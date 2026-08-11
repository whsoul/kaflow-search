//! What an engine can fail with.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("workspace error: {0}")]
    Workspace(String),

    #[error("kafka error: {0}")]
    Kafka(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("indexing error: {0}")]
    Indexing(String),

    #[error("search error: {0}")]
    Search(String),

    #[error("deserialize error: {0}")]
    Deserialize(String),

    #[error("schema error: {0}")]
    Schema(String),

    /// The feature is not available to this caller.
    #[error("feature not entitled: {0}")]
    NotEntitled(String),

    /// Something else holds what this needed.
    #[error("lock conflict: {0}")]
    LockConflict(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// More buckets were asked for than may be returned.
    ///
    /// ⚠️ **This fails rather than returning part of the answer.** A truncated aggregate
    /// looks like a complete one, and a caller cannot tell the difference — better to say
    /// so and let the range or the granularity change. The message begins
    /// `"bucket overflow"` so a caller can recognise it without reading the wording.
    #[error("{0}")]
    BucketOverflow(String),

    /// The caller asked for this to stop.
    ///
    /// Not a failure — a caller should clear whatever was pending and say nothing, since
    /// the user already knows. Stringifies to `"cancelled"`.
    #[error("cancelled")]
    Cancelled,

    #[error("internal: {0}")]
    Internal(String),
}

impl EngineError {
    /// For a boundary that can only carry a string.
    pub fn into_string(self) -> String {
        self.to_string()
    }
}

impl From<EngineError> for String {
    fn from(e: EngineError) -> Self {
        e.to_string()
    }
}
