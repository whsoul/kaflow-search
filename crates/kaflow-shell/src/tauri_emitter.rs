//! Carries progress events out to the window. The engine reports progress without knowing
//! this exists — that is what keeps it usable elsewhere.

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
