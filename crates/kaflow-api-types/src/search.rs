//! Search 도메인 DTO — 검색 / browse / prefetch / fetch_messages 시그니처.
//!
//! `kaflow-engine-impl` 의 `search` / `browse` 도메인 함수가 입출력으로 사용하고,
//! Tauri command / 미래 HTTP handler 도 동일 타입을 직렬화한다.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchMode {
    Exact,
    Prefix,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    NewestFirst,
    OldestFirst,
}

/// 브라우즈 모드 페이지네이션 커서 (topic + partition + offset)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionCursor {
    pub topic: String,
    pub partition: u32,
    pub offset: u64,
}

/// 키워드 검색 모드 페이지네이션 커서 ((topic, field) 단위 마지막 I-key)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFieldCursor {
    pub topic: String,
    pub field: String,
    pub last_key_hex: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexRequest {
    pub query: String,
    /// 검색 대상 필드 목록 (OR 매칭).
    /// `None` = 인덱싱된 모든 필드, `Some(empty)` = 결과 없음, `Some(vec![...])` = 지정 필드들.
    pub fields: Option<Vec<String>>,
    pub topics: Option<Vec<String>>,
    pub mode: SearchMode,
    pub sort_order: SortOrder,
    pub limit: usize,
    /// 브라우즈 모드 커서 (빈 검색 시 사용)
    #[serde(default)]
    pub browse_cursors: Vec<PartitionCursor>,
    /// 키워드 검색 모드 커서 (검색어 있을 때 사용)
    #[serde(default)]
    pub search_cursors: Vec<SearchFieldCursor>,
    /// 메시지 생성시간 범위 (글로벌 선행 필터). multi_search 의 `ts_range` 와 동일.
    /// browse 모드는 value.ts_millis 로 in-iter filter, keyword 모드는 cursor seek/stop.
    #[serde(default)]
    pub ts_range: Option<TsRange>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResult {
    pub matched_field: Option<String>,
    pub raw_index_key_hex: Option<String>,
    pub pretty_index_key: Option<String>,
    pub raw_meta_key_hex: String,
    pub pretty_meta_key: String,
    /// "full" = Kafka 까지 다녀온 payload/key/headers 포함.
    /// "meta-only" = RocksDB M-key 만 — list 표시용. payload/key/headers 는 빈 값.
    /// "missing" = fetch 시도했지만 Kafka 가 record 반환 안 함 (retention 등).
    pub content_source: String,
    /// browse 모드 전용: M-key에서 복원한 (field, pretty_index_key) 목록 (I-key 칩 표시용)
    pub meta_index_entries: Vec<(String, String)>,
    /// M-key value 실제 저장 JSON (raw 덤프용)
    pub raw_meta_value_json: Option<String>,
    /// 인덱싱된 필드의 (path, leaf_value) 목록 — list row 의 대표 필드 표시용.
    /// Kafka 호출 없이 list 표시 가능하게 하기 위해 M-key 의 `field_values` 를 그대로 전달.
    /// path 형식: `K`, `K.foo`, `P`, `P.bar`, `H.x` (lookupField 와 동일 notation).
    #[serde(default)]
    pub field_values: Vec<(String, String)>,
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub timestamp: String,
    pub key: String,
    pub payload: String,
    pub headers: Vec<(String, String)>,
    pub json: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexResponse {
    pub results: Vec<MessageResult>,
    /// browse 모드: fetch_messages에 넘길 loc 목록 (results는 비어있음)
    pub locs: Vec<LocItem>,
    pub has_more: bool,
    pub next_browse_cursors: Vec<PartitionCursor>,
    pub next_search_cursors: Vec<SearchFieldCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocItem {
    pub ts_millis: u64,
    pub partition: u32,
    pub offset: u64,
    /// 검색 히트가 있을 때만 설정됨. browse 모드에서는 None.
    /// browse 모드에서 1M 건 직렬화 시 약 60B/loc 의 `,"…":null` 비용을 없애기 위해 skip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pretty_index_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_index_key_hex: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefetchSearchLocsResponse {
    pub locs: Vec<LocItem>,
    pub total: usize,
    pub capped: bool,
    /// 검색어가 인덱스 entry 를 1건이라도 가졌던 필드 (전 토픽 union, dedup 이전 기준).
    /// 테이블 컬럼 / 미리보기 하이라이트의 필드 제한용. `MultiSearchResponse::hit_fields`
    /// 와 동일 의미.
    pub hit_fields: Vec<String>,
}

// ── 다중 키워드 검색 (multi-keyword search) ────────────────────────────────
//
// `docs/multi_keyword_search_design.md` Phase 2 — `I` key 기반 ES bool subset
// AST 검색. 엔진은 streaming merge-join + 재귀 그룹 평가 + exists / mustNot 차집합.
// browse/keyword 와 달리 단일 호출로 매치 loc 전체를 반환.
//
// 외부 태그 사용 — JSON 모양이 ES Query DSL 과 동일:
//   { "bool":   { "must": [...], "should": [...], "mustNot": [...] } }
//   { "term":   { "field": "f"|null, "value": "v" } }
//   { "prefix": { "field": "f"|null, "value": "v" } }
//   { "exists": { "field": "f" } }
//   { "range":  { "field": "f", "gte": "lo"|null, "lte": "hi"|null } }
//
// `range` 는 인덱스의 term (utf-8 bytes) lex 정렬 위에 동작한다. 즉 ISO 8601
// 날짜 / 같은 자릿수 zero-padded 숫자는 자연스럽게 동작하지만, 가변 자릿수
// 숫자(`"2"` vs `"10"`)는 lex 정렬상 잘못된 결과 — 사용자가 인덱싱 시점에
// 패딩해야 한다. gte / lte 둘 다 None 은 reject (exists 와 같아짐), gte > lte
// 도 reject.

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryNode {
    Bool(BoolNode),
    Term(TermLeaf),
    Prefix(PrefixLeaf),
    Exists(ExistsLeaf),
    Range(RangeLeaf),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BoolNode {
    #[serde(default)]
    pub must: Vec<QueryNode>,
    #[serde(default)]
    pub should: Vec<QueryNode>,
    #[serde(default)]
    pub must_not: Vec<QueryNode>,
    /// `should` 매치 최소 개수. None = 기본(1, must 가 비었을 때만 적용).
    #[serde(default)]
    pub minimum_should_match: Option<u32>,
}

/// `field: None` = 필드미지정(전체 인덱싱 필드 fan-out).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TermLeaf {
    pub field: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefixLeaf {
    pub field: Option<String>,
    pub value: String,
}

/// 인덱스 entry 가 1건이라도 있는 필드 매치. 1차 PR 미구현 — 검증기에서 reject.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistsLeaf {
    pub field: String,
}

/// term lex range — `field` 안에서 term ∈ [gte, lte] (양끝 inclusive) 매치.
/// cross-field 의미가 없어 `field` 필수. gte / lte 중 하나는 반드시 있어야
/// 한다 (둘 다 None 은 exists 와 동치 — reject). 인덱스는 utf-8 bytes lex 정렬.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeLeaf {
    pub field: String,
    pub gte: Option<String>,
    pub lte: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    /// `None` = 전체 토픽.
    pub topics: Option<Vec<String>>,
    pub r#where: QueryNode,
    pub sort_order: SortOrder,
    /// 결과 상한. `0` = 엔진 기본 cap.
    #[serde(default)]
    pub limit: usize,
    /// 메시지 생성시간(`ts_millis`) 범위 — 글로벌 선행 필터. ES bool 트리의 일부가
    /// 아니며 모든 매치에 곱해진다. cursor 의 시작점·끝점을 좁혀 효율적으로 적용.
    /// `None` 또는 `gte_ms`/`lte_ms` 모두 `None` 이면 필터 없음.
    #[serde(default)]
    pub ts_range: Option<TsRange>,
    /// 위치(partition/offset) 범위 — `ts_range` 와 같은 성질의 글로벌 선행 필터.
    /// bool 트리 밖이며 모든 매치에 곱해진다. R `__ts` cursor 가 ts 우선 정렬이라
    /// seek 로 좁힐 수 없어 hit emit 직전 **post-filter**(매치 소비는 정상, push 만 거름.
    /// `limit` 은 필터 후 카운트). `None` 또는 빈 조건이면 필터 없음.
    #[serde(default)]
    pub pos_filter: Option<PosFilter>,
}

/// 위치 필터 — partition 집합(빈 = 전체) ∧ offset 범위(inclusive, `None` = 무제한).
/// partition/offset 은 Kafka 메시지의 불변 좌표라 의미가 모드(browse/keyword/multi)·
/// 상황(상세검색 입력 / 차트 막대 drill)과 무관하게 동일 — 순수 위치 술어로 공유한다.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PosFilter {
    /// 허용 partition 집합. 빈 벡터 = 전체 partition (필터 없음).
    #[serde(default)]
    pub partitions: Vec<u32>,
    pub offset_gte: Option<u64>,
    pub offset_lte: Option<u64>,
}

impl PosFilter {
    /// 필터가 실질적으로 아무 것도 거르지 않으면 `true` (호출부가 `None` 으로 접어 비용 0 처리 가능).
    pub fn is_noop(&self) -> bool {
        self.partitions.is_empty() && self.offset_gte.is_none() && self.offset_lte.is_none()
    }

    /// 이 (partition, offset) 가 필터를 통과하는지. 공유 술어 — 모든 emit chokepoint 가 호출.
    pub fn passes(&self, partition: u32, offset: u64) -> bool {
        if !self.partitions.is_empty() && !self.partitions.contains(&partition) {
            return false;
        }
        if let Some(lo) = self.offset_gte {
            if offset < lo {
                return false;
            }
        }
        if let Some(hi) = self.offset_lte {
            if offset > hi {
                return false;
            }
        }
        true
    }
}

/// 메시지 생성시간 범위 (inclusive). lex 비교가 아닌 epoch ms 정수 비교.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TsRange {
    pub gte_ms: Option<u64>,
    pub lte_ms: Option<u64>,
}

/// 조건별 실행 비용 — 검증/디버깅용.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSearchCondCost {
    pub field: Option<String>,
    pub term: String,
    /// "exact" | "prefix"
    pub match_type: String,
    /// "iter" = lazy raw-iterator cursor, "mem" = 1회 materialize 한 cursor.
    pub cursor_kind: String,
    /// `Mem` cursor 가 메모리에 적재한 loc 수 (`Iter` 는 0).
    pub materialized: u64,
    /// 이 조건의 검색어가 인덱스 entry 를 1건이라도 가졌던 필드 (combine 이전 기준).
    /// 필드미지정 조건은 fan-out 된 필드들. AND 최종 결과와 무관 — 조건 단독 매치.
    pub hit_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSearchResponse {
    /// 매치 loc 전체 (sort_order 적용, cap 한도). browse loc 과 동일하게
    /// matched_field/index_key 는 None — list 는 fetch_meta_rows 로 본문화.
    pub locs: Vec<LocItem>,
    pub total: usize,
    pub capped: bool,
    /// QueryPlanner 가 선택한 전략 — 현재는 항상 "streaming".
    pub strategy: String,
    pub per_condition: Vec<MultiSearchCondCost>,
    /// 이번 검색에서 검색어가 인덱스 entry 를 1건이라도 가졌던 필드 (전 토픽 union).
    /// 필드미지정(전체필드) 조건의 fan-out 결과도 포함 — 테이블 컬럼 하이라이트용.
    pub hit_fields: Vec<String>,
    pub scan_ms: u64,
    /// 입구 gate 정규화로 조건식이 **변경된 경우** 실제 실행된 식의 압축 표현
    /// (flatten/unwrap/dedup 후). 변경 없으면 None. FE 는 App LOG 로 노출.
    pub normalized_where: Option<String>,
}

// ── Offset buckets (flat leaf=100 단위, hierarchical 은 FE 빌드) ───────────
//
// 검색 / browse 매치의 partition × offset 단위 집계. 응답은 leaf (보통 100 offset
// 단위) 만 sparse 하게 담는다 (count > 0 인 것만). 사용처:
//   - Explore "Partition" 트리 — FE 가 leaf 를 자릿수별 (1억/1000만/.../1000) 그룹핑
//   - Map 셀 시각화 — cellSize ≥ 100 이면 leaf 합산 정확, < 100 이면 균등분배 근사
//   - drill — 한 partition 의 한 범위 [lo, hi] 안의 leaf 만 응답

/// 한 partition × offset 범위의 bucket. 응답 안에 수천~수만개 반복되므로 IPC
/// 페이로드 절감 위해 필드명 약어 적용 (`f` / `t` / `c` / `os`).
#[derive(Debug, Clone, Serialize)]
pub struct OffsetBucket {
    /// 범위 시작 offset (inclusive). 보통 bucket_unit 의 배수.
    #[serde(rename = "f")]
    pub from_offset: u64,
    /// 범위 끝 offset (inclusive). 보통 `from_offset + bucket_unit - 1`. drill
    /// 응답에서 마지막 bucket 이 partition max_offset 으로 잘리는 케이스 + bucket_unit
    /// 향후 변경 대비 명시 포함.
    #[serde(rename = "t")]
    pub to_offset: u64,
    /// 이 범위 안의 매치 갯수. sparse 응답이므로 `count > 0` 만 포함됨.
    #[serde(rename = "c")]
    pub count: u32,
    /// 이 범위 안 매치들의 실제 offsets — `OffsetsInline` 모드만 채움.
    /// FE 는 응답 안에서 local slice 로 leaf 본문 fetch 가능 (drill invoke 0).
    /// `CountOnly` 모드면 `None` (직렬화 시 skip).
    #[serde(rename = "os", skip_serializing_if = "Option::is_none")]
    pub offsets: Option<Vec<u64>>,
}

/// 한 partition 의 offset bucket 집계. leaf (보통 100 단위) 만 담는다 —
/// hierarchical level (1000 / 10000 / ...) 은 FE 가 derive.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionOffsetBuckets {
    pub partition: u32,
    pub count: u64,
    pub min_offset: u64,
    pub max_offset: u64,
    /// bucket 의 offset 단위 (보통 100). 향후 가변 가능성 대비 응답에 명시.
    pub bucket_unit: u32,
    pub buckets: Vec<OffsetBucket>,
}

/// `OffsetBucketsResponse` 의 전달 모드 — FE 가 drill 흐름을 결정할 때 명시 신호로 사용.
///
/// **버킷의 `offsets` 가 None 인 경우의 모호성을 제거**한다:
///   - `CountOnly` — 모든 leaf 가 `offsets = None`. FE 는 leaf 펼침 시 drill 호출 필요.
///   - `OffsetsInline` — 모든 leaf 의 `offsets` 가 `Some(Vec<u64>)`. FE 는 local slice 로
///     leaf 본문 fetch 가능 (drill 호출 0).
///   - `PartialInline` — **버킷별 혼재** (메모리 cap 안에서 채운 버킷만 inline). FE 는 **버킷별**로
///     `offsets` 가 `Some` 이면 local slice, `None`(count-only)이면 그 셀만 drill. (cap 초과분/불완전
///     버킷은 서버가 strip 해 `None` 으로 내려보냄 → `Some` 은 항상 완전.) `PLANNER_DECISIONS.md §16-8`.
///   - `Drill` — drill 응답 (한 partition 의 한 [lo, hi] 범위). 펼침 흐름과 다름.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OffsetBucketsDeliveryMode {
    CountOnly,
    OffsetsInline,
    PartialInline,
    Drill,
}

/// partition × offset 단위 bucket aggregate 응답. ExploreTab Partition 트리 +
/// MapTab 셀 시각화의 공용 데이터 source.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OffsetBucketsResponse {
    pub topic: String,
    pub total: u64,
    pub partitions: Vec<PartitionOffsetBuckets>,
    /// **FE 분기 결정용 명시 모드**. 버킷별 `offsets` 유무로 추정 금지.
    pub delivery_mode: OffsetBucketsDeliveryMode,
    /// `query_planner::offset_buckets` 가 선택한 plan 이름. 디버그/로깅용.
    /// 예: `"offset_buckets.count_only"` / `"offset_buckets.offsets_inline"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_plan: Option<String>,
}

/// R `__ts` cursor 기반 **시간 bucket** 집계 1행 — `(partition, gran_start_ms)` 별 메시지 수.
/// LocItem 본문 없이 카운트만 (시계열/탐색(시간) 골격용). browse 모드 전용
/// (R 은 모든 메시지 색인 — 쿼리 매치 필터 X).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBucketRow {
    pub partition: u32,
    /// bucket 시작 epoch ms — `floor(ts / gran_ms) * gran_ms`.
    pub gran_start_ms: u64,
    pub count: u64,
    /// `OffsetsInlinePlan` (threshold 안) 일 때만 채워짐. 이 bucket 안 메시지들의 offsets.
    /// 채워졌으면 FE 가 chart bar 클릭 drill 시 invoke 없이 local slice → LocItem 합성 →
    /// fetch_meta_rows 직행. CountOnly fallback 시 None (직렬화 skip).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offsets: Option<Vec<u64>>,
}

/// `TimeBucketsResponse` 의 전달 모드 — FE 가 chart bar drill 흐름을 결정할 때 명시 신호.
///   - `CountOnly` — 모든 bar 의 `offsets = None`. FE 는 bar 클릭 시 drill 호출 (현재 동작).
///   - `OffsetsInline` — 모든 bar 의 `offsets = Some(Vec<u64>)` (count=0 이면 None). FE 는
///     local slice 만으로 LocItem 합성 가능 (drill invoke 0).
///   - `PartialInline` — **bar 별 혼재** (cap 안에서 채운 bar 만 inline). FE 는 **bar 별**로
///     `offsets` 가 `Some` 이면 local slice, `None` 이면 그 bar 만 drill. (불완전 bar 는 서버가 strip
///     → `Some` 은 항상 완전.) `PLANNER_DECISIONS.md §16-8`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimeBucketDeliveryMode {
    CountOnly,
    OffsetsInline,
    PartialInline,
}

/// `fetch_time_buckets` 응답 — gran_ms 와 행 목록. FE 는 gran_start_ms 로 차트 컬럼 구성.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBucketsResponse {
    pub gran_ms: u64,
    pub total: u64,
    pub rows: Vec<TimeBucketRow>,
    /// **FE 분기 결정용 명시 모드**. bar 의 `offsets` 유무로 추정하지 말고 이걸 봐야 함.
    pub delivery_mode: TimeBucketDeliveryMode,
    /// 선택된 plan 이름 — `"time_bucket.count_only"` / `"time_bucket.offsets_inline"` 등.
    /// 디버그/로깅용 (FE 분기엔 `delivery_mode` 사용).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_plan: Option<String>,
}

/// `*_combined_buckets`(multi/keyword/browse 공통) 요청 — 어떤 축을 집계할지. **단일 scan** 후
/// 같은 매치집합에서 grouping 만 분기하므로, `All` 도 `Offset` 단독 대비 cheap grouping 1회만
/// 추가될 뿐 scan 은 1회.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BucketsKind {
    Offset,
    Time,
    All,
}

/// 버킷 응답에 offset 을 얼마나 inline 할지 — **호출 의도** 표현 (cap 숫자가 아니라 "원하는 정보 형태").
/// 엔진 내부 `inline_cap` 노브를 의미 단위로 노출한다 (`PLANNER_DECISIONS.md` §16-8 PartialInline).
///
///   - `CountOnly` — offset inline 0. 트리/요약 위젯용 (drill 은 lazy / 탭 이동이라 offset 불필요).
///   - `Inline { cap }` — `cap` 까지만 inline, 초과 버킷은 서버가 count-only 로 strip (PartialInline).
///     보이는 만큼만 즉시 drill 하고 payload 는 가볍게.
///   - `Default` — 엔진 기본 cap(`STREAM_INLINE_CAP`)까지 inline. **미지정 시 기본값** = 기존 동작 보존.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum BucketDetail {
    CountOnly,
    Inline { cap: usize },
    Default,
}

impl BucketDetail {
    /// inline 상한으로 해석. `Default` 면 호출자가 넘긴 엔진 기본 cap 을 그대로 쓴다
    /// (`STREAM_INLINE_CAP` 은 엔진 crate 소유라 여기서 직접 참조하지 않는다).
    pub fn inline_cap(self, default_cap: usize) -> usize {
        match self {
            BucketDetail::CountOnly => 0,
            BucketDetail::Inline { cap } => cap,
            BucketDetail::Default => default_cap,
        }
    }
}

/// `*_combined_buckets`(multi/keyword/browse 공통) 응답 — **단일 scan** 에서 offset/time buckets 를
/// 함께 derive(검색 시 1회 → 차트 탭 즉시). `kind` 가 가리키는 축만 `Some`(나머지는 직렬화 skip).
/// **root 전용** — drill 은 단축 엔드포인트(`*_offset_buckets` / `*_time_buckets`)를 유지한다
/// (multi 는 offset 셀 범위가 time 축까지 자르는 비대칭, keyword/browse 는 단축 호출 자체가 drill 단위).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CombinedBucketsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<OffsetBucketsResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeBucketsResponse>,
}

// ── test_deserializers (deser 콤보 옆 [테스트] 버튼) ──────────────────────────
//
// 사용자가 picker drawer 에서 선택 중인 deserializer 가 실제 토픽 메시지에
// 올바르게 동작하는지 확인하기 위한 도구. 토픽에서 메시지 N개를 fetch 한 후
// 주어진 spec 으로 디코드하여 결과를 그대로 보여준다 (garbage 검출은 사용자 육안).

/// 한 메시지에 대한 key / value 디코드 시도 결과.
/// 디코드 성공 시 `*_decoded = Some(text)`, 실패 시 `*_error = Some(err_msg)`.
/// key 또는 value 가 None (Kafka tombstone 등) 이면 `*_decoded = Some("")`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestDeserializeSample {
    pub partition: i32,
    pub offset: i64,
    pub key_decoded: Option<String>,
    pub key_error: Option<String>,
    pub value_decoded: Option<String>,
    pub value_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestDeserializeResponse {
    /// 토픽에서 fetch 된 샘플별 디코드 결과 (최대 limit 개).
    pub samples: Vec<TestDeserializeSample>,
}

#[cfg(test)]
mod pos_filter_tests {
    use super::PosFilter;

    #[test]
    fn empty_filter_passes_everything_and_is_noop() {
        let f = PosFilter::default();
        assert!(f.is_noop());
        assert!(f.passes(0, 0));
        assert!(f.passes(7, u64::MAX));
    }

    #[test]
    fn partition_set_filters() {
        let f = PosFilter {
            partitions: vec![1, 3],
            ..Default::default()
        };
        assert!(!f.is_noop());
        assert!(f.passes(1, 100));
        assert!(f.passes(3, 0));
        assert!(!f.passes(2, 100));
        assert!(!f.passes(0, 100));
    }

    #[test]
    fn offset_range_is_inclusive() {
        let f = PosFilter {
            offset_gte: Some(10),
            offset_lte: Some(20),
            ..Default::default()
        };
        assert!(!f.passes(0, 9));
        assert!(f.passes(0, 10)); // 하한 inclusive
        assert!(f.passes(0, 15));
        assert!(f.passes(0, 20)); // 상한 inclusive
        assert!(!f.passes(0, 21));
    }

    #[test]
    fn open_ended_offset_bounds() {
        let lo_only = PosFilter {
            offset_gte: Some(100),
            ..Default::default()
        };
        assert!(!lo_only.passes(0, 99));
        assert!(lo_only.passes(0, u64::MAX));

        let hi_only = PosFilter {
            offset_lte: Some(100),
            ..Default::default()
        };
        assert!(hi_only.passes(0, 0));
        assert!(!hi_only.passes(0, 101));
    }

    #[test]
    fn partition_and_offset_both_apply() {
        let f = PosFilter {
            partitions: vec![2],
            offset_gte: Some(50),
            offset_lte: None,
        };
        assert!(f.passes(2, 50));
        assert!(!f.passes(2, 49)); // partition ok, offset 미달
        assert!(!f.passes(3, 100)); // offset ok, partition 불일치
    }
}
