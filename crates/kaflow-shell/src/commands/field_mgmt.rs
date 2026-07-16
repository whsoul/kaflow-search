//! Field 관리 Tauri 어댑터 — `Arc<dyn KafkaToolEngine>` 경유.

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

/// 인덱싱 **전** 어절(tokenize) 대상 필드 지정 (picker). 한도 초과는 Err.
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

/// 사용자가 picker 에서 선택한 schema 파일 본문을 읽어 frontend 로 반환.
/// frontend 가 DeserializerSpec.schema_text 인라인 필드를 채워 set_topic_deserializers 로 보낸다.
/// → T-META 에 schema 본문이 인라인 저장되어 외부 파일 의존이 사라진다.
#[tauri::command]
pub async fn read_schema_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("failed to read schema file: {path} → {e}"))
}

/// `.proto` 한 줄에서 `import "..."` (public/weak 포함) 경로를 추출.
fn parse_proto_import(line: &str) -> Option<String> {
    let l = line.trim();
    if l.starts_with("//") {
        return None;
    }
    let rest = l.strip_prefix("import")?.trim_start();
    // 다음이 식별자 시작이면 안 됨 (예: `importance`) — 공백/따옴표/public/weak 만 허용.
    let rest = rest
        .strip_prefix("public")
        .or_else(|| rest.strip_prefix("weak"))
        .unwrap_or(rest)
        .trim_start();
    let start = rest.find('"')?;
    let end = rest[start + 1..].find('"')?;
    Some(rest[start + 1..start + 1 + end].to_string())
}

/// 로컬 `.proto` 의 import 폐쇄집합을 읽어 반환 (메인 제외, well-known 제외).
///
/// import 경로는 메인 파일의 부모 디렉터리(include root) 기준으로 해석한다 (protoc 관습).
/// `name` 은 `import "..."` 문자열 그대로 — 디코더의 in-memory resolver 키와 일치해야 한다.
/// `google/protobuf/*` 는 디코더의 `GoogleFileResolver` 가 자동 해석하므로 폐쇄집합에서 제외.
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
            continue; // well-known: 디코더가 자동 해석
        }
        if !visited.insert(imp.clone()) {
            continue; // 다이아몬드/사이클 dedup
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

/// Confluent Schema Registry 의 `{topic}-{side}` subject 최신 schema 조회.
///
/// `side` 는 `"key"` 또는 `"value"` (기타 값이면 그대로 subject suffix 로 사용).
/// TopicMetaConfigDrawer 의 read-only 미리보기 용도. 결정/저장 없음.
///
/// trait `RegistryApi::fetch_registry_subject_latest` 경유 (형제 명령
/// `commands/registry.rs::fetch_registry_subject_latest` 와 동일 패턴) — 호출자는
/// `{topic}-{side}` subject 조합만 담당하고, `RegistrySchemaView` 를 좁은
/// `RegistrySubjectSchema`(FE 계약 유지) 로 매핑해 반환한다.
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
