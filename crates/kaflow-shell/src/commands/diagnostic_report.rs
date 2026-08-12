//! Writing a diagnostic report to a file.
//!
//! It goes to the user's downloads folder, not into the app's own directory: this is
//! something being handed to the user, and the app's directory is where its data lives.
//! The filename is prefixed so its origin is obvious in a shared folder.
//!
//! ⚠️ This writes what it is given. Whatever removes sensitive detail has to have done so
//! before the report reaches here.

use tauri::Manager;

#[tauri::command]
pub fn export_diagnostic_report(
    app: tauri::AppHandle,
    report_json: String,
) -> Result<String, String> {
    // Resolved per platform rather than assumed.
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
