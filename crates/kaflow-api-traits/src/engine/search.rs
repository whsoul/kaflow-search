//! Search 도메인 API — 검색 / browse / prefetch / fetch_messages.
//! `browse::fetch_kafka_messages` 도 본 trait 에 흡수.

use async_trait::async_trait;
use kaflow_api_types::{
    BucketDetail, BucketsKind, CombinedBucketsResponse, LocItem, MessageResult,
    MultiSearchResponse, OffsetBucketsResponse, PosFilter, PrefetchSearchLocsResponse,
    SearchIndexResponse, SearchMode, SearchQuery, SortOrder, TestDeserializeResponse,
    TimeBucketsResponse, TopicDeserializers, TsRange,
};

use crate::error::EngineError;

#[async_trait]
pub trait SearchApi: Send + Sync {
    /// 진행 중인 검색/buckets 작업을 **workspace 단위로 전부 취소**한다 (명시적 [취소] 버튼).
    /// 반환 = 실제로 cancel 발화한 in-flight 작업 수 (idempotent — 없으면 0). 취소된 작업은
    /// `EngineError::Cancelled`(stringify `"cancelled"`)로 반환되며 FE 는 조용히 무시한다.
    /// 등록 대상은 오래 걸릴 수 있는 스캔류(buckets / multi head·next·drill / keyword drill)뿐 —
    /// limit-bounded head/next(prefetch·browse cursor)나 topic-meta·인덱싱은 영향받지 않는다.
    async fn cancel_search(&self, workspace: &str) -> Result<u32, EngineError>;

    /// k-way merge 로 (partition, offset) 목록을 시간순 수집. 진행률 emit.
    ///
    /// `fields`:
    ///   - `None` = 인덱싱된 모든 필드 OR 매칭
    ///   - `Some(slice)` = slice 의 필드만 OR 매칭 (빈 slice → 결과 없음)
    /// `limit`:
    ///   - `None` = 내부 cap 까지 (`ORDERED_LOCS_CAP`, 기본 5M)
    ///   - `Some(N)` = 첫 N건만 채우고 즉시 반환 (head fetch — list 첫 페이지용)
    #[allow(clippy::too_many_arguments)]
    async fn prefetch_search_locs(
        &self,
        workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        sort_order: SortOrder,
        limit: Option<usize>,
        ts_range: Option<TsRange>,
        // 위치(partition/offset) 필터 — 상세검색 위치 조건. None = 전 위치(head 전체).
        pos_filter: Option<PosFilter>,
    ) -> Result<PrefetchSearchLocsResponse, EngineError>;

    /// 다중 키워드 검색 — ES bool subset AST 평가.
    /// `I` key 기반 streaming merge-join + 재귀 그룹 평가 + exists / mustNot.
    /// **head 100 + 메타(per_condition/hit_fields/scan_ms) 반환** — `locs` 는 galloping head
    /// (FE 가 `[..100]` slice). next 는 `multi_search_page`, drill 은
    /// `multi_search_locs_in_range`. ⚠️ `total` 은 head cap(100) 에서 끊긴 **근사값** —
    /// 정확 매치 수는 `multi_search_offset_buckets` 가 derive. 전체 맵: `docs/search_modes_current_map.md`.
    async fn multi_search(
        &self,
        workspace: &str,
        query: SearchQuery,
    ) -> Result<MultiSearchResponse, EngineError>;

    /// 다중검색 매치의 시간 bucket — keyword 의 `fetch_search_time_buckets` 대칭.
    /// ChartTab 의 windowed 차트가 multi 모드에서 호출.
    async fn multi_search_time_buckets(
        &self,
        workspace: &str,
        query: SearchQuery,
        gran_ms: u64,
        // distinct bar 수 호출자 cap(tighten-only, None=라이브러리 천장). 초과 시 BucketOverflow.
        hard_cap: Option<u64>,
    ) -> Result<TimeBucketsResponse, EngineError>;

    /// 다중검색 buckets 통합 — **단일 full-drain eval** 에서 offset/time buckets 를 함께 derive.
    /// FE 가 검색 시점에 1회(kind=All) 호출 → offset 단축 + time 단축이 같은 쿼리를 각각
    /// full-drain(2×) 하던 낭비를 1×로. root 전용(drill 없음).
    async fn multi_search_combined_buckets(
        &self,
        workspace: &str,
        query: SearchQuery,
        gran_ms: u64,
        kind: BucketsKind,
    ) -> Result<CombinedBucketsResponse, EngineError>;

    /// 다중검색 매치 drill — `partition=None` → 차트 막대 ts 폭 / `Some` → 맵 셀 또는
    /// 탐색 오프셋 폭. `extra_gte_ms`/`extra_lte_ms` 는 query.ts_range 와 추가 교차 (차트
    /// 막대 클릭). keyword 의 `fetch_search_locs_in_range` 대칭.
    #[allow(clippy::too_many_arguments)]
    async fn multi_search_locs_in_range(
        &self,
        workspace: &str,
        query: SearchQuery,
        partition: Option<u32>,
        from_offset: Option<u64>,
        to_offset: Option<u64>,
        extra_gte_ms: Option<u64>,
        extra_lte_ms: Option<u64>,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// 시간순 보장 다중검색(AND/OR)의 keyset 페이징 — head 와 동일 merge-join 을
    /// `after_loc`(직전 페이지 마지막 loc) 으로 seek 후 다음 `limit` 개만 produce. 전수 평가
    /// 없이 head 만 walk → 빠르고 시간순. flat AND/OR 만 직접 처리, 중첩/mustNot 은 전수 폴백.
    /// (구현체 = driver-walk: `multi_search_drive_walk_page`.)
    /// 위치(partition/offset) 필터는 `query.pos_filter` 로 운반 (상세검색 위치 조건 / 차트 막대
    /// partition drill 공용 — 별도 param 없음).
    #[allow(clippy::too_many_arguments)]
    async fn multi_search_page(
        &self,
        workspace: &str,
        query: SearchQuery,
        after_ts: Option<u64>,
        after_partition: Option<u32>,
        after_offset: Option<u64>,
        limit: usize,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// 다중검색 partition × offset bucket aggregate (leaf=100 sparse).
    /// `drill`:
    ///   - `None` = 모든 partition 전수 평가
    ///   - `Some((p, lo, hi))` = 한 partition 의 [lo, hi] 범위만 평가
    /// hierarchical (1000/10000/...) 그룹은 FE 가 leaf 들을 derive.
    async fn multi_search_offset_buckets(
        &self,
        workspace: &str,
        query: SearchQuery,
        drill: Option<(u32, u64, u64)>,
    ) -> Result<OffsetBucketsResponse, EngineError>;

    /// 키워드 검색 매치의 partition × offset bucket aggregate (leaf=100 sparse).
    /// FE 가 hierarchical 그룹은 leaf 들을 derive.
    #[allow(clippy::too_many_arguments)]
    async fn search_offset_buckets(
        &self,
        workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        ts_range: Option<TsRange>,
        sort_order: SortOrder,
    ) -> Result<OffsetBucketsResponse, EngineError>;

    /// 키워드 buckets 통합 — 단일 scan 에서 offset/time 함께 derive (`multi_search_combined_buckets` 대칭).
    /// FE 가 검색 시점 1회(kind=All) 호출 → offset+time 단축이 같은 쿼리를 2× scan 하던 낭비 1×로.
    #[allow(clippy::too_many_arguments)]
    async fn search_combined_buckets(
        &self,
        workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        ts_range: Option<TsRange>,
        gran_ms: u64,
        kind: BucketsKind,
        sort_order: SortOrder,
        // 위치(파티션/offset) 필터 — 키워드 단일 파티션 콤보 등. None=전 위치.
        pos_filter: Option<PosFilter>,
    ) -> Result<CombinedBucketsResponse, EngineError>;

    /// browse 모드 (쿼리 없음) 의 partition × offset bucket aggregate.
    /// R `__ts` cursor 기반 — ts_range 안 모든 메시지를 leaf=100 sparse buckets.
    /// `sort_order` = evict-newest(§16-10) 방향 (cap 초과 시 어느 끝 offset 을 inline 으로).
    #[allow(clippy::too_many_arguments)]
    async fn browse_offset_buckets(
        &self,
        workspace: &str,
        topic: &str,
        ts_range: Option<TsRange>,
        sort_order: SortOrder,
        detail: BucketDetail,
        // 파티션 위치 필터(둘러보기 파티션 선택). None = 전 파티션.
        pos_filter: Option<PosFilter>,
    ) -> Result<OffsetBucketsResponse, EngineError>;

    /// browse buckets 통합 — 단일 R `__ts` scan 에서 offset/time 함께 derive.
    #[allow(clippy::too_many_arguments)]
    async fn browse_combined_buckets(
        &self,
        workspace: &str,
        topic: &str,
        ts_range: Option<TsRange>,
        gran_ms: u64,
        kind: BucketsKind,
        sort_order: SortOrder,
        detail: BucketDetail,
        // 파티션 위치 필터(둘러보기 파티션 선택). None = 전 파티션.
        pos_filter: Option<PosFilter>,
    ) -> Result<CombinedBucketsResponse, EngineError>;

    /// LocItem 목록 → 실제 메시지 본문 fetch.
    async fn fetch_messages(
        &self,
        workspace: &str,
        topic: &str,
        locs: Vec<LocItem>,
        bootstrap: Option<&str>,
    ) -> Result<Vec<MessageResult>, EngineError>;

    /// LocItem 목록 → RocksDB M-key 만 batch 조회 (Kafka 호출 없음).
    /// list row 표시 / 대표 필드 값 derive 용 — payload/key/headers 는 빈 값.
    /// content_source = "meta-only". detail panel 진입 시점에 fetch_messages 로 lazy fetch.
    async fn fetch_meta_rows(
        &self,
        workspace: &str,
        topic: &str,
        locs: Vec<LocItem>,
    ) -> Result<Vec<MessageResult>, EngineError>;

    /// browse 도메인 흡수 — 인덱싱 없이 Kafka 에서 직접 fetch.
    async fn fetch_kafka_messages_raw(
        &self,
        workspace: &str,
        topic: &str,
        limit: Option<usize>,
    ) -> Result<SearchIndexResponse, EngineError>;

    /// 특정 파티션의 `[from_offset, to_offset]` 범위에 속하는 M-key 만 LocItem 으로 반환.
    /// 탐색 탭에서 버킷 노드를 펼칠 때 호출.
    async fn fetch_browse_locs_in_bucket(
        &self,
        workspace: &str,
        topic: &str,
        partition: u32,
        from_offset: u64,
        to_offset: u64,
        sort_order: SortOrder,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// `[gte_ms, lte_ms]` (inclusive) 시간 범위의 LocItem 을 R `__ts` cursor 로 반환.
    /// 시계열 막대 클릭 / 탐색(시간) 노드 펼침의 drill-down — 그 범위만 서버 조회
    /// (orderedLocs 전체 전송 없이). value 미접근, ts 는 키에서 직접. `limit` 으로 상한.
    #[allow(clippy::too_many_arguments)]
    async fn fetch_browse_locs_in_ts_range(
        &self,
        workspace: &str,
        topic: &str,
        gte_ms: Option<u64>,
        lte_ms: Option<u64>,
        sort_order: SortOrder,
        limit: Option<usize>,
        // 파티션 위치 필터(둘러보기 파티션 선택). None = 전 파티션.
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// 목록 loadMore — R `__ts` cursor 페이징. `after_*` 셋이 다 Some 이면 그 loc 다음부터
    /// `limit` 건. (B-3 본체: orderedLocs 전수 전송 없이 서버에서 다음 페이지만.)
    #[allow(clippy::too_many_arguments)]
    async fn fetch_browse_locs_page(
        &self,
        workspace: &str,
        topic: &str,
        gte_ms: Option<u64>,
        lte_ms: Option<u64>,
        sort_order: SortOrder,
        limit: Option<usize>,
        after_ts: Option<u64>,
        after_partition: Option<u32>,
        after_offset: Option<u64>,
        // 위치(partition/offset) 필터 — 상세검색 위치 조건 / 차트 막대 partition drill 공용.
        // None = 전 위치(목록 페이징). 필터 후 limit 카운트.
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// R `__ts` cursor 로 `(partition, gran_start_ms)` 별 메시지 수 집계 (browse 시계열/탐색시간 골격).
    /// value 미접근 — 카운트만. **검색 모드는 R 이 매치를 못 거르므로 사용 금지**
    /// (검색 시계열은 매치 loc 을 FE 가 직접 집계).
    #[allow(clippy::too_many_arguments)]
    async fn fetch_time_buckets(
        &self,
        workspace: &str,
        topic: &str,
        gte_ms: Option<u64>,
        lte_ms: Option<u64>,
        gran_ms: u64,
        sort_order: SortOrder,
        detail: BucketDetail,
        // distinct bar 수 호출자 cap(tighten-only, None=라이브러리 천장). 초과 시 BucketOverflow.
        hard_cap: Option<u64>,
        // 파티션 위치 필터(둘러보기 파티션 선택). None = 전 파티션.
        pos_filter: Option<PosFilter>,
    ) -> Result<TimeBucketsResponse, EngineError>;

    /// 키워드 검색 매치의 `(partition, gran_start_ms)` 시간 bucket 집계 (count-only).
    /// `fetch_time_buckets`(browse, R `__ts`) 의 키워드 대칭 — R 은 매치를 못 거르므로
    /// I-key 매치 스캔으로 집계한다. ts 출처는 엔진 내부 seam (현재 I-key revTs).
    /// `ts_range` 안의 매치만 (글로벌 ts 필터 + 차트 windowed 창 공용).
    #[allow(clippy::too_many_arguments)]
    async fn fetch_search_time_buckets(
        &self,
        workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        ts_range: Option<TsRange>,
        gran_ms: u64,
        sort_order: SortOrder,
        // distinct bar 수 호출자 cap(tighten-only, None=라이브러리 천장). 초과 시 BucketOverflow.
        hard_cap: Option<u64>,
        // 위치(파티션/offset) 필터 — 키워드 단일 파티션 콤보 등. None=전 위치.
        pos_filter: Option<PosFilter>,
    ) -> Result<TimeBucketsResponse, EngineError>;

    /// 키워드 검색 매치 중 `ts_range` ∩ 선택 범위의 LocItem (match-aware drill-down).
    /// `partition=None` → `[gte_ms, lte_ms]` 시간 폭(차트 막대). `partition=Some` →
    /// 그 파티션의 `[from_offset, to_offset]` 오프셋 폭(맵 셀 / 탐색 오프셋 노드).
    /// browse 의 `fetch_browse_locs_in_ts_range` / `fetch_browse_locs_in_bucket` 와 달리
    /// 쿼리 매치만 반환한다. matched_field/index key 채움.
    #[allow(clippy::too_many_arguments)]
    async fn fetch_search_locs_in_range(
        &self,
        workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        sort_order: SortOrder,
        ts_range: Option<TsRange>,
        partition: Option<u32>,
        from_offset: Option<u64>,
        to_offset: Option<u64>,
        // 위치(파티션/offset) 필터 — 키워드 단일 파티션 콤보 등. None=전 위치.
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// 키워드 검색 목록 cursor 페이징 — browse 의 `fetch_browse_locs_page`(R `__ts`) 의 키워드 대칭.
    /// `after_*` 셋이 다 Some 이면 그 loc *다음* 부터 `limit` 건. 셋 중 하나라도 None 이면
    /// 첫 페이지(seek_after 없음). browse 와 달리 sorted single stream 이 없어 매 페이지
    /// dedup 전수 스캔 후 정렬 + cursor seek 으로 slice 한다.
    #[allow(clippy::too_many_arguments)]
    async fn fetch_search_locs_page(
        &self,
        workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        sort_order: SortOrder,
        ts_range: Option<TsRange>,
        limit: Option<usize>,
        after_ts: Option<u64>,
        after_partition: Option<u32>,
        after_offset: Option<u64>,
        // 위치(partition/offset) 필터 — 상세검색 위치 조건 / 차트 막대 partition drill 공용.
        // None = 전 위치(목록 페이징). 필터 후 limit 카운트.
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// 토픽 전체 M-key 스캔으로 모든 LocItem 을 반환 (matched_field/index_key 는 None).
    /// browse 모드의 chart/map 이 정확한 시간 분포로 그릴 수 있도록 raw loc 메타 제공.
    /// 상한 초과 시 `capped = true` 로 반환되며 호출부는 bucket 추정 fallback 사용.
    /// `limit`:
    ///   - `None` = 내부 cap 까지 (`ORDERED_LOCS_CAP`, 기본 5M)
    ///   - `Some(N)` = 첫 N건만 채우고 즉시 반환 (head fetch — list 첫 페이지용)
    async fn prefetch_browse_locs(
        &self,
        workspace: &str,
        topic: &str,
        sort_order: SortOrder,
        limit: Option<usize>,
        ts_range: Option<TsRange>,
        // 위치(partition/offset) 필터 — 상세검색 위치 조건. None = 전 위치(head 전체).
        pos_filter: Option<PosFilter>,
    ) -> Result<PrefetchSearchLocsResponse, EngineError>;

    /// 토픽에서 메시지 `limit` 개 fetch 후 사용자가 임시로 지정한 `deserializers` 로 디코드.
    /// picker drawer 의 [테스트] 버튼이 호출 — 인덱싱 진입 전 schema/wire format 적합성 육안 확인용.
    /// T-META 의 저장된 deserializer 와 무관하게 동작 (인자 spec 만 사용).
    async fn test_deserializers(
        &self,
        workspace: &str,
        topic: &str,
        deserializers: TopicDeserializers,
        bootstrap: Option<&str>,
        limit: usize,
    ) -> Result<TestDeserializeResponse, EngineError>;
}
