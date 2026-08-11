//! Resource ceilings.
//!
//! These are structural limits rather than settings: a caller may ask for less, but
//! nothing can raise them. They are enforced by the engine — a check in a user interface
//! is a convenience, not a guarantee.

/// How many cluster connection profiles can be stored.
///
/// Saving past this is **rejected**; nothing is evicted to make room. Each cluster keeps
/// its own index, so the disk budget applies per profile rather than in total.
pub const CLUSTER_PROFILE_CAP: usize = crate::settings::CLUSTER_PROFILE_CAP_B.default;

/// How many topics one workspace can keep indexed and in sync at the same time.
pub const INDEXED_TOPIC_CAP: usize = crate::settings::INDEXED_TOPIC_CAP_B.default;

/// Upper bound on how many messages one topic keeps in the index.
///
/// Search and browse return at most this many, so indexing past it buys disk usage rather
/// than reach.
pub const TOPIC_INDEX_COUNT_CAP: u64 = 5_000_000;

/// Ceiling on how many fields of one topic can be word-tokenized.
pub const TOKENIZE_FIELD_CAP: usize = crate::settings::TOKENIZE_FIELD_CAP_B.default;

/// Prefix marking an error as "a plan limit was reached".
///
/// Locale-invariant contract: branch on this token, never on the wording after it.
pub const PLAN_LIMIT_MSG: &str = "plan limit:";

/// Prefix marking "the broker listed its topics and this one was not among them".
///
/// Locale-invariant contract. It must stay distinguishable from a failure to reach the
/// cluster at all — this token is how a caller tells a real deletion from a connection
/// problem, and acting on the wrong one deletes an index that should have been kept.
pub const TOPIC_NOT_FOUND_MSG: &str = "topic not found:";

// ── Kafka connection error sentinels — locale-invariant contract ─────────────
// Prefixed to a user-facing connection error. Callers branch on these tokens and may
// show their own wording; what follows the prefix is English technical detail meant to
// be shown alongside, not parsed. Matching on that text is not supported — only these
// constants are the contract.

/// The broker did not answer in time.
pub const KAFKA_CONNECT_TIMEOUT_MSG: &str = "kafka connect timeout:";
/// SASL authentication was refused.
pub const KAFKA_AUTH_FAILED_MSG: &str = "kafka auth failed:";
/// The broker does not support the SASL mechanism that was chosen.
pub const KAFKA_SASL_MECHANISM_MSG: &str = "kafka sasl mechanism rejected:";
/// The TLS handshake or certificate check failed.
pub const KAFKA_TLS_FAILED_MSG: &str = "kafka tls failed:";
/// Connection failed for a reason none of the above covers.
pub const KAFKA_CONNECT_FAILED_MSG: &str = "kafka connect failed:";
/// AWS MSK IAM refused the request. The cause is deliberately not narrowed — several
/// unrelated problems produce the same response, so guessing one would mislead.
pub const MSK_ACCESS_DENIED_MSG: &str = "msk access denied:";
