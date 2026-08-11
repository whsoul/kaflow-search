//! Signs that a topic is no longer the one that was indexed, and that the index should be
//! rebuilt rather than continued.
//!
//! Each of these means the same offsets no longer refer to the same messages, so carrying
//! on would mix two topics together in one index.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum FullResyncTrigger {
    /// The cluster's id for the topic changed — it was recreated.
    TopicIdChanged { prev: String, curr: String },
    /// The number of partitions changed, so messages are addressed differently.
    PartitionCountChanged { prev: usize, curr: usize },
    /// The topic is configured differently and has to be read differently.
    TopicTypeChanged { prev: String, curr: String },
    /// The cluster's latest offset went backwards, which appending alone cannot do.
    LatestOffsetRegression {
        partition: u32,
        prev_indexed_latest: i64,
        curr_latest: i64,
    },
    /// The earliest offset went backwards. Retention only ever moves it forward, so this
    /// says the topic was recreated. Used only where the cluster is too old to report a
    /// topic id, which is the stronger signal.
    EarliestOffsetRegression {
        partition: u32,
        prev_earliest: i64,
        curr_earliest: i64,
    },
}
