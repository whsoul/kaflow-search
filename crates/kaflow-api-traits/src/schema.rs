//! Schema source 추상화 — Confluent / Apicurio / AWS Glue / 로컬 파일 / inline 등 plug-in.
//!
//! 구현체 위치 (모두 미래):
//! - `kaflow-schema-registry` (private) — Confluent / Apicurio / AWS Glue HTTP 클라이언트
//! - 로컬 파일 source — `.avsc` / `.proto` 디렉토리 watch
//! - inline source — 사용자가 UI 에 paste 한 단일 schema (디버그용)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// schema 식별자. 현재는 Confluent 의 `u32 schema id` 를 기준으로 하되,
/// 다른 source 도 변종으로 수용한다.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchemaId {
    /// Confluent / Apicurio 등 numeric ID
    Numeric(u32),
    /// Subject + version 형태 (Confluent subjects API)
    Subject { subject: String, version: i32 },
    /// 로컬 파일 / inline 식별자
    Name(String),
}

/// 가져온 schema 본체. Avro / Protobuf 등 raw 표현을 담는다.
/// 디코더 측에서 자기 포맷에 맞는 변환을 수행한다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub id: SchemaId,
    /// schema 형식 (`"avro"`, `"protobuf"`, ...).
    pub kind: String,
    /// schema raw 텍스트 / 직렬화 표현.
    pub raw: String,
}

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("schema not found: {0:?}")]
    NotFound(SchemaId),

    #[error("schema source unreachable: {0}")]
    Unreachable(String),

    #[error("schema source unauthorized: {0}")]
    Unauthorized(String),

    #[error("schema parse failed: {0}")]
    Parse(String),

    #[error("schema source error: {0}")]
    Other(String),
}

/// Schema 조회 인터페이스. 구현체 내부에서 캐시 / refresh / fallback 정책을 가진다.
#[async_trait]
pub trait SchemaSource: Send + Sync {
    /// source 식별자 — UI 에 표시되는 이름 (`"confluent"`, `"apicurio"`, `"file:./schemas"` 등).
    fn id(&self) -> String;

    async fn fetch_schema(&self, id: &SchemaId) -> Result<Schema, SchemaError>;
}
