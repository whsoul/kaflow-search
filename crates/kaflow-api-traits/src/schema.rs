//! Where a schema comes from — a registry, a file, or one supplied directly.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// How a schema is named. Which form applies depends on where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchemaId {
    /// A numeric id, as most registries use.
    Numeric(u32),
    /// A subject and a version.
    Subject { subject: String, version: i32 },
    /// A path, or a name given to something supplied directly.
    Name(String),
}

/// A schema as it was fetched, left in its own form for whoever decodes with it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub id: SchemaId,
    /// Which kind it is.
    pub kind: String,
    /// The schema itself, unmodified.
    pub raw: String,
}

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("schema not found: {0:?}")]
    NotFound(SchemaId),

    #[error("schema source unreachable: {0}")]
    Unreachable(String),

    #[error("schema source unauthorized: {0}")]
    Unauthorized(String),

    #[error("schema parse failed: {0}")]
    Parse(String),

    #[error("schema source error: {0}")]
    Other(String),
}

/// Fetching schemas. Whether and how they are cached is left to the implementation.
#[async_trait]
pub trait SchemaSource: Send + Sync {
    /// A name for this source, to show to a user.
    fn id(&self) -> String;

    async fn fetch_schema(&self, id: &SchemaId) -> Result<Schema, SchemaError>;
}
