//! Config 도메인 API — 글로벌 / 토픽 단위 설정 read / write.

use async_trait::async_trait;
use kaflow_api_types::settings::{EffectiveLimitsView, ProfileLimits, SystemLimitsView};
use kaflow_api_types::{GlobalConfigInput, GlobalConfigView, TopicMetaConfigInput};

use crate::error::EngineError;

#[async_trait]
pub trait ConfigApi: Send + Sync {
    /// 현재 글로벌 ILM 설정 view.
    async fn get_global_ilm_config(&self) -> Result<GlobalConfigView, EngineError>;

    /// 운영자 profile 의 한도 제약 적용 (FE 가 profile fetch 직후 호출).
    ///
    /// 설정 3층 모델(`docs/settings_layers_design.md`)의 최상위 층. 엔진은 **받은 값을 신뢰하지
    /// 않는다** — 모르는 키 무시 / 허용 안 된 type 무시 / hard bounds clamp 후 적용한다.
    async fn apply_profile_limits(
        &self,
        limits: ProfileLimits,
    ) -> Result<EffectiveLimitsView, EngineError>;

    /// 3층 해석 결과(플랜 한도 + 디스크 한도 실효값 + FE 입력 범위 + debug 여부).
    /// FE 의 `resourceLimits.ts` / `searchLimits.ts` / `tokenizeLimits.ts` 하드코딩을 대체한다.
    async fn get_effective_limits(&self) -> Result<EffectiveLimitsView, EngineError>;

    /// 유형1 시스템 한도 — **읽기 전용 진단 목록** (설정 화면 debug 섹션).
    /// 편집을 노출하지 않는 이유 = 핫패스의 `const` 라 값만 바꿔서는 동작이 따라오지 않는다.
    async fn get_system_limits(&self) -> Result<SystemLimitsView, EngineError>;

    /// 글로벌 ILM 설정 갱신. 반환값은 적용 후 view.
    async fn set_global_ilm_config(
        &self,
        input: GlobalConfigInput,
    ) -> Result<GlobalConfigView, EngineError>;

    /// 토픽 단위 T-META 사용자 설정 갱신.
    async fn set_topic_meta_config(
        &self,
        workspace: &str,
        topic: &str,
        input: TopicMetaConfigInput,
    ) -> Result<(), EngineError>;

    /// 토픽 cleanup_policy 단독 설정 (None = 정책 unset).
    async fn set_topic_cleanup_policy(
        &self,
        workspace: &str,
        topic: &str,
        policy: Option<&str>,
    ) -> Result<(), EngineError>;
}
