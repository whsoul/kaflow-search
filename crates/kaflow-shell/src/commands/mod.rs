//! Tauri command shim 모음 (**public shell**).
//!
//! 모든 비즈니스는 `KafkaToolEngine` trait impl 에 위임하고, 본 layer 는
//! `tauri::State<Arc<dyn KafkaToolEngine>>` 를 풀어 trait method 를 호출한다.
//!
//! **private crate import 0건** — 이 불변식이 이 crate 가 public 일 수 있는 이유다.
//! debug / bench 명령은 엔진 내부(`DbState`, `encode_index_key` 등)를 직접 잡으므로
//! 여기 있을 수 없다. 바이너리(private) 쪽 `debug_commands.rs` / `bench*.rs` 에 산다.
//!
//! 모듈이 `pub` 인 이유: `crate::all_handlers!` 매크로가 `$crate::commands::<모듈>::<명령>`
//! 경로로 참조하기 때문.

pub mod auth;
pub mod browse;
pub mod build_info;
pub mod config;
pub mod consistency;
pub mod diagnostic_report;
pub mod export;
pub mod field_mgmt;
pub mod find_menu;
pub mod ilm;
pub mod indexing;
pub mod profiles;
pub mod recovery;
pub mod registry;
pub mod search;
pub mod storage;
pub mod topic_meta;

pub use auth::*;
pub use browse::*;
pub use build_info::*;
pub use config::*;
pub use consistency::*;
pub use diagnostic_report::*;
pub use export::*;
pub use field_mgmt::*;
pub use find_menu::*;
pub use ilm::*;
pub use indexing::*;
pub use profiles::*;
pub use recovery::*;
pub use registry::*;
pub use search::*;
pub use storage::*;
pub use topic_meta::*;
