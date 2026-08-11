//! Export commands. Each one hands straight to the engine.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{ExportRequest, ExportResult};
use std::sync::Arc;

/// Writes every result of the current search to a file.
#[tauri::command]
pub async fn export_search_results(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    req: ExportRequest,
) -> Result<ExportResult, String> {
    engine
        .export_search_results(&workspace, req)
        .await
        .map_err(|e| e.into_string())
}

/// Cancels any export in progress, returning how many were signalled.
#[tauri::command]
pub async fn cancel_export(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
) -> Result<u32, String> {
    engine
        .cancel_export(&workspace)
        .await
        .map_err(|e| e.into_string())
}
