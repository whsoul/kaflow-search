//! 플랜(등급) 리소스 한도 — **구조적 천장(안전 바닥)의 단일 출처.**
//!
//! CLAUDE.md "리소스 한도 — 2층 캡" 규약의 라이브러리 쪽 층이다.
//! FE 대칭 = `src/resourceLimits.ts` / `src/tokenizeLimits.ts` (UX 사전 차단 — 보장 아님).
//! 여기 상수들이 우회 경로(직접 invoke / JSON 조작)를 막는 실제 강제선이다.
//!
//! 향후 라이선스 등급(`docs/requriement/mvp-license-and-admin-policy.md`) 도입 시:
//! 등급별 한도는 gate/appProfile 봉투에서 내려오고(tighten-only), 이 상수들은
//! "최상위 등급도 못 넘는 천장"으로 남는다.

/// 저장 가능한 클러스터 연결 프로필 수. 초과 시 저장 **거부** (silent eviction 금지).
/// 출시값 2 (2026-07-13 확정): 워크스페이스마다 별도 RocksDB → 디스크 한도(10GB)가 프로필
/// 수만큼 곱해진다(2 × 10GB = 20GB 상한). 무료 등급에서 멀티클러스터는 "맛보기" 수준으로 두고,
/// 상향은 유료 등급의 차별화 축으로 남긴다.
pub const CLUSTER_PROFILE_CAP: usize = crate::settings::CLUSTER_PROFILE_CAP_B.default;

/// 워크스페이스당 동시 인덱싱(watched = T-META 보유 + 자동정리 안 됨) 토픽 수.
/// 강제 지점: `ensure_topic_watched`(피커 등록) + `open_kafka_topic`(직접 인덱싱 경로).
pub const INDEXED_TOPIC_CAP: usize = crate::settings::INDEXED_TOPIC_CAP_B.default;

// (구 `PER_TOPIC_MAX_COUNT_CAP`(파티션당 10만)은 2026-07-13 제거 — 총량 캡이 성능 축을,
//  10GB/추천 500MB 가 디스크 축을 담당해 고유 축이 없었고, 소수 파티션 토픽의 정당한
//  디스크 역산 추천(예: 2파티션 × 250만)을 자의적으로 막았다. 파티션당 한도는
//  `effective_max_count` 의 `TOPIC_INDEX_COUNT_CAP / partition_count` 환산으로 일원화.)

/// 토픽당 인덱스 **총 건수** 상한 — 조회 캡(`ORDERED_LOCS_CAP`, engine-impl search)과 같은 값.
/// 이 건수를 넘겨 인덱싱해도 검색/탐색은 상한까지만 반환되므로(잘림), 초과 보관은 낭비다.
///
/// ⚠ 티어 도입 시 이 값의 **상향은 상수/profile 교체만으로 불가** — `ORDERED_LOCS_CAP` 은
/// loc 리스트 IPC 페이로드(건수 선형 ≈ 80MB/1M)를 막는 물리적 안전장치라, >5M 티어는
/// loc 스트리밍/페이징 재설계가 선행돼야 한다. (하향/동결은 profile 로 자유 — tighten-only.)
///
/// 현재 강제 범위(2026-07-13, 오픈 전 보수 결정 — 인덱싱 핫패스 무접촉):
/// - CountBased 토픽: `effective_max_count()` 가 파티션당 `min(…, 이 값 / partition_count)` 클램프
///   → 총량이 이 값을 넘지 않음 (기존 회귀검증된 trim 기계가 집행).
/// - DropIndex/FieldBased 토픽: **강제 없음** — FE 상태 패널 경고(조회 상한 도달 + count_based
///   전환 권장)로 가시화만. rolling trim(클램프+trim 상시화) + size-shrink 에스컬레이션은
///   오픈 후 named task (`docs/index_lifecycle_management_policy.md` 참조).
pub const TOPIC_INDEX_COUNT_CAP: u64 = 5_000_000;

/// 어절(tokenize) 대상 필드 수 천장 (토픽당). FE 대칭 = `src/tokenizeLimits.ts`.
pub const TOKENIZE_FIELD_CAP: usize = crate::settings::TOKENIZE_FIELD_CAP_B.default;

/// 플랜 한도 위반 에러 메시지의 sentinel prefix.
/// engine_impl shim 이 이 prefix 를 보고 `EngineError::NotEntitled` 로 매핑하고,
/// FE 는 토큰 매칭으로 "플랜 한도" 안내 UI 로 분기한다 (BucketOverflow 와 같은 패턴).
pub const PLAN_LIMIT_MSG: &str = "plan limit:";

/// "토픽이 브로커 목록에 없음" 에러의 sentinel prefix — **로케일 불변 계약**.
/// FE 폴링이 이 토큰으로 "진짜 삭제 신호"(list 성공 + 부재)를 판별해 고아 인덱스
/// 정리 모달을 띄운다. 통신 실패류 메시지와 반드시 구분되어야 한다 (i18n_design.md §5-2).
pub const TOPIC_NOT_FOUND_MSG: &str = "topic not found:";

// ── Kafka 연결 대표 에러 sentinel (2026-07-16) — **로케일 불변 계약** ─────────
// kafka-client 가 사용자 대면 연결 에러 메시지 맨 앞에 붙이고, FE 연결 실패 모달
// (ConnectScreen)이 이 토큰으로 로케일 안내문(4개 언어)을 원문 위에 표시한다.
// BE 원문 = 영어 기술 상세로 병기 노출. FE 의 자연어 문장 매칭은 금지 — 이 상수만 계약.

/// 연결 시간 초과 (broker 무응답 — 네트워크/방화벽/자격증명 silent close 후보).
pub const KAFKA_CONNECT_TIMEOUT_MSG: &str = "kafka connect timeout:";
/// SASL 인증 실패 (사용자/비밀번호/메커니즘).
pub const KAFKA_AUTH_FAILED_MSG: &str = "kafka auth failed:";
/// broker 가 선택한 SASL 메커니즘을 지원하지 않음.
pub const KAFKA_SASL_MECHANISM_MSG: &str = "kafka sasl mechanism rejected:";
/// TLS 연결/인증서 실패.
pub const KAFKA_TLS_FAILED_MSG: &str = "kafka tls failed:";
/// 그 외 연결 실패 (원인 미분류 폴백).
pub const KAFKA_CONNECT_FAILED_MSG: &str = "kafka connect failed:";
/// AWS MSK IAM 인가 거부 (원인 4종 비구분 — msk_iam::explain_access_denied).
pub const MSK_ACCESS_DENIED_MSG: &str = "msk access denied:";
