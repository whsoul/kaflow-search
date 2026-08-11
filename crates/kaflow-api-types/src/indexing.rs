//! Progress and results of indexing and of reclaiming space.

use crate::decode_failure::DecodeFailureContext;
use crate::full_resync::FullResyncTrigger;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenKafkaTopicResponse {
    pub topic: String,
    /// What the cluster holds.
    pub kafka_total_count: i64,
    /// Newly indexed by this call.
    pub newly_indexed: usize,
    pub key_fields: Vec<String>,
    pub payload_fields: Vec<String>,
    pub header_fields: Vec<String>,
    /// Index entries removed.
    pub cleaned_up_i: usize,
    /// Message records removed.
    pub cleaned_up_m: usize,
    /// Reasons the index may no longer line up with the topic. **When this is not empty,
    /// nothing was cleaned up** — the situation needs deciding before anything is removed,
    /// since the wrong choice discards an index that was fine.
    #[serde(default)]
    pub full_resync_triggers: Vec<FullResyncTrigger>,
    /// Set when indexing stopped on a message it could not decode. What was indexed
    /// before that point is kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decode_failure_context: Option<DecodeFailureContext>,
    /// Whether it stopped at the requested limit rather than at the end, so there may be
    /// more to do. False when no limit was given.
    #[serde(default)]
    pub has_more: bool,
}

/// Progress while space is being reclaimed.
/// phase: "scanning" | "cleaning" | "done"
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupProgressEvent {
    pub topic: String,
    pub deleted: usize,
    pub phase: String,
}

// ── History kept for a compacted topic ──────────────────────────────────────

/// What is remembered about one key of a compacted topic, where each new value replaces
/// the last.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactKeyHistory {
    /// Where its current value sits.
    pub latest_offset: u64,
    /// How many times it has been replaced. Kept even when the list below is trimmed.
    pub superseded_total: u64,
    /// Whether the latest record deletes the key rather than setting it.
    #[serde(default)]
    pub latest_is_tombstone: bool,
    /// Recent replaced versions, newest first. Only as many as fit are kept.
    pub entries: Vec<CompactSupersededVersion>,
}

/// One replaced version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactSupersededVersion {
    pub offset: u64,
    pub ts_millis: u64,
    /// The value it held, where there was room to keep it. `None` leaves only the
    /// position and the time.
    pub field_values: Option<Vec<(String, String)>>,
    /// True when the record was a deletion, which has no value to begin with — this is
    /// what tells that apart from a value that was dropped for space.
    #[serde(default)]
    pub tombstone: bool,
}

// ── Keys that have been deleted ─────────────────────────────────────────────

/// A key whose latest record deletes it.
///
/// Nothing else in the index refers to such a key — the history above is the only place
/// it survives, which is why it is listed separately rather than searched for.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactDeletedKeyRow {
    pub partition: u32,
    /// The key as its deserializer rendered it.
    pub key_raw: String,
    /// Where the deletion sits.
    pub tombstone_offset: u64,
    /// When it was deleted, where that is still known.
    pub deleted_ts_millis: Option<u64>,
    /// How many times it was replaced before being deleted.
    pub superseded_total: u64,
    /// The last value it held, where that is still kept.
    pub last_value: Option<CompactSupersededVersion>,
}

/// Where a listing left off. The next page starts after this position.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactDeletedKeyCursor {
    pub partition: u32,
    pub key_raw: String,
}

/// One page of deleted keys. `next_cursor` of `None` means the end was reached.
///
/// ⚠️ A short page with a cursor still set means the search ran out of budget, not that
/// there is nothing more — ask again rather than concluding the list is complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactDeletedKeysPage {
    pub rows: Vec<CompactDeletedKeyRow>,
    pub next_cursor: Option<CompactDeletedKeyCursor>,
    /// How many entries were examined. Diagnostic only.
    pub scanned: u64,
}
