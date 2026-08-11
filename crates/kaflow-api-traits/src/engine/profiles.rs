//! Saved cluster connections.

use async_trait::async_trait;
use kaflow_api_types::{ClusterProfile, StoredAuthConfig};

use crate::error::EngineError;

#[async_trait]
pub trait ProfilesApi: Send + Sync {
    /// Saved connections, most recently used first.
    async fn list_cluster_profiles(&self) -> Result<Vec<ClusterProfile>, EngineError>;

    /// Saves or updates one.
    ///
    /// ⚠️ **Past the limit this must fail, and nothing may be removed to make room.**
    /// A saved connection disappearing on its own is indistinguishable from losing it, so
    /// the refusal is deliberate — evicting the oldest would be the friendlier-looking
    /// behaviour and the wrong one.
    ///
    /// `auth_config` carries no secrets — see `StoredAuthConfig`.
    async fn save_cluster_profile(
        &self,
        workspace: &str,
        bootstrap: &str,
        cluster_id: Option<&str>,
        auth_config: Option<StoredAuthConfig>,
    ) -> Result<(), EngineError>;

    /// Removes one.
    async fn delete_cluster_profile(&self, workspace: &str) -> Result<(), EngineError>;
}
