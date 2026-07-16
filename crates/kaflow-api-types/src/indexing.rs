//! Indexing / cleanup 진행 이벤트 DTO.

use crate::decode_failure::DecodeFailureContext;
use crate::full_resync::FullResyncTrigger;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenKafkaTopicResponse {
    pub topic: String,
    /// kafka 클러스터 기준 실제 total (sum of latest - earliest per partition)
    pub kafka_total_count: i64,
    /// 이번 호출로 새로 인덱싱된 메시지 수
    pub newly_indexed: usize,
    pub key_fields: Vec<String>,
    pub payload_fields: Vec<String>,
    pub header_fields: Vec<String>,
    /// Retention Cleanup으로 삭제된 I-key 수
    pub cleaned_up_i: usize,
    /// Retention Cleanup으로 삭제된 M-key 수
    pub cleaned_up_m: usize,
    /// Full resync 필요 감지 결과. 비어있으면 정상 흐름.
    /// 비어있지 않으면 ILM 이 실행되지 않았으며, 프론트에서 사용자 승인 후
    /// drop + 재호출하거나 무시해야 한다.
    #[serde(default)]
    pub full_resync_triggers: Vec<FullResyncTrigger>,
    /// Deserialize 실패로 sync 가 partial commit 후 중단된 경우의 컨텍스트.
    /// Some 이면 frontend 가 모달로 표시 + 사용자 결정에 따른 별도 명령 호출.
    /// 정책: `docs/deserialize_failure_policy.md`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decode_failure_context: Option<DecodeFailureContext>,
    /// `max_messages` cap 에 걸려 slice 단위로 조기 종료됐고 아직 더 인덱싱할
    /// 데이터가 남았을 수 있음(round-robin 자동싱크가 다음 라운드에 이어감).
    /// cap 미지정(None) 이거나 자연 종료면 항상 false.
    #[serde(default)]
    pub has_more: bool,
}

/// kafka-cleanup-progress 이벤트 페이로드.
/// phase: "scanning" | "cleaning" | "done"
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupProgressEvent {
    pub topic: String,
    pub deleted: usize,
    pub phase: String,
}

// ── compact dedup 값 히스토리 (KC view) ────────────────────────────────────

/// compact 토픽 dedup-on-write 가 (partition, key) 별로 보관한 값 히스토리.
/// KC 레코드의 public view — `docs/compact_topic_sync_repair_policy.md` §이력 보관.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactKeyHistory {
    /// 이 key 의 현재 최신 offset.
    pub latest_offset: u64,
    /// superseded 누적 횟수 (cap 무관 — 리스트가 잘려도 총량 보존).
    pub superseded_total: u64,
    /// 최신 레코드가 tombstone(삭제 마커)인지 — true 면 이 key 는 "삭제됨" 상태.
    #[serde(default)]
    pub latest_is_tombstone: bool,
    /// 최근 superseded 버전 (최신순, 개수 cap + byte budget 적용된 보관분).
    pub entries: Vec<CompactSupersededVersion>,
}

/// superseded 된 옛 버전 하나.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactSupersededVersion {
    pub offset: u64,
    pub ts_millis: u64,
    /// 삭제 직전 값 (field, leaf value). byte budget 강등 시 None (offset/ts breadcrumb 만).
    pub field_values: Option<Vec<(String, String)>>,
    /// true = tombstone(삭제 마커) 자체의 breadcrumb — 값이 원래 없음 (budget 강등과 구분).
    #[serde(default)]
    pub tombstone: bool,
}

// ── compact 삭제 key 뷰 (KC 스캔) ──────────────────────────────────────────

/// 삭제됨(tombstone latest) key 한 건 — 간편검색 "대상 유형: 삭제됨" 리스트 row.
/// M/I/R 에 흔적이 없는 key 라 KC 가 유일한 출처다.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactDeletedKeyRow {
    pub partition: u32,
    /// key deserializer 가 만든 key 문자열 (KC 키의 key_raw).
    pub key_raw: String,
    /// tombstone(삭제 마커) 레코드의 offset (= KC latest_offset).
    pub tombstone_offset: u64,
    /// 삭제 시각 — tombstone breadcrumb 의 ts. 이력 cap 으로 breadcrumb 이
    /// 밀려났으면 None (offset 만 앎).
    pub deleted_ts_millis: Option<u64>,
    /// 이 key 의 superseded 누적 횟수 (값 대체 총량 — tombstone 은 미포함).
    pub superseded_total: u64,
    /// 삭제 직전 마지막 실제 값 (보관돼 있으면) — 리스트 요약 표시용.
    pub last_value: Option<CompactSupersededVersion>,
}

/// 삭제 key 스캔 재개 커서 — KC 저장 순서(partition asc, key asc)의 마지막 위치.
/// 다음 페이지는 이 위치 **다음**부터.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactDeletedKeyCursor {
    pub partition: u32,
    pub key_raw: String,
}

/// 삭제 key 스캔 한 페이지. `next_cursor = None` 이면 끝까지 봤다는 뜻.
/// 페이지가 덜 찼는데 next_cursor 가 있으면 스캔 예산 소진(계속하려면 재호출).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactDeletedKeysPage {
    pub rows: Vec<CompactDeletedKeyRow>,
    pub next_cursor: Option<CompactDeletedKeyCursor>,
    /// 이번 호출이 살펴본 KC 엔트리 수 (예산/진행 진단용).
    pub scanned: u64,
}
