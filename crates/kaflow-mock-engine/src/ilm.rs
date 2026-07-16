//! `IlmApi` mock impl.

use async_trait::async_trait;
use kaflow_api_traits::engine::IlmApi;
use kaflow_api_traits::error::EngineError;

use crate::MockEngine;

#[async_trait]
impl IlmApi for MockEngine {
    async fn trigger_background_ilm(
        &self,
        _workspace: &str,
        _bootstrap: &str,
    ) -> Result<usize, EngineError> {
        Ok(0)
    }

    async fn trigger_size_based_cleanup(
        &self,
        _workspace: &str,
        _current_topic: Option<&str>,
    ) -> Result<(), EngineError> {
        Ok(())
    }

    async fn force_topic_cleanup(&self, _workspace: &str, _topic: &str) -> Result<(), EngineError> {
        Ok(())
    }

    async fn trigger_topic_retention_cleanup(
        &self,
        _workspace: &str,
        _topic: &str,
        _bootstrap: &str,
    ) -> Result<bool, EngineError> {
        Ok(false)
    }
}
