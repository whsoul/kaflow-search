//! Topic / Cluster meta read API — destructive 아님.

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

    /// 클러스터 토폴로지(브로커 목록 + 토픽별 파티션 배치). `topics` 비면 브로커/클러스터
    /// 정보만, 토픽명 주면 해당 토픽 파티션 leader/replica/ISR 도 함께. (맵 화면 정식 기능.)
    async fn fetch_cluster_topology(
        &self,
        bootstrap: &str,
        topics: &[String],
    ) -> Result<ClusterTopology, EngineError>;

    /// 인덱스 대상 선택 시 보여줄 경량 메시지 카운트 (Kafka ListOffsets 만, RocksDB 미접근).
    async fn get_topic_message_count(
        &self,
        bootstrap: &str,
        topic: &str,
    ) -> Result<TopicMessageCount, EngineError>;

    /// 인덱싱 정책 추천 입력값 — 샘플 메시지 raw bytes 평균 + cleanup.policy (RocksDB 미접근).
    async fn get_topic_size_profile(
        &self,
        bootstrap: &str,
        topic: &str,
    ) -> Result<TopicSizeProfile, EngineError>;

    /// 어절(tokenize) 대상 필드 추천 — picker 시점(인덱싱 전) 샘플을 선택한 deserializer 로
    /// decode → 긴 텍스트 필드 랭크. 초기 인덱싱부터 어절 적용을 위한 picker 진입점.
    async fn suggest_tokenize_fields(
        &self,
        bootstrap: &str,
        topic: &str,
        key_spec: DeserializerSpec,
        value_spec: DeserializerSpec,
    ) -> Result<SuggestTokenizeFieldsResponse, EngineError>;

    async fn get_cluster_id(&self, bootstrap: &str) -> Result<String, EngineError>;

    /// 브로커 버전 정보(ApiVersions key 18) — 추정 버전 + 핵심 API 지원맵.
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

    /// 워치 토픽들의 인덱싱 lag(미인덱싱 추정 메시지 수) 배치 조회. 칩 lag 배지용 경량 폴링.
    /// Kafka latest offset(캐시 연결 1개 배치) − RocksDB 인덱싱 offset 으로 partition별 차감.
    async fn list_watched_lag(
        &self,
        workspace: &str,
        bootstrap: &str,
        topics: Vec<String>,
    ) -> Result<Vec<TopicLagStatus>, EngineError>;
}
