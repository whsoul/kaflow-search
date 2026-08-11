//! Searching the fixture. Everything is scanned; nothing is indexed.

use async_trait::async_trait;
use kaflow_api_traits::engine::SearchApi;
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::{
    BucketDetail, BucketsKind, CombinedBucketsResponse, LocItem, MessageResult,
    MultiSearchResponse, OffsetBucketsResponse, PosFilter, PrefetchSearchLocsResponse, QueryNode,
    SearchIndexResponse, SearchMode, SearchQuery, SortOrder, TestDeserializeResponse,
    TestDeserializeSample, TimeBucketsResponse, TopicDeserializers, TsRange,
};

use crate::data::{build_offset_buckets, build_time_buckets, sort_locs, MockStore};
use crate::MockEngine;

// ── Shared helpers ──────────────────────────────────────────────────────────

fn ts_range_of(gte: Option<u64>, lte: Option<u64>) -> Option<TsRange> {
    if gte.is_none() && lte.is_none() {
        None
    } else {
        Some(TsRange {
            gte_ms: gte,
            lte_ms: lte,
        })
    }
}

/// A limit of zero means the engine decides, which here means no limit.
fn limit_opt(limit: usize) -> Option<usize> {
    if limit == 0 {
        None
    } else {
        Some(limit)
    }
}

/// Keeps only what comes after a given position.
fn after_cursor(
    locs: Vec<LocItem>,
    after_ts: Option<u64>,
    after_partition: Option<u32>,
    after_offset: Option<u64>,
) -> Vec<LocItem> {
    if after_ts.is_none() && after_offset.is_none() {
        return locs;
    }
    if let Some(idx) = locs.iter().position(|l| {
        Some(l.ts_millis) == after_ts
            && (after_partition.is_none() || Some(l.partition) == after_partition)
            && Some(l.offset) == after_offset
    }) {
        locs.into_iter().skip(idx + 1).collect()
    } else {
        locs
    }
}

/// Runs a keyword search across topics and merges the results.
#[allow(clippy::too_many_arguments)]
fn keyword_all(
    store: &MockStore,
    query: &str,
    fields: Option<&[String]>,
    topics: Option<&[String]>,
    mode: &SearchMode,
    ts: Option<&TsRange>,
    pos: Option<&PosFilter>,
    sort: &SortOrder,
    limit: Option<usize>,
) -> (Vec<LocItem>, Vec<String>, usize) {
    let mut all: Vec<LocItem> = Vec::new();
    let mut hit_fields: Vec<String> = Vec::new();
    let mut total = 0usize;
    for t in store.topics_for(topics) {
        let tok = store.tokenized_for(&t.name);
        let (locs, hf, tot) = t.keyword_locs(query, fields, mode, &tok, ts, pos, sort, None);
        total += tot;
        for h in hf {
            if !hit_fields.contains(&h) {
                hit_fields.push(h);
            }
        }
        all.extend(locs);
    }
    sort_locs(&mut all, sort);
    if let Some(l) = limit {
        all.truncate(l);
    }
    (all, hit_fields, total)
}

/// The same for a boolean query.
fn multi_all(
    store: &MockStore,
    node: &QueryNode,
    topics: Option<&[String]>,
    ts: Option<&TsRange>,
    pos: Option<&PosFilter>,
    sort: &SortOrder,
    limit: Option<usize>,
) -> (Vec<LocItem>, usize) {
    let mut all: Vec<LocItem> = Vec::new();
    let mut total = 0usize;
    for t in store.topics_for(topics) {
        let tok = store.tokenized_for(&t.name);
        let (locs, tot) = t.multi_locs(node, &tok, ts, pos, sort, None);
        total += tot;
        all.extend(locs);
    }
    sort_locs(&mut all, sort);
    if let Some(l) = limit {
        all.truncate(l);
    }
    (all, total)
}

fn first_topic_name(topics: Option<&[String]>, store: &MockStore) -> String {
    topics
        .and_then(|t| t.first())
        .cloned()
        .or_else(|| store.topics.first().map(|t| t.name.clone()))
        .unwrap_or_default()
}

fn empty_response() -> SearchIndexResponse {
    SearchIndexResponse {
        results: Vec::new(),
        locs: Vec::new(),
        has_more: false,
        next_browse_cursors: Vec::new(),
        next_search_cursors: Vec::new(),
    }
}

#[async_trait]
impl SearchApi for MockEngine {
    async fn cancel_search(&self, _workspace: &str) -> Result<u32, EngineError> {
        Ok(0)
    }

    #[allow(clippy::too_many_arguments)]
    async fn prefetch_search_locs(
        &self,
        _workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        sort_order: SortOrder,
        limit: Option<usize>,
        ts_range: Option<TsRange>,
        pos_filter: Option<PosFilter>,
    ) -> Result<PrefetchSearchLocsResponse, EngineError> {
        let (locs, hit_fields, total) = keyword_all(
            &self.store,
            query,
            fields,
            topics,
            &mode,
            ts_range.as_ref(),
            pos_filter.as_ref(),
            &sort_order,
            limit,
        );
        let capped = total > locs.len();
        Ok(PrefetchSearchLocsResponse {
            locs,
            total,
            capped,
            hit_fields,
        })
    }

    async fn multi_search(
        &self,
        _workspace: &str,
        query: SearchQuery,
    ) -> Result<MultiSearchResponse, EngineError> {
        let (locs, total) = multi_all(
            &self.store,
            &query.r#where,
            query.topics.as_deref(),
            query.ts_range.as_ref(),
            query.pos_filter.as_ref(),
            &query.sort_order,
            limit_opt(query.limit),
        );
        let capped = total > locs.len();
        Ok(MultiSearchResponse {
            locs,
            total,
            capped,
            strategy: "streaming".to_string(),
            per_condition: Vec::new(),
            hit_fields: Vec::new(),
            scan_ms: 0,
            normalized_where: None,
        })
    }

    async fn multi_search_time_buckets(
        &self,
        _workspace: &str,
        query: SearchQuery,
        gran_ms: u64,
        _hard_cap: Option<u64>,
    ) -> Result<TimeBucketsResponse, EngineError> {
        let (locs, _) = multi_all(
            &self.store,
            &query.r#where,
            query.topics.as_deref(),
            query.ts_range.as_ref(),
            query.pos_filter.as_ref(),
            &query.sort_order,
            None,
        );
        Ok(build_time_buckets(&locs, gran_ms))
    }

    async fn multi_search_combined_buckets(
        &self,
        _workspace: &str,
        query: SearchQuery,
        gran_ms: u64,
        kind: BucketsKind,
    ) -> Result<CombinedBucketsResponse, EngineError> {
        let topic = first_topic_name(query.topics.as_deref(), &self.store);
        let (locs, _) = multi_all(
            &self.store,
            &query.r#where,
            query.topics.as_deref(),
            query.ts_range.as_ref(),
            query.pos_filter.as_ref(),
            &query.sort_order,
            None,
        );
        Ok(combined(&topic, &locs, gran_ms, kind))
    }

    async fn multi_search_locs_in_range(
        &self,
        _workspace: &str,
        query: SearchQuery,
        partition: Option<u32>,
        from_offset: Option<u64>,
        to_offset: Option<u64>,
        extra_gte_ms: Option<u64>,
        extra_lte_ms: Option<u64>,
    ) -> Result<Vec<LocItem>, EngineError> {
        let pos = range_pos(partition, from_offset, to_offset);
        let ts = merge_ts(query.ts_range.as_ref(), extra_gte_ms, extra_lte_ms);
        let (locs, _) = multi_all(
            &self.store,
            &query.r#where,
            query.topics.as_deref(),
            ts.as_ref(),
            Some(&pos),
            &query.sort_order,
            None,
        );
        Ok(locs)
    }

    #[allow(clippy::too_many_arguments)]
    async fn multi_search_page(
        &self,
        _workspace: &str,
        query: SearchQuery,
        after_ts: Option<u64>,
        after_partition: Option<u32>,
        after_offset: Option<u64>,
        limit: usize,
    ) -> Result<Vec<LocItem>, EngineError> {
        let (locs, _) = multi_all(
            &self.store,
            &query.r#where,
            query.topics.as_deref(),
            query.ts_range.as_ref(),
            query.pos_filter.as_ref(),
            &query.sort_order,
            None,
        );
        let mut page = after_cursor(locs, after_ts, after_partition, after_offset);
        page.truncate(limit.max(1));
        Ok(page)
    }

    async fn multi_search_offset_buckets(
        &self,
        _workspace: &str,
        query: SearchQuery,
        drill: Option<(u32, u64, u64)>,
    ) -> Result<OffsetBucketsResponse, EngineError> {
        let topic = first_topic_name(query.topics.as_deref(), &self.store);
        let pos = drill.map(|(p, lo, hi)| range_pos(Some(p), Some(lo), Some(hi)));
        let (locs, _) = multi_all(
            &self.store,
            &query.r#where,
            query.topics.as_deref(),
            query.ts_range.as_ref(),
            pos.as_ref().or(query.pos_filter.as_ref()),
            &query.sort_order,
            None,
        );
        Ok(build_offset_buckets(&topic, &locs))
    }

    async fn search_offset_buckets(
        &self,
        _workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        ts_range: Option<TsRange>,
        sort_order: SortOrder,
    ) -> Result<OffsetBucketsResponse, EngineError> {
        let topic = first_topic_name(topics, &self.store);
        let (locs, _, _) = keyword_all(
            &self.store,
            query,
            fields,
            topics,
            &mode,
            ts_range.as_ref(),
            None,
            &sort_order,
            None,
        );
        Ok(build_offset_buckets(&topic, &locs))
    }

    #[allow(clippy::too_many_arguments)]
    async fn search_combined_buckets(
        &self,
        _workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        ts_range: Option<TsRange>,
        gran_ms: u64,
        kind: BucketsKind,
        sort_order: SortOrder,
        pos_filter: Option<PosFilter>,
    ) -> Result<CombinedBucketsResponse, EngineError> {
        let topic = first_topic_name(topics, &self.store);
        let (locs, _, _) = keyword_all(
            &self.store,
            query,
            fields,
            topics,
            &mode,
            ts_range.as_ref(),
            pos_filter.as_ref(),
            &sort_order,
            None,
        );
        Ok(combined(&topic, &locs, gran_ms, kind))
    }

    #[allow(clippy::too_many_arguments)]
    async fn browse_offset_buckets(
        &self,
        _workspace: &str,
        topic: &str,
        ts_range: Option<TsRange>,
        sort_order: SortOrder,
        _detail: BucketDetail,
        pos_filter: Option<PosFilter>,
    ) -> Result<OffsetBucketsResponse, EngineError> {
        let locs = match self.store.topic(topic) {
            Some(t) => {
                t.browse_locs(ts_range.as_ref(), pos_filter.as_ref(), &sort_order, None)
                    .0
            }
            None => Vec::new(),
        };
        Ok(build_offset_buckets(topic, &locs))
    }

    #[allow(clippy::too_many_arguments)]
    async fn browse_combined_buckets(
        &self,
        _workspace: &str,
        topic: &str,
        ts_range: Option<TsRange>,
        gran_ms: u64,
        kind: BucketsKind,
        sort_order: SortOrder,
        _detail: BucketDetail,
        pos_filter: Option<PosFilter>,
    ) -> Result<CombinedBucketsResponse, EngineError> {
        let locs = match self.store.topic(topic) {
            Some(t) => {
                t.browse_locs(ts_range.as_ref(), pos_filter.as_ref(), &sort_order, None)
                    .0
            }
            None => Vec::new(),
        };
        Ok(combined(topic, &locs, gran_ms, kind))
    }

    async fn fetch_messages(
        &self,
        _workspace: &str,
        topic: &str,
        locs: Vec<LocItem>,
        _bootstrap: Option<&str>,
    ) -> Result<Vec<MessageResult>, EngineError> {
        Ok(self
            .store
            .topic(topic)
            .map(|t| t.message_rows(&locs))
            .unwrap_or_default())
    }

    async fn fetch_meta_rows(
        &self,
        _workspace: &str,
        topic: &str,
        locs: Vec<LocItem>,
    ) -> Result<Vec<MessageResult>, EngineError> {
        Ok(self
            .store
            .topic(topic)
            .map(|t| t.message_rows(&locs))
            .unwrap_or_default())
    }

    async fn fetch_kafka_messages_raw(
        &self,
        _workspace: &str,
        topic: &str,
        limit: Option<usize>,
    ) -> Result<SearchIndexResponse, EngineError> {
        let t = match self.store.topic(topic) {
            Some(t) => t,
            None => return Ok(empty_response()),
        };
        let (locs, _) = t.browse_locs(None, None, &SortOrder::NewestFirst, limit);
        let results = t.message_rows(&locs);
        Ok(SearchIndexResponse {
            results,
            locs: Vec::new(),
            has_more: false,
            next_browse_cursors: Vec::new(),
            next_search_cursors: Vec::new(),
        })
    }

    async fn fetch_browse_locs_in_bucket(
        &self,
        _workspace: &str,
        topic: &str,
        partition: u32,
        from_offset: u64,
        to_offset: u64,
        sort_order: SortOrder,
    ) -> Result<Vec<LocItem>, EngineError> {
        let pos = range_pos(Some(partition), Some(from_offset), Some(to_offset));
        Ok(match self.store.topic(topic) {
            Some(t) => t.browse_locs(None, Some(&pos), &sort_order, None).0,
            None => Vec::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_browse_locs_in_ts_range(
        &self,
        _workspace: &str,
        topic: &str,
        gte_ms: Option<u64>,
        lte_ms: Option<u64>,
        sort_order: SortOrder,
        limit: Option<usize>,
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError> {
        let ts = ts_range_of(gte_ms, lte_ms);
        Ok(match self.store.topic(topic) {
            Some(t) => {
                t.browse_locs(ts.as_ref(), pos_filter.as_ref(), &sort_order, limit)
                    .0
            }
            None => Vec::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_browse_locs_page(
        &self,
        _workspace: &str,
        topic: &str,
        gte_ms: Option<u64>,
        lte_ms: Option<u64>,
        sort_order: SortOrder,
        limit: Option<usize>,
        after_ts: Option<u64>,
        after_partition: Option<u32>,
        after_offset: Option<u64>,
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError> {
        let ts = ts_range_of(gte_ms, lte_ms);
        let all = match self.store.topic(topic) {
            Some(t) => {
                t.browse_locs(ts.as_ref(), pos_filter.as_ref(), &sort_order, None)
                    .0
            }
            None => Vec::new(),
        };
        let mut page = after_cursor(all, after_ts, after_partition, after_offset);
        if let Some(l) = limit {
            page.truncate(l);
        }
        Ok(page)
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_time_buckets(
        &self,
        _workspace: &str,
        topic: &str,
        gte_ms: Option<u64>,
        lte_ms: Option<u64>,
        gran_ms: u64,
        sort_order: SortOrder,
        _detail: BucketDetail,
        _hard_cap: Option<u64>,
        pos_filter: Option<PosFilter>,
    ) -> Result<TimeBucketsResponse, EngineError> {
        let ts = ts_range_of(gte_ms, lte_ms);
        let locs = match self.store.topic(topic) {
            Some(t) => {
                t.browse_locs(ts.as_ref(), pos_filter.as_ref(), &sort_order, None)
                    .0
            }
            None => Vec::new(),
        };
        Ok(build_time_buckets(&locs, gran_ms))
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_search_time_buckets(
        &self,
        _workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        ts_range: Option<TsRange>,
        gran_ms: u64,
        sort_order: SortOrder,
        _hard_cap: Option<u64>,
        pos_filter: Option<PosFilter>,
    ) -> Result<TimeBucketsResponse, EngineError> {
        let (locs, _, _) = keyword_all(
            &self.store,
            query,
            fields,
            topics,
            &mode,
            ts_range.as_ref(),
            pos_filter.as_ref(),
            &sort_order,
            None,
        );
        Ok(build_time_buckets(&locs, gran_ms))
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_search_locs_in_range(
        &self,
        _workspace: &str,
        query: &str,
        fields: Option<&[String]>,
        topics: Option<&[String]>,
        mode: SearchMode,
        sort_order: SortOrder,
        ts_range: Option<TsRange>,
        partition: Option<u32>,
        from_offset: Option<u64>,
        to_offset: Option<u64>,
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError> {
        let pos = match (partition, from_offset, to_offset) {
            (None, None, None) => pos_filter,
            _ => Some(range_pos(partition, from_offset, to_offset)),
        };
        let (locs, _, _) = keyword_all(
            &self.store,
            query,
            fields,
            topics,
            &mode,
            ts_range.as_ref(),
            pos.as_ref(),
            &sort_order,
            None,
        );
        Ok(locs)
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_search_locs_page(
        &self,
        _workspace: &str,
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
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError> {
        let (all, _, _) = keyword_all(
            &self.store,
            query,
            fields,
            topics,
            &mode,
            ts_range.as_ref(),
            pos_filter.as_ref(),
            &sort_order,
            None,
        );
        let mut page = after_cursor(all, after_ts, after_partition, after_offset);
        if let Some(l) = limit {
            page.truncate(l);
        }
        Ok(page)
    }

    async fn prefetch_browse_locs(
        &self,
        _workspace: &str,
        topic: &str,
        sort_order: SortOrder,
        limit: Option<usize>,
        ts_range: Option<TsRange>,
        pos_filter: Option<PosFilter>,
    ) -> Result<PrefetchSearchLocsResponse, EngineError> {
        let (locs, total) = match self.store.topic(topic) {
            Some(t) => t.browse_locs(ts_range.as_ref(), pos_filter.as_ref(), &sort_order, limit),
            None => (Vec::new(), 0),
        };
        let capped = total > locs.len();
        Ok(PrefetchSearchLocsResponse {
            locs,
            total,
            capped,
            hit_fields: Vec::new(),
        })
    }

    async fn test_deserializers(
        &self,
        _workspace: &str,
        topic: &str,
        _deserializers: TopicDeserializers,
        _bootstrap: Option<&str>,
        limit: usize,
    ) -> Result<TestDeserializeResponse, EngineError> {
        let samples = self
            .store
            .topic(topic)
            .map(|t| {
                t.messages
                    .iter()
                    .take(limit.max(1))
                    .map(|m| TestDeserializeSample {
                        partition: m.partition as i32,
                        offset: m.offset as i64,
                        key_decoded: Some(m.key.clone().unwrap_or_default()),
                        key_error: None,
                        value_decoded: Some(m.value_json.clone()),
                        value_error: None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(TestDeserializeResponse { samples })
    }
}

// ── Small helpers ───────────────────────────────────────────────────────────

fn range_pos(partition: Option<u32>, from: Option<u64>, to: Option<u64>) -> PosFilter {
    PosFilter {
        partitions: partition.map(|p| vec![p]).unwrap_or_default(),
        offset_gte: from,
        offset_lte: to,
    }
}

/// Where two time ranges overlap.
fn merge_ts(
    base: Option<&TsRange>,
    extra_gte: Option<u64>,
    extra_lte: Option<u64>,
) -> Option<TsRange> {
    let bg = base.and_then(|r| r.gte_ms);
    let bl = base.and_then(|r| r.lte_ms);
    let gte = match (bg, extra_gte) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    };
    let lte = match (bl, extra_lte) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    };
    ts_range_of(gte, lte)
}

fn combined(
    topic: &str,
    locs: &[LocItem],
    gran_ms: u64,
    kind: BucketsKind,
) -> CombinedBucketsResponse {
    let offset = matches!(kind, BucketsKind::Offset | BucketsKind::All)
        .then(|| build_offset_buckets(topic, locs));
    let time = matches!(kind, BucketsKind::Time | BucketsKind::All)
        .then(|| build_time_buckets(locs, gran_ms));
    CombinedBucketsResponse { offset, time }
}
