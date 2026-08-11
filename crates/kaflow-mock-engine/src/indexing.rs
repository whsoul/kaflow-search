//! Indexing, for data that is already loaded. Nothing is read and nothing is written.

use async_trait::async_trait;
use kaflow_api_traits::engine::IndexingApi;
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::{
    CompactDeletedKeyCursor, CompactDeletedKeysPage, CompactKeyHistory, DecodeFailureKind,
    DecodeFailureMode, OpenKafkaTopicResponse,
};

use crate::MockEngine;

#[async_trait]
impl IndexingApi for MockEngine {
    async fn open_kafka_topic(
        &self,
        _workspace: &str,
        topic: &str,
        _bootstrap: &str,
        _skip_resync_check: Option<bool>,
        _decode_failure_mode: Option<DecodeFailureMode>,
        _max_messages: Option<u64>,
    ) -> Result<OpenKafkaTopicResponse, EngineError> {
        // Report what is already there rather than pretending to have done work.
        let t = self.store.topic(topic);
        let count = t.map(|t| t.messages.len()).unwrap_or(0);
        Ok(OpenKafkaTopicResponse {
            topic: topic.to_string(),
            kafka_total_count: count as i64,
            newly_indexed: count,
            key_fields: t.map(|t| t.key_fields.clone()).unwrap_or_default(),
            payload_fields: t.map(|t| t.payload_fields.clone()).unwrap_or_default(),
            header_fields: Vec::new(),
            cleaned_up_i: 0,
            cleaned_up_m: 0,
            full_resync_triggers: Vec::new(),
            decode_failure_context: None,
            has_more: false,
        })
    }

    async fn cancel_indexing(&self, _workspace: &str, _topic: &str) -> Result<(), EngineError> {
        // Nothing runs long enough to stop.
        Ok(())
    }

    async fn record_placeholder(
        &self,
        _workspace: &str,
        _topic: &str,
        _partition: i32,
        _offset: i64,
        _kind: DecodeFailureKind,
        _reason: String,
    ) -> Result<(), EngineError> {
        // Nothing is persisted, so there is nothing to record.
        Ok(())
    }

    async fn fetch_compact_key_history(
        &self,
        _workspace: &str,
        _topic: &str,
        _partition: u32,
        _key_raw: &str,
    ) -> Result<Option<CompactKeyHistory>, EngineError> {
        // The fixture keeps no history of replaced values.
        Ok(None)
    }

    async fn fetch_compact_deleted_keys_page(
        &self,
        _workspace: &str,
        _topic: &str,
        _key_query: &str,
        _partitions: Option<Vec<u32>>,
        _cursor: Option<CompactDeletedKeyCursor>,
        _limit: Option<u32>,
    ) -> Result<CompactDeletedKeysPage, EngineError> {
        // Nor of deleted keys.
        Ok(CompactDeletedKeysPage {
            rows: Vec::new(),
            next_cursor: None,
            scanned: 0,
        })
    }
}
