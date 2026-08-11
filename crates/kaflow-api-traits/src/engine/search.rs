//! Search: querying the index, browsing a topic, and fetching message bodies.

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
    /// Cancels every in-flight search and bucket job for this workspace.
    ///
    /// Returns how many were actually signalled. Zero is not an error — cancelling when
    /// nothing is running is allowed. Cancelled work fails with [`EngineError::Cancelled`].
    ///
    /// Only long-running work is cancellable. Bounded operations may run to completion
    /// regardless, so a non-zero return is not proof that everything stopped.
    async fn cancel_search(&self, workspace: &str) -> Result<u32, EngineError>;

    /// Collects the positions matching `query`, in time order. Progress is emitted while
    /// it runs.
    ///
    /// `fields`:
    ///   - `None` = match against every indexed field
    ///   - `Some(slice)` = match against those fields only (an empty slice matches nothing)
    ///
    /// `limit`:
    ///   - `None` = up to an internal safety cap; the response reports whether it was hit
    ///   - `Some(n)` = stop after `n` and return immediately
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
        // Restricts to a partition or offset span. None = every position.
        pos_filter: Option<PosFilter>,
    ) -> Result<PrefetchSearchLocsResponse, EngineError>;

    /// Evaluates a boolean query — nested groups, must / should / must_not, and exists.
    ///
    /// Returns the first page of matches, together with per-condition counts, the fields
    /// that were hit, and how long the scan took.
    ///
    /// ⚠️ `total` is an approximation and must not be relied on as an exact count —
    /// counting may stop once that first page is filled. The exact number comes from
    /// [`SearchApi::multi_search_offset_buckets`]. Later pages come from
    /// [`SearchApi::multi_search_page`], and drilling into one range from
    /// [`SearchApi::multi_search_locs_in_range`].
    async fn multi_search(
        &self,
        workspace: &str,
        query: SearchQuery,
    ) -> Result<MultiSearchResponse, EngineError>;

    /// Time buckets over the matches of a boolean query — the counterpart of
    /// [`SearchApi::fetch_search_time_buckets`] for single-keyword search.
    async fn multi_search_time_buckets(
        &self,
        workspace: &str,
        query: SearchQuery,
        gran_ms: u64,
        // Caller cap on the number of distinct bars. Tighten-only — None means the
        // library ceiling. Exceeding it fails with `EngineError::BucketOverflow`.
        hard_cap: Option<u64>,
    ) -> Result<TimeBucketsResponse, EngineError>;

    /// Offset and time buckets for one boolean query in a single call; `kind` selects
    /// which of the two to return. Root only — this does not drill.
    async fn multi_search_combined_buckets(
        &self,
        workspace: &str,
        query: SearchQuery,
        gran_ms: u64,
        kind: BucketsKind,
    ) -> Result<CombinedBucketsResponse, EngineError>;

    /// The matches of a boolean query inside one range.
    ///
    /// `partition = None` narrows by time; `Some(p)` narrows to an offset span within that
    /// partition. `extra_gte_ms` / `extra_lte_ms` intersect with `query.ts_range` rather
    /// than replacing it. Counterpart of [`SearchApi::fetch_search_locs_in_range`].
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

    /// Keyset pagination over the same boolean query, still in time order.
    ///
    /// Returns up to `limit` positions strictly after `(after_ts, after_partition,
    /// after_offset)` — the last position of the previous page. All three `None` returns
    /// the first page.
    ///
    /// Position filters travel inside `query.pos_filter`; there is no separate parameter.
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

    /// Counts the matches of a boolean query per partition and offset range. Only the
    /// finest level is returned, and only where the count is non-zero; how wide that level
    /// is comes back with the response rather than being fixed here.
    ///
    /// `drill`:
    ///   - `None` = every partition
    ///   - `Some((partition, from, to))` = that offset span of one partition only
    ///
    /// Coarser levels are the caller's to derive from the leaves.
    async fn multi_search_offset_buckets(
        &self,
        workspace: &str,
        query: SearchQuery,
        drill: Option<(u32, u64, u64)>,
    ) -> Result<OffsetBucketsResponse, EngineError>;

    /// The same aggregate for a single-keyword search: finest level only, non-zero counts
    /// only, coarser levels the caller's to derive.
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

    /// Offset and time buckets for a single-keyword search in one call; `kind` selects
    /// which. Counterpart of [`SearchApi::multi_search_combined_buckets`].
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
        // Restricts to a partition or offset span. None = every position.
        pos_filter: Option<PosFilter>,
    ) -> Result<CombinedBucketsResponse, EngineError>;

    /// The same aggregate with no query: every indexed message inside `ts_range`.
    ///
    /// `sort_order` decides which end survives when the inline cap is reached — oldest
    /// first keeps the low offsets, newest first keeps the high ones.
    #[allow(clippy::too_many_arguments)]
    async fn browse_offset_buckets(
        &self,
        workspace: &str,
        topic: &str,
        ts_range: Option<TsRange>,
        sort_order: SortOrder,
        detail: BucketDetail,
        // Restricts to selected partitions. None = every partition.
        pos_filter: Option<PosFilter>,
    ) -> Result<OffsetBucketsResponse, EngineError>;

    /// Offset and time buckets for a browse (no query) in one call.
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
        // Restricts to selected partitions. None = every partition.
        pos_filter: Option<PosFilter>,
    ) -> Result<CombinedBucketsResponse, EngineError>;

    /// Fetches the full messages at the given positions. This one reads from the cluster.
    async fn fetch_messages(
        &self,
        workspace: &str,
        topic: &str,
        locs: Vec<LocItem>,
        bootstrap: Option<&str>,
    ) -> Result<Vec<MessageResult>, EngineError>;

    /// Reads only what the index already holds about those positions.
    ///
    /// **Must not contact the cluster.** Payload, key and headers come back empty and
    /// `content_source` is `"meta-only"`. Callers that need the bodies ask for them later
    /// with [`SearchApi::fetch_messages`].
    async fn fetch_meta_rows(
        &self,
        workspace: &str,
        topic: &str,
        locs: Vec<LocItem>,
    ) -> Result<Vec<MessageResult>, EngineError>;

    /// Reads messages straight from the cluster, without going through the index.
    async fn fetch_kafka_messages_raw(
        &self,
        workspace: &str,
        topic: &str,
        limit: Option<usize>,
    ) -> Result<SearchIndexResponse, EngineError>;

    /// The indexed positions inside `[from_offset, to_offset]` of one partition.
    async fn fetch_browse_locs_in_bucket(
        &self,
        workspace: &str,
        topic: &str,
        partition: u32,
        from_offset: u64,
        to_offset: u64,
        sort_order: SortOrder,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// The indexed positions inside `[gte_ms, lte_ms]`, inclusive at both ends.
    ///
    /// Answers a drill into one time range without returning the whole result set;
    /// `limit` bounds it.
    #[allow(clippy::too_many_arguments)]
    async fn fetch_browse_locs_in_ts_range(
        &self,
        workspace: &str,
        topic: &str,
        gte_ms: Option<u64>,
        lte_ms: Option<u64>,
        sort_order: SortOrder,
        limit: Option<usize>,
        // Restricts to selected partitions. None = every partition.
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// One page of a browse, in time order. With all three `after_*` set, returns up to
    /// `limit` positions strictly after that one; otherwise the first page.
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
        // Restricts to a partition or offset span. None = every position.
        // `limit` counts what survives the filter.
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// Counts indexed messages per `(partition, bucket start)`, with no query applied.
    ///
    /// ⚠️ **Every message in the range is counted, and a query must not be applied here.**
    /// A caller that has a query and reaches for this anyway gets numbers that look right
    /// and are not — [`SearchApi::fetch_search_time_buckets`] is the one that counts
    /// matches.
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
        // Caller cap on the number of distinct bars. Tighten-only — None means the
        // library ceiling. Exceeding it fails with `EngineError::BucketOverflow`.
        hard_cap: Option<u64>,
        // Restricts to selected partitions. None = every partition.
        pos_filter: Option<PosFilter>,
    ) -> Result<TimeBucketsResponse, EngineError>;

    /// Counts the matches of a single-keyword search per `(partition, bucket start)`.
    ///
    /// Only matches inside `ts_range` are counted.
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
        // Caller cap on the number of distinct bars. Tighten-only — None means the
        // library ceiling. Exceeding it fails with `EngineError::BucketOverflow`.
        hard_cap: Option<u64>,
        // Restricts to a partition or offset span. None = every position.
        pos_filter: Option<PosFilter>,
    ) -> Result<TimeBucketsResponse, EngineError>;

    /// The matches of a single-keyword search inside one range, intersected with
    /// `ts_range`.
    ///
    /// `partition = None` narrows by time; `Some(p)` narrows to an offset span within that
    /// partition. Unlike the browse calls, only matching positions come back, and each one
    /// carries the field it matched on and its index key.
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
        // Restricts to a partition or offset span. None = every position.
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// One page of a single-keyword search, in time order — the counterpart of
    /// [`SearchApi::fetch_browse_locs_page`].
    ///
    /// With all three `after_*` set, returns up to `limit` positions strictly after that
    /// one; if any is `None`, the first page.
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
        // Restricts to a partition or offset span. None = every position.
        // `limit` counts what survives the filter.
        pos_filter: Option<PosFilter>,
    ) -> Result<Vec<LocItem>, EngineError>;

    /// Every indexed position in the topic, with no query. `matched_field` and the index
    /// key are `None` — there was no match to report.
    ///
    /// When the cap is reached the response sets `capped = true`; callers should fall back
    /// to bucket counts rather than treat the list as complete.
    ///
    /// `limit`:
    ///   - `None` = up to an internal safety cap
    ///   - `Some(n)` = stop after `n` and return immediately
    async fn prefetch_browse_locs(
        &self,
        workspace: &str,
        topic: &str,
        sort_order: SortOrder,
        limit: Option<usize>,
        ts_range: Option<TsRange>,
        // Restricts to a partition or offset span. None = every position.
        pos_filter: Option<PosFilter>,
    ) -> Result<PrefetchSearchLocsResponse, EngineError>;

    /// Fetches `limit` messages and decodes them with the given deserializers, so a schema
    /// can be checked against real data before any indexing starts.
    ///
    /// Uses only the deserializers passed in — the topic's saved configuration is neither
    /// read nor changed.
    async fn test_deserializers(
        &self,
        workspace: &str,
        topic: &str,
        deserializers: TopicDeserializers,
        bootstrap: Option<&str>,
        limit: usize,
    ) -> Result<TestDeserializeResponse, EngineError>;
}
