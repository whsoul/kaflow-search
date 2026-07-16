//! Full resync 트리거 DTO — `kafka-ilm::full_resync_detector::detect` 가 반환한다.
//!
//! `docs/index_full_resync_policy.md` 에 정의된 트리거 중 토픽 단위 (Tier 1+2):
//! - `TopicIdChanged` — topicId 변경 (토픽 재생성 의심)
//! - `PartitionCountChanged` — 파티션 수 변경 (주소 체계 변경)
//! - `TopicTypeChanged` — cleanup.policy / topic_type 변경 (해석 규칙 변경)
//! - `LatestOffsetRegression` — 현재 latest < stored latestIndexedOffsets+1 (append-only 깨짐)
//! - `EarliestOffsetRegression` — 현재 earliest < stored earliest (재생성 대체 신호;
//!   topicId 비교가 불가능한 구버전(<2.8) 클러스터에서만 평가됨)
//!
//! 프론트 노출을 전제로 camelCase 직렬화. `rename_all_fields` 로 variant 내부 필드까지 변환.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum FullResyncTrigger {
    /// topicId 가 기존 값과 다름.
    TopicIdChanged { prev: String, curr: String },
    /// 파티션 수가 다름.
    PartitionCountChanged { prev: usize, curr: usize },
    /// cleanup.policy / topic_type 이 다름.
    TopicTypeChanged { prev: String, curr: String },
    /// 해당 파티션의 현재 broker latest 가 기존 indexed latest 보다 작음.
    LatestOffsetRegression {
        partition: u32,
        prev_indexed_latest: i64,
        curr_latest: i64,
    },
    /// 해당 파티션의 현재 broker earliest 가 저장된 earliest 보다 작음.
    /// 정상 운영에서 log start offset 은 감소하지 않으므로(retention 은 전진만)
    /// 토픽 재생성 신호. topicId 비교가 불가능할 때만 평가되는 대체 신호.
    EarliestOffsetRegression {
        partition: u32,
        prev_earliest: i64,
        curr_earliest: i64,
    },
}
