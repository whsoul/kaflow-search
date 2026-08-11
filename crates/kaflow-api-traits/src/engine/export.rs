//! Writing search results out to a file.

use async_trait::async_trait;
use kaflow_api_types::{ExportRequest, ExportResult};

use crate::error::EngineError;

#[async_trait]
pub trait ExportApi: Send + Sync {
    /// Writes every result of a search to a file.
    ///
    /// ⚠️ **An export must not be bounded by what fits in memory.** The file grows as it is
    /// written rather than being assembled first — that is the whole point, since the
    /// results worth exporting are the ones too large to look through. Progress is emitted
    /// while it runs.
    async fn export_search_results(
        &self,
        workspace: &str,
        req: ExportRequest,
    ) -> Result<ExportResult, EngineError>;

    /// Cancels any export in progress, returning how many were signalled. Zero is not an
    /// error. A cancelled export fails with `EngineError::Cancelled`.
    async fn cancel_export(&self, workspace: &str) -> Result<u32, EngineError>;
}
