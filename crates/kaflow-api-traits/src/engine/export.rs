//! Export 도메인 API — 검색결과(browse/keyword) 파일 스트리밍 내보내기.
//!
//! v1 = browse+keyword 단일 토픽(현재 검색과 동일 파라미터). multi(상세검색)는 별도 생산자라
//! named 후속. 설계 = `docs/pre_launch_specs.md §2`.

use async_trait::async_trait;
use kaflow_api_types::{ExportRequest, ExportResult};

use crate::error::EngineError;

#[async_trait]
pub trait ExportApi: Send + Sync {
    /// 현재 검색(browse/keyword)의 전체결과를 파일로 **스트리밍** export.
    /// loc 은 head 와 동일 생산자로 전량 뽑고(최대 내부 cap), 본문은 배치 단위로 fetch→파일 append
    /// (전량 미적재, 파일이 자라며 기록). 진행은 `events::EXPORT_PROGRESS` 로 emit, 취소는 `cancel_export`.
    async fn export_search_results(
        &self,
        workspace: &str,
        req: ExportRequest,
    ) -> Result<ExportResult, EngineError>;

    /// 진행 중인 export 를 workspace 단위로 취소. 반환 = 실제 취소된 작업 수(idempotent).
    /// 취소된 작업은 `EngineError::Cancelled` 로 반환되며 FE 는 조용히 무시한다.
    async fn cancel_export(&self, workspace: &str) -> Result<u32, EngineError>;
}
