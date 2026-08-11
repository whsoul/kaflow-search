// Kaflow Search — public demo binary.
//
// Wires the mock engine (local JSON fixtures) + the public Tauri shell
// (command shims over the KafkaToolEngine trait) + the prebuilt frontend
// (../../dist). No private crates, no real Kafka connection.
//
// This is the runnable proof that the public repo is self-sufficient: the
// same shell that drives the real product also drives this demo, differing
// only in which KafkaToolEngine implementation is managed.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use kaflow_api_traits::KafkaToolEngine;
use tauri::{Emitter, Manager};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // In-app find — native right-click menu routing (FindBar listens on `find_menu`).
        .on_menu_event(|app, event| match event.id().0.as_str() {
            "find_scope" => {
                let _ = app.emit("find_menu", "scope");
            }
            "find_all" => {
                let _ = app.emit("find_menu", "all");
            }
            "to_multi" => {
                let _ = app.emit("find_menu", "to_multi");
            }
            "to_multi_drill" => {
                let _ = app.emit("find_menu", "to_multi_drill");
            }
            _ => {}
        })
        .setup(|app| {
            let engine: Arc<dyn KafkaToolEngine> = Arc::new(kaflow_mock_engine::MockEngine::new());
            app.manage(engine);
            Ok(())
        })
        // Production command set — single source is kaflow-shell's all_handlers!.
        // The demo adds no extra (debug) commands.
        .invoke_handler(kaflow_shell::all_handlers![])
        .run(tauri::generate_context!())
        .expect("error while running kaflow-demo");
}
