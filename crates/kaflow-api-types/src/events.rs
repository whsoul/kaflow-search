//! Event channel names.
//!
//! Declare a new one here and use the constant. A literal written at the point of use is
//! how a listener and an emitter come to disagree by one character.

#![allow(dead_code)]

pub const KAFKA_INDEXING_PROGRESS: &str = "kafka-indexing-progress";
pub const KAFKA_CLEANUP_PROGRESS: &str = "kafka-cleanup-progress";
pub const TOPIC_DROP_PROGRESS: &str = "topic-drop-progress";
pub const TOPIC_INDEX_REFRESHED: &str = "topic-index-refreshed";
pub const ILM_ACTION_LOG: &str = "ilm-action-log";
pub const SEARCH_PREFETCH_PROGRESS: &str = "search-prefetch-progress";
pub const TIME_BUCKETS_PROGRESS: &str = "time-buckets-progress";
/// Progress of a boolean search — `{ processed, matched, done }`, ending with `done`.
pub const MULTI_SEARCH_PROGRESS: &str = "multi-search-progress";
/// A short note when a drill finishes: what it chose, how much it found, how long.
pub const SEARCH_DRILL_LOG: &str = "search-drill-log";
pub const FIELD_REINDEX_PROGRESS: &str = "field-reindex-progress";
pub const DB_RESYNC_PROGRESS: &str = "db-resync-progress";
pub const DB_CLEAR_PROGRESS: &str = "db-clear-progress";
/// Progress of an export — `{ written, total, done }`, ending with `done`.
pub const EXPORT_PROGRESS: &str = "export-progress";
