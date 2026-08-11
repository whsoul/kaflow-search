//! What is known about a topic: its fields, how its index is looked after, and how far
//! indexing has got.
//!
//! ⚠️ `TopicFieldMeta` is persisted as-is, so a change to it has to keep reading what
//! earlier versions wrote. The compatibility layer below exists for that reason and is
//! not optional.

use crate::domain::{FieldSearchStat, IndexState, IndexedField, IndexedFieldKind};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ── DeserializerSpec ──────────────────────────────────────────────────────

/// How a topic's messages are decoded.
///
/// The variants are combinations of two independent choices: whether the bytes carry a
/// Confluent prefix (a magic byte and schema id) or are a bare datum, and whether the
/// schema comes from a local file or from a Schema Registry.
///
/// Bare bytes with a registry is not offered — a registry is looked up by the schema id
/// that only the prefixed form carries.
///
/// Serialized as `{ "kind": "json" }`, `{ "kind": "avro_local_file", "schemaPath": … }`
/// and so on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeserializerSpec {
    /// The default: bytes are read as UTF-8, replacing anything invalid rather than
    /// failing. Suits plain text and JSON alike.
    Json,
    /// A bare Avro datum, with the schema from a local `.avsc`.
    ///
    /// `schema_text` is what decoding uses; `schema_path` only says where it came from, so
    /// moving or deleting that file does not break an already configured topic. A value
    /// written by an older version may have no `schema_text`, and then the path is read.
    #[serde(rename_all = "camelCase")]
    AvroLocalFile {
        schema_path: String,
        #[serde(default)]
        schema_text: Option<String>,
    },
    /// Confluent-framed bytes (magic byte `0x00`, then a four-byte schema id), with the
    /// schema from a local `.avsc` — for reaching a registry-formatted topic without
    /// calling the registry. The id in the message is ignored.
    ///
    /// The framing must be checked strictly: anything but `0x00` is an error rather than
    /// something to decode anyway.
    #[serde(rename_all = "camelCase")]
    AvroConfluentLocal {
        schema_path: String,
        #[serde(default)]
        schema_text: Option<String>,
    },
    /// Confluent-framed bytes with the schema fetched from a registry by the id in each
    /// message. `basic_auth` is `"user:password"` when the registry needs it.
    #[serde(rename_all = "camelCase")]
    AvroConfluentRegistry {
        registry_url: String,
        basic_auth: Option<String>,
    },
    /// The envelope Kafka Connect writes with `schemas.enable=true`. Only `payload` is
    /// kept — `schema` describes the shape rather than the data, and indexing it would
    /// fill the index with field names.
    JsonConnectEnvelope,
    /// Confluent's JSON Schema serialization: the same framing, but the payload is JSON
    /// and describes itself, so no registry call is needed.
    ///
    /// The framing check here is deliberately forgiving — if the bytes begin like a
    /// prefix, it is stripped; otherwise the whole thing is read as JSON. A topic that
    /// mixes framed and unframed JSON still decodes.
    JsonSchemaConfluent,
    /// A bare Protobuf datum, with the schema from a local `.proto`.
    ///
    /// One `.proto` can define several messages, so `message_name` says which to decode —
    /// an input Avro does not need.
    #[serde(rename_all = "camelCase")]
    ProtobufLocalFile {
        schema_path: String,
        #[serde(default)]
        schema_text: Option<String>,
        message_name: String,
        /// Every file the main `.proto` imports, stored inline so the set is closed and
        /// stays readable once the originals move. Empty means it imports nothing.
        #[serde(default)]
        import_files: Vec<ProtoFile>,
    },
    /// Protobuf with the schema fetched from a registry.
    ///
    /// The framing carries a magic byte, a four-byte schema id, and a message-index array
    /// that picks which message in the schema to decode.
    ///
    /// ⚠️ A schema that imports others is not resolved. That must fail outright rather
    /// than decode against the wrong message — a wrong guess produces plausible garbage.
    #[serde(rename_all = "camelCase")]
    ProtobufConfluentRegistry {
        registry_url: String,
        basic_auth: Option<String>,
    },
    /// The same as above, but pointing at a saved registry rather than repeating its URL.
    /// The reference is resolved when it is used, so editing the registry reaches every
    /// topic that names it. This is the form to prefer.
    #[serde(rename_all = "camelCase")]
    AvroConfluentRegistryRef { resource_id: String },
    /// Protobuf against a saved registry, resolved the same way.
    #[serde(rename_all = "camelCase")]
    ProtobufConfluentRegistryRef { resource_id: String },
}

impl Default for DeserializerSpec {
    fn default() -> Self {
        DeserializerSpec::Json
    }
}

/// One imported `.proto`, stored with the text it was read from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtoFile {
    /// Must match the path in the importing `import "…"` line, such as
    /// `common/address.proto` — it is how the two are tied together.
    pub name: String,
    pub text: String,
}

/// A topic's key and value deserializers.
///
/// They are separate because the bytes are: a key and a value are produced independently,
/// and a registry keeps them under separate subjects. Mixing them — JSON keys with Avro
/// values, say — is ordinary rather than exceptional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TopicDeserializers {
    #[serde(default)]
    pub key: DeserializerSpec,
    #[serde(default)]
    pub value: DeserializerSpec,
}

// ── CleanupPolicy ─────────────────────────────────────────────────────────

/// What to reclaim when a topic's index has to give back space. Set per topic; falls back
/// to the global default when unset.
///
/// `DropIndex` ends the matter — nothing else runs after it. The other two may run
/// together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupPolicy {
    /// Drop the topic's index entirely. The default.
    DropIndex,
    /// Drop only the fields that are rarely searched.
    FieldBased,
    /// Keep the newest `max_count` per partition, trimming the oldest first. Nothing is
    /// trimmed until space is actually needed. A zero or absent limit falls back to the
    /// global default.
    CountBased { max_count: u64 },
}

// ── IlmUpdate ─────────────────────────────────────────────────────────────

/// A partial update: only the fields set here change, the rest keep their values.
#[derive(Debug, Default)]
pub struct IlmUpdate {
    /// Whether indexing has ever completed.
    pub indexed: Option<bool>,
    /// Whether only some fields are indexed.
    pub field_subset: Option<bool>,
    /// partition → latest indexed offset (i64)
    pub latest_indexed_offsets: Option<HashMap<u32, i64>>,
    /// partition → Kafka earliest offset
    pub earliest_offsets: Option<HashMap<u32, i64>>,
    /// The cluster's latest offset per partition, as of the last sync.
    pub kafka_latest_offsets: Option<HashMap<u32, i64>>,
    pub last_incremental_sync_at: Option<i64>,
    pub last_cleanup_at: Option<i64>,
    pub topic_type: Option<String>,
    pub topic_policy_checked_at: Option<i64>,
    /// When a search was last run against this topic. Browsing does not count.
    pub last_search_at: Option<i64>,
    /// `(when, fields)` — records that these fields were searched, rather than replacing
    /// a value.
    pub field_search_increments: Option<(i64, Vec<String>)>,
    // ── Read from the cluster's own topic configuration ─────────────────────
    /// Retention in milliseconds; `-1` means unlimited.
    pub retention_ms: Option<i64>,
    /// Retention in bytes per partition; `-1` means unlimited.
    pub retention_bytes: Option<i64>,
    pub compression_type: Option<String>,
    pub message_timestamp_type: Option<String>,
    pub partition_count: Option<usize>,
    pub replication_factor: Option<usize>,
    /// The cluster's own id for the topic. A topic deleted and recreated under the same
    /// name gets a new one, which is how that is told apart from the topic simply being
    /// emptied.
    pub topic_id: Option<String>,
    /// Added to the running total rather than replacing it.
    pub delivery_lost_add: Option<u64>,
}

// ── TopicFieldMeta ────────────────────────────────────────────────────────

/// Everything kept about one topic.
///
/// Index state is held as independent flags rather than one enum, because the two answer
/// different questions and a single word cannot say both. Use `derived_index_state()`
/// where a label is wanted; branch on the flags where behaviour depends on it.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", from = "TopicFieldMetaRaw")]
pub struct TopicFieldMeta {
    pub topic: String,
    pub key_fields: Vec<String>,
    pub payload_fields: Vec<String>,
    pub header_fields: Vec<String>,
    #[serde(default)]
    pub message_count: usize,
    #[serde(default)]
    pub index_entry_count: usize,
    // ── Index state, as independent flags ──────────────────────
    /// Whether indexing has ever completed.
    #[serde(default)]
    pub indexed: bool,
    /// Whether only some fields are indexed.
    #[serde(default)]
    pub field_subset: bool,
    /// The furthest offset indexed, per partition.
    #[serde(default)]
    pub latest_indexed_offsets: HashMap<u32, i64>,
    /// The cluster's earliest offset per partition, as of the last sync.
    #[serde(default)]
    pub earliest_offsets: HashMap<u32, i64>,
    /// The cluster's latest offset per partition, as of the last sync. Kept so that how
    /// far behind the index is can still be shown while the cluster is unreachable.
    #[serde(default)]
    pub kafka_latest_offsets: HashMap<u32, i64>,
    #[serde(default)]
    pub topic_type: Option<String>,
    #[serde(default)]
    pub topic_policy_checked_at: Option<i64>,
    #[serde(default)]
    pub last_incremental_sync_at: Option<i64>,
    #[serde(default)]
    pub last_cleanup_at: Option<i64>,
    #[serde(default)]
    pub retention_priority: Option<i32>,
    /// When a search was last run against this topic. Browsing does not count.
    #[serde(default)]
    pub last_search_at: Option<i64>,
    /// How much each field's index is searched.
    #[serde(default)]
    pub field_search_stats: HashMap<String, FieldSearchStat>,
    // ── Read from the cluster's own topic configuration ─────────────────────
    #[serde(default)]
    pub retention_ms: Option<i64>,
    #[serde(default)]
    pub retention_bytes: Option<i64>,
    #[serde(default)]
    pub compression_type: Option<String>,
    #[serde(default)]
    pub message_timestamp_type: Option<String>,
    #[serde(default)]
    pub partition_count: Option<usize>,
    #[serde(default)]
    pub replication_factor: Option<usize>,
    /// The cluster's own id for the topic. A topic deleted and recreated under the same
    /// name gets a new one.
    #[serde(default)]
    pub topic_id: Option<String>,
    /// This topic's cleanup policy; `None` uses the global default.
    ///
    /// Always serialized as a plain string (`"drop_index" | "field_based" |
    /// "count_based" | null`), never as an object — the limit for `count_based` travels in
    /// `max_count` beside it. An older list form is still accepted on the way in, taking
    /// the first entry.
    #[serde(
        default,
        alias = "cleanupPolicies",
        deserialize_with = "deserialize_cleanup_policy",
        serialize_with = "serialize_cleanup_policy_as_tag"
    )]
    pub cleanup_policy: Option<CleanupPolicy>,
    /// The limit for `count_based`, per partition. `None` uses the global default.
    #[serde(default)]
    pub max_count: Option<u64>,
    /// Which fields are indexed — a subset of those discovered. **Empty means all of
    /// them**, which is also what an older value with no list means.
    ///
    /// A plain list of names is still accepted on the way in and read as ordinary fields.
    #[serde(default, deserialize_with = "deserialize_indexed_fields")]
    pub indexed_fields: Vec<IndexedField>,
    /// When this topic's index was last removed to reclaim space. `None` means it never
    /// was.
    ///
    /// **Must be cleared when the topic is indexed again** — left set, the topic goes on
    /// being reported as cleaned up after it is no longer true.
    #[serde(default)]
    pub auto_cleanup_removed_at: Option<i64>,
    /// How to decode this topic's keys and values. `None` decodes both as JSON.
    ///
    /// An older value held a single deserializer meant for the value alone; it is read as
    /// a value deserializer with JSON keys.
    #[serde(default)]
    pub deserializers: Option<TopicDeserializers>,
    /// Fields to split into words, named directly rather than through `indexed_fields`.
    ///
    /// The two are independent and both apply. This one exists because the choice can be
    /// made before any field has been discovered — that is, before there is an
    /// `indexed_fields` list to mark. It works even when that list is empty.
    #[serde(default)]
    pub tokenize_fields: Vec<String>,
    /// On a compacted topic, how many superseded versions have been dropped in total.
    #[serde(default)]
    pub compact_superseded_total: u64,
    /// How many deletion markers have been handled. They are processed but store nothing,
    /// so they are counted separately — otherwise they would look like messages that were
    /// never reached.
    #[serde(default)]
    pub compact_tombstone_total: u64,
    /// Offsets the cluster never delivered although they were asked for — already
    /// compacted away, or transaction bookkeeping that was never a message. Counted so
    /// they are not mistaken for messages still to be indexed. The two causes cannot be
    /// told apart and are counted together.
    #[serde(default)]
    pub delivery_lost_total: u64,
}

impl TopicFieldMeta {
    /// The names of the indexed fields.
    pub fn indexed_field_names(&self) -> HashSet<String> {
        self.indexed_fields.iter().map(|f| f.name.clone()).collect()
    }

    /// Every field to be split into words, from both places it can be asked for.
    ///
    /// ⚠️ **Indexing and re-indexing must both go through this.** If one of them works out
    /// the set differently, a message ends up indexed one way and re-indexed another, and
    /// the same search stops agreeing with itself.
    pub fn effective_tokenized_fields(&self) -> HashSet<String> {
        let mut set: HashSet<String> = self
            .indexed_fields
            .iter()
            .filter(|f| f.tokenize)
            .map(|f| f.name.clone())
            .collect();
        set.extend(self.tokenize_fields.iter().cloned());
        set
    }

    /// The names of the fields that must never be dropped.
    pub fn required_field_names(&self) -> HashSet<String> {
        self.indexed_fields
            .iter()
            .filter(|f| f.kind == IndexedFieldKind::Required)
            .map(|f| f.name.clone())
            .collect()
    }

    /// The flags collapsed into one label, for display.
    pub fn derived_index_state(&self) -> IndexState {
        if !self.indexed {
            IndexState::NotIndexed
        } else if self.field_subset {
            IndexState::PartialField
        } else {
            IndexState::Full
        }
    }
}

// ── Reading what earlier versions wrote ─────────────────────────────────────

/// Reads what earlier versions wrote. Fields they had and this one does not are simply
/// dropped.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct TopicFieldMetaRaw {
    topic: String,
    key_fields: Vec<String>,
    payload_fields: Vec<String>,
    header_fields: Vec<String>,
    message_count: usize,
    index_entry_count: usize,

    indexed: Option<bool>,
    field_subset: Option<bool>,

    latest_indexed_offsets: HashMap<u32, i64>,
    earliest_offsets: HashMap<u32, i64>,
    kafka_latest_offsets: HashMap<u32, i64>,
    topic_type: Option<String>,
    topic_policy_checked_at: Option<i64>,
    last_incremental_sync_at: Option<i64>,
    last_cleanup_at: Option<i64>,
    retention_priority: Option<i32>,
    last_search_at: Option<i64>,
    field_search_stats: HashMap<String, FieldSearchStat>,
    retention_ms: Option<i64>,
    retention_bytes: Option<i64>,
    compression_type: Option<String>,
    message_timestamp_type: Option<String>,
    partition_count: Option<usize>,
    replication_factor: Option<usize>,
    topic_id: Option<String>,
    #[serde(
        alias = "cleanupPolicies",
        deserialize_with = "deserialize_cleanup_policy"
    )]
    cleanup_policy: Option<CleanupPolicy>,
    max_count: Option<u64>,
    #[serde(deserialize_with = "deserialize_indexed_fields")]
    indexed_fields: Vec<IndexedField>,
    auto_cleanup_removed_at: Option<i64>,
    deserializers: Option<TopicDeserializers>,
    /// The older single deserializer, read as the value one.
    deserializer: Option<DeserializerSpec>,
    tokenize_fields: Vec<String>,
    compact_superseded_total: u64,
    compact_tombstone_total: u64,
    delivery_lost_total: u64,
}

impl From<TopicFieldMetaRaw> for TopicFieldMeta {
    fn from(raw: TopicFieldMetaRaw) -> Self {
        let indexed = raw.indexed.unwrap_or(false);
        let field_subset = raw.field_subset.unwrap_or(false);
        TopicFieldMeta {
            topic: raw.topic,
            key_fields: raw.key_fields,
            payload_fields: raw.payload_fields,
            header_fields: raw.header_fields,
            message_count: raw.message_count,
            index_entry_count: raw.index_entry_count,
            indexed,
            field_subset,
            latest_indexed_offsets: raw.latest_indexed_offsets,
            earliest_offsets: raw.earliest_offsets,
            kafka_latest_offsets: raw.kafka_latest_offsets,
            topic_type: raw.topic_type,
            topic_policy_checked_at: raw.topic_policy_checked_at,
            last_incremental_sync_at: raw.last_incremental_sync_at,
            last_cleanup_at: raw.last_cleanup_at,
            retention_priority: raw.retention_priority,
            last_search_at: raw.last_search_at,
            field_search_stats: raw.field_search_stats,
            retention_ms: raw.retention_ms,
            retention_bytes: raw.retention_bytes,
            compression_type: raw.compression_type,
            message_timestamp_type: raw.message_timestamp_type,
            partition_count: raw.partition_count,
            replication_factor: raw.replication_factor,
            topic_id: raw.topic_id,
            cleanup_policy: raw.cleanup_policy,
            max_count: raw.max_count,
            indexed_fields: raw.indexed_fields,
            auto_cleanup_removed_at: raw.auto_cleanup_removed_at,
            deserializers: raw.deserializers.or_else(|| {
                // The older single deserializer described the value; keys were JSON.
                raw.deserializer.map(|value| TopicDeserializers {
                    key: DeserializerSpec::Json,
                    value,
                })
            }),
            tokenize_fields: raw.tokenize_fields,
            compact_superseded_total: raw.compact_superseded_total,
            compact_tombstone_total: raw.compact_tombstone_total,
            delivery_lost_total: raw.delivery_lost_total,
        }
    }
}

// ── serde helpers ─────────────────────────────────────────────────────────

/// Accepts either a list of names or a list of full entries.
fn deserialize_indexed_fields<'de, D>(deserializer: D) -> Result<Vec<IndexedField>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either {
        Name(String),
        Full(IndexedField),
    }

    let raw: Option<Vec<Either>> = Option::deserialize(deserializer)?;
    let list = raw.unwrap_or_default();
    let mut out = Vec::with_capacity(list.len());
    for item in list {
        match item {
            Either::Name(name) => out.push(IndexedField::optional(name)),
            Either::Full(f) => {
                if f.name.is_empty() {
                    return Err(D::Error::custom("indexed field name must not be empty"));
                }
                out.push(f);
            }
        }
    }
    Ok(out)
}

/// Accepts a null, a single policy, a plain string, or an older list (taking the first).
///
/// A plain `"count_based"` carries no limit, so it comes back with zero — the real limit
/// is read from `max_count` beside it, or from the global default. Zero here means
/// "unspecified", not "keep nothing".
fn deserialize_cleanup_policy<'de, D>(deserializer: D) -> Result<Option<CleanupPolicy>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AnyShape {
        One(CleanupPolicy),
        Many(Vec<CleanupPolicy>),
        Tag(String),
    }

    let v: Option<AnyShape> = Option::deserialize(deserializer)?;
    Ok(match v {
        None => None,
        Some(AnyShape::One(p)) => Some(p),
        Some(AnyShape::Many(v)) => v.into_iter().next(),
        Some(AnyShape::Tag(s)) => match s.as_str() {
            "drop_index" => Some(CleanupPolicy::DropIndex),
            "field_based" => Some(CleanupPolicy::FieldBased),
            "count_based" => Some(CleanupPolicy::CountBased { max_count: 0 }),
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "unknown cleanup_policy tag: {s}"
                )))
            }
        },
    })
}

/// Always writes a plain string. Nothing is lost by it: the only policy carrying a value
/// has that value beside it in `max_count`.
fn serialize_cleanup_policy_as_tag<S>(
    value: &Option<CleanupPolicy>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let tag: Option<&'static str> = value.as_ref().map(|p| match p {
        CleanupPolicy::DropIndex => "drop_index",
        CleanupPolicy::FieldBased => "field_based",
        CleanupPolicy::CountBased { .. } => "count_based",
    });
    tag.serialize(serializer)
}

// ── What reading a topic's state gives back ─────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicIndexSize {
    pub topic: String,
    pub index_bytes: u64,
    pub meta_bytes: u64,
    pub total_bytes: u64,
    pub index_entry_count: u64,
    pub meta_entry_count: u64,
    pub total_entry_count: u64,
}

/// Whether a workspace's stored data can still be read by this version.
///
/// `requires_reset` means it cannot, and the user has to be told before anything else is
/// attempted — reading on regardless produces wrong answers rather than errors.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStorageStatus {
    pub schema_version: u32,
    pub current_schema_version: u32,
    pub topic_cf_count: usize,
    pub has_legacy_data: bool,
    pub requires_reset: bool,
    pub reason: String,
}

/// How far indexing has got in one partition, against where the cluster is.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionOffsetStatus {
    pub partition: i32,
    pub earliest: i64,
    pub latest: i64,
    /// The oldest offset still indexed. `None` when nothing is. It can sit later than
    /// the cluster's earliest, since older entries may have been trimmed on purpose.
    pub min_indexed_offset: Option<i64>,
    /// How far indexing has got — including offsets that were handled but stored nothing,
    /// such as deletion markers. This answers "how far", not "how much is kept".
    pub max_indexed_offset: Option<i64>,
    /// Messages after that point which have not been indexed yet.
    pub gap: i64,
}

/// The same for a whole topic.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicOffsetStatus {
    pub topic: String,
    /// What the cluster holds, summed across partitions.
    pub kafka_total: i64,
    /// How many are in the index.
    pub indexed_total: i64,
    /// How much of the offset range has been gone over. This counts ground covered, not
    /// messages kept, so it stays meaningful where offsets exist that never held a message
    /// a reader would recognise.
    ///
    /// The identity that holds: `kafka_total = processed_total + gap`.
    #[serde(default)]
    pub processed_total: i64,
    /// What has arrived since indexing last caught up.
    pub gap: i64,
    /// Messages that exist but were deliberately not indexed.
    pub skipped: i64,
    /// Why, broken down. ⚠️ The parts need not add up to `skipped` — some causes cannot be
    /// worked out from what is on hand, and are left unaccounted rather than guessed at.
    pub skip_breakdown: SkipBreakdown,
    /// Whether every partition is indexed up to the cluster's latest.
    pub is_caught_up: bool,
    pub partitions: Vec<PartitionOffsetStatus>,
}

/// Why messages that exist are not in the index. Only causes that can be worked out
/// afterwards are listed — what cleanup removed leaves no trace to read back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// The cluster dropped them before indexing reached them.
    Retention,
    /// Older than the count the topic is set to keep. Deliberate.
    CountBased,
    /// Indexing is held back until the user decides on a full re-index.
    FullResyncPending,
    /// Superseded versions on a compacted topic. Handled, but only the newest per key is
    /// kept. Scattered, so only a total is available.
    CompactDeduped,
    /// Deletion markers. Handled, but there is nothing to store. Total only.
    CompactTombstone,
    /// Offsets the cluster never delivered. Total only, and the causes behind it cannot
    /// be told apart.
    DeliveryLost,
    /// Whatever is left over once the causes above are subtracted.
    CompactGapOrUnknown,
}

/// One skipped range of one partition. `start..end` excludes `end`, as offsets do.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffsetRangeSkip {
    pub partition: i32,
    pub start: i64,
    pub end: i64,
    /// `end - start`.
    pub count: i64,
}

/// Skipped messages grouped by cause.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipBreakdown {
    /// Ranges, per cause and partition.
    pub by_reason: Vec<SkipReasonGroup>,
    /// The same totalled, including causes that have no ranges to give.
    pub totals: Vec<SkipReasonTotal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipReasonGroup {
    pub reason: SkipReason,
    pub ranges: Vec<OffsetRangeSkip>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipReasonTotal {
    pub reason: SkipReason,
    pub count: i64,
}

/// A topic's configuration as the cluster reports it. Re-read rather than remembered: a
/// topic switched to compaction has to be looked after differently, and nothing announces
/// that change.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicConfigInfoResponse {
    pub topic: String,
    pub cleanup_policy: String,
    pub retention_ms: Option<i64>,
    pub retention_bytes: Option<i64>,
    pub compression_type: Option<String>,
    pub message_timestamp_type: Option<String>,
    pub partition_count: usize,
    pub replication_factor: usize,
    /// The cluster's own id for the topic. A topic deleted and recreated under the
    /// same name gets a new one.
    pub topic_id: Option<String>,
    pub checked_at: i64,
}

/// A field being selected for indexing.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedFieldInput {
    pub name: String,
    #[serde(default)]
    pub kind: IndexedFieldKind,
    #[serde(default)]
    pub is_representative: bool,
    /// Whether to split this field into words; see `IndexedField::tokenize`.
    #[serde(default)]
    pub tokenize: bool,
}

/// The latest schema a registry holds for one subject. For looking at — nothing is
/// decided or stored from it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySubjectSchema {
    /// The subject that was asked for, such as `"orders-value"`.
    pub subject: String,
    /// The registry's id for this schema.
    pub schema_id: u32,
    /// Its version within the subject.
    pub version: i32,
    /// The schema itself, exactly as the registry returned it.
    pub schema_text: String,
}

/// Roughly how large a topic is, for deciding whether to index it.
///
/// Worked out from offsets alone: no messages are read and nothing is created locally, so
/// asking about a topic costs nothing and leaves no trace.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicMessageCount {
    pub topic: String,
    /// An upper bound — on a compacted topic the real number is lower.
    pub message_count: i64,
    pub partition_count: u32,
}

/// How far behind a topic's index is.
///
/// ⚠️ `behind` means nothing when `reachable` is false — the cluster could not be asked
/// this time, and zero there is absence of an answer rather than being up to date.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicLagStatus {
    pub topic: String,
    /// Estimated messages not yet indexed. Zero when unreachable.
    pub behind: i64,
    /// Whether the cluster answered this time.
    pub reachable: bool,
}

/// A sample of a topic, used to suggest how it should be indexed.
///
/// Measured on raw bytes without decoding — the suggestion is needed before a
/// deserializer has been chosen, so it cannot depend on one.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicSizeProfile {
    pub topic: String,
    /// Average bytes per message, keys and headers included. Zero if nothing was read.
    pub avg_bytes: u64,
    /// How many messages were actually read.
    pub sampled: u32,
    /// The topic's own `cleanup.policy`, when it could be read.
    pub cleanup_policy: Option<String>,
    /// A guess at the value deserializer, named as the variant would be. `None` when the
    /// bytes did not say clearly enough.
    ///
    /// ⚠️ **A suggestion, not a determination.** It has to be confirmed before it is
    /// used — a wrong deserializer indexes plausible nonsense rather than failing.
    pub suggested_value_deserializer: Option<String>,
    /// One line on why, for showing beside it.
    pub suggested_reason: Option<String>,
    /// The same guess for keys. `None` where most messages have no key.
    pub suggested_key_deserializer: Option<String>,
    /// One line on why.
    pub suggested_key_reason: Option<String>,
}

/// Which fields look worth splitting into words, judged from a sample before indexing
/// starts.
///
/// Long, spaced-out text ranks highest; identifiers and numbers are left out, since
/// splitting those only costs space. ⚠️ A suggestion — the choice stays the user's.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestTokenizeFieldsResponse {
    pub topic: String,
    /// How many messages decoded well enough to look at. Zero means no suggestion.
    pub sampled: u32,
    /// Candidates, best first.
    pub candidates: Vec<TokenizeFieldCandidate>,
}

/// One candidate field.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenizeFieldCandidate {
    /// The field's path, such as `"P.message"`.
    pub field: String,
    /// Average length seen in the sample.
    pub avg_len: u32,
    /// Average number of spaces — how likely it is to hold more than one word.
    pub avg_whitespace: u32,
    /// Set on the single best candidate, as a starting point for the choice.
    pub recommended: bool,
}
