//! `ProfilesApi` mock impl — 빈 목록 / no-op.

use async_trait::async_trait;
use kaflow_api_traits::engine::ProfilesApi;
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::{ClusterProfile, StoredAuthConfig};

use crate::MockEngine;

#[async_trait]
impl ProfilesApi for MockEngine {
    async fn list_cluster_profiles(&self) -> Result<Vec<ClusterProfile>, EngineError> {
        Ok(Vec::new())
    }

    async fn save_cluster_profile(
        &self,
        _workspace: &str,
        _bootstrap: &str,
        _cluster_id: Option<&str>,
        _auth_config: Option<StoredAuthConfig>,
    ) -> Result<(), EngineError> {
        Ok(())
    }

    async fn delete_cluster_profile(&self, _workspace: &str) -> Result<(), EngineError> {
        Ok(())
    }
}
