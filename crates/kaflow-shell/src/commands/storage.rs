//! Storage Tauri 어댑터 — `Arc<dyn KafkaToolEngine>` 경유.

use kaflow_api_traits::KafkaToolEngine;
use std::sync::Arc;

/// 인덱스가 저장되는 볼륨의 시스템 디스크 총량/여유 (바이트).
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpace {
    pub total_bytes: u64,
    pub free_bytes: u64,
}

/// 로컬 머신 디스크 총량/여유 조회. **엔진 trait 을 거치지 않는다** — 로컬 디스크는
/// desktop shell 관심사라 공개 엔진 계약(`KafkaToolEngine`)에 넣지 않는다. offline 에서도 동작.
/// 인덱스 볼륨 기준(`~/.kaflow/<workspace>`); 아직 없으면 베이스→홈 순 폴백.
/// 경로는 `crate::app_paths` 단일 출처 (B-1 seam).
#[tauri::command]
pub async fn get_disk_space(workspace: String) -> Result<DiskSpace, String> {
    let base = crate::app_paths::app_data_dir()?;
    let ws_dir = crate::app_paths::workspace_dir(&workspace)?;
    let path = if ws_dir.exists() {
        ws_dir
    } else if base.exists() {
        base
    } else {
        // 둘 다 없으면 홈 볼륨 기준 — base 의 부모가 곧 홈.
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
