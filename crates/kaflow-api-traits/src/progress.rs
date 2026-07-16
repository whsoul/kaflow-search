//! Progress emit 추상화 — Tauri / WebSocket / no-op 등 transport 별 구현체 분리.
//!
//! 도메인 함수가 progress 이벤트 발생 시 본 trait 의 `&dyn ProgressEmit` 을 받아
//! 호출하면, 어댑터 (`src-tauri/commands/...` 등 transport layer) 가 구체 구현체
//! (TauriProgressEmitter / WebSocketEmitter 등) 를 만들어 넘긴다.
//!
//! 단순한 동기 trait — `serde_json::Value` 페이로드 + 정적 채널명. payload 직렬화는
//! 호출 측에서 미리 수행.
//!
//! 비고: 사용 패턴이 fire-and-forget (실패 무시 / 결과 대기 없음) 이라 sync 로 충분.
//! async 가 필요한 transport 가 등장하면 그때 별 trait 추가.
//!
//! 본 trait 는 `library_split_design.md` §3 의 `ProgressReporter` (async) skeleton 을
//! 대체한다. `trait_redesign_2026_05.md` §2.1 결정.

use std::sync::Arc;

/// 채널 식별자. `&'static str` (주로 `kaflow_api_types::events::*` 상수)을 받는다.
/// 하드코딩 문자열을 직접 넘기는 것은 금지 — `events.rs` 상수만 허용.
pub type ProgressChannel = &'static str;

pub trait ProgressEmit: Send + Sync {
    /// `channel` 은 `kaflow_api_types::events::*` 상수.
    /// 실패는 무시 (UI 측 listener 가 없거나 transport 단절은 비즈니스 흐름과 무관).
    fn emit(&self, channel: ProgressChannel, payload: serde_json::Value);
}

/// 테스트 / mock 용 no-op 구현체.
pub struct NoopProgressEmit;

impl ProgressEmit for NoopProgressEmit {
    fn emit(&self, _channel: ProgressChannel, _payload: serde_json::Value) {}
}

/// `Arc<dyn ProgressEmit>` 형태의 alias — spawn 등 ownership 공유 시 유용.
pub type SharedEmit = Arc<dyn ProgressEmit>;
