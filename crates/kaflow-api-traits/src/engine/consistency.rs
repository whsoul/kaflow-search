//! Checking that the index still agrees with itself and with the topic.

use async_trait::async_trait;
use kaflow_api_types::{ConsistencyReport, FullResyncTrigger};

use crate::error::EngineError;

#[async_trait]
pub trait ConsistencyApi: Send + Sync {
    /// Compares what is stored against what the counters claim.
    async fn verify_topic_consistency(
        &self,
        workspace: &str,
        topic: &str,
    ) -> Result<ConsistencyReport, EngineError>;

    /// Looks for changes on the cluster that mean the index no longer describes the same
    /// topic.
    async fn detect_topic_drift(
        &self,
        workspace: &str,
        topic: &str,
        bootstrap: &str,
    ) -> Result<Vec<FullResyncTrigger>, EngineError>;
}
