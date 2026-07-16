//! Cluster connection profile API — `~/.kaflow/clusters.json` CRUD (구 profiles.json — 2026-07-07 rename).

use async_trait::async_trait;
use kaflow_api_types::{ClusterProfile, StoredAuthConfig};

use crate::error::EngineError;

#[async_trait]
pub trait ProfilesApi: Send + Sync {
    /// 저장된 클러스터 프로필 목록 (최신 연결순).
    async fn list_cluster_profiles(&self) -> Result<Vec<ClusterProfile>, EngineError>;

    /// 클러스터 프로필 저장 / 갱신 (max 3, 초과 시 가장 오래된 항목 제거).
    /// `auth_config` 는 protocol / mechanism / cert 경로 등 비-비밀 정보만.
    async fn save_cluster_profile(
        &self,
        workspace: &str,
        bootstrap: &str,
        cluster_id: Option<&str>,
        auth_config: Option<StoredAuthConfig>,
    ) -> Result<(), EngineError>;

    /// 클러스터 프로필 삭제.
    async fn delete_cluster_profile(&self, workspace: &str) -> Result<(), EngineError>;
}
