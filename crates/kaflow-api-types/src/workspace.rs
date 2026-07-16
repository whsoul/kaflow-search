//! Workspace 단위 메타 / 스키마 버전.

use serde::{Deserialize, Serialize};

/// Storage layout schema 버전.
/// 1 = 단일 DB + 키 prefix (legacy, pre CF migration)
/// 2 = Column Family per topic
/// 3 = M-key value 구조 변경 (`index_entries` → `field_values` + `tokenized_fields`, 세션 36)
/// 신규 워크스페이스 = 현재값으로 세팅. 더 낮은 버전 워크스페이스 만나면 reset 안내 후 진행 차단.
pub const CURRENT_SCHEMA_VERSION: u32 = 3;

fn default_schema_version_v1() -> u32 {
    1
}

/// 워크스페이스 단위 메타 — 토픽과 무관한 클러스터/환경 정보를 저장한다.
/// RocksDB 키: default CF 의 `W\x00` (단일 엔트리).
///
/// `schema_version` 필드가 누락된 (legacy) JSON 은 `serde(default)` 로 자동으로 1 매핑 →
/// FE 가 reset 안내 모달을 띄운다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMeta {
    /// Kafka 클러스터 ID (MetadataResponse v3+ 에서 조회). 클러스터 재생성 감지에 활용.
    pub cluster_id: Option<String>,
    /// Storage layout schema version. `None`/누락 → legacy(1) 로 매핑.
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
