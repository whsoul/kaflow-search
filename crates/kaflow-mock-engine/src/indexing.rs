//! `IndexingApi` mock impl — kafka 대신 로컬 fixture 를 "인덱싱"(이미 적재됨)한 것으로 응답.

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
        // fixture 는 이미 메모리에 적재돼 있으므로 그 토픽의 필드/건수를 그대로 보고한다.
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
        // mock 은 즉시 0건 응답이라 실제 in-flight 없음 — no-op.
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
        // mock 은 영속 stub — no-op.
        Ok(())
    }

    async fn fetch_compact_key_history(
        &self,
        _workspace: &str,
        _topic: &str,
        _partition: u32,
        _key_raw: &str,
    ) -> Result<Option<CompactKeyHistory>, EngineError> {
        // mock fixture 는 compact dedup 이력을 갖지 않는다 — 항상 None.
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
        // mock fixture 는 KC(삭제 key 흔적)를 갖지 않는다 — 항상 빈 페이지.
        Ok(CompactDeletedKeysPage {
            rows: Vec::new(),
            next_cursor: None,
            scanned: 0,
        })
    }
}
