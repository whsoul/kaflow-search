//! Kaflow public API types — IPC / 인덱스 도메인 DTO.
//!
//! 이 crate 는 Tauri / RocksDB / Kafka 등 인프라 의존이 없는 순수 데이터 타입만 둔다.
//! - 신규 DTO 는 여기에 추가한다.
//! - impl 메서드는 trait / from / into / 단순 helper 만 허용한다. 인프라 의존 로직은 금지.
//! - 추후 ts-rs / specta 도입 시 이 crate 가 진실의 원천 (single source of truth) 이 된다.

pub mod auth;
pub mod cluster;
pub mod config;
pub mod consistency;
pub mod decode_failure;
pub mod domain;
pub mod events;
pub mod export;
pub mod full_resync;
pub mod indexing;
pub mod limits;
pub mod message;
pub mod profile;
pub mod registry;
pub mod search;
pub mod settings;
pub mod topic_meta;
pub mod workspace;

// 평면 re-export — 호출처가 `kaflow_api_types::DocLoc` 처럼 짧게 쓸 수 있게 한다.
pub use auth::*;
pub use cluster::*;
pub use config::*;
pub use consistency::*;
pub use decode_failure::*;
pub use domain::*;
pub use export::*;
pub use full_resync::*;
pub use indexing::*;
pub use limits::*;
pub use message::*;
pub use profile::*;
pub use registry::*;
pub use search::*;
pub use topic_meta::*;
pub use workspace::*;
