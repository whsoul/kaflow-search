use serde::{Deserialize, Serialize};

/// One broker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerInfo {
    pub node_id: i32,
    pub host: String,
    pub port: i32,
    pub rack: Option<String>,
    pub is_controller: bool,
}

/// Where one partition lives.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionTopology {
    pub partition_id: i32,
    pub leader_id: i32,
    pub replica_ids: Vec<i32>,
    pub isr_ids: Vec<i32>,
}

/// How one topic's partitions are spread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicTopology {
    pub name: String,
    pub is_internal: bool,
    pub partitions: Vec<PartitionTopology>,
}

/// The shape of a cluster: its brokers, and where each topic's partitions sit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterTopology {
    pub cluster_id: Option<String>,
    pub controller_id: i32,
    pub brokers: Vec<BrokerInfo>,
    pub topics: Vec<TopicTopology>,
}

/// One API the broker supports, and the versions of it that it accepts.
///
/// Both ends are kept: deciding what a cluster can do needs the range, not just the
/// newest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiVersionEntry {
    pub api_key: i16,
    /// The API's name where it is known.
    pub name: Option<String>,
    pub min_version: i16,
    pub max_version: i16,
}

/// What version a broker appears to be.
///
/// ⚠️ `inferred_version` is a guess for reading, not for deciding by — behaviour must be
/// chosen from `api_versions`, which is what the broker actually said.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaVersionInfo {
    pub inferred_version: Option<String>,
    pub api_versions: Vec<ApiVersionEntry>,
}
