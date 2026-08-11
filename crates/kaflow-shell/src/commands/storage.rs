//! Storage commands.

use kaflow_api_traits::KafkaToolEngine;
use std::sync::Arc;

/// Total and free space on the volume the index lives on.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpace {
    pub total_bytes: u64,
    pub free_bytes: u64,
}

/// Reads the local disk directly rather than through the engine: what is on this machine
/// is a desktop concern, and this has to answer when nothing is reachable.
///
/// Measured at the workspace, falling back outward when it does not exist yet.
#[tauri::command]
pub async fn get_disk_space(workspace: String) -> Result<DiskSpace, String> {
    let base = crate::app_paths::app_data_dir()?;
    let ws_dir = crate::app_paths::workspace_dir(&workspace)?;
    let path = if ws_dir.exists() {
        ws_dir
    } else if base.exists() {
        base
    } else {
        // With neither present, the home volume is the honest answer.
        base.parent()
            .map(std::path::Path::to_path_buf)
            .ok_or_else(|| "home directory not found".to_string())?
    };
    let total_bytes =
        fs2::total_space(&path).map_err(|e| format!("disk total query failed: {e}"))?;
    let free_bytes =
        fs2::available_space(&path).map_err(|e| format!("disk free query failed: {e}"))?;
    Ok(DiskSpace {
        total_bytes,
        free_bytes,
    })
}

#[tauri::command]
pub async fn clear_workspace_index(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
) -> Result<usize, String> {
    engine
        .clear_workspace_index(&workspace)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn clear_workspace_meta(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
) -> Result<usize, String> {
    engine
        .clear_workspace_meta(&workspace)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn clear_topic_index(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
) -> Result<usize, String> {
    engine
        .clear_topic_index(&workspace, &topic)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn clear_topic_meta(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
) -> Result<usize, String> {
    engine
        .clear_topic_meta(&workspace, &topic)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn reset_workspace(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
) -> Result<(), String> {
    engine
        .reset_workspace(&workspace)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn clear_all(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: Option<String>,
) -> Result<usize, String> {
    engine
        .clear_all(&workspace, topic.as_deref())
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn resync_meta(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: Option<String>,
) -> Result<usize, String> {
    engine
        .resync_meta(&workspace, topic.as_deref())
        .await
        .map_err(|e| e.into_string())
}
