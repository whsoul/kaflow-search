//! Reading and changing settings, globally and per topic.

use async_trait::async_trait;
use kaflow_api_types::settings::{EffectiveLimitsView, ProfileLimits, SystemLimitsView};
use kaflow_api_types::{GlobalConfigInput, GlobalConfigView, TopicMetaConfigInput};

use crate::error::EngineError;

#[async_trait]
pub trait ConfigApi: Send + Sync {
    /// The current global settings.
    async fn get_global_ilm_config(&self) -> Result<GlobalConfigView, EngineError>;

    /// Applies an operator profile's constraints.
    ///
    /// ⚠️ **The profile is not trusted.** Keys that are not recognised and constraints that
    /// do not apply are ignored, and every value is held inside the hard bounds before it
    /// takes effect.
    async fn apply_profile_limits(
        &self,
        limits: ProfileLimits,
    ) -> Result<EffectiveLimitsView, EngineError>;

    /// The limits actually in force. Callers should read these directly rather than
    /// caching their own copy — a profile change can shift them at any time.
    async fn get_effective_limits(&self) -> Result<EffectiveLimitsView, EngineError>;

    /// System limits, for diagnosis only — there is no way to change them through this API.
    async fn get_system_limits(&self) -> Result<SystemLimitsView, EngineError>;

    /// Changes the global settings, returning what actually took effect.
    async fn set_global_ilm_config(
        &self,
        input: GlobalConfigInput,
    ) -> Result<GlobalConfigView, EngineError>;

    /// Changes one topic's settings.
    async fn set_topic_meta_config(
        &self,
        workspace: &str,
        topic: &str,
        input: TopicMetaConfigInput,
    ) -> Result<(), EngineError>;

    /// Sets a topic's cleanup policy alone; `None` unsets it.
    async fn set_topic_cleanup_policy(
        &self,
        workspace: &str,
        topic: &str,
        policy: Option<&str>,
    ) -> Result<(), EngineError>;
}
