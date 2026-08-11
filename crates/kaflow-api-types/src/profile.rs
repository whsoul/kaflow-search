//! A saved cluster connection.
//!
//! ⚠️ **A password must not be kept here.** What may be stored is only what it takes to
//! offer the connection again and ask for the rest.

use crate::auth::StoredAuthConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ClusterProfile {
    pub workspace: String,
    pub bootstrap: String,
    pub cluster_id: Option<String>,
    pub last_connected_at: Option<String>,
    /// How to connect, secrets excluded. Absent in older saved profiles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_config: Option<StoredAuthConfig>,
}
