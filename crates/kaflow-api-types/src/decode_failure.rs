//! What happens when a message cannot be decoded.

use serde::{Deserialize, Serialize};

/// What to do when a message will not decode.
///
/// `Stop` keeps what was indexed so far and returns, with enough context to ask the user
/// what they want. `SkipAll` records a placeholder in its place and carries on, and is
/// meant to be chosen deliberately rather than defaulted to — it is how a topic ends up
/// indexed with holes in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecodeFailureMode {
    #[default]
    Stop,
    SkipAll,
}

/// Why a message would not decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecodeFailureKind {
    /// The bytes are not framed the way the deserializer expects.
    WireFormat,
    /// The schema could not be fetched.
    SchemaLookup,
    /// The schema was found, but the data did not match it.
    DecodeFailed,
    /// The deserializer cannot handle this at all.
    UnsupportedFormat,
    /// Anything else.
    Other,
}

impl DecodeFailureKind {
    /// The form this is indexed and searched under.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WireFormat => "wire_format",
            Self::SchemaLookup => "schema_lookup",
            Self::DecodeFailed => "decode_failed",
            Self::UnsupportedFormat => "unsupported_format",
            Self::Other => "other",
        }
    }
}

/// What is known about the failure that stopped indexing, enough to decide what to do.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeFailureContext {
    pub partition: i32,
    pub offset: i64,
    pub kind: DecodeFailureKind,
    /// The underlying error, unmodified.
    pub reason: String,
    /// The bytes that failed, base64-encoded, so they can be looked at. Truncated when
    /// large.
    pub raw_key_base64: Option<String>,
    pub raw_value_base64: Option<String>,
    /// Which side failed: `"key"` or `"value"`.
    pub failed_side: String,
}

// ── Reserved field names ────────────────────────────────────────────────────
//
// Names beginning `__` belong to the engine. They are searchable if asked for by name,
// but must be kept out of field listings and must never be dropped to reclaim space —
// they are how the engine accounts for what it did.

/// The prefix that marks a reserved field. The only test for one.
pub const SYSTEM_FIELD_PREFIX: &str = "__";

/// The reserved field holding a message's time. Recorded for every message, and what
/// anything ordered by time reads.
pub const SYSTEM_FIELD_TS: &str = "__ts";

/// Marks a message that stood in for one that would not decode.
pub const SYSTEM_FIELD_DECODE_FAILED: &str = "__decode_failed";

/// Which kind of failure it was.
pub const SYSTEM_FIELD_DECODE_ERROR_KIND: &str = "__decode_error_kind";

/// The underlying error.
pub const SYSTEM_FIELD_DECODE_ERROR_REASON: &str = "__decode_error_reason";

/// Whether a field name is reserved.
pub fn is_system_field(name: &str) -> bool {
    name.starts_with(SYSTEM_FIELD_PREFIX)
}
