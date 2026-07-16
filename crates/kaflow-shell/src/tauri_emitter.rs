//! Tauri 어댑터 — `kaflow_api_traits::ProgressEmit` 를 `tauri::AppHandle.emit` 으로 구현.
//!
//! engine-impl 의 비즈니스 함수가 progress 이벤트를 emit 할 때, transport-agnostic 한
//! `Arc<dyn ProgressEmit>` 를 받는다. 본 어댑터가 그 trait object 의 Tauri 측 구현체.

use kaflow_api_traits::progress::ProgressEmit;
use tauri::Emitter;

pub struct TauriProgressEmitter {
    pub app: tauri::AppHandle,
}

impl ProgressEmit for TauriProgressEmitter {
    fn emit(&self, channel: &'static str, payload: serde_json::Value) {
        let _ = self.app.emit(channel, payload);
    }
}
