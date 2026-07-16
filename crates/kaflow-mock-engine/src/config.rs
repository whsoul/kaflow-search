//! `ConfigApi` mock impl — 기본값 view 만 반환.

use async_trait::async_trait;
use kaflow_api_traits::engine::ConfigApi;
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::settings::{EffectiveLimitsView, ProfileLimits, SystemLimitsView};
use kaflow_api_types::{GlobalConfigInput, GlobalConfigView, TopicMetaConfigInput};

use crate::MockEngine;

fn default_view() -> GlobalConfigView {
    GlobalConfigView {
        indexing_batch_size: 1000,
        auto_sync_slice_size: 50_000,
        incremental_sync_enabled: true,
        retention_cleanup_enabled: true,
        retention_cleanup_interval_secs: 60,
        size_cleanup_enabled: false,
        size_cleanup_interval_secs: 60,
        max_index_bytes_per_cluster: 0,
        default_cleanup_policies: vec!["drop_index".to_string()],
        default_keep_count_per_partition: 50_000,
        topic_index_count_cap: kaflow_api_types::TOPIC_INDEX_COUNT_CAP,
        retention_cleanup_wait_timeout_secs: 30,
        size_cleanup_wait_timeout_secs: 30,
        foreground_acquire_wait_timeout_secs: 30,
        compression_mode: "snappy".to_string(),
    }
}

#[async_trait]
impl ConfigApi for MockEngine {
    /// mock 도 **같은 해석기**를 태운다 — 층 모델이 mock 에서만 다르게 보이면 데모가 거짓이 된다.
    async fn apply_profile_limits(
        &self,
        limits: ProfileLimits,
    ) -> Result<EffectiveLimitsView, EngineError> {
        Ok(EffectiveLimitsView::resolve(
            kaflow_api_types::settings::DISK_LIMIT_BYTES.default,
            &limits,
            false,
        ))
    }

    async fn get_effective_limits(&self) -> Result<EffectiveLimitsView, EngineError> {
        Ok(EffectiveLimitsView::resolve(
            kaflow_api_types::settings::DISK_LIMIT_BYTES.default,
            &Default::default(),
            false,
        ))
    }

    /// mock 은 엔진 내부 캡을 갖지 않는다 — 빈 목록(화면은 "표시할 항목 없음").
    async fn get_system_limits(&self) -> Result<SystemLimitsView, EngineError> {
        Ok(SystemLimitsView::default())
    }

    async fn get_global_ilm_config(&self) -> Result<GlobalConfigView, EngineError> {
        Ok(default_view())
    }

    async fn set_global_ilm_config(
        &self,
        _input: GlobalConfigInput,
    ) -> Result<GlobalConfigView, EngineError> {
        Ok(default_view())
    }

    async fn set_topic_meta_config(
        &self,
        _workspace: &str,
        _topic: &str,
        _input: TopicMetaConfigInput,
    ) -> Result<(), EngineError> {
        Ok(())
    }

    async fn set_topic_cleanup_policy(
        &self,
        _workspace: &str,
        _topic: &str,
        _policy: Option<&str>,
    ) -> Result<(), EngineError> {
        Ok(())
    }
}
