//! Config Tauri 어댑터 — `Arc<dyn KafkaToolEngine>` 경유.

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

/// 운영자 profile 의 한도 제약 적용 — FE 가 profile 을 fetch 한 직후 호출한다.
/// (강제는 BE 가 하므로 값이 BE 로 와야 한다. 엔진이 받은 즉시 검증·clamp 한다.)
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

/// 3층 해석 결과 — FE 의 하드코딩 상수(resourceLimits/searchLimits/tokenizeLimits)를 대체.
#[tauri::command]
pub async fn get_effective_limits(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<EffectiveLimitsView, String> {
    engine
        .get_effective_limits()
        .await
        .map_err(|e| e.into_string())
}

/// 유형1 시스템 한도 — 읽기 전용 진단 목록 (설정 화면 debug 섹션).
#[tauri::command]
pub async fn get_system_limits(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<SystemLimitsView, String> {
    engine
        .get_system_limits()
        .await
        .map_err(|e| e.into_string())
}
