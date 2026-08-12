//! The types that cross the boundary between a caller and an engine.
//!
//! Data only: nothing here may depend on a store, a transport, or a cluster. Methods are
//! limited to conversions and small helpers — anything that needs infrastructure belongs
//! to whoever implements the engine, not to the types it speaks in.

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

// Re-exported flat, so callers can name a type without knowing which module it sits in.
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
