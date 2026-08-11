//! Settings commands. Each one hands straight to the engine.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::settings::{EffectiveLimitsView, ProfileLimits, SystemLimitsView};
use kaflow_api_types::{GlobalConfigInput, GlobalConfigView, TopicMetaConfigInput};
use std::sync::Arc;

#[tauri::command]
pub async fn get_global_ilm_config(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<GlobalConfigView, String> {
    engine
        .get_global_ilm_config()
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn set_global_ilm_config(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    input: GlobalConfigInput,
) -> Result<GlobalConfigView, String> {
    engine
        .set_global_ilm_config(input)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn set_topic_meta_config(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    input: TopicMetaConfigInput,
) -> Result<(), String> {
    engine
        .set_topic_meta_config(&workspace, &topic, input)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn set_topic_cleanup_policy(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    policy: Option<String>,
) -> Result<(), String> {
    engine
        .set_topic_cleanup_policy(&workspace, &topic, policy.as_deref())
        .await
        .map_err(|e| e.into_string())
}

/// Applies an operator profile's constraints.
///
/// The values have to reach the engine because that is where they are enforced — a limit
/// applied only in the interface is a suggestion.
#[tauri::command]
pub async fn apply_profile_limits(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    limits: ProfileLimits,
) -> Result<EffectiveLimitsView, String> {
    engine
        .apply_profile_limits(limits)
        .await
        .map_err(|e| e.into_string())
}

/// The limits actually in force. Callers read these instead of carrying their own.
#[tauri::command]
pub async fn get_effective_limits(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<EffectiveLimitsView, String> {
    engine
        .get_effective_limits()
        .await
        .map_err(|e| e.into_string())
}

/// System limits, for diagnosis. Read-only.
#[tauri::command]
pub async fn get_system_limits(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<SystemLimitsView, String> {
    engine
        .get_system_limits()
        .await
        .map_err(|e| e.into_string())
}
