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
    /// takes effect. A profile arrives over a network; acting on it as given would let a
    /// mistake elsewhere reconfigure the engine.
    async fn apply_profile_limits(
        &self,
        limits: ProfileLimits,
    ) -> Result<EffectiveLimitsView, EngineError>;

    /// The limits actually in force, resolved once. Callers read these rather than
    /// carrying their own copies, which would go stale the moment a profile changed.
    async fn get_effective_limits(&self) -> Result<EffectiveLimitsView, EngineError>;

    /// System limits, for diagnosis. Read-only by nature: they are compiled in, so
    /// offering to edit them would promise a change that nothing acts on.
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
