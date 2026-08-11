//! Indexing commands. Each one hands straight to the engine.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{
    CompactDeletedKeyCursor, CompactDeletedKeysPage, CompactKeyHistory, DecodeFailureKind,
    DecodeFailureMode, OpenKafkaTopicResponse,
};
use std::sync::Arc;

#[tauri::command]
pub async fn open_kafka_topic(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    bootstrap: String,
    skip_resync_check: Option<bool>,
    decode_failure_mode: Option<DecodeFailureMode>,
    max_messages: Option<u64>,
) -> Result<OpenKafkaTopicResponse, String> {
    engine
        .open_kafka_topic(
            &workspace,
            &topic,
            &bootstrap,
            skip_resync_check,
            decode_failure_mode,
            max_messages,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn cancel_indexing(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
) -> Result<(), String> {
    engine
        .cancel_indexing(&workspace, &topic)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn record_placeholder(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    partition: i32,
    offset: i64,
    kind: DecodeFailureKind,
    reason: String,
) -> Result<(), String> {
    engine
        .record_placeholder(&workspace, &topic, partition, offset, kind, reason)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn fetch_compact_key_history(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    partition: u32,
    key_raw: String,
) -> Result<Option<CompactKeyHistory>, String> {
    engine
        .fetch_compact_key_history(&workspace, &topic, partition, &key_raw)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn fetch_compact_deleted_keys_page(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    key_query: String,
    partitions: Option<Vec<u32>>,
    cursor: Option<CompactDeletedKeyCursor>,
    limit: Option<u32>,
) -> Result<CompactDeletedKeysPage, String> {
    engine
        .fetch_compact_deleted_keys_page(&workspace, &topic, &key_query, partitions, cursor, limit)
        .await
        .map_err(|e| e.into_string())
}
