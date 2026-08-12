//! Consistency checks that always pass, since nothing can drift.

use async_trait::async_trait;
use kaflow_api_traits::engine::ConsistencyApi;
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::{ConsistencyReport, FullResyncTrigger};

use crate::MockEngine;

#[async_trait]
impl ConsistencyApi for MockEngine {
    async fn verify_topic_consistency(
        &self,
        _workspace: &str,
        topic: &str,
    ) -> Result<ConsistencyReport, EngineError> {
        Ok(ConsistencyReport {
            topic: topic.to_string(),
            overall_ok: true,
            index_state: "Empty".to_string(),
            checks: Vec::new(),
        })
    }

    async fn detect_topic_drift(
        &self,
        _workspace: &str,
        _topic: &str,
        _bootstrap: &str,
    ) -> Result<Vec<FullResyncTrigger>, EngineError> {
        Ok(Vec::new())
    }
}
