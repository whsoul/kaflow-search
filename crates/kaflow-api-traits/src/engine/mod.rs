//! `KafkaToolEngine` — public/private 경계의 핵심 trait + 도메인별 sub-trait.
//!
//! 설계 (`docs/trait_redesign_2026_05.md`):
//! - 도메인별 sub-trait (TopicMetaApi / StorageApi / ...) 으로 분리.
//! - `KafkaToolEngine` 은 모든 sub-trait 을 묶는 supertrait + capability / license.
//! - 호출자는 `&dyn KafkaToolEngine` 한 객체만 들고 다니면 모든 도메인 method 호출 가능.
//!
//! 설계 원칙:
//! - 모든 메서드 시그니처는 transport-agnostic (Tauri / HTTP / WebSocket 무관).
//! - DTO 는 `kaflow-api-types` 의 타입만 사용. 본 crate 안에 신규 데이터 타입 정의 금지.
//! - 진행 이벤트가 필요한 메서드는 `Arc<dyn ProgressEmit>` 을 받지 않는다 — engine 인스턴스
//!   가 보유한 emitter 로 내부 emit 한다 (state 보유 모델 결정 2).
//! - 에러는 `EngineError`. boundary (Tauri command 어댑터) 에서만 stringify.

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

/// Engine 이 노출하는 가용 기능 / 한도 — UI 가 이 값으로 화면을 적응시킨다.
///
/// 라이선스 / FeatureGate 와 연동되어 plan 별로 다른 값을 반환할 수 있다.
/// `kaflow-feature-gate` (미래) 가 token 을 검증하면 그 entitlement 가 여기 반영된다.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineCapabilities {
    /// 등록된 deserializer id 목록 (`"json"`, `"avro-confluent"`, ...).
    pub supported_deserializers: Vec<String>,
    /// Schema Registry 연동 가능 여부 (Confluent / Apicurio / AWS Glue).
    pub supports_schema_registry: bool,
    /// AI 기능 (query builder / schema discovery 등) 가용 여부.
    pub supports_ai: bool,
    /// 인덱싱 가능한 최대 메시지 수 (라이선스 한도). `None` = 무제한.
    pub max_indexed_messages: Option<u64>,
    /// Engine 구현체 식별자 — `"mock"` / `"real"` / `"real-pro"` 등 디버그/표시용.
    pub engine_id: String,
}

/// Kaflow engine 의 외부 인터페이스 — 9 도메인 sub-trait 의 aggregate.
///
/// `Arc<dyn KafkaToolEngine>` 형태로 Tauri / HTTP / WebSocket 어댑터에 주입.
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
    /// 현재 engine 이 지원하는 capability 목록.
    fn capabilities(&self) -> EngineCapabilities;

    /// 라이선스 토큰 주입 — engine 내부 entitlement enforcement 에 사용.
    /// 토큰 자체의 서명 검증은 외부 (`kaflow-feature-gate`) 가 수행한 뒤 호출된다.
    async fn set_license(&self, _token: Option<String>) -> Result<(), EngineError> {
        Ok(())
    }
}
