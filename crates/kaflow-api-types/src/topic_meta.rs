//! 토픽 단위 메타 / cleanup 정책 / ILM 부분 업데이트 DTO.
//!
//! 호환성 주의:
//! - `TopicFieldMeta` 는 RocksDB 의 T-META 키에 그대로 직렬화되어 저장된다.
//!   기존 워크스페이스 호환을 위해 `TopicFieldMetaRaw` + `From` 변환 layer 가 있다.
//! - `cleanup_policy` 직렬화는 항상 discriminator string (FE 호환).

use crate::domain::{FieldSearchStat, IndexState, IndexedField, IndexedFieldKind};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ── DeserializerSpec ──────────────────────────────────────────────────────

/// 토픽별 메시지 페이로드 디시리얼라이저 선택.
///
/// `TopicFieldMeta::deserializer` 에 영속 저장되며, 인덱싱 / browse 진입부의
/// `resolve_deserializer` 가 이 값을 보고 실제 구현체를 빌드한다.
///
/// 직렬화는 `{ "kind": "json" }` / `{ "kind": "avro_local_file", "schemaPath": "..." }` 등
/// 형태 (snake_case discriminator + camelCase 필드).
///
/// 두 직교 축의 조합:
///   wire_format ∈ {raw datum, confluent (magic+schema_id prefix)}
///   schema_source ∈ {local .avsc file, Schema Registry HTTP}
///
/// 의미 있는 조합:
/// - `AvroLocalFile`: raw datum + local file (Phase 2b — 기본)
/// - `AvroConfluentLocal`: confluent wire + local file (Registry 없는 환경 + Confluent format)
/// - `AvroConfluentRegistry`: confluent wire + Registry HTTP (production)
///
/// raw + registry 조합은 없음 — Registry 는 wire 의 schema_id 로 fetch 하므로 raw datum 과 무의미.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeserializerSpec {
    /// 기본값 — UTF-8 (lossy) 로 통과. plain text / JSON 텍스트 양쪽 수용.
    Json,
    /// 로컬 `.avsc` 파일 1개 + **raw Avro datum** (Confluent wire format prefix 없음).
    /// `schema_path` 는 사용자 식별/표시용 (선택 시점 경로). 실 디코드는 `schema_text` 인라인 본문 사용.
    /// `schema_text` = None 은 legacy 경로 (앱 이전 버전 저장값) — `build_from_spec` 가 path 에서
    /// lazy load 하지만 신규 적용은 항상 Some 으로 들어와 외부 파일 의존이 사라진다.
    #[serde(rename_all = "camelCase")]
    AvroLocalFile {
        schema_path: String,
        #[serde(default)]
        schema_text: Option<String>,
    },
    /// 로컬 `.avsc` 파일 1개 + **Confluent wire format** (magic byte 0x00 + 4-byte schema_id prefix).
    /// `schema_path` 는 사용자 식별/표시용. 실 디코드는 `schema_text` 인라인 본문 사용 (schema_id 는 무시).
    /// Schema Registry 가 없거나 호출하고 싶지 않을 때 사용. wire format 검증은 strict —
    /// magic byte ≠ 0x00 이면 명시 에러.
    #[serde(rename_all = "camelCase")]
    AvroConfluentLocal {
        schema_path: String,
        #[serde(default)]
        schema_text: Option<String>,
    },
    /// Confluent Schema Registry HTTP + wire format.
    /// 메시지의 schema_id 로 Registry 에서 `.avsc` 자동 fetch (LRU 캐시).
    /// `registry_url` 예: `http://localhost:8081`. basic_auth 형식: `"user:password"`.
    #[serde(rename_all = "camelCase")]
    AvroConfluentRegistry {
        registry_url: String,
        basic_auth: Option<String>,
    },
    /// Kafka Connect 의 JSON converter envelope (`schemas.enable=true`).
    /// 메시지가 `{"schema": {...}, "payload": {...}}` 형태일 때 `payload` 만 벗겨 인덱싱한다
    /// (`schema` 는 Connect struct 메타라 검색 대상이 아님). 외부 schema 불필요 — unit variant.
    JsonConnectEnvelope,
    /// Confluent Schema Registry 의 "JSON Schema" 직렬화.
    /// wire format = `0x00 magic + 4-byte schema_id + JSON payload`. payload 가 이미
    /// self-describing JSON 이라 Registry 호출이 불필요 — prefix(있으면) 만 떼고 JSON 파싱.
    /// magic 체크는 느슨함: `0x00` 으로 시작 + len≥5 면 prefix 제거, 아니면 전체를 JSON 으로.
    /// 외부 schema / Registry URL 불필요 — unit variant.
    JsonSchemaConfluent,
    /// raw Protobuf datum + 로컬 `.proto` 파일.
    ///
    /// `.proto` 는 메시지를 여러 개 정의할 수 있어 `message_name` 으로 디코드 대상 메시지를
    /// 지정한다 (Avro `.avsc` 엔 없던 추가 입력). `schema_text` 는 `.proto` 본문 인라인.
    ///
    /// 디코더: `protox` 로 `.proto` → descriptor 컴파일 + `prost-reflect` `DynamicMessage` 디코드.
    /// `schema_text` 인라인은 single-file `.proto` (import 없음), `schema_path` fallback 은
    /// import 를 파일 디렉터리 기준으로 해석한다.
    #[serde(rename_all = "camelCase")]
    ProtobufLocalFile {
        schema_path: String,
        #[serde(default)]
        schema_text: Option<String>,
        message_name: String,
        /// 메인 `.proto` 가 `import` 하는 파일들 `(import 경로, 본문)`. picker 가 파일 선택 시
        /// 디스크에서 폐쇄집합을 읽어 인라인 저장. 비어 있으면 단일 파일(역호환).
        #[serde(default)]
        import_files: Vec<ProtoFile>,
    },
    /// Protobuf + Confluent Schema Registry.
    ///
    /// wire format = `0x00 magic + 4-byte schema_id (BE) + message-index 배열 + payload`.
    /// schema_id 로 Registry 에서 `.proto` fetch (schema_id 별 캐시), message-index (zigzag varint)
    /// 로 디코드 대상 메시지를 결정한 뒤 `DynamicMessage` 디코드.
    /// **제약**: 현재 single-file schema 만 지원 (import/schema 참조는 미해석 — fail-fast).
    #[serde(rename_all = "camelCase")]
    ProtobufConfluentRegistry {
        registry_url: String,
        basic_auth: Option<String>,
    },
    /// Avro Confluent Registry — 등록된 `RegistryResource` 를 **id 로 참조** (살아있는 참조).
    /// `build_from_spec` 가 `resource_id` → 리소스 조회 후 url/auth 로 `AvroConfluentRegistry`
    /// 디코더를 빌드한다. URL 직접입력 변종(`AvroConfluentRegistry`)을 대체하는 신규 기본형.
    #[serde(rename_all = "camelCase")]
    AvroConfluentRegistryRef { resource_id: String },
    /// Protobuf Confluent Registry — 등록된 `RegistryResource` 를 id 로 참조 (살아있는 참조).
    #[serde(rename_all = "camelCase")]
    ProtobufConfluentRegistryRef { resource_id: String },
}

impl Default for DeserializerSpec {
    fn default() -> Self {
        DeserializerSpec::Json
    }
}

/// `protobuf_local_file` 의 import 파일 1개 — `(import 경로, .proto 본문)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtoFile {
    /// `.proto` 안 `import "..."` 와 일치하는 경로 (예: `common/address.proto`).
    pub name: String,
    pub text: String,
}

/// 토픽의 key/value deserializer 페어.
///
/// Kafka 메시지는 key 와 value 가 별도 바이트로 produce 되며, Confluent Schema Registry
/// TopicNameStrategy 도 `{topic}-key` / `{topic}-value` 두 subject 로 분리한다. 따라서
/// 디코딩도 두 spec 을 따로 적용해야 한다 (한쪽은 JSON 다른 쪽은 AVRO 같은 mix 도 가능).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TopicDeserializers {
    #[serde(default)]
    pub key: DeserializerSpec,
    #[serde(default)]
    pub value: DeserializerSpec,
}

// ── CleanupPolicy ─────────────────────────────────────────────────────────

/// 토픽별로 적용할 크기 기반 정리 전략.
/// `TopicFieldMeta::cleanup_policy`에 설정하거나,
/// 없으면 `GlobalIlmConfig::default_cleanup_policies`를 사용한다.
///
/// 실행 규칙:
/// - `DropIndex`가 포함되면 전체 인덱스 삭제 후 나머지는 수행하지 않음
/// - `FieldBased` / `CountBased` 는 `DropIndex` 없이 동시에 수행 가능
///
/// **세션 25 (ILM 리뉴얼)**: 시간 차원 (`TimeBased`) 제거.
/// 시간 범위 검색은 별도의 ad-hoc 임시 인덱스 (후속 작업) 로 분리.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupPolicy {
    /// 토픽 전체 인덱스 삭제 (기본값)
    DropIndex,
    /// 검색 빈도가 낮은 필드만 선택적으로 삭제
    FieldBased,
    /// 파티션당 최신 `max_count` 건만 유지 (세션 25 ILM 리뉴얼).
    /// 평상시는 무한 누적, Size cleanup 트리거 시 partition 별 earliest 부터 trim.
    /// `max_count` 가 토픽 메타에서 0 이거나 None 이면 글로벌 `default_keep_count_per_partition` 폴백.
    CountBased { max_count: u64 },
}

// ── IlmUpdate ─────────────────────────────────────────────────────────────

/// ILM 필드만 골라서 부분 업데이트할 때 사용. None 필드는 기존값 보존.
///
/// **세션 23**: 단일 `index_state` 를 직교 플래그로 분리.
/// **세션 25 ILM 리뉴얼**: `time_subset` 제거 (시간 차원 분리됨).
#[derive(Debug, Default)]
pub struct IlmUpdate {
    /// 인덱싱 완료 여부
    pub indexed: Option<bool>,
    /// 일부 필드만 인덱싱 (`indexed_fields` non-empty) 인지
    pub field_subset: Option<bool>,
    /// partition → latest indexed offset (i64)
    pub latest_indexed_offsets: Option<HashMap<u32, i64>>,
    /// partition → Kafka earliest offset
    pub earliest_offsets: Option<HashMap<u32, i64>>,
    /// partition → Kafka latest offset (sync 시점 snapshot)
    pub kafka_latest_offsets: Option<HashMap<u32, i64>>,
    pub last_incremental_sync_at: Option<i64>,
    pub last_cleanup_at: Option<i64>,
    pub topic_type: Option<String>,
    pub topic_policy_checked_at: Option<i64>,
    /// 사용자가 키워드 검색을 수행한 마지막 시각 (browse 제외)
    pub last_search_at: Option<i64>,
    /// (now_ms, [field, ...]) — 각 필드의 search_count++, last_searched_at 갱신
    pub field_search_increments: Option<(i64, Vec<String>)>,
    // ── Kafka 토픽 설정 (get_topic_config_info로 갱신) ─────────────────────
    /// 보존 기간(ms). -1 = 무제한. Some(v) = 설정 확인됨.
    pub retention_ms: Option<i64>,
    /// 파티션당 최대 크기(bytes). -1 = 무제한.
    pub retention_bytes: Option<i64>,
    pub compression_type: Option<String>,
    pub message_timestamp_type: Option<String>,
    pub partition_count: Option<usize>,
    pub replication_factor: Option<usize>,
    /// Kafka 토픽 UUID (Metadata v9). 토픽 삭제 후 재생성 시 변경됨.
    pub topic_id: Option<String>,
    /// `delivery_lost_total` 에 더할 **증분** (다른 필드처럼 교체가 아님 — 완주 시
    /// 낙관 watermark 가 덮는 꼬리 미배달 구간을 배치 실측에 추가하는 용도).
    pub delivery_lost_add: Option<u64>,
}

// ── TopicFieldMeta ────────────────────────────────────────────────────────

/// 토픽별 필드 메타 — T | topic \x00 키에 value로 저장
///
/// **인덱스 상태 직교 플래그 (세션 23)**: 단일 `index_state: IndexState` 를 분리.
/// - `indexed`: 인덱싱이 한 번이라도 완료됐는지
/// - `field_subset`: 일부 필드만 인덱싱 중인지 (`indexed_fields` non-empty 와 동기)
///
/// **세션 25 ILM 리뉴얼**: `time_subset` 제거 (시간 차원 분리됨).
///
/// 표시용 라벨이 필요한 곳은 `derived_index_state()` 메서드를 사용한다.
/// 구버전 JSON 의 `index_state: "..."` 는 `TopicFieldMetaRaw` 가 자동 매핑한다.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", from = "TopicFieldMetaRaw")]
pub struct TopicFieldMeta {
    pub topic: String,
    pub key_fields: Vec<String>,
    pub payload_fields: Vec<String>,
    pub header_fields: Vec<String>,
    #[serde(default)]
    pub message_count: usize,
    #[serde(default)]
    pub index_entry_count: usize,
    // ── 직교 인덱스 상태 플래그 ───────────────────────────────
    /// 인덱싱이 한 번이라도 완료됐는지 (이전 `IndexState != NotIndexed`)
    #[serde(default)]
    pub indexed: bool,
    /// 일부 필드만 인덱싱 중 (이전 `PartialField` 차원)
    #[serde(default)]
    pub field_subset: bool,
    /// partition → latest indexed offset
    #[serde(default)]
    pub latest_indexed_offsets: HashMap<u32, i64>,
    /// partition → Kafka earliest offset (마지막 open_kafka_topic 시점 기준)
    #[serde(default)]
    pub earliest_offsets: HashMap<u32, i64>,
    /// partition → Kafka latest offset (마지막 open_kafka_topic 시점 기준).
    /// `latest_indexed_offsets` 와 비교해 partial-offset 여부 derive 에 사용.
    /// 영속 캐시 — Kafka 끊긴 상태에서도 직전 sync 시점의 partial 여부 표시 가능.
    #[serde(default)]
    pub kafka_latest_offsets: HashMap<u32, i64>,
    #[serde(default)]
    pub topic_type: Option<String>,
    #[serde(default)]
    pub topic_policy_checked_at: Option<i64>,
    #[serde(default)]
    pub last_incremental_sync_at: Option<i64>,
    #[serde(default)]
    pub last_cleanup_at: Option<i64>,
    #[serde(default)]
    pub retention_priority: Option<i32>,
    /// 사용자가 키워드 검색을 수행한 마지막 시각 (browse 제외)
    #[serde(default)]
    pub last_search_at: Option<i64>,
    /// field → 검색 사용 통계 (search_count, last_searched_at)
    #[serde(default)]
    pub field_search_stats: HashMap<String, FieldSearchStat>,
    // ── Kafka 토픽 설정 (get_topic_config_info로 갱신) ─────────────────────
    #[serde(default)]
    pub retention_ms: Option<i64>,
    #[serde(default)]
    pub retention_bytes: Option<i64>,
    #[serde(default)]
    pub compression_type: Option<String>,
    #[serde(default)]
    pub message_timestamp_type: Option<String>,
    #[serde(default)]
    pub partition_count: Option<usize>,
    #[serde(default)]
    pub replication_factor: Option<usize>,
    /// Kafka 토픽 UUID (Metadata v9). 토픽 삭제 후 재생성 시 변경됨.
    #[serde(default)]
    pub topic_id: Option<String>,
    /// 크기 기반 정리 시 이 토픽에 적용할 정책 (단일). None 이면 GlobalIlmConfig::default_cleanup_policies 사용.
    /// 구버전 `cleanupPolicies: Vec` 포맷도 수용 (첫 번째 항목만 사용).
    /// 직렬화는 항상 discriminator string (`"drop_index"|"field_based"|"count_based"|null`) — FE 호환.
    /// CountBased 의 실제 한도는 형제 필드 `max_count` 가 결정.
    #[serde(
        default,
        alias = "cleanupPolicies",
        deserialize_with = "deserialize_cleanup_policy",
        serialize_with = "serialize_cleanup_policy_as_tag"
    )]
    pub cleanup_policy: Option<CleanupPolicy>,
    /// CountBased 정책 — 파티션당 최신 N건 보존 한도. None 이면 글로벌 `default_keep_count_per_partition` 폴백.
    /// **세션 25 ILM 리뉴얼 신규**.
    #[serde(default)]
    pub max_count: Option<u64>,
    /// 현재 인덱싱 대상 필드 목록 (key/payload/header fields 의 부분집합).
    /// 비어있으면 발견된 모든 필드를 대상으로 함 (하위 호환).
    /// FieldBased SizeCleanup 또는 사용자 선택으로 변경됨.
    ///
    /// 각 엔트리는 `(name, kind)`. 유형 구분자(`IndexedFieldKind`)로
    /// 크기 기반 부분 정리 시 `Required` 필드를 보호한다.
    /// 레거시 JSON(`Vec<String>`)도 자동 수용한다 (`IndexedFieldKind::Optional` 로 변환).
    #[serde(default, deserialize_with = "deserialize_indexed_fields")]
    pub indexed_fields: Vec<IndexedField>,
    /// 자동 정리 (size cleanup / drop_topic_index) 로 인덱스가 제거된 타임스탬프.
    /// None 이면 자동 정리 이력 없음. picker UI 의 "자동정리됨" 뱃지 판단에 사용.
    /// 재인덱싱되면 `drop_topic_index`/`open_kafka_topic` 로직에서 다시 None 으로 리셋해야 함.
    #[serde(default)]
    pub auto_cleanup_removed_at: Option<i64>,
    /// 토픽의 key/value deserializer 페어. None 이면 기본 (`Json` × 2) 으로 동작.
    /// 인덱싱 / browse 진입부의 `resolve_deserializers` 가 이 값을 본다.
    /// **세션 40**: 이전 `deserializer: Option<DeserializerSpec>` (단일 spec, value 전용) 에서
    /// `deserializers: Option<TopicDeserializers>` (key/value 페어) 로 확장. legacy 필드는
    /// `TopicFieldMetaRaw` 가 `{ key: Json, value: legacy }` 로 자동 마이그레이션.
    #[serde(default)]
    pub deserializers: Option<TopicDeserializers>,
    /// 어절(tokenize) 대상 필드 — `indexed_fields` 와 **직교**. 인덱서는
    /// `indexed_fields` 의 `tokenize=true` ∪ 이 목록을 어절 대상으로 본다.
    /// `indexed_fields` 가 비어있어도(=전체 필드 인덱싱) 여기에 든 필드는 어절로 인덱싱된다.
    /// picker 에서 인덱싱 전(필드 미발견 시점) 어절 대상을 지정하기 위한 경로.
    /// 출시 한도 = 1개(`TOKENIZE_FIELD_CAP`), 향후 유료에서 다중 확장.
    #[serde(default)]
    pub tokenize_fields: Vec<String>,
    /// compact 토픽 dedup-on-write 로 이 토픽에서 superseded(옛 버전) 삭제된 누적 횟수.
    /// per-key 카운트(`CompactKeyState.superseded_total`)와 별개인 토픽 전체 집계.
    /// UI 표시("N개 stale 정리됨") 용. 상세: `docs/compact_topic_sync_repair_policy.md`.
    #[serde(default)]
    pub compact_superseded_total: u64,
    /// compact 토픽에서 tombstone(null value = key 삭제 마커)으로 처리한 누적 횟수.
    /// tombstone 은 M/I/R 미기록(KC 만 갱신)이라 "처리했지만 저장 안 함" — 회계에서
    /// 미처리(gap)로 오인되지 않도록 별도 집계. 상세: `docs/compact_topic_sync_repair_policy.md`.
    #[serde(default)]
    pub compact_tombstone_total: u64,
    /// sync fetch 가 훑고 지나간 offset 범위 중 **broker 가 레코드를 배달하지 않은 구간**(배달
    /// 구멍)의 누적 실측치 — broker 측 compaction 으로 이미 삭제됐거나 트랜잭션 control
    /// record(애초 사용자 레코드 아님)인 offset 자리. 배치 처리와 같은 WriteBatch 로 누적되어
    /// 처리 watermark 와 크래시 정합이 같다. "소실" 회계 버킷의 실측 출처 (훑은 구간 한정 —
    /// txn marker 와 합산, 구분 불가). 상세: `docs/compact_topic_sync_repair_policy.md`.
    #[serde(default)]
    pub delivery_lost_total: u64,
}

// TOKENIZE_FIELD_CAP 은 플랜 한도 단일 출처인 `limits.rs` 로 이동 (2026-07-13).
// 평면 re-export 라 `kaflow_api_types::TOKENIZE_FIELD_CAP` 경로는 그대로다.

impl TopicFieldMeta {
    /// 인덱싱 대상 필드 이름 집합 (유형 무관).
    pub fn indexed_field_names(&self) -> HashSet<String> {
        self.indexed_fields.iter().map(|f| f.name.clone()).collect()
    }

    /// 어절(tokenize) 대상 필드 이름 집합 — `indexed_fields` 의 `tokenize=true` ∪ `tokenize_fields`.
    /// 인덱싱 / reindex 가 **동일 set** 으로 M-value `tokenized_fields` 를 채우도록 단일 출처로 둔다
    /// (둘이 다르면 세션 65 류 정합성 어긋남 재발). `tokenize_fields` 는 `indexed_fields` 가
    /// 비어있어도(=전체 필드 인덱싱) 어절을 가능케 하는 직교 경로.
    pub fn effective_tokenized_fields(&self) -> HashSet<String> {
        let mut set: HashSet<String> = self
            .indexed_fields
            .iter()
            .filter(|f| f.tokenize)
            .map(|f| f.name.clone())
            .collect();
        set.extend(self.tokenize_fields.iter().cloned());
        set
    }

    /// 필수 인덱스 필드 이름 집합.
    pub fn required_field_names(&self) -> HashSet<String> {
        self.indexed_fields
            .iter()
            .filter(|f| f.kind == IndexedFieldKind::Required)
            .map(|f| f.name.clone())
            .collect()
    }

    /// 직교 플래그에서 derive 되는 표시용 라벨.
    /// FE 응답 / 로그 / consistency 라벨 등에 사용.
    pub fn derived_index_state(&self) -> IndexState {
        if !self.indexed {
            IndexState::NotIndexed
        } else if self.field_subset {
            IndexState::PartialField
        } else {
            IndexState::Full
        }
    }
}

// ── TopicFieldMetaRaw — deserialize 호환 layer ────────────────────────────

/// `TopicFieldMeta` 의 deserialize 호환 layer.
/// 세션 25 ILM 리뉴얼 이후로는 `indexed`/`fieldSubset` 만 신경.
/// `time_subset` / `index_state` 는 deprecated — 사용자가 워크스페이스 reset 으로 처리.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct TopicFieldMetaRaw {
    topic: String,
    key_fields: Vec<String>,
    payload_fields: Vec<String>,
    header_fields: Vec<String>,
    message_count: usize,
    index_entry_count: usize,

    indexed: Option<bool>,
    field_subset: Option<bool>,

    latest_indexed_offsets: HashMap<u32, i64>,
    earliest_offsets: HashMap<u32, i64>,
    kafka_latest_offsets: HashMap<u32, i64>,
    topic_type: Option<String>,
    topic_policy_checked_at: Option<i64>,
    last_incremental_sync_at: Option<i64>,
    last_cleanup_at: Option<i64>,
    retention_priority: Option<i32>,
    last_search_at: Option<i64>,
    field_search_stats: HashMap<String, FieldSearchStat>,
    retention_ms: Option<i64>,
    retention_bytes: Option<i64>,
    compression_type: Option<String>,
    message_timestamp_type: Option<String>,
    partition_count: Option<usize>,
    replication_factor: Option<usize>,
    topic_id: Option<String>,
    #[serde(
        alias = "cleanupPolicies",
        deserialize_with = "deserialize_cleanup_policy"
    )]
    cleanup_policy: Option<CleanupPolicy>,
    max_count: Option<u64>,
    #[serde(deserialize_with = "deserialize_indexed_fields")]
    indexed_fields: Vec<IndexedField>,
    auto_cleanup_removed_at: Option<i64>,
    /// 신규 (세션 40): key/value 페어.
    deserializers: Option<TopicDeserializers>,
    /// 레거시 (세션 35~40 사이): 단일 spec — value 전용으로 해석해 자동 migrate.
    deserializer: Option<DeserializerSpec>,
    /// 어절 대상 필드 (indexed_fields 직교). 구버전 JSON 엔 없어 default 빈 vec.
    tokenize_fields: Vec<String>,
    /// compact 토픽 dedup 누적 카운트. 구버전 JSON 엔 없어 default 0.
    compact_superseded_total: u64,
    /// compact 토픽 tombstone 처리 누적 카운트. 구버전 JSON 엔 없어 default 0.
    compact_tombstone_total: u64,
    /// sync fetch 배달 구멍(소실) 누적 실측. 구버전 JSON 엔 없어 default 0.
    delivery_lost_total: u64,
}

impl From<TopicFieldMetaRaw> for TopicFieldMeta {
    fn from(raw: TopicFieldMetaRaw) -> Self {
        let indexed = raw.indexed.unwrap_or(false);
        let field_subset = raw.field_subset.unwrap_or(false);
        TopicFieldMeta {
            topic: raw.topic,
            key_fields: raw.key_fields,
            payload_fields: raw.payload_fields,
            header_fields: raw.header_fields,
            message_count: raw.message_count,
            index_entry_count: raw.index_entry_count,
            indexed,
            field_subset,
            latest_indexed_offsets: raw.latest_indexed_offsets,
            earliest_offsets: raw.earliest_offsets,
            kafka_latest_offsets: raw.kafka_latest_offsets,
            topic_type: raw.topic_type,
            topic_policy_checked_at: raw.topic_policy_checked_at,
            last_incremental_sync_at: raw.last_incremental_sync_at,
            last_cleanup_at: raw.last_cleanup_at,
            retention_priority: raw.retention_priority,
            last_search_at: raw.last_search_at,
            field_search_stats: raw.field_search_stats,
            retention_ms: raw.retention_ms,
            retention_bytes: raw.retention_bytes,
            compression_type: raw.compression_type,
            message_timestamp_type: raw.message_timestamp_type,
            partition_count: raw.partition_count,
            replication_factor: raw.replication_factor,
            topic_id: raw.topic_id,
            cleanup_policy: raw.cleanup_policy,
            max_count: raw.max_count,
            indexed_fields: raw.indexed_fields,
            auto_cleanup_removed_at: raw.auto_cleanup_removed_at,
            deserializers: raw.deserializers.or_else(|| {
                // legacy 단일 spec → { key: Json, value: legacy } 로 마이그레이션
                raw.deserializer.map(|value| TopicDeserializers {
                    key: DeserializerSpec::Json,
                    value,
                })
            }),
            tokenize_fields: raw.tokenize_fields,
            compact_superseded_total: raw.compact_superseded_total,
            compact_tombstone_total: raw.compact_tombstone_total,
            delivery_lost_total: raw.delivery_lost_total,
        }
    }
}

// ── serde helpers ─────────────────────────────────────────────────────────

/// 레거시 JSON (`Vec<String>`) ↔ 신규 JSON (`Vec<IndexedField>`) 모두 수용하는 custom deserializer.
fn deserialize_indexed_fields<'de, D>(deserializer: D) -> Result<Vec<IndexedField>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either {
        Name(String),
        Full(IndexedField),
    }

    let raw: Option<Vec<Either>> = Option::deserialize(deserializer)?;
    let list = raw.unwrap_or_default();
    let mut out = Vec::with_capacity(list.len());
    for item in list {
        match item {
            Either::Name(name) => out.push(IndexedField::optional(name)),
            Either::Full(f) => {
                if f.name.is_empty() {
                    return Err(D::Error::custom("indexed field name must not be empty"));
                }
                out.push(f);
            }
        }
    }
    Ok(out)
}

/// `cleanup_policy` 역직렬화: `null` / 단일값 / Vec (구버전) 모두 수용.
/// Vec 이면 첫 번째 항목만 사용.
///
/// **세션 25 ILM 리뉴얼**: FE/IPC 호환을 위해 plain string (`"count_based"`) 도 수용.
/// 이 경우 `CountBased { max_count: 0 }` sentinel 로 복원되며, 실제 한도는
/// 형제 필드 `max_count` (또는 글로벌 `default_keep_count_per_partition`) 가 결정한다.
/// (`set_topic_meta_config` / `set_global_ilm_config` 가 사용하는 동일한 sentinel 규칙)
fn deserialize_cleanup_policy<'de, D>(deserializer: D) -> Result<Option<CleanupPolicy>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AnyShape {
        One(CleanupPolicy),
        Many(Vec<CleanupPolicy>),
        Tag(String),
    }

    let v: Option<AnyShape> = Option::deserialize(deserializer)?;
    Ok(match v {
        None => None,
        Some(AnyShape::One(p)) => Some(p),
        Some(AnyShape::Many(v)) => v.into_iter().next(),
        Some(AnyShape::Tag(s)) => match s.as_str() {
            "drop_index" => Some(CleanupPolicy::DropIndex),
            "field_based" => Some(CleanupPolicy::FieldBased),
            "count_based" => Some(CleanupPolicy::CountBased { max_count: 0 }),
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "unknown cleanup_policy tag: {s}"
                )))
            }
        },
    })
}

/// `cleanup_policy` 직렬화: 항상 discriminator string 만 emit (변종 내부 데이터 drop).
/// FE 는 `cleanupPolicy: string | null` 로 가정하며, CountBased 의 실제 한도는
/// 형제 필드 `max_count` 로 별도 노출되므로 정보 손실 없음.
fn serialize_cleanup_policy_as_tag<S>(
    value: &Option<CleanupPolicy>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let tag: Option<&'static str> = value.as_ref().map(|p| match p {
        CleanupPolicy::DropIndex => "drop_index",
        CleanupPolicy::FieldBased => "field_based",
        CleanupPolicy::CountBased { .. } => "count_based",
    });
    tag.serialize(serializer)
}

// ── Topic Meta read DTOs ──────────────────────────────────────────────────
//
// `kaflow-engine-impl::topic_meta` 의 read API 가 반환하는 응답 타입들.
// Phase 3 trait redesign (`docs/trait_redesign_2026_05.md`) 에 따라 engine-impl 에서
// 이전됨.

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicIndexSize {
    pub topic: String,
    pub index_bytes: u64,
    pub meta_bytes: u64,
    pub total_bytes: u64,
    pub index_entry_count: u64,
    pub meta_entry_count: u64,
    pub total_entry_count: u64,
}

/// CF per topic 마이그레이션 가드 — 워크스페이스 storage layout 점검 결과.
/// FE 가 connect 시점에 호출해 v1 (legacy 단일 CF + prefix) 인지 판정한다.
/// `requires_reset=true` 면 reset_workspace 모달을 띄워야 한다.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStorageStatus {
    pub schema_version: u32,
    pub current_schema_version: u32,
    pub topic_cf_count: usize,
    pub has_legacy_data: bool,
    pub requires_reset: bool,
    pub reason: String,
}

/// 파티션별 offset 상태 (kafka latest vs DB max/min indexed)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionOffsetStatus {
    pub partition: i32,
    pub earliest: i64,
    pub latest: i64,
    /// DB에서 확인된 최소 indexed offset. None = 아직 아무것도 인덱싱 안됨.
    /// CountBased trim / PartialTime boundary 등으로 인해 `earliest` 보다 클 수 있음.
    pub min_indexed_offset: Option<i64>,
    /// 처리 완료된 최대 offset — M-key 최대(M-seek)와 T-META 처리 watermark 의 max.
    /// tombstone 처럼 저장 없이 처리된 offset 도 포함 ("어디까지 진행했나").
    /// None = 아직 아무것도 인덱싱/처리 안됨.
    pub max_indexed_offset: Option<i64>,
    /// 이 파티션의 미인덱싱 메시지 수 (`max_indexed_offset+1 .. latest`)
    pub gap: i64,
}

/// 토픽 전체 offset 상태 요약
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicOffsetStatus {
    pub topic: String,
    /// kafka 클러스터 실제 total (sum of latest - earliest per partition)
    pub kafka_total: i64,
    /// DB에 인덱싱된 메시지 수 추정
    pub indexed_total: i64,
    /// 인덱싱이 커버(진행)한 offset 범위 크기 — Σ per partition `(max_indexed − earliest + 1)`
    /// (현재 서버 earliest 기준 clamp). record 수가 아니라 **범위**라서 compact 중복 dedup /
    /// broker 선행 compaction / 트랜잭션 control record 와 무관하게 "어디까지 훑었나" 를 나타냄.
    /// 항등식: `kafka_total = processed_total + gap`.
    #[serde(default)]
    pub processed_total: i64,
    /// 신규 미인덱싱 메시지 수 (catch-up gap, partition 별 latest - max_indexed - 1 합)
    pub gap: i64,
    /// 정책으로 인덱싱하지 않은 메시지 수 (kafka_total - indexed_total - gap).
    /// PartialTime 의 boundary 이전 옛 데이터 등.
    pub skipped: i64,
    /// 사유별 skip 분해. partition 별 offset range + 추정 count.
    /// `skipped` 숫자와 합산값이 일치하지 않을 수 있음 (compaction stale 등 derive 불가 영역 존재).
    pub skip_breakdown: SkipBreakdown,
    /// 모든 파티션이 최신 offset까지 인덱싱됨
    pub is_caught_up: bool,
    pub partitions: Vec<PartitionOffsetStatus>,
}

/// skip 사유 분류.
///
/// 사유 카테고리는 derive 가능한 것만 enumerate. cleanup 으로 사라진 이력은 derive 불가라
/// 별도 영속 저장 없이 ILM 로그로만 보존 (Option C 영역).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// retention 으로 broker earliest 가 우리 indexed 범위 앞으로 가버린 구간.
    /// `[max_indexed+1, earliest)` — 인덱싱이 닿기도 전에 사라졌거나, 인덱스 후 broker 측이 정리.
    Retention,
    /// CountBased 정책의 `latest - effective_max` floor 보다 앞에 있는 옛 offset.
    /// `[earliest, count_floor)` — 의도적 skip (정책으로 인덱싱 대상 아님).
    CountBased,
    /// FullResync 감지 → 인덱싱 차단 중인 구간 (사용자 승인 대기).
    /// 전체 partition 의 indexed 범위 밖.
    FullResyncPending,
    /// compact 토픽 dedup-on-write 로 superseded(옛 버전) 삭제된 수 (정상 — 오해 아님).
    /// = `TopicFieldMeta.compact_superseded_total`. "인덱싱 처리는 했으나 key 당 최신만 남김".
    /// range 는 흩어져 있어 derive 불가 → totals 로만 표시.
    CompactDeduped,
    /// compact 토픽 tombstone(null value = key 삭제 마커) 처리 수 (정상).
    /// = `TopicFieldMeta.compact_tombstone_total`. "처리했으나 삭제 마커라 저장 안 함".
    /// range 는 흩어져 있어 derive 불가 → totals 로만 표시.
    CompactTombstone,
    /// sync fetch 가 훑은 범위에서 broker 가 배달하지 않은 offset 자리(배달 구멍) 실측.
    /// = `TopicFieldMeta.delivery_lost_total`. broker compaction 소실 / 트랜잭션 control
    /// record 합산 (구분 불가). "소실" 버킷의 실측 부분 — 잔여 추정은 CompactGapOrUnknown.
    /// range 는 흩어져 있어 derive 불가 → totals 로만 표시.
    DeliveryLost,
    /// compact 토픽 derive 불가 영역 (broker 측 compaction stale 등).
    /// kafka_total - indexed_total - gap - (위 사유 합) 의 잔여.
    CompactGapOrUnknown,
}

/// 단일 partition 의 skip range. `start..end` 는 반열림 구간 (Kafka offset 컨벤션).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffsetRangeSkip {
    pub partition: i32,
    pub start: i64,
    pub end: i64,
    /// `end - start`. 0 이면 표시 생략 가능.
    pub count: i64,
}

/// 사유별 skip 분해. derive 가능한 사유만 enumerate.
/// 표시는 FE 가 tooltip / 상세 패널에 활용.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipBreakdown {
    /// 사유별 partition × range 목록.
    pub by_reason: Vec<SkipReasonGroup>,
    /// 사유별 합산 count — UI 요약용 (FE 가 by_reason 에서 derive 도 가능).
    pub totals: Vec<SkipReasonTotal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipReasonGroup {
    pub reason: SkipReason,
    pub ranges: Vec<OffsetRangeSkip>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipReasonTotal {
    pub reason: SkipReason,
    pub count: i64,
}

/// 토픽의 Kafka 설정을 조회한 결과 — `topic_meta::get_topic_config_info` 응답.
/// cleanup.policy / retention.ms / retention.bytes / compression.type / message.timestamp.type
/// 토픽 선택 시마다 호출해 topic_type을 최신 상태로 유지하며,
/// ILM resolve_ilm_actions가 올바른 분기를 타도록 한다.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicConfigInfoResponse {
    pub topic: String,
    pub cleanup_policy: String,
    pub retention_ms: Option<i64>,
    pub retention_bytes: Option<i64>,
    pub compression_type: Option<String>,
    pub message_timestamp_type: Option<String>,
    pub partition_count: usize,
    pub replication_factor: usize,
    /// Kafka 토픽 UUID (Metadata v9). 토픽 삭제 후 재생성 시 변경됨.
    pub topic_id: Option<String>,
    pub checked_at: i64,
}

/// `set_indexed_fields` 입력 — 토픽별 인덱싱 대상 필드 설정.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedFieldInput {
    pub name: String,
    #[serde(default)]
    pub kind: IndexedFieldKind,
    #[serde(default)]
    pub is_representative: bool,
    /// 어절 토큰화 대상 여부 (세션 36). 자세한 의미는 `IndexedField::tokenize`.
    #[serde(default)]
    pub tokenize: bool,
}

/// `fetch_registry_subject_schema` 응답 — Confluent Schema Registry 의 `{topic}-{side}`
/// subject 의 최신 schema 정보.
///
/// TopicMetaConfigDrawer 의 read-only 미리보기 용도. 결정/저장은 안 함.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySubjectSchema {
    /// 호출된 subject 이름 (e.g. `"orders-value"`).
    pub subject: String,
    /// Registry 가 부여한 schema id.
    pub schema_id: u32,
    /// subject 내 버전 번호.
    pub version: i32,
    /// AVRO schema text (JSON 표현 그대로).
    pub schema_text: String,
}

/// 인덱스 대상 선택(picker) 시 토픽의 대략적 규모를 보여주기 위한 경량 카운트.
/// Kafka ListOffsets 메타데이터만으로 산출 — record fetch / RocksDB 접근(CF 생성) 없음.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicMessageCount {
    pub topic: String,
    /// sum(latest - earliest) over partitions. compact 토픽은 실제 보관 건수보다 클 수 있음(상한 추정).
    pub message_count: i64,
    pub partition_count: u32,
}

/// 워치 토픽의 인덱싱 지연(lag) 폴링 결과 — 칩 lag 배지용 경량 DTO.
///
/// `behind` = Kafka latest offset 합 − (인덱싱된 마지막 offset+1) 합. 즉 "마지막 인덱싱
/// 이후 새로 들어왔지만 아직 인덱싱되지 않은 메시지" 추정 수. `reachable=false` 면 이번
/// 폴링에서 해당 토픽의 latest offset 을 얻지 못함(토픽 없음/연결 실패) → `behind` 무의미(0).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicLagStatus {
    pub topic: String,
    /// 미인덱싱 추정 메시지 수(>=0). reachable=false 면 0.
    pub behind: i64,
    /// 이번 폴링에서 Kafka latest offset 을 받았는지.
    pub reachable: bool,
}

/// 인덱스 대상 선택(picker) 시 인덱싱 정책 추천 입력값으로 쓰는 경량 사이즈 프로파일.
/// 토픽에서 샘플 메시지 N개를 fetch 하여 **raw bytes 평균**만 산출한다 (deserialize 불필요 —
/// 정책 추천은 deserializer 선택 전 단계라 raw 바이트가 deserializer-무관한 "구조 복잡도" proxy).
/// `cleanup_policy` 는 DescribeConfigs 의 표시 전용 값(실패 시 None).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicSizeProfile {
    pub topic: String,
    /// 샘플 메시지당 평균 바이트 (key + value + headers). 샘플 0건이면 0.
    pub avg_bytes: u64,
    /// 실제 fetch 된 샘플 메시지 수.
    pub sampled: u32,
    /// Kafka 토픽의 `cleanup.policy` ("delete" | "compact" | "compact,delete"). 조회 실패 시 None.
    pub cleanup_policy: Option<String>,
    /// 샘플 value 바이트 휴리스틱으로 추정한 value deserializer kind (DeserializerSpec discriminator).
    /// 예: "json" / "json_connect_envelope" / "json_schema_confluent" / "avro_confluent_registry" /
    /// "protobuf_confluent_registry". 확신 불가/샘플 없음이면 None. **추천일 뿐 — 사용자 확인 필요.**
    pub suggested_value_deserializer: Option<String>,
    /// 추천 근거 한 줄 (UI tooltip). None 이면 추천 없음.
    pub suggested_reason: Option<String>,
    /// 샘플 key 바이트 휴리스틱으로 추정한 key deserializer kind (value 와 동일 분류).
    /// key=null(키 없음) 다수면 None. **추천일 뿐 — 사용자 확인 필요.**
    pub suggested_key_deserializer: Option<String>,
    /// key 추천 근거 한 줄. None 이면 추천 없음.
    pub suggested_key_reason: Option<String>,
}

/// 어절(tokenize) 대상 필드 추천 — picker 시점(인덱싱 전) 샘플 기반.
///
/// 토픽 샘플 N개를 선택된 deserializer 로 decode → `flatten_json` 으로 필드별 텍스트를 모아,
/// "긴 텍스트 필드"(평균 길이 ↑ + 공백 多, UUID/numeric/enum 제외)를 어절 후보로 랭크한다.
/// **추천일 뿐 — 사용자가 picker 에서 1개(출시) 선택**. 초기 인덱싱부터 어절 적용 진입점.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestTokenizeFieldsResponse {
    pub topic: String,
    /// 실제 decode 에 성공해 필드 추출에 쓰인 샘플 수. 0 이면 후보 없음(추천 불가).
    pub sampled: u32,
    /// 어절 후보 필드 (점수 내림차순). 비-텍스트(numeric/uuid/enum/짧은 식별자)는 제외.
    pub candidates: Vec<TokenizeFieldCandidate>,
}

/// 어절 후보 1개. `recommended=true` 는 최상위 추천(자동선택 seed).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenizeFieldCandidate {
    /// 필드 경로 (예: "P.message"). `extract_all_fields` 의 필드명과 동일.
    pub field: String,
    /// 샘플에서 관측된 평균 문자 길이.
    pub avg_len: u32,
    /// 샘플에서 관측된 평균 공백(어절 구분) 수 — 다중 어절 가능성 지표.
    pub avg_whitespace: u32,
    /// 최상위 추천 여부 (점수 1위 1개만 true). FE 자동선택 seed.
    pub recommended: bool,
}
