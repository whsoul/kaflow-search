//! `FieldMgmtApi` mock impl.

use async_trait::async_trait;
use kaflow_api_traits::engine::FieldMgmtApi;
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::{IndexedFieldInput, TopicDeserializers};

use crate::MockEngine;

#[async_trait]
impl FieldMgmtApi for MockEngine {
    async fn set_indexed_fields(
        &self,
        _workspace: &str,
        topic: &str,
        fields: Vec<IndexedFieldInput>,
    ) -> Result<(), EngineError> {
        // The field list is what decides; the separate list is folded into it and cleared,
        // as a real engine does.
        let tok: Vec<String> = fields
            .iter()
            .filter(|f| f.tokenize)
            .map(|f| f.name.clone())
            .collect();
        self.store.set_tokenized(topic, tok);
        Ok(())
    }

    async fn reindex_fields_from_meta(
        &self,
        _workspace: &str,
        topic: &str,
        _fields: Vec<String>,
    ) -> Result<usize, EngineError> {
        // Nothing to rebuild — only a count, so the caller has something to report.
        Ok(self
            .store
            .topic(topic)
            .map(|t| t.messages.len())
            .unwrap_or(0))
    }

    async fn drop_fields_from_index(
        &self,
        _workspace: &str,
        _topic: &str,
        _fields: Vec<String>,
    ) -> Result<usize, EngineError> {
        Ok(0)
    }

    async fn remove_topic_from_index(
        &self,
        _workspace: &str,
        _topic: &str,
    ) -> Result<(usize, usize), EngineError> {
        Ok((0, 0))
    }

    async fn ensure_topic_watched(
        &self,
        _workspace: &str,
        _topic: &str,
    ) -> Result<(), EngineError> {
        Ok(())
    }

    async fn set_topic_deserializers(
        &self,
        _workspace: &str,
        _topic: &str,
        _deserializers: Option<TopicDeserializers>,
    ) -> Result<(), EngineError> {
        Ok(())
    }

    async fn set_tokenize_fields(
        &self,
        _workspace: &str,
        topic: &str,
        fields: Vec<String>,
    ) -> Result<(), EngineError> {
        // Takes effect immediately, which is the thing worth showing.
        self.store.set_tokenized(topic, fields);
        Ok(())
    }

    async fn unwatch_topic(
        &self,
        _workspace: &str,
        _topic: &str,
    ) -> Result<(usize, usize), EngineError> {
        Ok((0, 0))
    }

    async fn mark_topic_auto_cleaned(
        &self,
        _workspace: &str,
        _topic: &str,
        _timestamp_ms: i64,
    ) -> Result<(), EngineError> {
        Ok(())
    }
}
