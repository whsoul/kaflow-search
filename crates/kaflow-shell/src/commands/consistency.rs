//! Consistency Tauri 어댑터 — `Arc<dyn KafkaToolEngine>` 경유.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{ConsistencyReport, FullResyncTrigger};
use std::sync::Arc;

#[tauri::command]
pub async fn verify_topic_consistency(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
) -> Result<ConsistencyReport, String> {
    engine
        .verify_topic_consistency(&workspace, &topic)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn detect_topic_drift(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    bootstrap: String,
) -> Result<Vec<FullResyncTrigger>, String> {
    engine
        .detect_topic_drift(&workspace, &topic, &bootstrap)
        .await
        .map_err(|e| e.into_string())
}
