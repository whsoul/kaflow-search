//! Browse Tauri 어댑터 — `SearchApi::fetch_kafka_messages_raw` 호출.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::SearchIndexResponse;
use std::sync::Arc;

#[tauri::command]
pub async fn fetch_kafka_messages(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    limit: Option<usize>,
) -> Result<SearchIndexResponse, String> {
    engine
        .fetch_kafka_messages_raw(&workspace, &topic, limit)
        .await
        .map_err(|e| e.into_string())
}
