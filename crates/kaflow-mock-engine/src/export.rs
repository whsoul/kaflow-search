//! Writing results to a file, by searching itself and formatting what comes back.
//!
//! **Compression is not offered** — providing it would mean taking on a dependency, which
//! this crate does not do. Files are written uncompressed whatever was asked for.

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
        // Everything at once: the fixture is small enough that streaming would be
        // ceremony. Which search to run follows from what was given.
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
        // Bodies come from what is stored, so exporting works with nothing reachable.
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
