//! Cleanup commands. Each one hands straight to the engine.

use kaflow_api_traits::KafkaToolEngine;
use std::sync::Arc;

#[tauri::command]
pub async fn trigger_background_ilm(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    bootstrap: String,
) -> Result<usize, String> {
    engine
        .trigger_background_ilm(&workspace, &bootstrap)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn trigger_size_based_cleanup(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    current_topic: Option<String>,
) -> Result<(), String> {
    engine
        .trigger_size_based_cleanup(&workspace, current_topic.as_deref())
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn force_topic_cleanup(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
) -> Result<(), String> {
    engine
        .force_topic_cleanup(&workspace, &topic)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn trigger_topic_retention_cleanup(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    bootstrap: String,
) -> Result<bool, String> {
    engine
        .trigger_topic_retention_cleanup(&workspace, &topic, &bootstrap)
        .await
        .map_err(|e| e.into_string())
}
