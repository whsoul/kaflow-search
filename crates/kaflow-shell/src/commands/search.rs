//! Search Tauri 어댑터 — `Arc<dyn KafkaToolEngine>` 경유.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{
    BucketDetail, BucketsKind, CombinedBucketsResponse, LocItem, MessageResult,
    MultiSearchResponse, OffsetBucketsResponse, PosFilter, PrefetchSearchLocsResponse, SearchMode,
    SearchQuery, SortOrder, TestDeserializeResponse, TimeBucketsResponse, TopicDeserializers,
    TsRange,
};
use std::sync::Arc;

/// 진행 중인 검색/buckets 를 workspace 단위로 전부 취소 (명시적 [취소] 버튼). 반환 = 취소된 작업 수.
#[tauri::command]
pub async fn cancel_search(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
) -> Result<u32, String> {
    engine
        .cancel_search(&workspace)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn prefetch_search_locs(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: String,
    fields: Option<Vec<String>>,
    topics: Option<Vec<String>>,
    mode: SearchMode,
    sort_order: SortOrder,
    limit: Option<usize>,
    ts_range: Option<TsRange>,
    pos_filter: Option<PosFilter>,
) -> Result<PrefetchSearchLocsResponse, String> {
    engine
        .prefetch_search_locs(
            &workspace,
            &query,
            fields.as_deref(),
            topics.as_deref(),
            mode,
            sort_order,
            limit,
            ts_range,
            pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn multi_search(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: SearchQuery,
) -> Result<MultiSearchResponse, String> {
    engine
        .multi_search(&workspace, query)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn multi_search_time_buckets(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: SearchQuery,
    gran_ms: u64,
    hard_cap: Option<u64>,
) -> Result<TimeBucketsResponse, String> {
    engine
        .multi_search_time_buckets(&workspace, query, gran_ms, hard_cap)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn multi_search_combined_buckets(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: SearchQuery,
    gran_ms: u64,
    kind: BucketsKind,
) -> Result<CombinedBucketsResponse, String> {
    engine
        .multi_search_combined_buckets(&workspace, query, gran_ms, kind)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn multi_search_locs_in_range(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: SearchQuery,
    partition: Option<u32>,
    from_offset: Option<u64>,
    to_offset: Option<u64>,
    extra_gte_ms: Option<u64>,
    extra_lte_ms: Option<u64>,
) -> Result<Vec<LocItem>, String> {
    engine
        .multi_search_locs_in_range(
            &workspace,
            query,
            partition,
            from_offset,
            to_offset,
            extra_gte_ms,
            extra_lte_ms,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn multi_search_page(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: SearchQuery,
    after_ts: Option<u64>,
    after_partition: Option<u32>,
    after_offset: Option<u64>,
    limit: usize,
) -> Result<Vec<LocItem>, String> {
    engine
        .multi_search_page(
            &workspace,
            query,
            after_ts,
            after_partition,
            after_offset,
            limit,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn multi_search_offset_buckets(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: SearchQuery,
    drill: Option<(u32, u64, u64)>,
) -> Result<OffsetBucketsResponse, String> {
    engine
        .multi_search_offset_buckets(&workspace, query, drill)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn search_offset_buckets(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: String,
    fields: Option<Vec<String>>,
    topics: Option<Vec<String>>,
    mode: SearchMode,
    ts_range: Option<TsRange>,
    sort_order: SortOrder,
) -> Result<OffsetBucketsResponse, String> {
    engine
        .search_offset_buckets(
            &workspace,
            &query,
            fields.as_deref(),
            topics.as_deref(),
            mode,
            ts_range,
            sort_order,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn browse_offset_buckets(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    ts_range: Option<TsRange>,
    sort_order: SortOrder,
    detail: Option<BucketDetail>,
    pos_filter: Option<PosFilter>,
) -> Result<OffsetBucketsResponse, String> {
    engine
        .browse_offset_buckets(
            &workspace,
            &topic,
            ts_range,
            sort_order,
            detail.unwrap_or(BucketDetail::Default),
            pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn search_combined_buckets(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: String,
    fields: Option<Vec<String>>,
    topics: Option<Vec<String>>,
    mode: SearchMode,
    ts_range: Option<TsRange>,
    gran_ms: u64,
    kind: BucketsKind,
    sort_order: SortOrder,
    pos_filter: Option<PosFilter>,
) -> Result<CombinedBucketsResponse, String> {
    engine
        .search_combined_buckets(
            &workspace,
            &query,
            fields.as_deref(),
            topics.as_deref(),
            mode,
            ts_range,
            gran_ms,
            kind,
            sort_order,
            pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn browse_combined_buckets(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    ts_range: Option<TsRange>,
    gran_ms: u64,
    kind: BucketsKind,
    sort_order: SortOrder,
    detail: Option<BucketDetail>,
    pos_filter: Option<PosFilter>,
) -> Result<CombinedBucketsResponse, String> {
    engine
        .browse_combined_buckets(
            &workspace,
            &topic,
            ts_range,
            gran_ms,
            kind,
            sort_order,
            detail.unwrap_or(BucketDetail::Default),
            pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn fetch_messages(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    locs: Vec<LocItem>,
    bootstrap: Option<String>,
) -> Result<Vec<MessageResult>, String> {
    engine
        .fetch_messages(&workspace, &topic, locs, bootstrap.as_deref())
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn fetch_meta_rows(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    locs: Vec<LocItem>,
) -> Result<Vec<MessageResult>, String> {
    engine
        .fetch_meta_rows(&workspace, &topic, locs)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn fetch_browse_locs_in_bucket(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    partition: u32,
    from_offset: u64,
    to_offset: u64,
    sort_order: SortOrder,
) -> Result<Vec<LocItem>, String> {
    engine
        .fetch_browse_locs_in_bucket(
            &workspace,
            &topic,
            partition,
            from_offset,
            to_offset,
            sort_order,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_browse_locs_in_ts_range(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    gte_ms: Option<u64>,
    lte_ms: Option<u64>,
    sort_order: SortOrder,
    limit: Option<usize>,
    pos_filter: Option<PosFilter>,
) -> Result<Vec<LocItem>, String> {
    engine
        .fetch_browse_locs_in_ts_range(
            &workspace, &topic, gte_ms, lte_ms, sort_order, limit, pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_browse_locs_page(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    gte_ms: Option<u64>,
    lte_ms: Option<u64>,
    sort_order: SortOrder,
    limit: Option<usize>,
    after_ts: Option<u64>,
    after_partition: Option<u32>,
    after_offset: Option<u64>,
    pos_filter: Option<PosFilter>,
) -> Result<Vec<LocItem>, String> {
    engine
        .fetch_browse_locs_page(
            &workspace,
            &topic,
            gte_ms,
            lte_ms,
            sort_order,
            limit,
            after_ts,
            after_partition,
            after_offset,
            pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_time_buckets(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    gte_ms: Option<u64>,
    lte_ms: Option<u64>,
    gran_ms: u64,
    sort_order: SortOrder,
    detail: Option<BucketDetail>,
    hard_cap: Option<u64>,
    pos_filter: Option<PosFilter>,
) -> Result<TimeBucketsResponse, String> {
    engine
        .fetch_time_buckets(
            &workspace,
            &topic,
            gte_ms,
            lte_ms,
            gran_ms,
            sort_order,
            detail.unwrap_or(BucketDetail::Default),
            hard_cap,
            pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_search_time_buckets(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: String,
    fields: Option<Vec<String>>,
    topics: Option<Vec<String>>,
    mode: SearchMode,
    ts_range: Option<TsRange>,
    gran_ms: u64,
    sort_order: SortOrder,
    hard_cap: Option<u64>,
    pos_filter: Option<PosFilter>,
) -> Result<TimeBucketsResponse, String> {
    engine
        .fetch_search_time_buckets(
            &workspace,
            &query,
            fields.as_deref(),
            topics.as_deref(),
            mode,
            ts_range,
            gran_ms,
            sort_order,
            hard_cap,
            pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_search_locs_in_range(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: String,
    fields: Option<Vec<String>>,
    topics: Option<Vec<String>>,
    mode: SearchMode,
    sort_order: SortOrder,
    ts_range: Option<TsRange>,
    partition: Option<u32>,
    from_offset: Option<u64>,
    to_offset: Option<u64>,
    pos_filter: Option<PosFilter>,
) -> Result<Vec<LocItem>, String> {
    engine
        .fetch_search_locs_in_range(
            &workspace,
            &query,
            fields.as_deref(),
            topics.as_deref(),
            mode,
            sort_order,
            ts_range,
            partition,
            from_offset,
            to_offset,
            pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_search_locs_page(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    query: String,
    fields: Option<Vec<String>>,
    topics: Option<Vec<String>>,
    mode: SearchMode,
    sort_order: SortOrder,
    ts_range: Option<TsRange>,
    limit: Option<usize>,
    after_ts: Option<u64>,
    after_partition: Option<u32>,
    after_offset: Option<u64>,
    pos_filter: Option<PosFilter>,
) -> Result<Vec<LocItem>, String> {
    engine
        .fetch_search_locs_page(
            &workspace,
            &query,
            fields.as_deref(),
            topics.as_deref(),
            mode,
            sort_order,
            ts_range,
            limit,
            after_ts,
            after_partition,
            after_offset,
            pos_filter,
        )
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn prefetch_browse_locs(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    sort_order: SortOrder,
    limit: Option<usize>,
    ts_range: Option<TsRange>,
    pos_filter: Option<PosFilter>,
) -> Result<PrefetchSearchLocsResponse, String> {
    engine
        .prefetch_browse_locs(&workspace, &topic, sort_order, limit, ts_range, pos_filter)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn test_deserializers(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    deserializers: TopicDeserializers,
    bootstrap: Option<String>,
    limit: usize,
) -> Result<TestDeserializeResponse, String> {
    engine
        .test_deserializers(
            &workspace,
            &topic,
            deserializers,
            bootstrap.as_deref(),
            limit,
        )
        .await
        .map_err(|e| e.into_string())
}
