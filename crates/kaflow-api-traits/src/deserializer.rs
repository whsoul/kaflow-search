//! Turning message bytes into something searchable.
//!
//! Some formats describe themselves and need nothing else; others carry a schema id and
//! need somewhere to look it up.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// What is known about a message besides its bytes. Some of it applies only to some
/// formats.
#[derive(Debug, Clone, Default)]
pub struct DecodeCtx {
    /// The topic it came from, which some registries key their schemas by.
    pub topic: Option<String>,
    /// A schema chosen explicitly, overriding whatever the bytes say.
    pub schema_id: Option<u32>,
}

/// A decoded message, normalized to one representation so that everything downstream
/// works the same whatever the format was.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedValue {
    /// The decoded value.
    pub json: serde_json::Value,
    /// Which schema was used, where one was.
    pub schema_id: Option<u32>,
    /// Which decoder did it. For showing and for diagnosis.
    pub deserializer_id: &'static str,
}

#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error("wire format mismatch: {0}")]
    WireFormat(String),

    #[error("schema lookup failed: {0}")]
    SchemaLookup(String),

    #[error("decode failed: {0}")]
    Decode(String),

    #[error("unsupported format: {0}")]
    Unsupported(String),
}

/// Decodes message bytes.
///
/// An implementation names itself with `id()`, and a topic's settings name the one to
/// use — so the id is a stable contract, not a label.
pub trait Deserializer: Send + Sync {
    /// This decoder's id. A topic refers to it by this, so it must not change.
    fn id(&self) -> &'static str;

    /// Decodes one message.
    fn decode(&self, bytes: &[u8], ctx: &DecodeCtx) -> Result<DecodedValue, DeserializeError>;
}
