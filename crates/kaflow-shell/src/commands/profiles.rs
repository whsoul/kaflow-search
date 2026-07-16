//! Cluster profile Tauri 어댑터 — `Arc<dyn KafkaToolEngine>` 경유.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{ClusterProfile, StoredAuthConfig};
use std::sync::Arc;

#[tauri::command]
pub async fn list_cluster_profiles(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<Vec<ClusterProfile>, String> {
    engine
        .list_cluster_profiles()
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn save_cluster_profile(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    bootstrap: String,
    cluster_id: Option<String>,
    auth_config: Option<StoredAuthConfig>,
) -> Result<(), String> {
    engine
        .save_cluster_profile(&workspace, &bootstrap, cluster_id.as_deref(), auth_config)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn delete_cluster_profile(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
) -> Result<(), String> {
    engine
        .delete_cluster_profile(&workspace)
        .await
        .map_err(|e| e.into_string())
}
