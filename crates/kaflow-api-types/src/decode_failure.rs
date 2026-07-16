//! Deserialize 실패 정책 DTO.
//!
//! `docs/deserialize_failure_policy.md` 참조.
//!
//! - `DecodeFailureKind` — 실패 종류 (5 종, picker substring 매칭과 동일 분류).
//! - `DecodeFailureContext` — `OpenKafkaTopicResponse` 에 첨부되는 실패 컨텍스트.
//!   사용자 결정은 별도 Tauri 명령으로 흐르므로 oneshot await 인프라 없음.
//! - 시스템 필드 상수 — placeholder 메시지의 I-key 필드 이름.

use serde::{Deserialize, Serialize};

/// `open_kafka_topic` 의 deserialize 실패 처리 모드.
///
/// - `Stop` (기본): fail 시 partial commit 후 즉시 종료. 응답에 컨텍스트 첨부 →
///   frontend 가 사용자 결정 모달 표시.
/// - `SkipAll`: fail 시 placeholder 인라인 기록 + sync 계속. frontend 모달의
///   "이후 자동 skip" 결정 시 재호출. 호출 1회 한정.
///
/// 정책 전문: `docs/deserialize_failure_policy.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecodeFailureMode {
    #[default]
    Stop,
    SkipAll,
}

/// Deserialize 실패 종류. picker substring 매칭과 동일 분류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecodeFailureKind {
    /// Confluent magic byte / prefix 불일치
    WireFormat,
    /// Schema Registry 조회 실패 (HTTP / not found)
    SchemaLookup,
    /// schema 는 찾았는데 데이터 디코드 실패
    DecodeFailed,
    /// deserializer 가 지원 못 함
    UnsupportedFormat,
    /// 그 외
    Other,
}

impl DecodeFailureKind {
    /// snake_case 문자열 표현. I-key 인덱싱과 직접 매칭.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WireFormat => "wire_format",
            Self::SchemaLookup => "schema_lookup",
            Self::DecodeFailed => "decode_failed",
            Self::UnsupportedFormat => "unsupported_format",
            Self::Other => "other",
        }
    }
}

/// `OpenKafkaTopicResponse.decode_failure_context` 에 실리는 실패 정보.
///
/// fail-stop 모델 — backend 가 sync 중단 후 응답에 1회성으로 첨부. frontend 가
/// 모달로 표시 + 사용자 결정 따라 별도 명령 호출. registry / oneshot 채널 인프라
/// 사용하지 않음.
///
/// 정책: `docs/deserialize_failure_policy.md`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeFailureContext {
    pub partition: i32,
    pub offset: i64,
    pub kind: DecodeFailureKind,
    /// 원본 에러 메시지 (raw reason).
    pub reason: String,
    /// 실패한 메시지의 raw bytes (base64 인코딩). preview 용.
    /// 너무 큰 경우 backend 가 truncate 한 뒤 전송.
    pub raw_key_base64: Option<String>,
    pub raw_value_base64: Option<String>,
    /// 어느 쪽에서 실패했는지 (`"key"` | `"value"`).
    pub failed_side: String,
}

// ── 시스템 예약 필드 (placeholder 인덱싱용) ──────────────────────────────────
//
// `__` prefix 컨벤션을 따른다. 일반 필드 목록 UI / 인기 통계에는 노출하지 않고,
// FieldBased cleanup 대상에서도 제외한다. 검색은 가능 (직접 입력).

/// 모든 시스템 필드 공통 prefix. `is_system_field` 의 단일 기준.
pub const SYSTEM_FIELD_PREFIX: &str = "__";

/// 메시지 생성시간 R-key 의 예약 필드명. 모든 메시지에 자동 등록 (R | topic | "__ts" | ...).
/// browse / 시계열 / 맵의 시간순 source. 사용자 정의 range_fields 와 같은 R prefix 공간에서
/// field 이름으로 자연 격리된다.
pub const SYSTEM_FIELD_TS: &str = "__ts";

/// placeholder 메시지 마커 (`true` 값 1 cardinality).
pub const SYSTEM_FIELD_DECODE_FAILED: &str = "__decode_failed";

/// 실패 종류 (`DecodeFailureKind::as_str()` 값).
pub const SYSTEM_FIELD_DECODE_ERROR_KIND: &str = "__decode_error_kind";

/// 원본 에러 메시지 (raw reason).
pub const SYSTEM_FIELD_DECODE_ERROR_REASON: &str = "__decode_error_reason";

/// 필드 이름이 시스템 예약 필드인지 판정.
///
/// `__` prefix 단일 기준. cleanup_selector / 필드 목록 UI 등이 사용.
pub fn is_system_field(name: &str) -> bool {
    name.starts_with(SYSTEM_FIELD_PREFIX)
}
