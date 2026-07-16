//! 메시지 페이로드 deserializer 추상화.
//!
//! 구현체 위치 (모두 미래):
//! - JSON / MessagePack / CBOR — self-describing, schema 불필요. private 또는 public 가능.
//! - AVRO / Protobuf (Confluent wire format) — schema_id 기반, `SchemaSource` 와 결합.
//!
//! 자동 감지 (`WireFormatDetector`) 는 별도 trait 으로 추후 도입.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Deserialize 호출 시 함께 전달되는 컨텍스트.
/// schema_id 가 wire 에 없는 포맷도 있고 (JSON 등), 있는 포맷도 있어서 (AVRO) 분기 처리한다.
#[derive(Debug, Clone, Default)]
pub struct DecodeCtx {
    /// 메시지가 속한 토픽 — schema subject 추론에 사용 가능.
    pub topic: Option<String>,
    /// 명시적으로 지정된 schema id (예: 사용자가 UI 에서 schema 선택).
    pub schema_id: Option<u32>,
}

/// Deserializer 결과. JSON 표현으로 normalize 한다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedValue {
    /// JSON 표현 (`serde_json::Value` 형태). UI 가 이 형태로 표시한다.
    pub json: serde_json::Value,
    /// 디코더가 사용한 schema 식별자 (있는 경우).
    pub schema_id: Option<u32>,
    /// 디코더 이름 (`"json"`, `"avro-confluent"`, ...) — 디버그/표시용.
    pub deserializer_id: &'static str,
}

#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error("wire format mismatch: {0}")]
    WireFormat(String),

    #[error("schema lookup failed: {0}")]
    SchemaLookup(String),

    #[error("decode failed: {0}")]
    Decode(String),

    #[error("unsupported format: {0}")]
    Unsupported(String),
}

/// 메시지 페이로드 deserializer.
///
/// 구현체는 `id()` 로 자기 자신을 식별하고, engine 은 토픽 설정에 따라
/// 등록된 deserializer 중 하나를 선택한다.
pub trait Deserializer: Send + Sync {
    /// 디코더 식별자. UI 에서 토픽별 디코더 선택 시 표시되는 키.
    fn id(&self) -> &'static str;

    /// 메시지 raw bytes 를 JSON value 로 변환.
    fn decode(&self, bytes: &[u8], ctx: &DecodeCtx) -> Result<DecodedValue, DeserializeError>;
}
