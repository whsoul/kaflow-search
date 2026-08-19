//! Where the app keeps its data — the one place this side works it out.
//!
//! The same path used to be assembled in six places, each with its own idea of what to do
//! when it failed. A path built independently in two places is a contract no compiler
//! checks: change the layout on one side and the other goes on **reporting a wrong answer
//! without anything breaking.**
//!
//! ⚠️ **These names have to match what the engine uses.** They are a contract between two
//! places that cannot see each other, so changing one alone is silent.
//!
//! These paths are used without going through the engine on purpose: looking at local
//! files is a desktop concern, and it has to work when nothing is reachable.

use std::path::PathBuf;

/// The app's data directory. **Must match what the engine uses.**
pub const APP_DATA_DIR_NAME: &str = ".kaflow";

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

/// Rejects a workspace name that would resolve outside the application data directory.
///
/// A workspace name reaches this crate as user input, and a workspace directory can be
/// deleted outright. A name containing a path separator or `..` would therefore delete a
/// directory the user never named. Validating where the path is built is the only place
/// that holds for every caller.
fn ensure_safe_workspace(workspace: &str) -> Result<(), String> {
    if workspace.trim().is_empty() {
        return Err("workspace name is empty".to_string());
    }
    if workspace.contains('/') || workspace.contains('\\') {
        return Err(format!("workspace name must not contain a path separator: {workspace}"));
    }
    if workspace == ".." || workspace == "." {
        return Err(format!("workspace name must not be a relative path: {workspace}"));
    }
    if workspace.contains(':') {
        return Err(format!("workspace name must not contain ':': {workspace}"));
    }
    Ok(())
}

/// A workspace's directory. Not created.
///
/// The name must pass [`ensure_safe_workspace`].
pub fn workspace_dir(workspace: &str) -> Result<PathBuf, String> {
    ensure_safe_workspace(workspace)?;
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
