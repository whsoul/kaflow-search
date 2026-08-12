//! Reporting progress, without knowing who is listening.
//!
//! Deliberately synchronous and fire-and-forget: nothing waits on a progress event and
//! nothing depends on one arriving, so making this async would add ceremony to every
//! caller for no gain.

use std::sync::Arc;

/// Reports progress on a named channel.
///
/// ⚠️ Use the declared constants, not a literal. A name typed out at the point of use is
/// how an emitter and a listener come to differ by one character, and neither side fails.
pub type ProgressChannel = &'static str;

pub trait ProgressEmit: Send + Sync {
    /// Failure is ignored — nobody listening is an ordinary state, not a problem to
    /// report back into the work being done.
    fn emit(&self, channel: ProgressChannel, payload: serde_json::Value);
}

/// Reports nothing.
pub struct NoopProgressEmit;

impl ProgressEmit for NoopProgressEmit {
    fn emit(&self, _channel: ProgressChannel, _payload: serde_json::Value) {}
}

/// A shareable emitter, for work that outlives the call that started it.
pub type SharedEmit = Arc<dyn ProgressEmit>;
