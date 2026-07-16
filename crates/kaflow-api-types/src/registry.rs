//! Schema Registry 리소스 DTO — `~/.kaflow/schema_registries.json` 직렬화 형태.
//!
//! deserializer 의 `*ConfluentRegistryRef` 변종이 `id` 로 이 리소스를 참조한다 (살아있는 참조).
//! 리소스의 `url` 을 바꾸면 그 id 를 쓰는 모든 토픽의 디코딩에 즉시 반영된다.

use serde::{Deserialize, Serialize};

/// 등록된 Schema Registry 연결 1건. password 같은 비밀 정보가 들어갈 수 있는
/// `basic_auth` (`"user:password"`) 는 cluster profile 과 달리 로컬 파일에 평문 저장된다
/// (테스트 도구 한정 — 운영 제품 전환 시 keychain 등으로 격상 검토).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryResource {
    /// 안정 식별자 (FE 에서 `crypto.randomUUID()` 로 생성). deserializer spec 이 이 값을 참조.
    pub id: String,
    /// 사용자 표시용 별칭 (예: `local`, `staging`). 수정 가능, 참조는 id 기준이라 rename 안전.
    pub name: String,
    /// Schema Registry base URL (예: `http://localhost:8081`).
    pub url: String,
    /// `"user:password"` 형식 basic auth. 없으면 None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_auth: Option<String>,
}

/// 연결 테스트 결과 — `GET <url>/subjects` 호출 후 UI 에 표시.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryTestResult {
    pub ok: bool,
    /// 성공/실패 사유 메시지 (UI 인라인 표시용).
    pub message: String,
    /// 성공 시 `/subjects` 응답의 subject 수 (파싱 가능할 때).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_count: Option<usize>,
}

/// 레지스트리 전체 스키마 인덱스 1건 — `GET /schemas?latestOnly=true` 응답의 한 항목.
/// 한 번 호출로 모든 subject 의 id/type + 참조 대상을 얻어 참조 그래프를 만든다(N+1 → 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySchemaIndexEntry {
    pub subject: String,
    pub version: i32,
    pub schema_id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    /// 이 스키마가 참조하는 다른 subject 들 (references[].subject). 역방향(referenced-by) 계산용.
    #[serde(default)]
    pub reference_subjects: Vec<String>,
}

/// 등록 화면의 스키마 브라우저 — `GET /subjects/{subject}/versions/latest` 결과 1건.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySchemaView {
    pub subject: String,
    pub schema_id: u32,
    pub version: i32,
    /// `"PROTOBUF"` / `"JSON"` / 없으면(AVRO) None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    pub schema_text: String,
}
