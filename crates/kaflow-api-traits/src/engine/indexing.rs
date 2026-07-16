//! Indexing API — Kafka → RocksDB 인덱싱 코어.

use async_trait::async_trait;
use kaflow_api_types::{
    CompactDeletedKeyCursor, CompactDeletedKeysPage, CompactKeyHistory, DecodeFailureKind,
    DecodeFailureMode, OpenKafkaTopicResponse,
};

use crate::error::EngineError;

#[async_trait]
pub trait IndexingApi: Send + Sync {
    /// 토픽 선택 시 호출. partition offset 조회 + IncrementalSync + retention cleanup.
    /// `skip_resync_check = Some(true)` 면 full-resync 감지 skip.
    /// `decode_failure_mode` 는 deserialize 실패 처리 모드. None → `Stop`.
    /// `max_messages = Some(n)` 이면 이번 호출에서 최대 n 건까지만 인덱싱하고 조기
    /// 종료(slice) — 응답 `has_more` 로 잔여 여부 전달, offset 은 자동 이어받음.
    /// None = 무제한(전건 catch-up, 포그라운드/수동싱크 기본). tighten-only 정책 노브.
    /// 정책 전문: `docs/deserialize_failure_policy.md`.
    async fn open_kafka_topic(
        &self,
        workspace: &str,
        topic: &str,
        bootstrap: &str,
        skip_resync_check: Option<bool>,
        decode_failure_mode: Option<DecodeFailureMode>,
        max_messages: Option<u64>,
    ) -> Result<OpenKafkaTopicResponse, EngineError>;

    /// 진행 중인 인덱싱을 협력적으로 중단 (F1).
    /// 호출 즉시 `Ok(())` 반환 — 실제 종료는 indexer 가 현재 batch 를 마무리한 뒤.
    /// 해당 토픽이 인덱싱 중이 아니면 no-op (idempotent).
    /// FE 는 `kafka-indexing-progress` 이벤트로 실제 종료 감지.
    ///
    /// atomic batch 보장: 진행 중이던 batch 까지는 commit 되고 멈추므로
    /// 다음 진입 시 `IncrementalSync` 가 `max_indexed_offset` 부터 자동 이어받음.
    async fn cancel_indexing(&self, workspace: &str, topic: &str) -> Result<(), EngineError>;

    /// 사용자가 "skip 1건" 결정 시 placeholder 메시지를 인라인으로 1건 영속화.
    ///
    /// 정상 메시지와 동일한 M-key + 시스템 I-key (3종) 만 가진다. raw bytes 는
    /// 보존하지 않으며 사후 raw 가 필요하면 Kafka 에서 on-demand fetch (retention 이내).
    /// 정책: `docs/deserialize_failure_policy.md`.
    async fn record_placeholder(
        &self,
        workspace: &str,
        topic: &str,
        partition: i32,
        offset: i64,
        kind: DecodeFailureKind,
        reason: String,
    ) -> Result<(), EngineError>;

    /// compact 토픽 dedup 이 (partition, key) 별로 보관한 값 히스토리(KC) 조회.
    /// `key_raw` 는 key deserializer 가 만든 문자열 (인덱싱 시점과 동일 규칙).
    /// KC 레코드가 없으면(비 compact 토픽 / dedup 도입 전 인덱싱 / 미등장 key) `None`.
    async fn fetch_compact_key_history(
        &self,
        workspace: &str,
        topic: &str,
        partition: u32,
        key_raw: &str,
    ) -> Result<Option<CompactKeyHistory>, EngineError>;

    /// 삭제됨(latest=tombstone) key 페이지 조회 — 간편검색 "대상 유형: 삭제됨" producer.
    /// M/I/R 미기록 key 라 KC 스캔이 유일한 출처. 순서 = (partition asc, key asc).
    /// `key_query` = key_raw 부분 문자열(contains) 필터("" = 전체 — json 구조체 key 도
    /// 중간 값으로 검색 가능), `partitions` = 비면/None 전 파티션,
    /// `cursor` = 직전 페이지 `next_cursor`. `limit` 은 tighten-only (라이브러리 천장 있음).
    async fn fetch_compact_deleted_keys_page(
        &self,
        workspace: &str,
        topic: &str,
        key_query: &str,
        partitions: Option<Vec<u32>>,
        cursor: Option<CompactDeletedKeyCursor>,
        limit: Option<u32>,
    ) -> Result<CompactDeletedKeysPage, EngineError>;
}
