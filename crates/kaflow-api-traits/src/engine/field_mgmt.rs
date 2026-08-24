//! Choosing what of a topic is indexed, and keeping that up to date.
//!
//! ⚠️ **Reserved fields are not the user's to manage.** A field recognised by
//! [`kaflow_api_types::is_system_field`] must be kept out of anything offered for
//! selection, and must not be removed by anything here or by reclaiming space.

use async_trait::async_trait;
use kaflow_api_types::{IndexedFieldInput, TopicDeserializers};

use crate::error::EngineError;

#[async_trait]
pub trait FieldMgmtApi: Send + Sync {
    /// Sets which fields a topic indexes.
    async fn set_indexed_fields(
        &self,
        workspace: &str,
        topic: &str,
        fields: Vec<IndexedFieldInput>,
    ) -> Result<(), EngineError>;

    /// Builds the index for fields that were not indexed before, from what is already
    /// stored — without reading the topic again. Returns how many entries were written.
    async fn reindex_fields_from_meta(
        &self,
        workspace: &str,
        topic: &str,
        fields: Vec<String>,
    ) -> Result<usize, EngineError>;

    /// Removes those fields from the index. Returns how many entries went.
    async fn drop_fields_from_index(
        &self,
        workspace: &str,
        topic: &str,
        fields: Vec<String>,
    ) -> Result<usize, EngineError>;

    /// Removes the topic and everything indexed for it.
    async fn remove_topic_from_index(
        &self,
        workspace: &str,
        topic: &str,
    ) -> Result<(usize, usize), EngineError>;

    /// Starts following a topic, before anything has been indexed for it.
    async fn ensure_topic_watched(&self, workspace: &str, topic: &str) -> Result<(), EngineError>;

    /// Changes how a topic decodes; `None` returns it to JSON.
    async fn set_topic_deserializers(
        &self,
        workspace: &str,
        topic: &str,
        deserializers: Option<TopicDeserializers>,
    ) -> Result<(), EngineError>;

    /// Names fields to split into words before any field has been discovered, leaving
    /// everything else indexed as it was. Past the limit this fails rather than truncating
    /// the list — silently indexing fewer fields than were asked for would be worse.
    async fn set_tokenize_fields(
        &self,
        workspace: &str,
        topic: &str,
        fields: Vec<String>,
    ) -> Result<(), EngineError>;

    /// Stops following a topic, at the user's request.
    async fn unwatch_topic(
        &self,
        workspace: &str,
        topic: &str,
    ) -> Result<(usize, usize), EngineError>;

    /// Records that a topic's index was removed to reclaim space, rather than by choice.
    async fn mark_topic_auto_cleaned(
        &self,
        workspace: &str,
        topic: &str,
        timestamp_ms: i64,
    ) -> Result<(), EngineError>;
}
