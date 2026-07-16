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
        // 어절 권위는 indexed_fields 의 tokenize=true (실 엔진과 동일 — tokenize_fields 흡수·클리어).
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
        // mock 은 fixture 가 이미 적재됨 — 어절 상태는 set_indexed_fields 에서 반영됨.
        // 재인덱싱 "건수" 만 보고(FE 의 "N건 재인덱싱 완료" 표시용).
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
        // picker 경로(인덱싱 전 어절 지정). mock 은 즉시 검색에 반영.
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
