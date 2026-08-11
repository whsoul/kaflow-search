//! An engine backed by a local file instead of a cluster.
//!
//! It exists for three reasons: to prove the interface can be implemented without any
//! infrastructure, to let the application be shown working without one, and to give tests
//! something to run against.
//!
//! Reading works for real — topics, searching, aggregating, fetching a message. Changing
//! things does not: the data is a fixture, and mutations report success without altering
//! it.
//!
//! **It depends on nothing.** No async runtime, no store, no client. That constraint is
//! the point rather than an accident — an engine that needs none of those proves the
//! interface asks for none of those.

use std::sync::Arc;

use async_trait::async_trait;
use kaflow_api_traits::{EngineCapabilities, KafkaToolEngine};

mod auth;
mod config;
mod consistency;
mod data;
mod export;
mod field_mgmt;
mod ilm;
mod indexing;
mod profiles;
mod registry;
mod search;
mod storage;
mod topic_meta;

/// An engine over fixture data.
#[derive(Debug, Clone)]
pub struct MockEngine {
    pub(crate) store: Arc<data::MockStore>,
}

impl MockEngine {
    pub fn new() -> Self {
        Self {
            store: Arc::new(data::MockStore::load()),
        }
    }
}

impl Default for MockEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KafkaToolEngine for MockEngine {
    fn capabilities(&self) -> EngineCapabilities {
        let total: usize = self.store.topics.iter().map(|t| t.messages.len()).sum();
        EngineCapabilities {
            supported_deserializers: vec!["json".to_string(), "plain".to_string()],
            supports_schema_registry: false,
            supports_ai: false,
            max_indexed_messages: Some(total as u64),
            engine_id: "kaflow-mock-engine".to_string(),
        }
    }
}
