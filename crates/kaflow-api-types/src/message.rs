//! A Kafka message, as indexing and searching both see it.

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
