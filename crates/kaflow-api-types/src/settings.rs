//! 설정 항목의 **코드 고정 범위(min/max/default) 단일 출처**.
//!
//! 설계 SSOT = `docs/settings_layers_design.md` (설정 3층 모델 + §10 최종 표).
//!
//! ## 왜 이 파일인가
//! 값이 `ilm.rs`(기본값) / `config.rs`(clamp) / FE 입력 `max=` 세 곳에 흩어져 있으면
//! 서로 어긋난다(실제로 FE 는 `max` 를 걸었는데 BE 는 `.max(1)` 만 하던 상태였다).
//! **범위는 여기 하나**만 보고, BE clamp·FE 입력·설정 화면 표시가 모두 이걸 참조한다.
//!
//! ## 층 모델 요약
//! - **유형1** = 코드 상수 (여기 `Bounds` 로 표현. 사용자·profile 모두 못 넘음)
//! - **유형2** = 요금제(profile)가 정하는 값 — `limits.rs`
//! - **유형3** = 사용자 설정 — 아래 `Bounds` 범위 **안에서** 자유. profile 이 더 좁힐 수 있음.
//!
//! `min`/`max` 는 **하드 범위**다. profile 은 이 안에서만 tighten 할 수 있고,
//! settings 파일에 범위 밖 값이 있어도 **로드 시점에 clamp** 한다
//! (debug 빌드에서 만진 값이 프로덕션에 그대로 살아남지 않게 — 입력 검증만으로는 못 막는다).

use crate::TOPIC_INDEX_COUNT_CAP;
use serde::{Deserialize, Serialize};

/// 한 설정 항목의 코드 고정 범위.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds<T> {
    pub min: T,
    pub max: T,
    pub default: T,
}

impl Bounds<u64> {
    pub const fn clamp(&self, v: u64) -> u64 {
        if v < self.min {
            self.min
        } else if v > self.max {
            self.max
        } else {
            v
        }
    }
    /// 하한만 우회 (debug 빌드 — cleanup/trim 테스트에 min 미만이 필요).
    /// 상한은 **우회하지 않는다** (시스템 안정성 축).
    pub const fn clamp_no_floor(&self, v: u64) -> u64 {
        let v = if v == 0 { 1 } else { v };
        if v > self.max {
            self.max
        } else {
            v
        }
    }

    /// **파괴적 한도 전용** 정규화 — 하한 위반은 `min` 이 아니라 **`default` 로 되돌린다.**
    ///
    /// 왜 비대칭인가 (2026-07-15 결정):
    /// - 이 값의 강제 수단은 **삭제(cleanup)** 다. **작을수록 더 많이 지운다.**
    ///   하한 위반값을 `min`(=합법적 최소)으로 붙이면 "가장 빡빡한" 방향으로 실패하는 셈이라,
    ///   profile 부재 시 fail-open 한 원칙과 정면으로 어긋난다.
    /// - 애초에 하한 미만은 **prod 사용자의 의사가 아니다** — UI 하한이 막고 있으므로, 파일에
    ///   그 값이 있다는 건 debug 빌드가 썼거나 손으로 편집했다는 뜻이다. 무효값으로 보고
    ///   **기본값으로 복귀**하는 편이 정직하다. (UI 로 고른 `min` 자체는 합법 — 그대로 존중된다.)
    /// - 상한 초과는 그대로 clamp — 위로 벗어난 값은 조여도 안정성 천장까지고, 아래로 벗어난
    ///   값은 조이면 **데이터가 사라진다.** 파괴 방향이 비대칭이므로 처리도 비대칭이다.
    pub const fn sanitize_destructive(&self, v: u64) -> u64 {
        if v < self.min {
            self.default
        } else if v > self.max {
            self.max
        } else {
            v
        }
    }
}

impl Bounds<usize> {
    pub const fn clamp(&self, v: usize) -> usize {
        if v < self.min {
            self.min
        } else if v > self.max {
            self.max
        } else {
            v
        }
    }
    /// 하한만 우회 (debug — 테스트 편의). 상한은 유지.
    pub const fn clamp_no_floor(&self, v: usize) -> usize {
        let v = if v == 0 { 1 } else { v };
        if v > self.max {
            self.max
        } else {
            v
        }
    }
}

const GB: u64 = 1_073_741_824;

// ── 유형3 (사용자 설정) ─────────────────────────────────────────────────────

/// 클러스터(워크스페이스)**당** 인덱스 디스크 한도.
///
/// - `min` = 1GB — 그 아래는 **thrash**(방금 인덱싱한 것을 size cleanup 이 즉시 삭제 →
///   "인덱싱은 되는데 검색하면 없음"). 다운로드 페이지 권장스펙에도 같은 값을 명시한다.
///   **debug 빌드는 이 하한을 우회**한다(`clamp_no_floor`) — cleanup 테스트에 필요.
/// - `max` = 100GB — 시스템 안정성 천장이자 **profile 부재 시 fail-open 값**.
///   강제 수단이 삭제(cleanup)라, 서버 장애로 한도가 조여지면 사용자 데이터가 사라진다.
///   → profile 이 없으면 **느슨한 쪽**으로 떨어져야 한다.
/// - `default` = 10GB — 토픽당 5M건 × ~500B ≈ 2.5GB, 인덱싱 토픽 5개 worst 12.5GB 를 방어.
pub const DISK_LIMIT_BYTES: Bounds<u64> = Bounds {
    min: GB,
    max: 100 * GB,
    default: 10 * GB,
};

/// size cleanup 주기 (초).
///
/// - `min` = 60 — CPU thrash 방지.
/// - `max` = 3,600 — 두 정리 사이에 인덱싱된 만큼은 한도를 넘는다
///   (`실제 최대 ≈ 한도 + 인덱싱속도 × 주기`). 주기를 무한정 늘리면 **디스크 한도를 우회**할 수 있다.
pub const SIZE_CLEANUP_INTERVAL_SECS: Bounds<u64> = Bounds {
    min: 60,
    max: 3_600,
    default: 600,
};

/// CountBased 정책의 **파티션당 보관 건수** 기본값 (토픽별 `max_count` 미지정 시 폴백).
/// 상위 클램프는 총량 캡이 담당 (`effective_max_count = min(N, topic_index_count_cap ÷ 파티션수)`).
pub const KEEP_COUNT_PER_PARTITION: Bounds<u64> = Bounds {
    min: 1,
    max: TOPIC_INDEX_COUNT_CAP,
    default: 100_000,
};

// ── 유형1 (debug 튜닝 — settings.json 미저장) ───────────────────────────────

pub const INDEXING_BATCH_SIZE: Bounds<usize> = Bounds {
    min: 100,
    max: 100_000,
    default: 1_000,
};

pub const AUTO_SYNC_SLICE_SIZE: Bounds<usize> = Bounds {
    min: 1_000,
    max: 1_000_000,
    default: 50_000,
};

/// retention cleanup **안전망 스윕** 주기 (초).
/// 실질 정리는 이벤트 기반 — FE 가 earliest offset 전진을 감지하면 즉시 트리거한다.
/// 이 주기는 "놓친 것을 줍는" 용도라 사용자 노브로 부적합 (유형1).
pub const RETENTION_CLEANUP_INTERVAL_SECS: Bounds<u64> = Bounds {
    min: 60,
    max: 86_400,
    default: 3_600,
};

pub const RETENTION_CLEANUP_WAIT_SECS: Bounds<u64> = Bounds {
    min: 1,
    max: 600,
    default: 60,
};

pub const SIZE_CLEANUP_WAIT_SECS: Bounds<u64> = Bounds {
    min: 1,
    max: 600,
    default: 30,
};

pub const FOREGROUND_ACQUIRE_WAIT_SECS: Bounds<u64> = Bounds {
    min: 1,
    max: 600,
    default: 30,
};

/// RocksDB 압축 — 변경 시 재연결 필요 + 기존 SST 는 그대로라 사용자 노출 부적합 (유형1, 벤치용).
pub const COMPRESSION_MODES: [&str; 3] = ["none", "snappy", "zstd"];
pub const COMPRESSION_MODE_DEFAULT: &str = "snappy";

// ── 유형2 (요금제 — profile 이 `fix` 로 내려줌) ─────────────────────────────
//
// `limits.rs` 의 상수들은 이 bounds 의 `default` 를 가리킨다 (숫자 중복 금지).
// min/max = **코드 고정 하드 범위** — profile 도 이 밖으로 못 나간다 (원격 데이터 불신).

pub const CLUSTER_PROFILE_CAP_B: Bounds<usize> = Bounds {
    min: 1,
    max: 10,
    default: 2,
};
pub const INDEXED_TOPIC_CAP_B: Bounds<usize> = Bounds {
    min: 1,
    max: 50,
    default: 5,
};
pub const TOKENIZE_FIELD_CAP_B: Bounds<usize> = Bounds {
    min: 1,
    max: 8,
    default: 1,
};
pub const SEARCH_MAX_GROUPS_B: Bounds<usize> = Bounds {
    min: 1,
    max: 10,
    default: 2,
};
pub const SEARCH_MAX_LEAVES_PER_GROUP_B: Bounds<usize> = Bounds {
    min: 1,
    max: 10,
    default: 3,
};
/// BE 안전망 — FE 빌더를 우회(JSON/AI 빌더)해도 이 leaf 총수를 넘으면 거부.
pub const QUERY_MAX_LEAVES_B: Bounds<usize> = Bounds {
    min: 1,
    max: 64,
    default: 10,
};
/// 토픽당 인덱스 건수. **상향 불가** — max = 조회 캡(`ORDERED_LOCS_CAP`)과 동치이며,
/// 넘기려면 loc IPC 페이로드(≈80MB/1M) 재설계가 선행돼야 한다. profile 은 tighten 만 가능.
pub const TOPIC_INDEX_COUNT_CAP_B: Bounds<u64> = Bounds {
    min: 1,
    max: TOPIC_INDEX_COUNT_CAP,
    default: TOPIC_INDEX_COUNT_CAP,
};

// ── profile 제약 (운영자/요금제가 내려주는 값) ──────────────────────────────

/// profile 이 한 항목에 거는 제약. **좁히는 방향으로만** 유효하다.
///
/// - `Fix`   — 값 고정 (사용자 노브가 없는 유형2 항목).
/// - `Max`   — 상한만 (사용자 노브가 있는 유형3 항목 — 그 안에서 사용자가 고른다).
/// - `Allow` — 허용 집합 (열거형 항목. 숫자가 아니라 "max" 가 성립하지 않는다.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProfileConstraint {
    Fix { value: u64 },
    Max { value: u64 },
    Allow { values: Vec<String> },
}

/// profile 봉투의 `limits` 섹션 — **항목 id 를 키로 한 열린 맵**.
///
/// 열린 맵인 이유: 나중에 새 제약을 추가할 때 **앱 재배포 없이** 서버만 바꾸면 되게.
/// 앱은 **아는 키만** 적용하고 모르는 키는 무시한다 (구버전 앱도 안전).
pub type ProfileLimits = std::collections::HashMap<String, ProfileConstraint>;

/// 요금제로 결정되는 값들 (유형2) — profile 해석 결과.
///
/// **profile 값도 hard bounds 로 clamp 한다.** 원격 데이터라 무조건 신뢰하지 않는다 —
/// 운영자가 실수로 토픽당 5,000만 건을 내려도 앱은 5M 에서 멈춘다.
///
/// ⚠ `rename_all = "camelCase"` 필수 — `EffectiveLimitsView` 안에 **중첩**되므로 부모의
/// rename 이 자식에 전파되지 않는다. 빠지면 FE 가 `plan.topicIndexCountCap` 을 `undefined` 로
///읽고 렌더 중 크래시한다 (실제 발생 — tsc 는 BE↔FE 계약 불일치를 못 잡는다).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanLimits {
    pub cluster_profile_cap: usize,
    pub indexed_topic_cap: usize,
    pub topic_index_count_cap: u64,
    pub tokenize_field_cap: usize,
    pub search_max_groups: usize,
    pub search_max_leaves_per_group: usize,
    pub query_max_leaves: usize,
}

impl Default for PlanLimits {
    /// profile 부재 시 = 출시 기본값. 유형2 는 **비파괴적**(저장/등록을 거부할 뿐 데이터를 지우지
    /// 않는다)이라 기본값으로 fail-closed 해도 안전하다.
    /// (파괴적 한도 — 디스크/보관 건수 — 는 반대로 fail-open. `effective_disk_limit` 참조.)
    fn default() -> Self {
        Self {
            cluster_profile_cap: CLUSTER_PROFILE_CAP_B.default,
            indexed_topic_cap: INDEXED_TOPIC_CAP_B.default,
            topic_index_count_cap: TOPIC_INDEX_COUNT_CAP_B.default,
            tokenize_field_cap: TOKENIZE_FIELD_CAP_B.default,
            search_max_groups: SEARCH_MAX_GROUPS_B.default,
            search_max_leaves_per_group: SEARCH_MAX_LEAVES_PER_GROUP_B.default,
            query_max_leaves: QUERY_MAX_LEAVES_B.default,
        }
    }
}

/// profile 의 항목 id (봉투 키). FE/서버와 공유하는 **문자열 계약**.
pub mod keys {
    pub const CLUSTER_COUNT: &str = "clusterCount";
    pub const INDEXED_TOPIC_COUNT: &str = "indexedTopicCount";
    pub const TOPIC_INDEX_COUNT: &str = "topicIndexCount";
    pub const TOKENIZE_FIELD_COUNT: &str = "tokenizeFieldCount";
    pub const SEARCH_GROUPS: &str = "searchGroups";
    pub const SEARCH_LEAVES_PER_GROUP: &str = "searchLeavesPerGroup";
    pub const QUERY_LEAVES: &str = "queryLeaves";
    /// 유형3 — 사용자 노브의 **상한**만 내린다 (`Max`).
    pub const DISK_BYTES: &str = "maxIndexBytesPerCluster";
}

/// profile 의 `Fix` 값을 usize bounds 로 해석. `Fix` 가 아니거나 없으면 default.
///
/// 유형2 항목에 `Max`/`Allow` 가 오면 **무시**한다 — 사용자 노브가 없는 항목에 상한은 의미가
/// 없고, 서버 오설정으로 앱 동작이 흔들리면 안 된다.
fn fix_usize(limits: &ProfileLimits, key: &str, bounds: Bounds<usize>) -> usize {
    match limits.get(key) {
        Some(ProfileConstraint::Fix { value }) => bounds.clamp(*value as usize),
        _ => bounds.default,
    }
}

fn fix_u64(limits: &ProfileLimits, key: &str, bounds: Bounds<u64>) -> u64 {
    match limits.get(key) {
        Some(ProfileConstraint::Fix { value }) => bounds.clamp(*value),
        _ => bounds.default,
    }
}

impl PlanLimits {
    /// profile 봉투 → 유형2 실효값. 모르는 키 / 허용 안 된 type 은 무시하고, 항상 bounds clamp.
    pub fn resolve(limits: &ProfileLimits) -> Self {
        Self {
            cluster_profile_cap: fix_usize(limits, keys::CLUSTER_COUNT, CLUSTER_PROFILE_CAP_B),
            indexed_topic_cap: fix_usize(limits, keys::INDEXED_TOPIC_COUNT, INDEXED_TOPIC_CAP_B),
            topic_index_count_cap: fix_u64(
                limits,
                keys::TOPIC_INDEX_COUNT,
                TOPIC_INDEX_COUNT_CAP_B,
            ),
            tokenize_field_cap: fix_usize(limits, keys::TOKENIZE_FIELD_COUNT, TOKENIZE_FIELD_CAP_B),
            search_max_groups: fix_usize(limits, keys::SEARCH_GROUPS, SEARCH_MAX_GROUPS_B),
            search_max_leaves_per_group: fix_usize(
                limits,
                keys::SEARCH_LEAVES_PER_GROUP,
                SEARCH_MAX_LEAVES_PER_GROUP_B,
            ),
            query_max_leaves: fix_usize(limits, keys::QUERY_LEAVES, QUERY_MAX_LEAVES_B),
        }
    }
}

/// FE 가 보는 **실효 한도** — 3층 해석 결과 한 벌.
///
/// FE 는 상수를 직접 읽지 않고 이것만 본다 (`resourceLimits.ts` / `searchLimits.ts` /
/// `tokenizeLimits.ts` 의 하드코딩 상수를 대체). 그래야 profile 이 바뀌면 화면도 따라온다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveLimitsView {
    /// 유형2 (요금제) — 읽기 전용 표시 + FE 사전 차단에 사용.
    pub plan: PlanLimits,
    /// 유형3 디스크 한도 실효값 (클러스터당).
    pub disk_limit_bytes: u64,
    /// FE 입력 범위 — `disk_max` 는 `min(profile.max ?? 100GB, 100GB)`.
    pub disk_min_bytes: u64,
    pub disk_max_bytes: u64,
    /// profile 이 `fix` 로 잠갔는가 — true 면 입력 비활성 + "운영자 설정" 배지.
    pub disk_locked: bool,
    /// debug 빌드 여부 — 하한 우회 / 유형1·2 편집 노출 게이팅에 사용.
    pub debug_build: bool,
}

/// 유형1 — **읽기 전용** 시스템 한도 한 벌 (설정 화면 debug 섹션 §G).
///
/// 왜 값만 보여주고 편집은 막나 (노출 규칙 ②, `docs/settings_layers_design.md` §9-0):
/// 이 상수들은 **검색 핫패스 곳곳에 박힌 `const`** 라, 화면에서 바꿔도 코드가 따라오지 않는다.
/// 편집 필드를 내면 거짓말이 된다. 성능 실험은 BenchPanel 이 담당하고, 여기서는 **진단용 표시**만.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemLimitEntry {
    /// 코드 상수 이름 (진단용 — 코드에서 바로 찾을 수 있게).
    pub name: String,
    pub value: u64,
    /// 한 줄 설명 (왜 이 값인가).
    pub note: String,
}

/// 설정 화면(debug §G)이 보여줄 유형1 한도 목록. 값의 출처는 각 crate 의 `const`.
/// 엔진이 채워 넣는다(여기서는 형태만 정의) — api-types 는 engine-impl 의 상수를 못 본다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SystemLimitsView {
    pub entries: Vec<SystemLimitEntry>,
}

/// 유형3 — 디스크 한도의 실효값.
///
/// `effective = clamp(사용자 설정, floor, min(profile.max ?? 100GB, 100GB))`
///
/// - profile 이 **없으면 상한은 100GB (fail-open)**. 이 한도의 강제 수단은 **삭제(cleanup)** 라,
///   서버 장애로 한도가 조여지면 사용자 인덱스가 지워진다. 느슨한 쪽으로 실패해야 한다.
/// - **하한 위반값은 `min` 이 아니라 `default`(10GB) 로 복귀**한다 (`sanitize_destructive`) —
///   같은 이유(작을수록 더 지운다 + prod 사용자 의사가 아니다).
/// - `debug_build = true` 면 **하한만** 우회 (cleanup/trim 테스트). 상한은 유지.
pub fn effective_disk_limit(user_value: u64, limits: &ProfileLimits, debug_build: bool) -> u64 {
    let profile_max = match limits.get(keys::DISK_BYTES) {
        Some(ProfileConstraint::Max { value }) => (*value).min(DISK_LIMIT_BYTES.max),
        // `Fix` = 운영자가 값을 잠근 경우 (기업 배포). 사용자 설정을 무시한다.
        Some(ProfileConstraint::Fix { value }) => {
            let fixed = DISK_LIMIT_BYTES.clamp(*value);
            return fixed;
        }
        _ => DISK_LIMIT_BYTES.max,
    };

    let v = if debug_build {
        DISK_LIMIT_BYTES.clamp_no_floor(user_value)
    } else {
        DISK_LIMIT_BYTES.sanitize_destructive(user_value)
    };
    v.min(profile_max)
}

/// 디스크 한도의 FE 입력 상한 — `min(profile.max ?? 100GB, 100GB)`.
pub fn disk_limit_max(limits: &ProfileLimits) -> u64 {
    match limits.get(keys::DISK_BYTES) {
        Some(ProfileConstraint::Max { value }) => (*value).min(DISK_LIMIT_BYTES.max),
        Some(ProfileConstraint::Fix { value }) => DISK_LIMIT_BYTES.clamp(*value),
        _ => DISK_LIMIT_BYTES.max,
    }
}

/// profile 이 디스크 한도를 잠갔는가 (`fix`).
pub fn disk_limit_locked(limits: &ProfileLimits) -> bool {
    matches!(
        limits.get(keys::DISK_BYTES),
        Some(ProfileConstraint::Fix { .. })
    )
}

impl EffectiveLimitsView {
    /// 3층 해석 한 벌 — `user_disk` = settings 층의 값(= 현재 GlobalConfig 값).
    pub fn resolve(user_disk: u64, limits: &ProfileLimits, debug_build: bool) -> Self {
        Self {
            plan: PlanLimits::resolve(limits),
            disk_limit_bytes: effective_disk_limit(user_disk, limits, debug_build),
            disk_min_bytes: if debug_build { 1 } else { DISK_LIMIT_BYTES.min },
            disk_max_bytes: disk_limit_max(limits),
            disk_locked: disk_limit_locked(limits),
            debug_build,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_inside_bounds() {
        assert!(DISK_LIMIT_BYTES.default >= DISK_LIMIT_BYTES.min);
        assert!(DISK_LIMIT_BYTES.default <= DISK_LIMIT_BYTES.max);
        assert!(SIZE_CLEANUP_INTERVAL_SECS.default >= SIZE_CLEANUP_INTERVAL_SECS.min);
        assert!(SIZE_CLEANUP_INTERVAL_SECS.default <= SIZE_CLEANUP_INTERVAL_SECS.max);
        assert!(KEEP_COUNT_PER_PARTITION.default <= KEEP_COUNT_PER_PARTITION.max);
        assert!(INDEXING_BATCH_SIZE.default >= INDEXING_BATCH_SIZE.min);
        assert!(AUTO_SYNC_SLICE_SIZE.default <= AUTO_SYNC_SLICE_SIZE.max);
    }

    #[test]
    fn clamp_pins_to_range() {
        assert_eq!(DISK_LIMIT_BYTES.clamp(0), GB); // 하한
        assert_eq!(DISK_LIMIT_BYTES.clamp(200 * GB), 100 * GB); // 상한
        assert_eq!(DISK_LIMIT_BYTES.clamp(5 * GB), 5 * GB);
        assert_eq!(SIZE_CLEANUP_INTERVAL_SECS.clamp(1), 60);
        assert_eq!(SIZE_CLEANUP_INTERVAL_SECS.clamp(86_400), 3_600);
    }

    /// debug 는 하한만 우회한다 — 상한(안정성)은 여전히 막힌다.
    #[test]
    fn debug_bypasses_floor_but_not_ceiling() {
        assert_eq!(
            DISK_LIMIT_BYTES.clamp_no_floor(100 * 1_048_576),
            104_857_600
        ); // 100MB 허용
        assert_eq!(DISK_LIMIT_BYTES.clamp_no_floor(0), 1); // 0 은 방어
        assert_eq!(DISK_LIMIT_BYTES.clamp_no_floor(500 * GB), 100 * GB); // 상한은 유지
    }

    /// 보관 건수의 상한은 총량 캡과 같아야 한다 (같은 축을 두 값이 다르게 말하면 안 됨).
    #[test]
    fn keep_count_ceiling_tracks_topic_cap() {
        assert_eq!(KEEP_COUNT_PER_PARTITION.max, TOPIC_INDEX_COUNT_CAP);
    }

    // ── 해석기 (3층) ────────────────────────────────────────────────────────

    fn limits_of(pairs: &[(&str, ProfileConstraint)]) -> ProfileLimits {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    /// profile 부재 = 출시 기본값 (유형2 는 비파괴적이라 fail-closed 안전).
    #[test]
    fn no_profile_yields_launch_defaults() {
        let plan = PlanLimits::resolve(&ProfileLimits::new());
        assert_eq!(plan, PlanLimits::default());
        assert_eq!(plan.cluster_profile_cap, 2);
        assert_eq!(plan.indexed_topic_cap, 5);
        assert_eq!(plan.topic_index_count_cap, TOPIC_INDEX_COUNT_CAP);
    }

    /// 유료 등급이 상향 — 단, **hard bounds 안에서만**.
    #[test]
    fn profile_fix_raises_within_bounds() {
        let plan = PlanLimits::resolve(&limits_of(&[
            (keys::CLUSTER_COUNT, ProfileConstraint::Fix { value: 5 }),
            (
                keys::INDEXED_TOPIC_COUNT,
                ProfileConstraint::Fix { value: 20 },
            ),
        ]));
        assert_eq!(plan.cluster_profile_cap, 5);
        assert_eq!(plan.indexed_topic_cap, 20);
    }

    /// **원격 데이터를 신뢰하지 않는다** — 운영자가 천장 밖 값을 내려도 앱은 천장에서 멈춘다.
    #[test]
    fn profile_cannot_exceed_hard_ceiling() {
        let plan = PlanLimits::resolve(&limits_of(&[
            (keys::CLUSTER_COUNT, ProfileConstraint::Fix { value: 9_999 }),
            (
                keys::TOPIC_INDEX_COUNT,
                ProfileConstraint::Fix { value: 50_000_000 }, // 5,000만
            ),
        ]));
        assert_eq!(plan.cluster_profile_cap, CLUSTER_PROFILE_CAP_B.max); // 10
        assert_eq!(plan.topic_index_count_cap, TOPIC_INDEX_COUNT_CAP); // 5M — 상향 불가
    }

    /// 유형2 항목에 `Max`/`Allow` 가 오면 무시 (사용자 노브가 없는 항목엔 상한이 무의미).
    #[test]
    fn wrong_constraint_type_is_ignored() {
        let plan = PlanLimits::resolve(&limits_of(&[
            (keys::CLUSTER_COUNT, ProfileConstraint::Max { value: 7 }),
            (
                keys::INDEXED_TOPIC_COUNT,
                ProfileConstraint::Allow {
                    values: vec!["x".into()],
                },
            ),
        ]));
        assert_eq!(plan.cluster_profile_cap, 2); // 기본값 유지
        assert_eq!(plan.indexed_topic_cap, 5);
    }

    /// 모르는 키는 무시 — 구버전 앱이 새 제약을 만나도 안전.
    #[test]
    fn unknown_keys_are_ignored() {
        let plan = PlanLimits::resolve(&limits_of(&[(
            "aiBuilderQuota",
            ProfileConstraint::Fix { value: 3 },
        )]));
        assert_eq!(plan, PlanLimits::default());
    }

    /// 디스크 한도: profile 부재 → **fail-open (100GB 상한)**.
    /// 강제 수단이 삭제라, 서버 장애로 조여지면 사용자 인덱스가 지워진다.
    #[test]
    fn disk_limit_fails_open_without_profile() {
        let none = ProfileLimits::new();
        assert_eq!(effective_disk_limit(50 * GB, &none, false), 50 * GB); // 사용자가 올려둔 값 유지
        assert_eq!(effective_disk_limit(200 * GB, &none, false), 100 * GB); // 안정성 천장은 clamp
    }

    /// **하한 위반값은 min 이 아니라 default(10GB) 로 복귀** — 파괴적 한도라 비대칭 (2026-07-15).
    /// min(1GB)으로 붙이면 "가장 많이 지우는" 방향으로 실패한다. 그리고 하한 미만은 애초에
    /// prod 사용자의 의사가 아니다(UI 가 막는다) → 무효값 → 기본값 복귀.
    #[test]
    fn sub_floor_persisted_value_falls_back_to_default_not_floor() {
        let none = ProfileLimits::new();
        assert_eq!(effective_disk_limit(100 * 1_048_576, &none, false), 10 * GB); // 100MB → 10GB
        assert_eq!(effective_disk_limit(0, &none, false), 10 * GB);
        // 사용자가 UI 로 고른 합법적 최소(1GB)는 그대로 존중된다.
        assert_eq!(effective_disk_limit(GB, &none, false), GB);
    }

    /// profile 이 상한을 좁히면 사용자 값이 그 안으로 clamp (다운그레이드).
    #[test]
    fn disk_limit_tightened_by_profile_max() {
        let free = limits_of(&[(keys::DISK_BYTES, ProfileConstraint::Max { value: 10 * GB })]);
        assert_eq!(effective_disk_limit(50 * GB, &free, false), 10 * GB);
        assert_eq!(effective_disk_limit(3 * GB, &free, false), 3 * GB); // 그 아래는 사용자 자유
    }

    /// profile `Fix` = 운영자 잠금 (기업 배포) — 사용자 설정을 무시한다.
    #[test]
    fn disk_limit_locked_by_profile_fix() {
        let locked = limits_of(&[(keys::DISK_BYTES, ProfileConstraint::Fix { value: 5 * GB })]);
        assert_eq!(effective_disk_limit(50 * GB, &locked, false), 5 * GB);
        assert_eq!(effective_disk_limit(1 * GB, &locked, false), 5 * GB);
    }

    /// **FE 계약 고정** — `EffectiveLimitsView` 는 FE 가 그대로 읽는 JSON 이다.
    /// 중첩된 `PlanLimits` 에 `rename_all` 을 빠뜨려 snake_case 로 내려갔고, FE 가
    /// `plan.topicIndexCountCap` 을 undefined 로 읽어 **설정 드로어가 렌더 중 크래시**했다
    /// (2026-07-15). tsc 는 BE↔FE 계약 불일치를 못 잡으므로 **키 이름을 여기서 고정**한다.
    #[test]
    fn effective_limits_view_serializes_camel_case_for_fe() {
        let v = EffectiveLimitsView::resolve(10 * GB, &ProfileLimits::new(), false);
        let json = serde_json::to_value(&v).unwrap();

        // 최상위
        for key in [
            "plan",
            "diskLimitBytes",
            "diskMinBytes",
            "diskMaxBytes",
            "diskLocked",
            "debugBuild",
        ] {
            assert!(json.get(key).is_some(), "최상위 키 누락/이름 불일치: {key}");
        }
        // **중첩된 plan** — 부모의 rename 은 자식에 전파되지 않는다.
        let plan = json.get("plan").unwrap();
        for key in [
            "clusterProfileCap",
            "indexedTopicCap",
            "topicIndexCountCap",
            "tokenizeFieldCap",
            "searchMaxGroups",
            "searchMaxLeavesPerGroup",
            "queryMaxLeaves",
        ] {
            assert!(plan.get(key).is_some(), "plan 키 누락/이름 불일치: {key}");
        }
        // snake_case 가 섞여 나가면 FE 는 undefined 를 읽는다.
        assert!(plan.get("cluster_profile_cap").is_none());
    }

    /// debug 는 하한만 우회 — 상한(안정성)과 profile 제약은 그대로.
    #[test]
    fn debug_bypasses_floor_only() {
        let none = ProfileLimits::new();
        assert_eq!(
            effective_disk_limit(100 * 1_048_576, &none, true),
            100 * 1_048_576
        );
        assert_eq!(effective_disk_limit(500 * GB, &none, true), 100 * GB);
    }
}
