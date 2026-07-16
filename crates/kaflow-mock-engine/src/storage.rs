//! `StorageApi` mock impl — 모든 mutation 은 no-op + dummy 카운터 반환.

use async_trait::async_trait;
use kaflow_api_traits::engine::StorageApi;
use kaflow_api_traits::error::EngineError;

use crate::MockEngine;

#[async_trait]
impl StorageApi for MockEngine {
    async fn clear_workspace_index(&self, _workspace: &str) -> Result<usize, EngineError> {
        Ok(0)
    }

    async fn clear_workspace_meta(&self, _workspace: &str) -> Result<usize, EngineError> {
        Ok(0)
    }

    async fn clear_topic_index(
        &self,
        _workspace: &str,
        _topic: &str,
    ) -> Result<usize, EngineError> {
        Ok(0)
    }

    async fn clear_topic_meta(&self, _workspace: &str, _topic: &str) -> Result<usize, EngineError> {
        Ok(0)
    }

    async fn reset_workspace(&self, _workspace: &str) -> Result<(), EngineError> {
        Ok(())
    }

    async fn clear_all(
        &self,
        _workspace: &str,
        _topic: Option<&str>,
    ) -> Result<usize, EngineError> {
        Ok(0)
    }

    async fn resync_meta(
        &self,
        _workspace: &str,
        _topic: Option<&str>,
    ) -> Result<usize, EngineError> {
        Ok(0)
    }
}
