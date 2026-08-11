//! The types a search speaks in: what is asked for, and what comes back.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchMode {
    Exact,
    Prefix,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    NewestFirst,
    OldestFirst,
}

/// Where a browse left off.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionCursor {
    pub topic: String,
    pub partition: u32,
    pub offset: u64,
}

/// Where a keyword search left off, per topic and field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFieldCursor {
    pub topic: String,
    pub field: String,
    pub last_key_hex: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexRequest {
    pub query: String,
    /// Which fields to match against; any of them counts as a match.
    /// `None` = every indexed field. An empty list matches nothing.
    pub fields: Option<Vec<String>>,
    pub topics: Option<Vec<String>>,
    pub mode: SearchMode,
    pub sort_order: SortOrder,
    pub limit: usize,
    /// Used when there is no query.
    #[serde(default)]
    pub browse_cursors: Vec<PartitionCursor>,
    /// Used when there is one.
    #[serde(default)]
    pub search_cursors: Vec<SearchFieldCursor>,
    /// Narrows by message time before anything else is considered.
    #[serde(default)]
    pub ts_range: Option<TsRange>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResult {
    pub matched_field: Option<String>,
    pub raw_index_key_hex: Option<String>,
    pub pretty_index_key: Option<String>,
    pub raw_meta_key_hex: String,
    pub pretty_meta_key: String,
    /// Where the content came from. This has to be set truthfully — a caller decides
    /// whether to ask again based on it, and nothing else tells it apart:
    /// `"full"` — read from the cluster; payload, key and headers are present.
    /// `"meta-only"` — from the index alone; payload, key and headers must be left empty.
    /// `"missing"` — it was asked for and the cluster did not return it. Retention makes
    /// this an ordinary outcome rather than a failure.
    pub content_source: String,
    /// The index keys this message is reachable by, as readable text.
    pub meta_index_entries: Vec<(String, String)>,
    /// Exactly what the index stored, for inspection.
    pub raw_meta_value_json: Option<String>,
    /// Indexed fields as `(path, value)`, so a row can be shown without reading the
    /// message itself. Paths look like `K`, `K.foo`, `P`, `P.bar`, `H.x` — key, payload
    /// and header, then the path within.
    #[serde(default)]
    pub field_values: Vec<(String, String)>,
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub timestamp: String,
    pub key: String,
    pub payload: String,
    pub headers: Vec<(String, String)>,
    pub json: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexResponse {
    pub results: Vec<MessageResult>,
    /// Positions to read messages for; set instead of `results` when there is no query.
    pub locs: Vec<LocItem>,
    pub has_more: bool,
    pub next_browse_cursors: Vec<PartitionCursor>,
    pub next_search_cursors: Vec<SearchFieldCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocItem {
    pub ts_millis: u64,
    pub partition: u32,
    pub offset: u64,
    /// Only set when this position came from a match. Omitted rather than sent as null —
    /// with a list this long, the nulls alone would be the larger part of the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pretty_index_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_index_key_hex: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefetchSearchLocsResponse {
    pub locs: Vec<LocItem>,
    pub total: usize,
    pub capped: bool,
    /// Fields the query matched in at least once, across every topic searched. Useful for
    /// deciding which columns are worth showing.
    pub hit_fields: Vec<String>,
}

// ── Boolean queries ─────────────────────────────────────────────────────────
//
// A query is a tree of conditions: groups combined with must / should / must_not, and
// leaves that match a term, a prefix, a range, or the mere presence of a field.
//
// The JSON shape is externally tagged:
//   { "bool":   { "must": [...], "should": [...], "mustNot": [...] } }
//   { "term":   { "field": "f"|null, "value": "v" } }
//   { "prefix": { "field": "f"|null, "value": "v" } }
//   { "exists": { "field": "f" } }
//   { "range":  { "field": "f", "gte": "lo"|null, "lte": "hi"|null } }
//
// A range with neither end is rejected — it would mean the same as `exists`. So is one
// whose lower end sits above its upper end. How the bounds compare is part of the
// contract and is stated on `RangeLeaf`.

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryNode {
    Bool(BoolNode),
    Term(TermLeaf),
    Prefix(PrefixLeaf),
    Exists(ExistsLeaf),
    Range(RangeLeaf),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BoolNode {
    #[serde(default)]
    pub must: Vec<QueryNode>,
    #[serde(default)]
    pub should: Vec<QueryNode>,
    #[serde(default)]
    pub must_not: Vec<QueryNode>,
    /// How many `should` clauses must match. `None` means one, and only when `must` is
    /// empty.
    #[serde(default)]
    pub minimum_should_match: Option<u32>,
}

/// `field: None` matches across every indexed field.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TermLeaf {
    pub field: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefixLeaf {
    pub field: Option<String>,
    pub value: String,
}

/// Matches messages that have anything at all indexed for the field.
/// Not implemented yet — a query using it is rejected rather than silently ignored.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistsLeaf {
    pub field: String,
}

/// Matches values between `gte` and `lte`, both ends included.
///
/// `field` is required — a range has no meaning spread across fields — and at least one
/// end must be given.
///
/// **Bounds are compared lexicographically, as text.** An implementation must not treat
/// values that look numeric as numbers: the same query has to select the same messages
/// whichever engine answers it.
///
/// ⚠️ A consequence worth stating plainly: unpadded numbers do not behave the way a
/// reader expects. `"10"` sorts before `"2"`, so such a range returns the wrong rows
/// rather than an error. Numbers have to be zero-padded to equal width when they are
/// indexed. ISO 8601 dates already sort correctly as text.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeLeaf {
    pub field: String,
    pub gte: Option<String>,
    pub lte: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    /// `None` searches every topic.
    pub topics: Option<Vec<String>>,
    pub r#where: QueryNode,
    pub sort_order: SortOrder,
    /// Maximum results. `0` means the engine's own limit.
    #[serde(default)]
    pub limit: usize,
    /// Narrows by message time. This sits outside the boolean tree and applies to every
    /// match, whatever the tree says.
    #[serde(default)]
    pub ts_range: Option<TsRange>,
    /// Narrows by position, in the same way and with the same reach as `ts_range`.
    /// `limit` counts what survives this filter, not what was examined.
    #[serde(default)]
    pub pos_filter: Option<PosFilter>,
}

/// Restricts results to certain partitions and a range of offsets, both ends included.
///
/// A partition and offset identify a message unchangeably, so this means the same thing
/// wherever it is applied.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PosFilter {
    /// Partitions to allow. Empty means all of them.
    #[serde(default)]
    pub partitions: Vec<u32>,
    pub offset_gte: Option<u64>,
    pub offset_lte: Option<u64>,
}

impl PosFilter {
    /// True when this filter would let everything through, so a caller can drop it.
    pub fn is_noop(&self) -> bool {
        self.partitions.is_empty() && self.offset_gte.is_none() && self.offset_lte.is_none()
    }

    /// Whether this position passes.
    pub fn passes(&self, partition: u32, offset: u64) -> bool {
        if !self.partitions.is_empty() && !self.partitions.contains(&partition) {
            return false;
        }
        if let Some(lo) = self.offset_gte {
            if offset < lo {
                return false;
            }
        }
        if let Some(hi) = self.offset_lte {
            if offset > hi {
                return false;
            }
        }
        true
    }
}

/// A range of message times, both ends included. Compared as numbers, not as text.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TsRange {
    pub gte_ms: Option<u64>,
    pub lte_ms: Option<u64>,
}

/// What one condition cost to run. Diagnostic only.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSearchCondCost {
    pub field: Option<String>,
    pub term: String,
    /// "exact" | "prefix"
    pub match_type: String,
    /// How the condition was read. Diagnostic only.
    pub cursor_kind: String,
    /// How many positions had to be held in memory, if any.
    pub materialized: u64,
    /// Fields this one condition matched in, before it was combined with the others —
    /// so a field here need not appear in the final result.
    pub hit_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSearchResponse {
    /// Every matching position, in the requested order and up to the cap. The per-hit
    /// fields are left empty here; content is asked for separately.
    pub locs: Vec<LocItem>,
    pub total: usize,
    pub capped: bool,
    /// Which strategy the engine chose. Diagnostic only.
    pub strategy: String,
    pub per_condition: Vec<MultiSearchCondCost>,
    /// Fields matched in at least once during this search, across every topic.
    pub hit_fields: Vec<String>,
    pub scan_ms: u64,
    /// Set when the query was rewritten before running — a compact form of what actually
    /// ran, so the difference can be shown. `None` when nothing changed.
    pub normalized_where: Option<String>,
}

// ── Counting matches by position ────────────────────────────────────────────
//
// Matches counted per partition and offset range. Only the finest level is sent, and only
// where the count is non-zero; coarser groupings are the caller's to build by adding
// leaves together.

/// One offset range of one partition. The field names are abbreviated because this
/// repeats tens of thousands of times in a single response.
#[derive(Debug, Clone, Serialize)]
pub struct OffsetBucket {
    /// First offset in the range, included.
    #[serde(rename = "f")]
    pub from_offset: u64,
    /// Last offset in the range, included. Sent rather than inferred: the final range of
    /// a partition is cut short, and the width is not guaranteed to stay what it is.
    #[serde(rename = "t")]
    pub to_offset: u64,
    /// Matches in this range. Ranges with none are left out entirely.
    #[serde(rename = "c")]
    pub count: u32,
    /// The actual offsets, when the response carries them. A caller that has these can
    /// open the range without asking again.
    #[serde(rename = "os", skip_serializing_if = "Option::is_none")]
    pub offsets: Option<Vec<u64>>,
}

/// One partition's counts. Only the finest level is here; coarser ones are derived by
/// the caller.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionOffsetBuckets {
    pub partition: u32,
    pub count: u64,
    pub min_offset: u64,
    pub max_offset: u64,
    /// How wide each range is. Sent rather than assumed, in case it changes.
    pub bucket_unit: u32,
    pub buckets: Vec<OffsetBucket>,
}

/// What the response carries, stated outright so a caller never has to guess from whether
/// `offsets` happens to be present.
///
///   - `CountOnly` — counts only; opening a range means asking again.
///   - `OffsetsInline` — every range carries its offsets.
///   - `PartialInline` — **some do and some do not.** Decide per range, not for the
///     response as a whole. ⚠️ **An incomplete set must never be sent**: where offsets are
///     present they have to be all of them, because a caller uses them instead of asking
///     again and cannot tell that some are missing.
///   - `Drill` — the answer to a request for one range, rather than an overview.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OffsetBucketsDeliveryMode {
    CountOnly,
    OffsetsInline,
    PartialInline,
    Drill,
}

/// Match counts across a topic, by partition and offset range.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OffsetBucketsResponse {
    pub topic: String,
    pub total: u64,
    pub partitions: Vec<PartitionOffsetBuckets>,
    /// ⚠️ Branch on this, not on whether a range happens to carry offsets.
    pub delivery_mode: OffsetBucketsDeliveryMode,
    /// Which plan produced this. Diagnostic only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_plan: Option<String>,
}

/// Messages counted in one time bucket of one partition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBucketRow {
    pub partition: u32,
    /// Start of the bucket, in epoch milliseconds.
    pub gran_start_ms: u64,
    pub count: u64,
    /// The offsets in this bucket, when the response carries them — enough to open the
    /// bucket without asking again.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offsets: Option<Vec<u64>>,
}

/// What the response carries, on the same terms as the offset version above:
///   - `CountOnly` — counts only.
///   - `OffsetsInline` — every bucket carries its offsets.
///   - `PartialInline` — some do and some do not; decide per bucket. Where offsets are
///     present they are complete.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimeBucketDeliveryMode {
    CountOnly,
    OffsetsInline,
    PartialInline,
}

/// Counts over time: the bucket width, and one row per bucket that has anything in it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBucketsResponse {
    pub gran_ms: u64,
    pub total: u64,
    pub rows: Vec<TimeBucketRow>,
    /// ⚠️ Branch on this, not on whether a bucket happens to carry offsets.
    pub delivery_mode: TimeBucketDeliveryMode,
    /// Which plan produced this. Diagnostic only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_plan: Option<String>,
}

/// Which of the two groupings to return. Asking for both costs little more than asking
/// for one — the work of finding the matches is shared.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BucketsKind {
    Offset,
    Time,
    All,
}

/// How much detail the buckets should carry — stated as intent rather than as a number.
///
///   - `CountOnly` — counts alone.
///   - `Inline { cap }` — offsets up to `cap`; buckets past it come back as counts only.
///     Enough to open what is on screen without making the response heavy.
///   - `Default` — the engine's own limit. Used when nothing is asked for.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum BucketDetail {
    CountOnly,
    Inline { cap: usize },
    Default,
}

impl BucketDetail {
    /// Resolves to a number, given the engine's default to fall back on.
    pub fn inline_cap(self, default_cap: usize) -> usize {
        match self {
            BucketDetail::CountOnly => 0,
            BucketDetail::Inline { cap } => cap,
            BucketDetail::Default => default_cap,
        }
    }
}

/// Both groupings from one request. Only what `kind` asked for is present.
///
/// This answers the whole topic; narrowing to one range is a separate call.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CombinedBucketsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<OffsetBucketsResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeBucketsResponse>,
}

// ── Trying a deserializer out ───────────────────────────────────────────────
//
// Reads a few real messages and decodes them with a given deserializer, showing the
// result as it came out. Whether it is right is for a person to see.

/// How one message decoded. On success the text is set, on failure the error is; a
/// message with no key or no value decodes to an empty string rather than an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestDeserializeSample {
    pub partition: i32,
    pub offset: i64,
    pub key_decoded: Option<String>,
    pub key_error: Option<String>,
    pub value_decoded: Option<String>,
    pub value_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestDeserializeResponse {
    /// One entry per message read.
    pub samples: Vec<TestDeserializeSample>,
}

#[cfg(test)]
mod pos_filter_tests {
    use super::PosFilter;

    #[test]
    fn empty_filter_passes_everything_and_is_noop() {
        let f = PosFilter::default();
        assert!(f.is_noop());
        assert!(f.passes(0, 0));
        assert!(f.passes(7, u64::MAX));
    }

    #[test]
    fn partition_set_filters() {
        let f = PosFilter {
            partitions: vec![1, 3],
            ..Default::default()
        };
        assert!(!f.is_noop());
        assert!(f.passes(1, 100));
        assert!(f.passes(3, 0));
        assert!(!f.passes(2, 100));
        assert!(!f.passes(0, 100));
    }

    #[test]
    fn offset_range_is_inclusive() {
        let f = PosFilter {
            offset_gte: Some(10),
            offset_lte: Some(20),
            ..Default::default()
        };
        assert!(!f.passes(0, 9));
        assert!(f.passes(0, 10)); // lower end included
        assert!(f.passes(0, 15));
        assert!(f.passes(0, 20)); // upper end included
        assert!(!f.passes(0, 21));
    }

    #[test]
    fn open_ended_offset_bounds() {
        let lo_only = PosFilter {
            offset_gte: Some(100),
            ..Default::default()
        };
        assert!(!lo_only.passes(0, 99));
        assert!(lo_only.passes(0, u64::MAX));

        let hi_only = PosFilter {
            offset_lte: Some(100),
            ..Default::default()
        };
        assert!(hi_only.passes(0, 0));
        assert!(!hi_only.passes(0, 101));
    }

    #[test]
    fn partition_and_offset_both_apply() {
        let f = PosFilter {
            partitions: vec![2],
            offset_gte: Some(50),
            offset_lte: None,
        };
        assert!(f.passes(2, 50));
        assert!(!f.passes(2, 49)); // right partition, offset too low
        assert!(!f.passes(3, 100)); // right offset, wrong partition
    }
}
