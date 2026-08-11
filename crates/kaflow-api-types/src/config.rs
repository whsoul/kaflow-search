//! Settings, global and per topic.

use serde::{Deserialize, Serialize};

fn default_auto_sync_slice_size() -> usize {
    50_000
}

fn default_topic_index_count_cap() -> u64 {
    crate::limits::TOPIC_INDEX_COUNT_CAP
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConfigView {
    // ── Indexing ──────────────────────────────────────────────
    pub indexing_batch_size: usize,
    /// How many messages one topic may index before the next gets a turn.
    pub auto_sync_slice_size: usize,
    // ── ILM ───────────────────────────────────────────────────
    pub incremental_sync_enabled: bool,
    pub retention_cleanup_enabled: bool,
    pub retention_cleanup_interval_secs: u64,
    pub size_cleanup_enabled: bool,
    pub size_cleanup_interval_secs: u64,
    pub max_index_bytes_per_cluster: u64,
    pub default_cleanup_policies: Vec<String>,
    /// How many messages to keep per partition, where that policy applies.
    pub default_keep_count_per_partition: u64,
    /// Total messages one topic may keep indexed. Can be lowered but not raised past
    /// `TOPIC_INDEX_COUNT_CAP`.
    pub topic_index_count_cap: u64,
    // ── Lock wait timeouts ────────────────────────────────────
    pub retention_cleanup_wait_timeout_secs: u64,
    pub size_cleanup_wait_timeout_secs: u64,
    pub foreground_acquire_wait_timeout_secs: u64,
    // ── RocksDB compression ───────────────────────────────────
    /// `"none"`, `"snappy"` or `"zstd"`. Takes effect on reconnect.
    pub compression_mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConfigInput {
    // ── Indexing ──────────────────────────────────────────────
    pub indexing_batch_size: usize,
    #[serde(default = "default_auto_sync_slice_size")]
    pub auto_sync_slice_size: usize,
    // ── ILM ───────────────────────────────────────────────────
    pub incremental_sync_enabled: bool,
    pub retention_cleanup_enabled: bool,
    pub retention_cleanup_interval_secs: u64,
    pub size_cleanup_enabled: bool,
    pub size_cleanup_interval_secs: u64,
    pub max_index_bytes_per_cluster: u64,
    pub default_cleanup_policies: Vec<String>,
    pub default_keep_count_per_partition: u64,
    /// Absent in older payloads, in which case the default stands.
    #[serde(default = "default_topic_index_count_cap")]
    pub topic_index_count_cap: u64,
    // ── Lock wait timeouts ────────────────────────────────────
    pub retention_cleanup_wait_timeout_secs: u64,
    pub size_cleanup_wait_timeout_secs: u64,
    pub foreground_acquire_wait_timeout_secs: u64,
    // ── RocksDB compression ───────────────────────────────────
    pub compression_mode: String,
}

/// The per-topic settings a user may change by hand.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicMetaConfigInput {
    // ── Chosen by the user ──────────────────────────────────────
    pub retention_priority: Option<i32>,
    pub topic_type: Option<String>,
    pub cleanup_policy: Option<String>,
    /// Messages to keep per partition; `None` uses the global default.
    pub max_count: Option<u64>,
    // ── Read from the cluster, but overridable ─────────────────
    pub retention_ms: Option<i64>,
    pub retention_bytes: Option<i64>,
    pub compression_type: Option<String>,
    pub message_timestamp_type: Option<String>,
    pub partition_count: Option<usize>,
    pub replication_factor: Option<usize>,
    pub topic_id: Option<String>,
}

impl From<GlobalConfigView> for GlobalConfigInput {
    /// Turns the current values back into a write request.
    ///
    /// Loading goes back through writing so that values are bounded in exactly one place.
    /// A second, load-only bounding step is how the two come to disagree.
    fn from(v: GlobalConfigView) -> Self {
        Self {
            indexing_batch_size: v.indexing_batch_size,
            auto_sync_slice_size: v.auto_sync_slice_size,
            incremental_sync_enabled: v.incremental_sync_enabled,
            retention_cleanup_enabled: v.retention_cleanup_enabled,
            retention_cleanup_interval_secs: v.retention_cleanup_interval_secs,
            size_cleanup_enabled: v.size_cleanup_enabled,
            size_cleanup_interval_secs: v.size_cleanup_interval_secs,
            max_index_bytes_per_cluster: v.max_index_bytes_per_cluster,
            default_cleanup_policies: v.default_cleanup_policies,
            default_keep_count_per_partition: v.default_keep_count_per_partition,
            topic_index_count_cap: v.topic_index_count_cap,
            retention_cleanup_wait_timeout_secs: v.retention_cleanup_wait_timeout_secs,
            size_cleanup_wait_timeout_secs: v.size_cleanup_wait_timeout_secs,
            foreground_acquire_wait_timeout_secs: v.foreground_acquire_wait_timeout_secs,
            compression_mode: v.compression_mode,
        }
    }
}
