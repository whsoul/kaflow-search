//! Clearing and rebuilding what is stored.

use async_trait::async_trait;

use crate::error::EngineError;

#[async_trait]
pub trait StorageApi: Send + Sync {
    async fn clear_workspace_index(&self, workspace: &str) -> Result<usize, EngineError>;

    async fn clear_workspace_meta(&self, workspace: &str) -> Result<usize, EngineError>;

    async fn clear_topic_index(&self, workspace: &str, topic: &str) -> Result<usize, EngineError>;

    async fn clear_topic_meta(&self, workspace: &str, topic: &str) -> Result<usize, EngineError>;

    /// Discards a workspace entirely. A last resort — nothing survives it.
    async fn reset_workspace(&self, workspace: &str) -> Result<(), EngineError>;

    /// Clears the index, for one topic or for all of them.
    async fn clear_all(&self, workspace: &str, topic: Option<&str>) -> Result<usize, EngineError>;

    /// Rebuilds message records from the index alone, without reading the topic again.
    async fn resync_meta(&self, workspace: &str, topic: Option<&str>)
        -> Result<usize, EngineError>;
}
