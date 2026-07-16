//! Consistency API — 정합성 검증 + drift 감지.

use async_trait::async_trait;
use kaflow_api_types::{ConsistencyReport, FullResyncTrigger};

use crate::error::EngineError;

#[async_trait]
pub trait ConsistencyApi: Send + Sync {
    /// RocksDB scan vs T-META counter 항목별 비교.
    async fn verify_topic_consistency(
        &self,
        workspace: &str,
        topic: &str,
    ) -> Result<ConsistencyReport, EngineError>;

    /// Kafka 측 metadata 변경 감지 (topicId / partitionCount / cleanup.policy / latestRegression).
    async fn detect_topic_drift(
        &self,
        workspace: &str,
        topic: &str,
        bootstrap: &str,
    ) -> Result<Vec<FullResyncTrigger>, EngineError>;
}
