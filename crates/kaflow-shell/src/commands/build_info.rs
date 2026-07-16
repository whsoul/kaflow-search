//! Build-time feature 정보 — FE 가 UI 게이팅에 사용.
//!
//! `debug-api` 가 꺼진 production 빌드에서는 BenchPanel 진입 버튼 등 디버그 UI 를 숨기기 위해
//! FE 가 앱 시작 시 본 command 를 한 번 호출해 분기한다.

use kaflow_api_traits::{EngineCapabilities, KafkaToolEngine};
use serde::Serialize;
use std::sync::Arc;

#[tauri::command]
pub fn is_debug_build_enabled() -> bool {
    cfg!(feature = "debug-api")
}

/// 주입된 엔진이 스스로 밝히는 신원/능력. FE 는 `engineId` 로 mock 데모 빌드를 식별해
/// "로컬 파일 데모" 배지를 띄운다 (mock = `"kaflow-mock-engine"`, 실엔진 = `"kaflow-engine-impl"`).
/// 어떤 엔진이 링크됐는지는 바이너리가 정하므로, shell 은 shell feature 가 아니라 **엔진에게 물어본다.**
#[tauri::command]
pub fn get_engine_info(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<EngineCapabilities, String> {
    Ok(engine.capabilities())
}

/// [임시] 빌드 모드 식별 라벨 — FE 타이틀에 표시해 dev/release · debug-api 여부를 한눈에.
/// `debug_assertions` = true 면 debug 프로파일(미최적화 Rust), false 면 release(최적화).
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

/// 호스트 정보 — appProfile 게이트 `clientInfo` 용 (FE 가 부팅 시 1회 조회).
/// `deviceIdHash` = OS 머신ID 의 SHA-256 (원본은 절대 노출하지 않는다). navigator 로는 못 얻는 값이라 shell 에서 제공.
#[derive(Serialize)]
pub struct HostInfo {
    /// "macos" | "windows" | "linux" | (기타 std OS 문자열)
    pub platform: String,
    /// "x86_64" | "aarch64" | …
    pub arch: String,
    /// 머신ID SHA-256 hex. 획득 실패 시 "" (FE 는 미제공으로 처리).
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

/// OS 머신ID 를 읽어 SHA-256. 실패(권한/미지원)면 "" — 게이트는 이 값을 하드 게이트로 쓰지 않으므로 빈값 허용.
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
