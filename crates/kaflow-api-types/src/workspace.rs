//! What is known about a workspace as a whole.

use serde::{Deserialize, Serialize};

/// The version of the stored layout.
/// 1 = the original layout
/// 2 = Column Family per topic
/// 3 = what is stored per message changed
///
/// A workspace written by an older version cannot be read: the user has to be told, and
/// nothing further attempted, rather than reading it as though it were current.
pub const CURRENT_SCHEMA_VERSION: u32 = 3;

fn default_schema_version_v1() -> u32 {
    1
}

/// Workspace-level information — what belongs to the cluster rather than to any topic.
///
/// A value with no version recorded is from before there were versions, and reads as 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMeta {
    /// The cluster's own id, which is how a different cluster at the same address is
    /// noticed.
    pub cluster_id: Option<String>,
    /// The stored layout version; absent means 1.
    #[serde(default = "default_schema_version_v1")]
    pub schema_version: u32,
}

impl Default for WorkspaceMeta {
    fn default() -> Self {
        Self {
            cluster_id: None,
            schema_version: CURRENT_SCHEMA_VERSION,
        }
    }
}
