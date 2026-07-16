//! Storage 도메인 API — workspace / topic 단위 raw I/M/T key 관리.

use async_trait::async_trait;

use crate::error::EngineError;

#[async_trait]
pub trait StorageApi: Send + Sync {
    async fn clear_workspace_index(&self, workspace: &str) -> Result<usize, EngineError>;

    async fn clear_workspace_meta(&self, workspace: &str) -> Result<usize, EngineError>;

    async fn clear_topic_index(&self, workspace: &str, topic: &str) -> Result<usize, EngineError>;

    async fn clear_topic_meta(&self, workspace: &str, topic: &str) -> Result<usize, EngineError>;

    /// 비상용 워크스페이스 초기화 — RocksDB 디렉토리 통째 삭제 + 캐시 drop.
    async fn reset_workspace(&self, workspace: &str) -> Result<(), EngineError>;

    /// I + M + T 키 전체 삭제 (topic 지정 시 해당 토픽만). 진행률 이벤트 emit.
    async fn clear_all(&self, workspace: &str, topic: Option<&str>) -> Result<usize, EngineError>;

    /// M + T 키 삭제 후 I-key 에서 M-key 재구성 (topic 지정 시 해당 토픽만).
    async fn resync_meta(&self, workspace: &str, topic: Option<&str>)
        -> Result<usize, EngineError>;
}
