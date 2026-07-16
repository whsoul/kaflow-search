//! `ExportApi` mock impl — 자기 자신의 검색/본문 fetch(trait 메서드) 를 재사용해 fixture 행을
//! 파일로 내보낸다. **압축 미지원**(mock 은 infra-free: flate2/zstd 미의존) — 포맷 무관 무압축 기록.
//! CI 컴파일 검증 + 데모용. 실 스트리밍/압축/취소는 engine-impl 담당.

use async_trait::async_trait;
use kaflow_api_traits::engine::{ExportApi, SearchApi};
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::{export_header_line, serialize_export_row, ExportRequest, ExportResult};

use crate::MockEngine;

#[async_trait]
impl ExportApi for MockEngine {
    async fn export_search_results(
        &self,
        workspace: &str,
        req: ExportRequest,
    ) -> Result<ExportResult, EngineError> {
        // loc 전량(limit=None) — 자기 SearchApi 재사용(단일 토픽). fixture 가 작아 한 번에.
        // multi(searchQuery) → browse(빈 쿼리) → keyword. engine-impl 3모드 분기와 대응.
        let topics = [req.topic.clone()];
        let locs = if let Some(sq) = req.search_query.clone() {
            self.multi_search_page(workspace, sq, None, None, None, usize::MAX)
                .await?
        } else if req.query.trim().is_empty() {
            self.prefetch_browse_locs(
                workspace,
                &req.topic,
                req.sort_order.clone(),
                None,
                req.ts_range.clone(),
                req.pos_filter.clone(),
            )
            .await?
            .locs
        } else {
            self.prefetch_search_locs(
                workspace,
                &req.query,
                req.fields.as_deref(),
                Some(&topics),
                req.mode.clone(),
                req.sort_order.clone(),
                None,
                req.ts_range.clone(),
                req.pos_filter.clone(),
            )
            .await?
            .locs
        };
        // 본문 = M-key 재조립(RocksDB only). engine-impl 과 동일 정책(오프라인 동작).
        let bodies = self.fetch_meta_rows(workspace, &req.topic, locs).await?;

        let mut out = String::new();
        if let Some(header) = export_header_line(req.format) {
            out.push_str(&header);
        }
        for row in &bodies {
            out.push_str(&serialize_export_row(row, req.format));
        }
        std::fs::write(&req.dest_path, out.as_bytes())
            .map_err(|e| EngineError::Internal(format!("mock export write failed: {e}")))?;

        Ok(ExportResult {
            rows: bodies.len(),
            bytes: out.len() as u64,
            path: req.dest_path,
        })
    }

    async fn cancel_export(&self, _workspace: &str) -> Result<u32, EngineError> {
        Ok(0)
    }
}
