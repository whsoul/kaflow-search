use serde::{Deserialize, Serialize};

/// Kafka 클러스터 내 단일 브로커 정보.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerInfo {
    pub node_id: i32,
    pub host: String,
    pub port: i32,
    pub rack: Option<String>,
    pub is_controller: bool,
}

/// 단일 파티션의 브로커 배치 정보.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionTopology {
    pub partition_id: i32,
    pub leader_id: i32,
    pub replica_ids: Vec<i32>,
    pub isr_ids: Vec<i32>,
}

/// 단일 토픽의 파티션 배치 요약.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicTopology {
    pub name: String,
    pub is_internal: bool,
    pub partitions: Vec<PartitionTopology>,
}

/// Kafka 클러스터 토폴로지 — 브로커 목록 + 토픽별 파티션 배치.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterTopology {
    pub cluster_id: Option<String>,
    pub controller_id: i32,
    pub brokers: Vec<BrokerInfo>,
    pub topics: Vec<TopicTopology>,
}

/// ApiVersions(API key 18) 응답의 단일 API 항목 — 브로커가 지원하는 버전 범위.
/// `min`/`max` 둘 다 담는다 — `kafka_cluster_version_strategy.md` 의 BrokerCapabilities
/// [min, max] 모델이 그대로 흡수할 수 있게 (진단 리포트는 주로 max 사용).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiVersionEntry {
    pub api_key: i16,
    /// 알려진 API 키의 이름 (Produce/Fetch/Metadata 등). 미상이면 None.
    pub name: Option<String>,
    pub min_version: i16,
    pub max_version: i16,
}

/// Kafka broker 버전 정보 — ApiVersions(key 18) 기반.
/// - `inferred_version`: marker API max 버전에서 추정한 **대략값**(fuzzy, "≈ 3.x" 식).
/// - `api_versions`: 핵심 API 의 지원 max 버전 맵 — "특정 버전 동작 X" 추적용 **정확값**.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaVersionInfo {
    pub inferred_version: Option<String>,
    pub api_versions: Vec<ApiVersionEntry>,
}
