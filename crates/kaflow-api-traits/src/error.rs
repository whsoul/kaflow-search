//! Engine 경계 통합 에러 타입.
//!
//! 현재는 카테고리별 stringify variant 만 둔다. private engine 측 thiserror enum 들이
//! 점진적으로 안정되면 더 세밀한 error 변종으로 교체될 예정.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("workspace error: {0}")]
    Workspace(String),

    #[error("kafka error: {0}")]
    Kafka(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("indexing error: {0}")]
    Indexing(String),

    #[error("search error: {0}")]
    Search(String),

    #[error("deserialize error: {0}")]
    Deserialize(String),

    #[error("schema error: {0}")]
    Schema(String),

    /// 기능이 license / capability 로 막혀있는 경우.
    /// `kaflow-feature-gate` (미래) 와 연동되어 raise 된다.
    #[error("feature not entitled: {0}")]
    NotEntitled(String),

    /// 락 관련 — `lock_policy.md` 의 충돌 매트릭스에서 차단된 케이스.
    #[error("lock conflict: {0}")]
    LockConflict(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// 시간버켓 집계 결과가 hard cap(구조적 천장 또는 호출자 cap)을 넘음. 부분 반환 대신 **명시적
    /// 실패**로 정합성을 보존한다(`.claude/CLAUDE.md` "리소스 한도"). 메시지에 sentinel prefix
    /// `"bucket overflow"` + 한도/안내가 담겨 FE 가 토큰 매칭해 범위 축소·단위 확대 안내 + 폴백.
    /// (Display = 메시지 그대로 — prefix 중복 방지.)
    #[error("{0}")]
    BucketOverflow(String),

    /// 사용자가 명시적으로 취소한 작업 (검색/buckets cancel 버튼). 실패가 아니라 정상 흐름이라
    /// FE 는 이 에러를 토스트 없이 조용히 무시하고 pending 상태만 해제한다. stringify =
    /// `"cancelled"` (FE 매칭 토큰). 발화 경로: `search_cancel::CANCELLED_MSG`.
    #[error("cancelled")]
    Cancelled,

    #[error("internal: {0}")]
    Internal(String),
}

impl EngineError {
    /// Tauri command 어댑터에서 `Result<_, String>` 으로 변환 시 사용.
    /// (Tauri IPC 가 stringified error 를 받는 관습)
    pub fn into_string(self) -> String {
        self.to_string()
    }
}

impl From<EngineError> for String {
    fn from(e: EngineError) -> Self {
        e.to_string()
    }
}
