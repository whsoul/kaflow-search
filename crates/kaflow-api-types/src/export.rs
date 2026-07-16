//! Export (검색결과 내보내기) DTO + 순수 직렬화 헬퍼.
//!
//! 설계: `docs/pre_launch_specs.md §2`. v1 = browse+keyword(단일 토픽) BE 스트리밍 export.
//! - 무손실 원본 = **JSONL**(한 줄 = 한 레코드), 사람용 평면 = **CSV/TSV**.
//! - 향후 유료 offline reader / 백업·복원의 lean 스키마 선례.
//!   record = topic/partition/offset/ts/key/headers/payload(+matchedField).
//!   방향 상세 = memory `project_monetization_roadmap`.
//!
//! ⚠️ 본 모듈 직렬화는 **인프라 의존 0 순수 함수**(api-types 규칙) — 파일 IO/압축/fetch 는
//!    engine-impl `export.rs` 담당. 두 엔진(impl/mock)이 이 직렬화를 공유한다.

use serde::{Deserialize, Serialize};

use crate::{MessageResult, PosFilter, SearchMode, SearchQuery, SortOrder, TsRange};

/// 출력 파일 포맷. JSONL = 무손실(구조 보존), CSV/TSV = 평면(사람/스프레드시트용).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Jsonl,
    Csv,
    Tsv,
}

/// 출력 압축. writer 를 한 겹 감싸는 부가 옵션.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportCompression {
    None,
    Gzip,
    Zstd,
}

/// 검색결과 export 요청 — 파라미터는 현재 검색과 동일 + 출력 대상.
/// 소스 분기: `search_query=Some` → **multi(상세검색)**(ts범위/위치필터는 SearchQuery 내장).
/// `None` + 빈 `query` → **browse**, `None` + 비어있지 않은 `query` → **keyword**.
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
    /// multi(상세검색) 소스 — 있으면 이 bool AST 로 매치 loc 페이징. browse/keyword 는 None.
    #[serde(default)]
    pub search_query: Option<SearchQuery>,
    /// 저장 대상 경로 (FE save dialog 결정).
    pub dest_path: String,
    pub format: ExportFormat,
    pub compression: ExportCompression,
}

/// export 완료 요약.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    /// 실제 기록한 레코드 수.
    pub rows: usize,
    /// 최종 파일 크기(bytes, 압축 후).
    pub bytes: u64,
    pub path: String,
}

/// CSV/TSV 컬럼 순서 (헤더 + 각 행 동일).
const COLUMNS: &[&str] = &[
    "topic",
    "partition",
    "offset",
    "timestamp",
    "key",
    "headers",
    "payload",
];

/// CSV/TSV 헤더 행 (JSONL 은 None). 끝에 `\n` 포함.
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

/// 한 레코드 → 한 줄(끝에 `\n`). JSONL=무손실 JSON, CSV/TSV=평면(headers 는 JSON 문자열 한 칸).
pub fn serialize_export_row(row: &MessageResult, format: ExportFormat) -> String {
    match format {
        ExportFormat::Jsonl => serialize_jsonl(row),
        ExportFormat::Csv => serialize_delimited(row, ','),
        ExportFormat::Tsv => serialize_delimited(row, '\t'),
    }
}

/// ⚠️ Kafka 헤더는 중복 키 가능하나 v1 은 object 로 표현(중복 시 후자 우선).
///    무손실이 중요해지면 array-of-pairs 로 승격(offline reader 도입 시 재검토).
fn headers_to_json(headers: &[(String, String)]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in headers {
        map.insert(k.clone(), serde_json::Value::String(v.clone()));
    }
    serde_json::Value::Object(map)
}

fn serialize_jsonl(row: &MessageResult) -> String {
    // matchedField 는 검색 화면 주석(어느 필드에 걸렸나)이라 export(순수 데이터)엔 넣지 않는다.
    // → keyword/multi 가 같은 rows 에 대해 byte-동일한 파일을 낸다.
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

/// CSV(delim=',')/TSV(delim='\t') 필드 escape.
/// CSV: `"`·delim·CR·LF 포함 시 큰따옴표로 감싸고 내부 `"`→`""`.
/// TSV: tab/CR/LF 를 공백으로 치환(엄격 quoting 규격이 없어 관습적 처리).
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
