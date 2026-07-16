//! Global / Topic 설정 DTO — `ConfigApi` 의 입출력 타입.

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
    /// 자동싱크 round-robin slice 상한 (토픽당 한 턴 최대 인덱싱 건수).
    pub auto_sync_slice_size: usize,
    // ── ILM ───────────────────────────────────────────────────
    pub incremental_sync_enabled: bool,
    pub retention_cleanup_enabled: bool,
    pub retention_cleanup_interval_secs: u64,
    pub size_cleanup_enabled: bool,
    pub size_cleanup_interval_secs: u64,
    pub max_index_bytes_per_cluster: u64,
    pub default_cleanup_policies: Vec<String>,
    /// CountBased 정책 — 파티션당 최신 N건 보존 한도.
    pub default_keep_count_per_partition: u64,
    /// 토픽당 인덱스 총 건수 상한 — 조회 캡 동치. 천장 = `TOPIC_INDEX_COUNT_CAP`(tighten-only).
    /// FE 경고(toast/배지) 임계 + count_based effective clamp 가 사용. 테스트용 하향 가능.
    pub topic_index_count_cap: u64,
    // ── Lock wait timeouts ────────────────────────────────────
    pub retention_cleanup_wait_timeout_secs: u64,
    pub size_cleanup_wait_timeout_secs: u64,
    pub foreground_acquire_wait_timeout_secs: u64,
    // ── RocksDB compression ───────────────────────────────────
    /// "none" | "snappy" | "zstd". 변경 시 재연결 필요.
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
    /// 구 FE 페이로드 호환 — 미전송 시 천장(=기본값) 유지.
    #[serde(default = "default_topic_index_count_cap")]
    pub topic_index_count_cap: u64,
    // ── Lock wait timeouts ────────────────────────────────────
    pub retention_cleanup_wait_timeout_secs: u64,
    pub size_cleanup_wait_timeout_secs: u64,
    pub foreground_acquire_wait_timeout_secs: u64,
    // ── RocksDB compression ───────────────────────────────────
    pub compression_mode: String,
}

/// 프론트엔드에서 사용자가 수동으로 설정할 수 있는 T-META 필드.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicMetaConfigInput {
    // ── 사용자 설정 ─────────────────────────────────────────────
    pub retention_priority: Option<i32>,
    pub topic_type: Option<String>,
    pub cleanup_policy: Option<String>,
    /// CountBased 한도 (파티션당 메시지 수). None 이면 글로벌 default_keep_count_per_partition 폴백.
    pub max_count: Option<u64>,
    // ── Kafka 토픽 설정 (자동 갱신되지만 수동 오버라이드 가능) ──────
    pub retention_ms: Option<i64>,
    pub retention_bytes: Option<i64>,
    pub compression_type: Option<String>,
    pub message_timestamp_type: Option<String>,
    pub partition_count: Option<usize>,
    pub replication_factor: Option<usize>,
    pub topic_id: Option<String>,
}

impl From<GlobalConfigView> for GlobalConfigInput {
    /// 현재 값을 그대로 다시 쓰기 위한 변환 (필드 동일).
    /// 용도: settings.json 로드 시 "기본값 + 파일값" 을 합쳐 `write_global_config` 로 태우기 —
    /// 그래야 **clamp 가 한 곳(write)에서만** 일어난다 (로드 전용 clamp 를 따로 두면 어긋난다).
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
