//! Kafka 인증 Tauri 어댑터 — `Arc<dyn KafkaToolEngine>` 경유.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{AwsPaths, AwsProfileCredentials, AwsProfileSummary, KafkaAuth};
use std::sync::Arc;

#[tauri::command]
pub async fn register_kafka_auth(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    auth: KafkaAuth,
) -> Result<(), String> {
    engine
        .register_kafka_auth(&bootstrap, auth)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn clear_kafka_auth(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
) -> Result<(), String> {
    engine
        .clear_kafka_auth(&bootstrap)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn verify_kafka_auth(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
) -> Result<(), String> {
    engine
        .verify_kafka_auth(&bootstrap)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn list_sasl_mechanisms(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    auth: KafkaAuth,
) -> Result<Vec<String>, String> {
    engine
        .list_sasl_mechanisms(&bootstrap, auth)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn list_aws_profiles(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    path: Option<String>,
) -> Result<Vec<AwsProfileSummary>, String> {
    engine
        .list_aws_profiles(path)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn resolve_aws_paths(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<AwsPaths, String> {
    Ok(engine.resolve_aws_paths().await)
}

#[tauri::command]
pub async fn load_aws_profile(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    path: Option<String>,
    profile: String,
) -> Result<AwsProfileCredentials, String> {
    engine
        .load_aws_profile(path, profile)
        .await
        .map_err(|e| e.into_string())
}
