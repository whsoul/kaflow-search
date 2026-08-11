//! Saved Schema Registry connections.
//!
//! A deserializer refers to one by id rather than repeating its URL, so editing the
//! registry reaches every topic that uses it.

use serde::{Deserialize, Serialize};

/// One saved registry connection.
///
/// ⚠️ Unlike a cluster's password, `basic_auth` **is** written to disk, and in the clear.
/// Anything given here should be treated as stored rather than entered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryResource {
    /// A stable id. Deserializers refer to this, so it must not change.
    pub id: String,
    /// A name for people. Safe to change — nothing refers to it.
    pub name: String,
    /// The registry's base URL.
    pub url: String,
    /// Basic auth as `"user:password"`, where the registry needs it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_auth: Option<String>,
}

/// The result of trying a registry out.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryTestResult {
    pub ok: bool,
    /// What happened, in words.
    pub message: String,
    /// How many subjects it reported, where that could be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_count: Option<usize>,
}

/// One entry of a registry's whole schema index — enough of every schema to work out how
/// they refer to one another.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySchemaIndexEntry {
    pub subject: String,
    pub version: i32,
    pub schema_id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    /// The subjects this schema refers to; enough to invert into "referred to by".
    #[serde(default)]
    pub reference_subjects: Vec<String>,
}

/// One subject's latest schema, for looking at.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySchemaView {
    pub subject: String,
    pub schema_id: u32,
    pub version: i32,
    /// `"PROTOBUF"` or `"JSON"`; absent for Avro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    pub schema_text: String,
}
