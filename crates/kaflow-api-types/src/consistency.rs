//! Consistency 도메인 DTO — verify_topic_consistency 응답.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsistencyCheck {
    pub name: String,
    pub ok: bool,
    pub expected: i64,
    pub actual: i64,
    pub diff: i64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsistencyReport {
    pub topic: String,
    pub overall_ok: bool,
    pub index_state: String,
    pub checks: Vec<ConsistencyCheck>,
}
