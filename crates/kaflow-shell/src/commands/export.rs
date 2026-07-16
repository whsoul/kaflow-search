//! Export Tauri 어댑터 — `Arc<dyn KafkaToolEngine>` 경유. thin shim.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{ExportRequest, ExportResult};
use std::sync::Arc;

/// 현재 검색(browse/keyword)의 전체결과를 파일로 스트리밍 export.
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

/// 진행 중인 export 를 workspace 단위로 취소. 반환 = 취소된 작업 수.
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
