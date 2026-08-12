//! `KafkaToolEngine` — one trait per domain, and a supertrait that gathers them, so a
//! caller can hold a single object and reach everything.
//!
//! Rules an implementation has to keep to:
//! - **Nothing in a signature may assume how it is called.** No transport belongs here.
//! - **Data types come from `kaflow-api-types`.** This crate defines behaviour, not shapes.
//! - **Progress is emitted by the engine itself**, not through an argument. A method that
//!   reports progress does so with whatever the engine was built with.
//! - **Errors are `EngineError`.** Turning one into a string is the caller's business, at
//!   its own boundary.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineError;

pub mod auth;
pub mod config;
pub mod consistency;
pub mod export;
pub mod field_mgmt;
pub mod ilm;
pub mod indexing;
pub mod profiles;
pub mod registry;
pub mod search;
pub mod storage;
pub mod topic_meta;

pub use auth::AuthApi;
pub use config::ConfigApi;
pub use consistency::ConsistencyApi;
pub use export::ExportApi;
pub use field_mgmt::FieldMgmtApi;
pub use ilm::IlmApi;
pub use indexing::IndexingApi;
pub use profiles::ProfilesApi;
pub use registry::RegistryApi;
pub use search::SearchApi;
pub use storage::StorageApi;
pub use topic_meta::TopicMetaApi;

/// What this engine can do, so a caller can adapt rather than discover by failing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineCapabilities {
    /// The deserializers this engine knows, by id.
    pub supported_deserializers: Vec<String>,
    /// Whether it can reach a schema registry.
    pub supports_schema_registry: bool,
    /// Whether it offers assisted features.
    pub supports_ai: bool,
    /// The most messages it will index. `None` means no limit of its own.
    pub max_indexed_messages: Option<u64>,
    /// Which implementation this is. For showing and for diagnosis, not for branching —
    /// behaviour should follow from the fields above, which say what is actually true.
    pub engine_id: String,
}

/// Everything an engine offers, gathered into one trait.
#[async_trait]
pub trait KafkaToolEngine:
    TopicMetaApi
    + StorageApi
    + FieldMgmtApi
    + IlmApi
    + IndexingApi
    + SearchApi
    + ExportApi
    + ConsistencyApi
    + ConfigApi
    + ProfilesApi
    + RegistryApi
    + AuthApi
    + Send
    + Sync
{
    /// What this engine can do.
    fn capabilities(&self) -> EngineCapabilities;

    /// Hands the engine a licence token, or clears it with `None`.
    ///
    /// ⚠️ **Nothing currently acts on this.** The default does nothing and no engine
    /// overrides it, so calling this gates nothing — a caller must not treat it as having
    /// done so.
    ///
    /// An engine that does come to act on a token has to be handed one that was already
    /// verified: nothing here checks a signature.
    async fn set_license(&self, _token: Option<String>) -> Result<(), EngineError> {
        Ok(())
    }
}
