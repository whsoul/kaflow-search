//! Kafka 메시지 DTO — sample 모드 / 인덱싱 / 검색 결과 공통 모델.
//!
//! 원래 `src-tauri/src/sample_data.rs` 에 있던 정의. domain 이주 과정에서
//! kaflow-api-types 로 격상 (DTO 성격이 명확하므로).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub timestamp: String,
    pub key: String,
    pub payload: String,
    pub headers: BTreeMap<String, String>,
}
