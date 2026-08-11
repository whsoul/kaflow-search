//! What this build is, so a caller can adapt to it rather than offering what is not
//! there.

use kaflow_api_traits::{EngineCapabilities, KafkaToolEngine};
use serde::Serialize;
use std::sync::Arc;

#[tauri::command]
pub fn is_debug_build_enabled() -> bool {
    cfg!(feature = "debug-api")
}

/// What the engine says about itself.
///
/// Which engine is present is decided when the binary is built, so this asks the engine
/// rather than inferring it from how this crate was compiled — the two can differ.
#[tauri::command]
pub fn get_engine_info(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<EngineCapabilities, String> {
    Ok(engine.capabilities())
}

/// A short label for the build, so an unoptimized one is not mistaken for a slow app.
#[tauri::command]
pub fn build_mode_label() -> String {
    let profile = if cfg!(debug_assertions) {
        "dev"
    } else {
        "release"
    };
    let api = if cfg!(feature = "debug-api") {
        " · debug-api"
    } else {
        ""
    };
    format!("{profile}{api}")
}

/// About the machine this is running on.
///
/// ⚠️ **The original machine id must never be returned** — only a hash of it. What leaves
/// here cannot then be turned back into an identifier for the device.
#[derive(Serialize)]
pub struct HostInfo {
    /// `"macos"`, `"windows"`, `"linux"`, or whatever else the platform reports.
    pub platform: String,
    /// "x86_64" | "aarch64" | …
    pub arch: String,
    /// The hashed machine id, or empty where it could not be read.
    #[serde(rename = "deviceIdHash")]
    pub device_id_hash: String,
}

#[tauri::command]
pub fn get_host_info() -> HostInfo {
    HostInfo {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        device_id_hash: device_id_hash(),
    }
}

/// Hashes the machine id. Empty on failure — nothing may depend on having it.
fn device_id_hash() -> String {
    match machine_uid::get() {
        Ok(id) if !id.is_empty() => {
            use sha2::{Digest, Sha256};
            let digest = Sha256::digest(id.as_bytes());
            format!("{digest:x}")
        }
        _ => String::new(),
    }
}
