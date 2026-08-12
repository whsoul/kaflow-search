//! Reading what a topic and its cluster look like. Nothing here changes anything.

use async_trait::async_trait;
use kaflow_api_types::{
    ClusterTopology, DeserializerSpec, KafkaVersionInfo, SuggestTokenizeFieldsResponse,
    TopicConfigInfoResponse, TopicFieldMeta, TopicIndexSize, TopicLagStatus, TopicMessageCount,
    TopicOffsetStatus, TopicSizeProfile, WorkspaceStorageStatus,
};

use crate::error::EngineError;

#[async_trait]
pub trait TopicMetaApi: Send + Sync {
    async fn list_topic_metas(&self, workspace: &str) -> Result<Vec<TopicFieldMeta>, EngineError>;

    async fn get_topic_meta(
        &self,
        workspace: &str,
        topic: &str,
    ) -> Result<Option<TopicFieldMeta>, EngineError>;

    async fn get_topic_index_sizes(
        &self,
        workspace: &str,
        topics: &[String],
    ) -> Result<Vec<TopicIndexSize>, EngineError>;

    async fn check_workspace_storage(
        &self,
        workspace: &str,
    ) -> Result<WorkspaceStorageStatus, EngineError>;

    async fn list_kafka_topics(&self, bootstrap: &str) -> Result<Vec<String>, EngineError>;

    /// The cluster's shape. With no topics named, only the brokers; name some and their
    /// partitions come too.
    async fn fetch_cluster_topology(
        &self,
        bootstrap: &str,
        topics: &[String],
    ) -> Result<ClusterTopology, EngineError>;

    /// Roughly how many messages a topic holds, from offsets alone — nothing is read and
    /// nothing is created locally.
    async fn get_topic_message_count(
        &self,
        bootstrap: &str,
        topic: &str,
    ) -> Result<TopicMessageCount, EngineError>;

    /// The same for many topics at once.
    ///
    /// ⚠️ **Partial success is normal**: a topic the cluster does not report is left out of
    /// the result rather than failing the call.
    ///
    /// Counts **must be current as of the call** — a caller uses them to decide what to
    /// index, and a remembered number would send it after a topic that has since gone.
    async fn list_topic_message_counts(
        &self,
        bootstrap: &str,
        topics: &[String],
    ) -> Result<Vec<TopicMessageCount>, EngineError>;

    /// A sample of a topic, enough to suggest how it should be indexed.
    async fn get_topic_size_profile(
        &self,
        bootstrap: &str,
        topic: &str,
    ) -> Result<TopicSizeProfile, EngineError>;

    /// Which fields look worth splitting into words, judged from a sample before indexing
    /// begins — so the choice can be made once rather than corrected later.
    async fn suggest_tokenize_fields(
        &self,
        bootstrap: &str,
        topic: &str,
        key_spec: DeserializerSpec,
        value_spec: DeserializerSpec,
    ) -> Result<SuggestTokenizeFieldsResponse, EngineError>;

    async fn get_cluster_id(&self, bootstrap: &str) -> Result<String, EngineError>;

    /// What version the broker appears to be, and what it actually supports.
    async fn get_kafka_version_info(
        &self,
        bootstrap: &str,
    ) -> Result<KafkaVersionInfo, EngineError>;

    async fn save_workspace_cluster_id(
        &self,
        workspace: &str,
        cluster_id: &str,
    ) -> Result<(), EngineError>;

    async fn get_workspace_cluster_id(
        &self,
        workspace: &str,
    ) -> Result<Option<String>, EngineError>;

    async fn get_topic_offset_status(
        &self,
        workspace: &str,
        topic: &str,
        bootstrap: &str,
    ) -> Result<TopicOffsetStatus, EngineError>;

    async fn get_topic_config_info(
        &self,
        workspace: &str,
        bootstrap: &str,
        topic: &str,
    ) -> Result<TopicConfigInfoResponse, EngineError>;

    /// How far behind each followed topic's index is.
    ///
    /// **Meant to be called repeatedly**, so it has to stay cheap enough that polling it
    /// costs nothing worth noticing.
    async fn list_watched_lag(
        &self,
        workspace: &str,
        bootstrap: &str,
        topics: Vec<String>,
    ) -> Result<Vec<TopicLagStatus>, EngineError>;
}
