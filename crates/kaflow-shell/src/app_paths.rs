//! 앱 런타임 데이터 디렉토리 — **desktop shell 측 단일 출처**.
//!
//! ## 왜 이 파일이 있나 (B-1 seam)
//! 예전에는 `env::var("HOME") + ".kafka-tool-test"` 조합이 **6곳에서 각자 독립적으로** 반복됐다
//! (shell 3곳 / engine 3곳, 공유 헬퍼 없음, 에러 메시지 제각각). repo 를 public/private 로 쪼개면
//! 그 문자열은 **컴파일러가 검사하지 않는 두 레포 사이의 계약**이 된다 — 한쪽이 레이아웃을 바꾸면
//! 다른 쪽은 아무것도 깨지지 않은 채 **조용히 틀린 값을 보고**한다.
//!
//! 그래서 진영별로 출처를 하나씩만 둔다:
//! - **shell(public)**: 이 파일.
//! - **engine(private)**: `kafka_index_store::db::{APP_DATA_DIR_NAME, app_data_dir, workspace_dir}`.
//!
//! ## ⚠️ 계약 (두 곳이 반드시 같은 값)
//! `APP_DATA_DIR_NAME` / `ROCKSDB_DIR_NAME` 은 engine 쪽 상수와 **같아야 한다.**
//! 바꿀 때는 반드시 양쪽을 함께 바꾼다. 상세 = `docs/library_split_design.md` §2.3 (B-1).
//!
//! shell 은 엔진 trait 을 거치지 않고 이 경로를 쓴다 — 로컬 디스크/파일 stat 은 desktop 관심사라
//! 공개 엔진 계약(`KafkaToolEngine`)에 넣지 않는다. offline 에서도 동작해야 한다.

use std::path::PathBuf;

/// 앱 데이터 디렉토리 이름. **engine 쪽 `kafka_index_store::db::APP_DATA_DIR_NAME` 과 동일해야 함.**
pub const APP_DATA_DIR_NAME: &str = ".kaflow";

/// 구 디렉토리 이름 (디버깅앱 시절). 1회 마이그레이션 대상.
pub const LEGACY_APP_DATA_DIR_NAME: &str = ".kafka-tool-test";

/// 워크스페이스 하위 RocksDB 디렉토리 이름. **engine 쪽 상수와 동일해야 함.**
pub const ROCKSDB_DIR_NAME: &str = "rocksdb";

/// 홈 디렉토리. Windows 에는 `HOME` 이 기본 설정되지 않으므로 `USERPROFILE` 로 폴백한다.
fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| "home directory not found (HOME / USERPROFILE unset)".to_string())
}

/// `~/.kaflow` — **생성하지 않는다** (존재 확인 / 조회용).
pub fn app_data_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(APP_DATA_DIR_NAME))
}

/// `~/.kafka-tool-test` (구 이름) — 마이그레이션 판단용.
fn legacy_app_data_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(LEGACY_APP_DATA_DIR_NAME))
}

/// `~/.kaflow/{workspace}` — 생성하지 않는다.
pub fn workspace_dir(workspace: &str) -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join(workspace))
}

/// `~/.kaflow/{workspace}/rocksdb` — 생성하지 않는다.
pub fn rocksdb_dir(workspace: &str) -> Result<PathBuf, String> {
    Ok(workspace_dir(workspace)?.join(ROCKSDB_DIR_NAME))
}

/// `~/.kaflow/reports` — 생성하지 않는다.
pub fn reports_dir() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("reports"))
}

/// 구 디렉토리(`~/.kafka-tool-test`) → 신 디렉토리(`~/.kaflow`) **1회 마이그레이션.**
///
/// 디렉토리 통째 rename 이라 하위 전부(인덱스 `{ws}/rocksdb`, `clusters.json`,
/// `schema_registries.json`, `reports/`)가 한 번에 옮겨진다.
///
/// **반드시 DB open / 엔진 생성 전에 호출한다** — RocksDB 가 열린 상태로 rename 하면 안 된다.
///
/// - 신 디렉토리가 이미 있으면 아무것도 안 한다 (마이그레이션 완료 상태).
/// - 구 디렉토리가 없으면 아무것도 안 한다 (신규 설치).
/// - rename 실패 시 `Err` — 호출자가 로깅한다. 구 데이터는 그대로 남으므로 파괴적이지 않다.
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

// ※ engine(private) 상수와의 **계약 테스트는 바이너리 쪽**(`src-tauri/src/lib.rs`)에 있다.
//    이 crate 는 public 이라 private crate 를 볼 수 없기 때문 (그게 이 crate 의 존재 이유다).
