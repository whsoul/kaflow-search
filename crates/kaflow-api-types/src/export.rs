//! Writing search results out to a file.
//!
//! JSONL keeps the record whole; CSV and TSV flatten it for reading elsewhere.
//!
//! The serialization here is deliberately free of any I/O so that every engine writes
//! byte-identical files — reading and compressing belong elsewhere.

use serde::{Deserialize, Serialize};

use crate::{MessageResult, PosFilter, SearchMode, SearchQuery, SortOrder, TsRange};

/// The output format. JSONL preserves structure; CSV and TSV flatten it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Jsonl,
    Csv,
    Tsv,
}

/// Optional compression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportCompression {
    None,
    Gzip,
    Zstd,
}

/// What to export: the same parameters the search used, plus where to write.
///
/// Which kind of search is meant follows from what is set — a boolean query if one is
/// given, otherwise a keyword search, or everything when the query is empty.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub topic: String,
    pub query: String,
    pub fields: Option<Vec<String>>,
    pub mode: SearchMode,
    pub sort_order: SortOrder,
    pub ts_range: Option<TsRange>,
    pub pos_filter: Option<PosFilter>,
    /// Set to export the results of a boolean query.
    #[serde(default)]
    pub search_query: Option<SearchQuery>,
    /// Where to write it.
    pub dest_path: String,
    pub format: ExportFormat,
    pub compression: ExportCompression,
}

/// What was written.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    /// Records written.
    pub rows: usize,
    /// Final file size in bytes, after compression.
    pub bytes: u64,
    pub path: String,
}

/// Column order, shared by the header and every row.
const COLUMNS: &[&str] = &[
    "topic",
    "partition",
    "offset",
    "timestamp",
    "key",
    "headers",
    "payload",
];

/// The header row for CSV and TSV; `None` for JSONL. Ends with a newline.
pub fn export_header_line(format: ExportFormat) -> Option<String> {
    let delim = match format {
        ExportFormat::Csv => ',',
        ExportFormat::Tsv => '\t',
        ExportFormat::Jsonl => return None,
    };
    let mut s = COLUMNS.join(&delim.to_string());
    s.push('\n');
    Some(s)
}

/// One record as one line, ending with a newline.
pub fn serialize_export_row(row: &MessageResult, format: ExportFormat) -> String {
    match format {
        ExportFormat::Jsonl => serialize_jsonl(row),
        ExportFormat::Csv => serialize_delimited(row, ','),
        ExportFormat::Tsv => serialize_delimited(row, '\t'),
    }
}

/// ⚠️ Message headers may repeat a name, but are written as an object — where a name
/// repeats, the last one wins and the others are lost.
fn headers_to_json(headers: &[(String, String)]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in headers {
        map.insert(k.clone(), serde_json::Value::String(v.clone()));
    }
    serde_json::Value::Object(map)
}

fn serialize_jsonl(row: &MessageResult) -> String {
    // Which field matched is an artefact of searching, not of the message. Leaving it
    // out is what makes two different searches over the same rows produce the same file.
    let obj = serde_json::json!({
        "topic": row.topic,
        "partition": row.partition,
        "offset": row.offset,
        "ts": row.timestamp,
        "key": row.key,
        "headers": headers_to_json(&row.headers),
        "payload": row.payload,
    });
    let mut s = serde_json::to_string(&obj).unwrap_or_else(|_| "{}".to_string());
    s.push('\n');
    s
}

fn serialize_delimited(row: &MessageResult, delim: char) -> String {
    let headers_json =
        serde_json::to_string(&headers_to_json(&row.headers)).unwrap_or_else(|_| "{}".to_string());
    let fields: [String; 7] = [
        row.topic.clone(),
        row.partition.to_string(),
        row.offset.to_string(),
        row.timestamp.clone(),
        row.key.clone(),
        headers_json,
        row.payload.clone(),
    ];
    let mut out = String::new();
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            out.push(delim);
        }
        out.push_str(&escape_field(f, delim));
    }
    out.push('\n');
    out
}

/// Escapes one field. CSV quotes where it must and doubles any quote inside; TSV has no
/// agreed quoting, so control characters are replaced with spaces instead.
fn escape_field(s: &str, delim: char) -> String {
    if delim == '\t' {
        return s.replace(['\t', '\r', '\n'], " ");
    }
    if s.contains('"') || s.contains(delim) || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
