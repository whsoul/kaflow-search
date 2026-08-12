//! The commands themselves. Each unwraps the engine and calls one method on it — there is
//! no logic here to get wrong.
//!
//! **Nothing here reaches inside an engine.** Anything that would have to is not a command
//! and belongs with the binary that has such access.
//!
//! The modules are public because the macro that gathers the commands names them by path.

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
