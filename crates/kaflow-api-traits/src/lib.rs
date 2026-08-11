//! The interface an engine implements, and the only thing a caller needs to know about
//! one.
//!
//! Definitions only: an implementation lives elsewhere, and more than one is expected.

pub mod deserializer;
pub mod engine;
pub mod error;
pub mod progress;
pub mod schema;

// Re-exported flat, so a caller can name a trait without knowing its module.
pub use deserializer::*;
pub use engine::*;
pub use error::*;
pub use progress::*;
pub use schema::*;
