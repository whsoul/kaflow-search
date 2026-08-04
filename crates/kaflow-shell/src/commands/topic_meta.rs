//! Topic / Cluster meta read Tauri 어댑터 — 모든 호출은 `Arc<dyn KafkaToolEngine>` 경유.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{
    ClusterTopology, DeserializerSpec, KafkaVersionInfo, SuggestTokenizeFieldsResponse,
    TopicConfigInfoResponse, TopicFieldMeta, TopicIndexSize, TopicLagStatus, TopicMessageCount,
    TopicOffsetStatus, TopicSizeProfile, WorkspaceStorageStatus,
};
use std::sync::Arc;

#[tauri::command]
pub async fn list_topic_fields(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
) -> Result<Vec<TopicFieldMeta>, String> {
    engine
        .list_topic_metas(&workspace)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn get_topic_index_sizes(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topics: Vec<String>,
) -> Result<Vec<TopicIndexSize>, String> {
    engine
        .get_topic_index_sizes(&workspace, &topics)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn check_workspace_storage(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
) -> Result<WorkspaceStorageStatus, String> {
    engine
        .check_workspace_storage(&workspace)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn list_kafka_topics(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
) -> Result<Vec<String>, String> {
    engine
        .list_kafka_topics(&bootstrap)
        .await
        .map_err(|e| e.into_string())
}

/// 클러스터 토폴로지(브로커 + 토픽 파티션 배치) — 맵 화면 정식 기능(prod 빌드 포함).
#[tauri::command]
pub async fn fetch_cluster_topology(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    topics: Vec<String>,
) -> Result<ClusterTopology, String> {
    engine
        .fetch_cluster_topology(&bootstrap, &topics)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn get_topic_message_count(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    topic: String,
) -> Result<TopicMessageCount, String> {
    engine
        .get_topic_message_count(&bootstrap, &topic)
        .await
        .map_err(|e| e.into_string())
}

/// 여러 토픽 메시지 카운트 배치 — picker 오픈 시 전 토픽 일괄 (연결/메타데이터 1회).
#[tauri::command]
pub async fn list_topic_message_counts(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    topics: Vec<String>,
) -> Result<Vec<TopicMessageCount>, String> {
    engine
        .list_topic_message_counts(&bootstrap, &topics)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn get_topic_size_profile(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    topic: String,
) -> Result<TopicSizeProfile, String> {
    engine
        .get_topic_size_profile(&bootstrap, &topic)
        .await
        .map_err(|e| e.into_string())
}

/// 어절(tokenize) 대상 필드 추천 — picker 시점(인덱싱 전) 샘플 기반.
#[tauri::command]
pub async fn suggest_tokenize_fields(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    topic: String,
    key_spec: DeserializerSpec,
    value_spec: DeserializerSpec,
) -> Result<SuggestTokenizeFieldsResponse, String> {
    engine
        .suggest_tokenize_fields(&bootstrap, &topic, key_spec, value_spec)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn get_cluster_id(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
) -> Result<String, String> {
    engine
        .get_cluster_id(&bootstrap)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn get_kafka_version_info(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
) -> Result<KafkaVersionInfo, String> {
    engine
        .get_kafka_version_info(&bootstrap)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn save_workspace_cluster_id(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    cluster_id: String,
) -> Result<(), String> {
    engine
        .save_workspace_cluster_id(&workspace, &cluster_id)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn get_workspace_cluster_id(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
) -> Result<Option<String>, String> {
    engine
        .get_workspace_cluster_id(&workspace)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn get_topic_offset_status(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    bootstrap: String,
) -> Result<TopicOffsetStatus, String> {
    engine
        .get_topic_offset_status(&workspace, &topic, &bootstrap)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn list_watched_lag(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    bootstrap: String,
    topics: Vec<String>,
) -> Result<Vec<TopicLagStatus>, String> {
    engine
        .list_watched_lag(&workspace, &bootstrap, topics)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn get_topic_config_info(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    bootstrap: String,
    topic: String,
) -> Result<TopicConfigInfoResponse, String> {
    engine
        .get_topic_config_info(&workspace, &bootstrap, &topic)
        .await
        .map_err(|e| e.into_string())
}
