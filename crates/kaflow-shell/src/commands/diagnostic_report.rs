//! 진단 리포트 파일 내보내기 (에러리포팅 v1 — `docs/error_reporting_design.md`).
//!
//! FE 가 조립한 스냅샷 JSON(메시지 본문 미포함, FE 빌더가 redaction)을
//! `~/.kaflow/reports/diagnostic-<unix_ms>.json` 에 기록하고 절대경로를 반환한다.
//! 백엔드는 파일 쓰기만 담당 — 엔진(trait) 비의존이라 thin shell command.
//! 경로는 `crate::app_paths` 단일 출처 (B-1 seam).

use crate::app_paths::reports_dir;

#[tauri::command]
pub fn export_diagnostic_report(report_json: String) -> Result<String, String> {
    let dir = reports_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create reports directory: {e}"))?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = dir.join(format!("diagnostic-{ts}.json"));
    std::fs::write(&path, report_json).map_err(|e| format!("failed to write report: {e}"))?;
    Ok(path.to_string_lossy().into_owned())
}
