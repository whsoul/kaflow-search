//! 진단 리포트 파일 내보내기 (에러리포팅 v1 — `docs/error_reporting_design.md`).
//!
//! FE 가 조립한 스냅샷 JSON(메시지 본문 미포함, FE 빌더가 redaction)을
//! **사용자 다운로드 디렉토리**의 `kaflow-diagnostic-<unix_ms>.json` 에 기록하고 절대경로를
//! 반환한다. 사용자에게 건네주는 산출물이라 앱 내부 데이터 디렉토리(`~/.kaflow` — DB/설정과
//! 섞임)가 아니라 다운로드 폴더가 맞다(2026-07-18). 다운로드 경로를 못 얻는 환경(드묾)에서만
//! 구 `~/.kaflow/reports/` 로 폴백. 파일명에 `kaflow-` prefix — 공용 폴더라 출처 식별.
//! 백엔드는 파일 쓰기만 담당 — 엔진(trait) 비의존이라 thin shell command.

use tauri::Manager;

#[tauri::command]
pub fn export_diagnostic_report(
    app: tauri::AppHandle,
    report_json: String,
) -> Result<String, String> {
    // OS 공통: tauri path resolver 가 macOS/Windows/Linux(XDG) 다운로드 폴더를 해석한다.
    let dir = match app.path().download_dir() {
        Ok(d) => d,
        Err(_) => crate::app_paths::reports_dir()?,
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create report directory: {e}"))?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = dir.join(format!("kaflow-diagnostic-{ts}.json"));
    std::fs::write(&path, report_json).map_err(|e| format!("failed to write report: {e}"))?;
    Ok(path.to_string_lossy().into_owned())
}
