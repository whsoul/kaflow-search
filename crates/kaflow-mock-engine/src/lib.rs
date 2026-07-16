//! Kaflow mock engine — `KafkaToolEngine` 의 standalone 구현체.
//!
//! 목적:
//! - **CI 빌드 검증** — 본 crate + api-types + api-traits 만으로 컴파일 가능한지 확인.
//! - **public repo 데모** — kafka 클러스터 / RocksDB 없이 UI 가 살아있는 상태로 구동.
//! - **테스트 base** — 향후 단위 테스트가 trait 단위로 mock impl 을 사용.
//!
//! 현재 fidelity:
//! - **read API**: 로컬 json fixture(`data::MockStore`) 기반 실제 조회 — 토픽 리스트 /
//!   인덱싱(open) / browse·keyword·multi 검색 / buckets / 본문 fetch 가 모두 동작.
//! - **mutation API**: `Ok(())` / dummy 카운터 (fixture 는 read-only 데모).
//!
//! 로컬 fixture 는 `fixtures/default.json` 번들(또는 `KAFLOW_MOCK_FIXTURES` 환경변수 경로).
//! 설계/가드레일은 `data.rs` 모듈 doc 참조 (실 엔진 parity 추구 금지).
//!
//! 인프라 의존 0 — `tokio` / `tauri` / `rocksdb` / `kafka-*` 어떤 것도 import 하지 않는다
//! (`std::fs`/`std::env`/`serde_json` 만 사용).

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

/// 로컬 json fixture 기반 mock engine. read API 는 `store` 에서 실제 조회한다.
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
