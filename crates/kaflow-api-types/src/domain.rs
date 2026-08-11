//! Core types: where a message sits, what the index keeps about it, and what a search
//! gives back.

use serde::{Deserialize, Serialize};

// ── Core message location / meta ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocLoc {
    pub partition: u32,
    pub offset: u64,
}

/// What the index keeps about a single message: when it arrived, and its field values as
/// they were before any tokenization.
///
/// This is what the original text is restored from, so it has to stay faithful to the
/// message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaValue {
    pub ts_millis: u64,
    /// Each field paired with its value before tokenization. Topic, partition and offset
    /// are not repeated here — they are already known from where this record was found.
    #[serde(default)]
    pub field_values: Vec<(String, String)>,
    /// Which of those fields were split into words. Empty means every value was indexed
    /// whole.
    #[serde(default)]
    pub tokenized_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaRow {
    pub topic: String,
    pub partition: u32,
    pub offset: u64,
    pub value: MetaValue,
}

// ── Search hit / decoded index key ────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SearchHit {
    pub rev_ts: u64,
    pub loc: DocLoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHitWithKey {
    pub raw_key_hex: String,
    pub pretty_key: String,
    pub hit: SearchHit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedIndexKey {
    pub topic: String,
    pub field: String,
    pub term: String,
    pub rev_ts: u64,
    pub partition: u32,
    pub offset: u64,
}

// ── Index state / field index meta ────────────────────────────────────────

/// A topic's index state, as a single label.
///
/// It is derived from the independent flags on `TopicFieldMeta`. Branch on those flags
/// rather than on this enum — this one exists for display and for responses, and a label
/// that collapses two flags into one word will not survive a third flag being added.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum IndexState {
    #[default]
    NotIndexed,
    /// Every selected field is indexed.
    Full,
    /// Only some of them — the rest were dropped to reclaim space.
    PartialField,
}

/// How much a field's index actually gets searched.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FieldSearchStat {
    pub search_count: u64,
    pub last_searched_at: Option<i64>,
}

/// What kind of index a field gets, which decides whether it may be dropped when space
/// has to be reclaimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexedFieldKind {
    /// Never dropped, even when space is being reclaimed.
    Required,
    /// The default. May be dropped to reclaim space.
    Optional,
    /// Indexed as `Optional`, and must be until this contract says otherwise — a stored
    /// field can still carry it, and an engine that gave it behaviour of its own would
    /// index the same topic differently from one that did not.
    Async,
}

impl Default for IndexedFieldKind {
    fn default() -> Self {
        Self::Optional
    }
}

/// One field selected for indexing.
///
/// `is_representative` marks a field as identifying enough to stand in for the whole
/// message where only a line of it fits. It changes nothing about what gets indexed.
///
/// `tokenize` splits the value into words, so a word in the middle of a sentence can be
/// found; without it the value is only findable as a whole.
///
/// Both are independent of `kind` — a field can be required and representative at once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedField {
    pub name: String,
    #[serde(default)]
    pub kind: IndexedFieldKind,
    #[serde(default)]
    pub is_representative: bool,
    #[serde(default)]
    pub tokenize: bool,
}

impl IndexedField {
    pub fn optional(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: IndexedFieldKind::Optional,
            is_representative: false,
            tokenize: false,
        }
    }

    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: IndexedFieldKind::Required,
            is_representative: false,
            tokenize: false,
        }
    }
}
