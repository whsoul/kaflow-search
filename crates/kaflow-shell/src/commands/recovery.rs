//! 워크스페이스 RocksDB 복구 상태 점검 (fs-only — DB open 없음).
//!
//! 강제 종료 시 memtable 이 SST 로 flush 되지 않으면 미반영 쓰기가 WAL(`*.log`) 에 남는다.
//! 재실행 시 RocksDB 가 이 WAL 을 replay(복구) 하느라 첫 DB open 이 오래 "멈춘 것처럼" 보인다.
//! 이 커맨드는 **DB 를 열지 않고** 파일만 stat 해서 미반영 WAL 크기 / SST 진척을 보고,
//! FE 가 boot 중 "복구 중" 안내 + 진행(SST 증가) 표시를 할 수 있게 한다.

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRecoveryInfo {
    /// 미flush WAL(`*.log`) 총 바이트. 크면 강제 종료 후 복구 필요 신호.
    pub wal_bytes: u64,
    /// 현재 SST 파일 수 (복구 flush 가 진행되면 증가 — 진행 신호).
    pub sst_count: u64,
    pub sst_bytes: u64,
}

/// `~/.kaflow/{workspace}/rocksdb` 의 WAL/SST 를 fs stat 으로 집계 (DB open 안 함).
/// 경로는 `crate::app_paths` 단일 출처 (B-1 seam — engine 의 디스크 레이아웃과의 계약).
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
        // 디렉토리 없음 = 신규 워크스페이스 → 복구할 것 없음.
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
