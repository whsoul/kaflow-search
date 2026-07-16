//! Core 도메인 DTO — 메시지 위치 / 인덱스 키 / 검색 hit / 필드 메타.

use serde::{Deserialize, Serialize};

// ── Core message location / meta ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocLoc {
    pub partition: u32,
    pub offset: u64,
}

/// M-key value — 메시지 1건의 토크나이즈 전 원문(field별 leaf value) + 토크나이즈 flag.
///
/// **세션 36 구조 변경**: 이전 `index_entries`(쪼갠 (field,term) 목록)를 제거하고,
/// 토크나이즈 전 leaf value 를 들고 있는 `field_values` 로 교체했다. I-key 참조 목록은
/// 영속 저장하지 않고 `field_values` + `tokenized_fields` 에서 derive 한다.
/// - 검색축 = I-key (term 정렬) / 복원축 = M-key (loc point lookup) — 정렬축 분리 유지.
/// - 원문 복원 책임은 M-key 가 진다. I-key 는 검색용 derived index (lossy 허용).
///
/// 이전 구조와 **비호환** — 기존 워크스페이스는 reset 후 재인덱싱이 필요하다
/// (`WorkspaceStorageStatus` schema_version 으로 감지).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaValue {
    pub ts_millis: u64,
    /// (field, 토크나이즈 전 leaf value) 목록. 원문 복원 / 검색 후처리 / I-key derive 의 source.
    /// topic/partition/offset/revTs 는 M-key 컨텍스트에서 재구성.
    #[serde(default)]
    pub field_values: Vec<(String, String)>,
    /// 이 메시지에서 어절 토큰화된 필드 이름 목록. `field_values` 의 값을 토크나이저
    /// 동일 규칙으로 쪼개 어절 I-key 참조를 derive 할 때 사용. 비어있으면 전부 통짜 인덱싱.
    #[serde(default)]
    pub tokenized_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaRow {
    pub topic: String,
    pub partition: u32,
    pub offset: u64,
    pub value: MetaValue,
}

// ── Search hit / decoded index key ────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SearchHit {
    pub rev_ts: u64,
    pub loc: DocLoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHitWithKey {
    pub raw_key_hex: String,
    pub pretty_key: String,
    pub hit: SearchHit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedIndexKey {
    pub topic: String,
    pub field: String,
    pub term: String,
    pub rev_ts: u64,
    pub partition: u32,
    pub offset: u64,
}

// ── Index state / field index meta ────────────────────────────────────────

/// 토픽 인덱스 상태 — `TopicFieldMeta` 의 `indexed` / `field_subset` 직교 플래그에서
/// derive 되는 표시용 라벨. 코드 분기는 가급적 직교 플래그를 직접 보고,
/// 이 enum 은 외부 응답 / 라벨 표시에만 사용한다.
///
/// **세션 25 ILM 리뉴얼**: `PartialTime` / `Both` 제거. 시간 차원은 별도 임시 인덱스로 분리 (후속 작업).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum IndexState {
    #[default]
    NotIndexed,
    /// 모든 필드 완전 인덱싱 완료
    Full,
    /// 일부 필드만 인덱싱 — FieldBased SizeCleanup 으로 일부 필드가 제거된 상태
    PartialField,
}

/// 필드별 검색 사용 통계 — 어떤 필드 인덱스가 실제로 사용되는지 추적
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FieldSearchStat {
    pub search_count: u64,
    pub last_searched_at: Option<i64>,
}

/// 인덱스 대상 필드의 유형 구분자.
/// FieldBased SizeCleanup 선정 시 `Required`는 삭제 후보에서 제외한다.
/// `Async`는 "지연 인덱싱" 을 위한 향후 확장 플레이스홀더 (현재는 Optional 과 동일 취급).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexedFieldKind {
    /// 필수 인덱스 필드 — 크기 기반 부분 정리에서도 절대 삭제되지 않음
    Required,
    /// 옵셔널 인덱스 필드 — 기본값. 크기 기반 정리 대상으로 포함 가능
    Optional,
    /// 비동기 인덱싱 대상 (향후 확장) — 현재는 Optional 과 동일 취급
    Async,
}

impl Default for IndexedFieldKind {
    fn default() -> Self {
        Self::Optional
    }
}

/// 인덱스 대상 필드 엔트리 — (이름, 유형, 대표 플래그).
///
/// 레거시 저장값(`["foo", "bar"]` 형태의 `Vec<String>`) 호환을 위해
/// `topic_meta::deserialize_indexed_fields` 에서 두 형식 모두 읽는다.
///
/// `is_representative` 는 식별용 "대표 필드" 마커. list view 의 displayCols 통합과
/// 향후 탐색 모드 leaf 라벨에 활용된다. kind 와 직교 (Required + Representative 동시 부여 가능).
/// 메시지 본문 fetch 후 derive 만 하므로 인덱싱 / I-key 발급 정책에는 영향 없음.
///
/// `tokenize` 는 어절 토큰화 대상 마커 (세션 36). true 면 인덱싱 시 leaf value 를
/// 어절(공백/구두점 단위)로 쪼개 어절마다 I-key 를 발급한다 — 문장 중간 단어 검색용.
/// false 면 leaf value 통짜로 단일 I-key. `is_representative` / `kind` 와 직교.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedField {
    pub name: String,
    #[serde(default)]
    pub kind: IndexedFieldKind,
    #[serde(default)]
    pub is_representative: bool,
    #[serde(default)]
    pub tokenize: bool,
}

impl IndexedField {
    pub fn optional(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: IndexedFieldKind::Optional,
            is_representative: false,
            tokenize: false,
        }
    }

    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: IndexedFieldKind::Required,
            is_representative: false,
            tokenize: false,
        }
    }
}
