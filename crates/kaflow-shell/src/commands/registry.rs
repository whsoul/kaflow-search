//! Schema registry commands. Each one hands straight to the engine.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{
    RegistryResource, RegistrySchemaIndexEntry, RegistrySchemaView, RegistryTestResult,
};
use std::sync::Arc;

#[tauri::command]
pub async fn list_registry_resources(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<Vec<RegistryResource>, String> {
    engine
        .list_registry_resources()
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn save_registry_resource(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    resource: RegistryResource,
) -> Result<(), String> {
    engine
        .save_registry_resource(resource)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn delete_registry_resource(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    id: String,
) -> Result<(), String> {
    engine
        .delete_registry_resource(&id)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn test_registry_resource(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    url: String,
    basic_auth: Option<String>,
) -> Result<RegistryTestResult, String> {
    engine
        .test_registry_resource(&url, basic_auth.as_deref())
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn list_registry_subjects(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    url: String,
    basic_auth: Option<String>,
) -> Result<Vec<String>, String> {
    engine
        .list_registry_subjects(&url, basic_auth.as_deref())
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn fetch_registry_subject_latest(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    url: String,
    basic_auth: Option<String>,
    subject: String,
) -> Result<RegistrySchemaView, String> {
    engine
        .fetch_registry_subject_latest(&url, basic_auth.as_deref(), &subject)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn list_registry_schema_index(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    url: String,
    basic_auth: Option<String>,
) -> Result<Vec<RegistrySchemaIndexEntry>, String> {
    engine
        .list_registry_schema_index(&url, basic_auth.as_deref())
        .await
        .map_err(|e| e.into_string())
}
