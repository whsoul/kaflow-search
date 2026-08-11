//! Field selection commands.

use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::{IndexedFieldInput, ProtoFile, RegistrySubjectSchema, TopicDeserializers};
use std::sync::Arc;

#[tauri::command]
pub async fn set_indexed_fields(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    fields: Vec<IndexedFieldInput>,
) -> Result<(), String> {
    engine
        .set_indexed_fields(&workspace, &topic, fields)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn reindex_fields_from_meta(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    fields: Vec<String>,
) -> Result<usize, String> {
    engine
        .reindex_fields_from_meta(&workspace, &topic, fields)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn drop_fields_from_index(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    fields: Vec<String>,
) -> Result<usize, String> {
    engine
        .drop_fields_from_index(&workspace, &topic, fields)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn remove_topic_from_index(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
) -> Result<(usize, usize), String> {
    engine
        .remove_topic_from_index(&workspace, &topic)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn ensure_topic_watched(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
) -> Result<(), String> {
    engine
        .ensure_topic_watched(&workspace, &topic)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn set_topic_deserializers(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    deserializers: Option<TopicDeserializers>,
) -> Result<(), String> {
    engine
        .set_topic_deserializers(&workspace, &topic, deserializers)
        .await
        .map_err(|e| e.into_string())
}

/// Names fields to split into words before indexing begins. Past the limit this fails.
#[tauri::command]
pub async fn set_tokenize_fields(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    fields: Vec<String>,
) -> Result<(), String> {
    engine
        .set_tokenize_fields(&workspace, &topic, fields)
        .await
        .map_err(|e| e.into_string())
}

/// Reads a schema file the user chose.
///
/// The text is stored with the topic rather than the path alone, so moving or deleting the
/// file afterwards does not break a topic that was already configured.
#[tauri::command]
pub async fn read_schema_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("failed to read schema file: {path} → {e}"))
}

/// Pulls the path out of an `import` line, `public` and `weak` forms included.
fn parse_proto_import(line: &str) -> Option<String> {
    let l = line.trim();
    if l.starts_with("//") {
        return None;
    }
    let rest = l.strip_prefix("import")?.trim_start();
    // Must not run straight into an identifier — `importance` is not an import.
    let rest = rest
        .strip_prefix("public")
        .or_else(|| rest.strip_prefix("weak"))
        .unwrap_or(rest)
        .trim_start();
    let start = rest.find('"')?;
    let end = rest[start + 1..].find('"')?;
    Some(rest[start + 1..start + 1 + end].to_string())
}

/// Reads everything a `.proto` imports, so the set is closed and no longer depends on
/// files staying where they are.
///
/// Paths are resolved against the main file's directory, as `protoc` does, and each is
/// kept **exactly as written in the import** — that string is how the decoder finds it
/// again. The well-known types are left out; a decoder is expected to know those.
#[tauri::command]
pub async fn read_proto_closure(path: String) -> Result<Vec<ProtoFile>, String> {
    use std::collections::HashSet;
    use std::path::PathBuf;

    let main_path = PathBuf::from(&path);
    let root = main_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let main_text = std::fs::read_to_string(&main_path)
        .map_err(|e| format!("failed to read .proto: {path} → {e}"))?;

    let mut out: Vec<ProtoFile> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = main_text.lines().filter_map(parse_proto_import).collect();

    while let Some(imp) = stack.pop() {
        if imp.starts_with("google/protobuf/") {
            continue; // well-known — the decoder supplies these
        }
        if !visited.insert(imp.clone()) {
            continue; // already seen — imports may form cycles
        }
        let p = root.join(&imp);
        let text = std::fs::read_to_string(&p)
            .map_err(|e| format!("failed to read import '{imp}' ({}): {e}", p.display()))?;
        for nested in text.lines().filter_map(parse_proto_import) {
            stack.push(nested);
        }
        out.push(ProtoFile { name: imp, text });
    }
    Ok(out)
}

/// The latest schema a registry holds for one side of a topic.
///
/// `side` is `"key"` or `"value"`; anything else is used as the suffix as given.
///
/// For looking at only — nothing is decided or stored from it. This command differs from
/// its sibling only in building the subject name from a topic.
#[tauri::command]
pub async fn fetch_registry_subject_schema(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    registry_url: String,
    basic_auth: Option<String>,
    topic: String,
    side: String,
) -> Result<RegistrySubjectSchema, String> {
    let subject = format!("{topic}-{side}");
    let view = engine
        .fetch_registry_subject_latest(&registry_url, basic_auth.as_deref(), &subject)
        .await
        .map_err(|e| e.into_string())?;
    Ok(RegistrySubjectSchema {
        subject: view.subject,
        schema_id: view.schema_id,
        version: view.version,
        schema_text: view.schema_text,
    })
}

#[tauri::command]
pub async fn unwatch_topic(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
) -> Result<(usize, usize), String> {
    engine
        .unwatch_topic(&workspace, &topic)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn mark_topic_auto_cleaned(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    workspace: String,
    topic: String,
    timestamp_ms: i64,
) -> Result<(), String> {
    engine
        .mark_topic_auto_cleaned(&workspace, &topic, timestamp_ms)
        .await
        .map_err(|e| e.into_string())
}
