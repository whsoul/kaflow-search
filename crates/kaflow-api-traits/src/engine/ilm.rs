//! Reclaiming space, and following what the cluster no longer holds.
//!
//! ⚠️ **Reserved fields are never candidates.** Whatever is chosen to drop, a field
//! matching [`kaflow_api_types::is_system_field`] must survive it.

use async_trait::async_trait;

use crate::error::EngineError;

#[async_trait]
pub trait IlmApi: Send + Sync {
    /// Drops indexed messages the cluster has since discarded, across every topic.
    async fn trigger_background_ilm(
        &self,
        workspace: &str,
        bootstrap: &str,
    ) -> Result<usize, EngineError>;

    /// Reclaims space. `current_topic` is spared — taking away what is being looked at is
    /// how a user watches results disappear as they read them.
    async fn trigger_size_based_cleanup(
        &self,
        workspace: &str,
        current_topic: Option<&str>,
    ) -> Result<(), EngineError>;

    /// Reclaims space for one topic, at the user's request.
    async fn force_topic_cleanup(&self, workspace: &str, topic: &str) -> Result<(), EngineError>;

    /// Does the same for one topic, once it is known to have dropped messages.
    async fn trigger_topic_retention_cleanup(
        &self,
        workspace: &str,
        topic: &str,
        bootstrap: &str,
    ) -> Result<bool, EngineError>;
}
