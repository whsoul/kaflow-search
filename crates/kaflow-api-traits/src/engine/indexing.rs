//! Reading a topic into the index.

use async_trait::async_trait;
use kaflow_api_types::{
    CompactDeletedKeyCursor, CompactDeletedKeysPage, CompactKeyHistory, DecodeFailureKind,
    DecodeFailureMode, OpenKafkaTopicResponse,
};

use crate::error::EngineError;

#[async_trait]
pub trait IndexingApi: Send + Sync {
    /// Opens a topic: works out where indexing stands, brings it up to date, and lets
    /// retention remove what the cluster no longer has.
    ///
    /// `max_messages` stops early after that many, with `has_more` saying whether there is
    /// more to do; the next call resumes on its own. `None` runs to the end.
    ///
    /// `skip_resync_check` bypasses the checks that decide whether the index still matches
    /// the topic. ⚠️ Only for a caller that has already made that decision — skipping it
    /// blindly is how two different topics end up in one index.
    async fn open_kafka_topic(
        &self,
        workspace: &str,
        topic: &str,
        bootstrap: &str,
        skip_resync_check: Option<bool>,
        decode_failure_mode: Option<DecodeFailureMode>,
        max_messages: Option<u64>,
    ) -> Result<OpenKafkaTopicResponse, EngineError>;

    /// Asks indexing to stop.
    ///
    /// **Must return without waiting for indexing to end** — a caller watches the progress
    /// events for that. Stopping a topic that is not indexing is allowed and does nothing.
    ///
    /// ⚠️ **What is indexed when it stops has to stand on its own.** Resuming afterwards
    /// must need no reconciliation: nothing left half-written, and nothing already read
    /// left out of the index.
    ///
    /// Whether work in flight is finished or thrown away is left to the implementation —
    /// **nothing may depend on either.**
    async fn cancel_indexing(&self, workspace: &str, topic: &str) -> Result<(), EngineError>;

    /// Records a placeholder for one message that would not decode, so indexing can pass
    /// over it rather than stopping again on the same message.
    ///
    /// The placeholder occupies the message's position and is searchable by the reserved
    /// fields alone. The bytes are not kept — they can still be read from the cluster for
    /// as long as it holds them.
    async fn record_placeholder(
        &self,
        workspace: &str,
        topic: &str,
        partition: i32,
        offset: i64,
        kind: DecodeFailureKind,
        reason: String,
    ) -> Result<(), EngineError>;

    /// The history kept for one key of a compacted topic.
    ///
    /// `key_raw` must be the key as its deserializer renders it — the same form indexing
    /// used, or nothing will match. `None` where no history is held for it.
    async fn fetch_compact_key_history(
        &self,
        workspace: &str,
        topic: &str,
        partition: u32,
        key_raw: &str,
    ) -> Result<Option<CompactKeyHistory>, EngineError>;

    /// One page of the keys that have been deleted.
    ///
    /// These cannot be found by searching — nothing else in the index refers to them — so
    /// this is the only way to list them. `key_query` matches anywhere within the key
    /// rather than at its start, which is what makes a structured key searchable by a part
    /// of it. `limit` may tighten the page size but not raise it past the engine's own.
    async fn fetch_compact_deleted_keys_page(
        &self,
        workspace: &str,
        topic: &str,
        key_query: &str,
        partitions: Option<Vec<u32>>,
        cursor: Option<CompactDeletedKeyCursor>,
        limit: Option<u32>,
    ) -> Result<CompactDeletedKeysPage, EngineError>;
}
