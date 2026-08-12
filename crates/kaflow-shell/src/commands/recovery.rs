//! Whether an index is recovering from an unclean shutdown.
//!
//! Writes that had not been written out survive in a log, and replaying that log is what
//! makes the first open after a crash take a long time — during which the app looks
//! frozen rather than busy.
//!
//! ⚠️ **This must not open the index.** It only looks at the files, precisely so it can
//! answer while the open it is describing is still in progress.

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRecoveryInfo {
    /// How much is waiting to be replayed. A large number means a crash to recover from.
    pub wal_bytes: u64,
    /// How many index files exist. It rises as recovery proceeds, which is the only sign
    /// of progress available.
    pub sst_count: u64,
    pub sst_bytes: u64,
}

/// Looks at the files of one workspace's index, without opening it.
#[tauri::command]
pub fn inspect_workspace_recovery(workspace: String) -> Result<WorkspaceRecoveryInfo, String> {
    if workspace.trim().is_empty() {
        return Err("workspace name is empty".into());
    }
    let dir = crate::app_paths::rocksdb_dir(&workspace)?;

    let mut info = WorkspaceRecoveryInfo {
        wal_bytes: 0,
        sst_count: 0,
        sst_bytes: 0,
    };
    let Ok(rd) = std::fs::read_dir(&dir) else {
        // No directory means nothing has been written yet, so nothing to recover.
        return Ok(info);
    };
    for ent in rd.flatten() {
        let path = ent.path();
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        let len = ent.metadata().map(|m| m.len()).unwrap_or(0);
        match ext {
            "log" => info.wal_bytes += len, // WAL
            "sst" => {
                info.sst_count += 1;
                info.sst_bytes += len;
            }
            _ => {}
        }
    }
    Ok(info)
}
