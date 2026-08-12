//! Where the app keeps its data — the one place this side works it out.
//!
//! The same path used to be assembled in six places, each with its own idea of what to do
//! when it failed. A path built independently in two places is a contract no compiler
//! checks: change the layout on one side and the other goes on **reporting a wrong answer
//! without anything breaking.**
//! - **engine(private)**: `kafka_index_store::db::{APP_DATA_DIR_NAME, app_data_dir, workspace_dir}`.
//!
//! ⚠️ **These names have to match what the engine uses.** They are a contract between two
//! places that cannot see each other, so changing one alone is silent.
//!
//! These paths are used without going through the engine on purpose: looking at local
//! files is a desktop concern, and it has to work when nothing is reachable.

use std::path::PathBuf;

/// The app's data directory. **Must match what the engine uses.**
pub const APP_DATA_DIR_NAME: &str = ".kaflow";

/// The name used before this one, moved from once.
pub const LEGACY_APP_DATA_DIR_NAME: &str = ".kafka-tool-test";

/// The index directory within a workspace. **Must match what the engine uses.**
pub const ROCKSDB_DIR_NAME: &str = "rocksdb";

/// The home directory. Windows does not set `HOME`, so `USERPROFILE` is tried too.
fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| "home directory not found (HOME / USERPROFILE unset)".to_string())
}

/// The app directory. **Not created** — this only says where it would be.
pub fn app_data_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(APP_DATA_DIR_NAME))
}

/// Where it used to be, for deciding whether to move it.
fn legacy_app_data_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(LEGACY_APP_DATA_DIR_NAME))
}

/// A workspace's directory. Not created.
pub fn workspace_dir(workspace: &str) -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join(workspace))
}

/// A workspace's index directory. Not created.
pub fn rocksdb_dir(workspace: &str) -> Result<PathBuf, String> {
    Ok(workspace_dir(workspace)?.join(ROCKSDB_DIR_NAME))
}

/// Where reports go. Not created.
pub fn reports_dir() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("reports"))
}

/// Moves the old directory to the current one, once.
///
/// The whole directory is renamed, so everything inside it travels together.
///
/// ⚠️ **Call this before anything opens the index.** Renaming a directory out from under
/// an open database is not something that fails cleanly.
///
/// Does nothing if the current directory already exists, or if the old one does not.
/// A failure leaves the old data untouched, so it is safe to report and carry on.
pub fn migrate_legacy_app_dir() -> Result<bool, String> {
    let new_dir = app_data_dir()?;
    let old_dir = legacy_app_data_dir()?;

    if new_dir.exists() || !old_dir.exists() {
        return Ok(false);
    }

    std::fs::rename(&old_dir, &new_dir).map_err(|e| {
        format!(
            "{} → {} migration failed: {e}",
            old_dir.display(),
            new_dir.display()
        )
    })?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_compose_under_app_data_dir() {
        let base = app_data_dir().expect("home");
        assert!(base.ends_with(APP_DATA_DIR_NAME));
        assert_eq!(workspace_dir("ws").unwrap(), base.join("ws"));
        assert_eq!(
            rocksdb_dir("ws").unwrap(),
            base.join("ws").join(ROCKSDB_DIR_NAME)
        );
        assert_eq!(reports_dir().unwrap(), base.join("reports"));
    }
}

// The test that these agree with the engine's own constants lives with the binary — it is
// the only place that can see both, which is exactly what this crate gives up by design.
