//! 프론트엔드 `listen(...)` 채널명 단일 진실의 소스.
//!
//! 규칙 (docs/library_split_design.md Phase 1):
//! - 새 이벤트 추가 시 여기에 `pub const` 로 선언 후 사용한다.
//! - `app.emit("...")` 처럼 문자열 리터럴 직접 사용 금지.
//! - FE 대응 상수: `src/events.ts`.

#![allow(dead_code)]

pub const KAFKA_INDEXING_PROGRESS: &str = "kafka-indexing-progress";
pub const KAFKA_CLEANUP_PROGRESS: &str = "kafka-cleanup-progress";
pub const TOPIC_DROP_PROGRESS: &str = "topic-drop-progress";
pub const TOPIC_INDEX_REFRESHED: &str = "topic-index-refreshed";
pub const ILM_ACTION_LOG: &str = "ilm-action-log";
pub const SEARCH_PREFETCH_PROGRESS: &str = "search-prefetch-progress";
pub const TIME_BUCKETS_PROGRESS: &str = "time-buckets-progress";
/// 다중검색 buckets/full-drain 의 실시간 진행 — `{ processed, matched, done }`.
/// driver-walk loop 가 ~100ms 마다 emit, 끝에 `done:true`. FE 는 카운트만 갱신, done 시 렌더.
pub const MULTI_SEARCH_PROGRESS: &str = "multi-search-progress";
/// 다중검색 drill (셀/막대 펼침) 완료 시 선택된 plan / 건수 / 소요시간 짧은 App LOG.
pub const SEARCH_DRILL_LOG: &str = "search-drill-log";
pub const FIELD_REINDEX_PROGRESS: &str = "field-reindex-progress";
pub const DB_RESYNC_PROGRESS: &str = "db-resync-progress";
pub const DB_CLEAR_PROGRESS: &str = "db-clear-progress";
/// 검색결과 export 진행 — `{ written, total, done }`. 배치마다 emit, 끝에 `done:true`.
pub const EXPORT_PROGRESS: &str = "export-progress";
