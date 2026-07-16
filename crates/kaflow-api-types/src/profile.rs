//! 클러스터 연결 프로필 DTO — `~/.kaflow/clusters.json` 직렬화 형태 (구 profiles.json).
//!
//! 비밀 정보 (password) 는 저장하지 않는다. `auth_config` 필드는 `StoredAuthConfig`
//! (protocol / mechanism / cert 경로 등 비-비밀 정보) 만.

use crate::auth::StoredAuthConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ClusterProfile {
    pub workspace: String,
    pub bootstrap: String,
    pub cluster_id: Option<String>,
    pub last_connected_at: Option<String>,
    /// 인증/보안 프로토콜 설정. password 등 비밀 정보는 미저장.
    /// 기존 clusters.json(구 profiles.json) 역호환을 위해 `default = None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_config: Option<StoredAuthConfig>,
}
