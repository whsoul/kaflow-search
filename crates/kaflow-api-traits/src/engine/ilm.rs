//! Index Lifecycle Management API — retention / size-based / 강제 cleanup.

use async_trait::async_trait;

use crate::error::EngineError;

#[async_trait]
pub trait IlmApi: Send + Sync {
    /// 주기 RetentionCleanup 트리거 (모든 토픽 대상). 반환값 = spawn 한 토픽 수.
    async fn trigger_background_ilm(
        &self,
        workspace: &str,
        bootstrap: &str,
    ) -> Result<usize, EngineError>;

    /// 크기 기반 정리 트리거. `current_topic` 은 용량 확보 대상에서 제외.
    async fn trigger_size_based_cleanup(
        &self,
        workspace: &str,
        current_topic: Option<&str>,
    ) -> Result<(), EngineError>;

    /// 사용자 수동 트리거 — 단일 토픽의 cleanup_policy 에 따라 정리 실행.
    async fn force_topic_cleanup(&self, workspace: &str, topic: &str) -> Result<(), EngineError>;

    /// 단일 토픽 retention cleanup 트리거 (earliest offset 변경 감지 후 호출).
    async fn trigger_topic_retention_cleanup(
        &self,
        workspace: &str,
        topic: &str,
        bootstrap: &str,
    ) -> Result<bool, EngineError>;
}
