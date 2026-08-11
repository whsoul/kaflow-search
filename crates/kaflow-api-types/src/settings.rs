//! The one place a setting's range lives: its minimum, maximum, and default.
//!
//! Keeping the range in a single place is the point. When the same number is written in a
//! default, in a clamp, and again in an input field, the three drift apart and the one
//! that is wrong is whichever nobody looked at.
//!
//! `min` and `max` are hard: a settings file holding a value outside them is clamped when
//! it is read, not merely rejected at input. Validation at the point of entry cannot
//! protect a file that was written by something else.

use crate::TOPIC_INDEX_COUNT_CAP;
use serde::{Deserialize, Serialize};

/// The fixed range of one setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds<T> {
    pub min: T,
    pub max: T,
    pub default: T,
}

impl Bounds<u64> {
    pub const fn clamp(&self, v: u64) -> u64 {
        if v < self.min {
            self.min
        } else if v > self.max {
            self.max
        } else {
            v
        }
    }
    /// Relaxes the lower bound only. The ceiling still applies — it is there for
    /// stability, not preference.
    pub const fn clamp_no_floor(&self, v: u64) -> u64 {
        let v = if v == 0 { 1 } else { v };
        if v > self.max {
            self.max
        } else {
            v
        }
    }

    /// Normalization for limits whose enforcement **deletes data**. A value below the
    /// minimum returns to `default`, not to `min`.
    ///
    /// The asymmetry is deliberate. For these limits, smaller means more gets deleted, so
    /// pinning an out-of-range value to the legal minimum would fail in the most
    /// destructive direction available. A value below the minimum was not chosen through
    /// the interface either — treating it as invalid and restoring the default is the
    /// honest reading. A legitimately chosen minimum is still respected.
    ///
    /// Above the maximum, ordinary clamping is correct: tightening downward only reaches
    /// a stability ceiling, while tightening upward from below would erase data.
    pub const fn sanitize_destructive(&self, v: u64) -> u64 {
        if v < self.min {
            self.default
        } else if v > self.max {
            self.max
        } else {
            v
        }
    }
}

impl Bounds<usize> {
    pub const fn clamp(&self, v: usize) -> usize {
        if v < self.min {
            self.min
        } else if v > self.max {
            self.max
        } else {
            v
        }
    }
    /// Relaxes the lower bound only; the ceiling still applies.
    pub const fn clamp_no_floor(&self, v: usize) -> usize {
        let v = if v == 0 { 1 } else { v };
        if v > self.max {
            self.max
        } else {
            v
        }
    }
}

const GB: u64 = 1_073_741_824;

// ── Settings a user chooses ─────────────────────────────────────────────────

/// How much disk one cluster's index may occupy.
///
/// The minimum exists to prevent thrashing: below it, cleanup removes what was just
/// indexed, and the topic appears to index successfully while searching finds nothing.
///
/// The maximum doubles as the value used when no operator profile is available. This
/// limit is enforced by deleting, so failing toward the looser end matters — a limit that
/// tightens because a server was unreachable would destroy a user's index.
pub const DISK_LIMIT_BYTES: Bounds<u64> = Bounds {
    min: GB,
    max: 100 * GB,
    default: 10 * GB,
};

/// How often the size limit is enforced, in seconds.
///
/// Whatever is indexed between two runs sits above the limit until the next one, so the
/// real peak is roughly `limit + indexing rate × interval`. That is why the interval has
/// a ceiling of its own: a long enough interval would make the size limit meaningless.
pub const SIZE_CLEANUP_INTERVAL_SECS: Bounds<u64> = Bounds {
    min: 60,
    max: 3_600,
    default: 600,
};

/// Default number of messages kept per partition when a topic does not set its own.
/// The per-topic total is what actually bounds this.
pub const KEEP_COUNT_PER_PARTITION: Bounds<u64> = Bounds {
    min: 1,
    max: TOPIC_INDEX_COUNT_CAP,
    default: 100_000,
};

// ── Tuning values, not user settings (never persisted) ──────────────────────

pub const INDEXING_BATCH_SIZE: Bounds<usize> = Bounds {
    min: 100,
    max: 100_000,
    default: 1_000,
};

pub const AUTO_SYNC_SLICE_SIZE: Bounds<usize> = Bounds {
    min: 1_000,
    max: 1_000_000,
    default: 50_000,
};

/// How often the safety-net sweep for retention cleanup runs, in seconds.
///
/// Cleanup normally happens as soon as a topic is seen to have dropped messages; this
/// sweep only picks up what that missed, which is why it is not a user-facing setting.
pub const RETENTION_CLEANUP_INTERVAL_SECS: Bounds<u64> = Bounds {
    min: 60,
    max: 86_400,
    default: 3_600,
};

pub const RETENTION_CLEANUP_WAIT_SECS: Bounds<u64> = Bounds {
    min: 1,
    max: 600,
    default: 60,
};

pub const SIZE_CLEANUP_WAIT_SECS: Bounds<u64> = Bounds {
    min: 1,
    max: 600,
    default: 30,
};

pub const FOREGROUND_ACQUIRE_WAIT_SECS: Bounds<u64> = Bounds {
    min: 1,
    max: 600,
    default: 30,
};

/// Compression for the local index. Changing it applies to newly written data only and
/// takes effect on reconnect, which makes it unsuitable as a user-facing setting.
pub const COMPRESSION_MODES: [&str; 3] = ["none", "snappy", "zstd"];
pub const COMPRESSION_MODE_DEFAULT: &str = "snappy";

// ── Limits an operator profile may set ──────────────────────────────────────
//
// The constants in `limits.rs` point at these defaults rather than repeating the numbers.
// `min` and `max` are hard: a profile cannot move a value outside them, because a profile
// arrives over the network and is not trusted on its own.

pub const CLUSTER_PROFILE_CAP_B: Bounds<usize> = Bounds {
    min: 1,
    max: 10,
    default: 2,
};
pub const INDEXED_TOPIC_CAP_B: Bounds<usize> = Bounds {
    min: 1,
    max: 50,
    default: 5,
};
pub const TOKENIZE_FIELD_CAP_B: Bounds<usize> = Bounds {
    min: 1,
    max: 8,
    default: 1,
};
pub const SEARCH_MAX_GROUPS_B: Bounds<usize> = Bounds {
    min: 1,
    max: 10,
    default: 2,
};
pub const SEARCH_MAX_LEAVES_PER_GROUP_B: Bounds<usize> = Bounds {
    min: 1,
    max: 10,
    default: 3,
};
/// A backstop: a query carrying more leaves than this is refused, whatever composed it.
pub const QUERY_MAX_LEAVES_B: Bounds<usize> = Bounds {
    min: 1,
    max: 64,
    default: 10,
};
/// Messages kept in one topic's index. **Cannot be raised** — the maximum is the same
/// number the engine will return in one response, so a higher value would only store what
/// could never be read back. A profile may tighten it.
pub const TOPIC_INDEX_COUNT_CAP_B: Bounds<u64> = Bounds {
    min: 1,
    max: TOPIC_INDEX_COUNT_CAP,
    default: TOPIC_INDEX_COUNT_CAP,
};

// ── What a profile may say about one limit ──────────────────────────────────

/// A constraint a profile places on one item. It only ever narrows.
///
/// - `Fix` — pins the value, for items a user has no say in.
/// - `Max` — caps it, leaving the user free below that.
/// - `Allow` — a permitted set, for items that are choices rather than numbers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProfileConstraint {
    Fix { value: u64 },
    Max { value: u64 },
    Allow { values: Vec<String> },
}

/// The `limits` section of a profile, keyed by item id.
///
/// It is deliberately open-ended: a new constraint can be introduced without shipping a
/// new build. Keys that are not recognised are ignored, so an older build stays safe when
/// it meets one.
pub type ProfileLimits = std::collections::HashMap<String, ProfileConstraint>;

/// The limits a profile resolved to.
///
/// **Profile values are clamped to the hard bounds as well.** A profile is remote data
/// and is not trusted on its own: a mistyped value stops at the ceiling instead of
/// becoming the new one.
///
/// ⚠ `rename_all = "camelCase"` is required here. This type nests inside
/// `EffectiveLimitsView`, and the parent's rename does not reach a child — without it the
/// nested keys go out in a different shape from every other key beside them, and a
/// consumer reads them as missing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanLimits {
    pub cluster_profile_cap: usize,
    pub indexed_topic_cap: usize,
    pub topic_index_count_cap: u64,
    pub tokenize_field_cap: usize,
    pub search_max_groups: usize,
    pub search_max_leaves_per_group: usize,
    pub query_max_leaves: usize,
}

impl Default for PlanLimits {
    /// Used when no profile is available. These limits only refuse to store or register
    /// something — they never delete — so falling back to the defaults is safe here.
    /// Limits that enforce by deleting do the opposite; see `effective_disk_limit`.
    fn default() -> Self {
        Self {
            cluster_profile_cap: CLUSTER_PROFILE_CAP_B.default,
            indexed_topic_cap: INDEXED_TOPIC_CAP_B.default,
            topic_index_count_cap: TOPIC_INDEX_COUNT_CAP_B.default,
            tokenize_field_cap: TOKENIZE_FIELD_CAP_B.default,
            search_max_groups: SEARCH_MAX_GROUPS_B.default,
            search_max_leaves_per_group: SEARCH_MAX_LEAVES_PER_GROUP_B.default,
            query_max_leaves: QUERY_MAX_LEAVES_B.default,
        }
    }
}

/// Item ids used as profile keys — a string contract shared with whatever supplies the
/// profile.
pub mod keys {
    pub const CLUSTER_COUNT: &str = "clusterCount";
    pub const INDEXED_TOPIC_COUNT: &str = "indexedTopicCount";
    pub const TOPIC_INDEX_COUNT: &str = "topicIndexCount";
    pub const TOKENIZE_FIELD_COUNT: &str = "tokenizeFieldCount";
    pub const SEARCH_GROUPS: &str = "searchGroups";
    pub const SEARCH_LEAVES_PER_GROUP: &str = "searchLeavesPerGroup";
    pub const QUERY_LEAVES: &str = "queryLeaves";
    /// A user setting: a profile caps it with `Max` rather than pinning it.
    pub const DISK_BYTES: &str = "maxIndexBytesPerCluster";
}

/// Reads a `Fix` value against its bounds; anything else falls back to the default.
///
/// A `Max` or `Allow` on an item the user cannot set is ignored — a cap means nothing
/// where there is no choice to cap, and a misconfigured profile should not move behaviour.
fn fix_usize(limits: &ProfileLimits, key: &str, bounds: Bounds<usize>) -> usize {
    match limits.get(key) {
        Some(ProfileConstraint::Fix { value }) => bounds.clamp(*value as usize),
        _ => bounds.default,
    }
}

fn fix_u64(limits: &ProfileLimits, key: &str, bounds: Bounds<u64>) -> u64 {
    match limits.get(key) {
        Some(ProfileConstraint::Fix { value }) => bounds.clamp(*value),
        _ => bounds.default,
    }
}

impl PlanLimits {
    /// Resolves a profile into effective limits. Unknown keys and constraint types that
    /// do not apply are ignored, and every result is clamped to its bounds.
    pub fn resolve(limits: &ProfileLimits) -> Self {
        Self {
            cluster_profile_cap: fix_usize(limits, keys::CLUSTER_COUNT, CLUSTER_PROFILE_CAP_B),
            indexed_topic_cap: fix_usize(limits, keys::INDEXED_TOPIC_COUNT, INDEXED_TOPIC_CAP_B),
            topic_index_count_cap: fix_u64(
                limits,
                keys::TOPIC_INDEX_COUNT,
                TOPIC_INDEX_COUNT_CAP_B,
            ),
            tokenize_field_cap: fix_usize(limits, keys::TOKENIZE_FIELD_COUNT, TOKENIZE_FIELD_CAP_B),
            search_max_groups: fix_usize(limits, keys::SEARCH_GROUPS, SEARCH_MAX_GROUPS_B),
            search_max_leaves_per_group: fix_usize(
                limits,
                keys::SEARCH_LEAVES_PER_GROUP,
                SEARCH_MAX_LEAVES_PER_GROUP_B,
            ),
            query_max_leaves: fix_usize(limits, keys::QUERY_LEAVES, QUERY_MAX_LEAVES_B),
        }
    }
}

/// The limits actually in force, resolved once.
///
/// Callers read this rather than the constants directly; otherwise a profile could change
/// while the interface went on showing the value it was compiled with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveLimitsView {
    /// Limits the profile decided; read-only, and useful for refusing early.
    pub plan: PlanLimits,
    /// The disk limit in force for one cluster.
    pub disk_limit_bytes: u64,
    /// The range an input should offer for the above.
    pub disk_min_bytes: u64,
    pub disk_max_bytes: u64,
    /// Whether a profile pinned the disk limit; if so, it is not the user's to change.
    pub disk_locked: bool,
    /// Whether this is a debug build.
    pub debug_build: bool,
}

/// A system limit, reported for diagnosis only.
///
/// These are shown but never made editable. They are compiled in, so an input field would
/// promise a change that nothing acts on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemLimitEntry {
    /// The constant's name, so a report can be traced back to it.
    pub name: String,
    pub value: u64,
    /// One line on why it is what it is.
    pub note: String,
}

/// The list of system limits to report. The engine fills this in; only its shape is
/// defined here, since the constants live where they are used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SystemLimitsView {
    pub entries: Vec<SystemLimitEntry>,
}

/// The disk limit actually in force: the user's value, held inside the hard range and
/// then inside whatever the profile allows.
///
/// With no profile the ceiling stays at its maximum rather than dropping — this limit is
/// enforced by deleting, so a server that cannot be reached must not shrink anyone's
/// index. A value below the floor returns to the default rather than to the floor, for
/// the reason given on `sanitize_destructive`.
pub fn effective_disk_limit(user_value: u64, limits: &ProfileLimits, debug_build: bool) -> u64 {
    let profile_max = match limits.get(keys::DISK_BYTES) {
        Some(ProfileConstraint::Max { value }) => (*value).min(DISK_LIMIT_BYTES.max),
        // Pinned by the profile: the user's own value does not apply.
        Some(ProfileConstraint::Fix { value }) => {
            let fixed = DISK_LIMIT_BYTES.clamp(*value);
            return fixed;
        }
        _ => DISK_LIMIT_BYTES.max,
    };

    let v = if debug_build {
        DISK_LIMIT_BYTES.clamp_no_floor(user_value)
    } else {
        DISK_LIMIT_BYTES.sanitize_destructive(user_value)
    };
    v.min(profile_max)
}

/// The highest disk limit an input should offer.
pub fn disk_limit_max(limits: &ProfileLimits) -> u64 {
    match limits.get(keys::DISK_BYTES) {
        Some(ProfileConstraint::Max { value }) => (*value).min(DISK_LIMIT_BYTES.max),
        Some(ProfileConstraint::Fix { value }) => DISK_LIMIT_BYTES.clamp(*value),
        _ => DISK_LIMIT_BYTES.max,
    }
}

/// Whether a profile pinned the disk limit.
pub fn disk_limit_locked(limits: &ProfileLimits) -> bool {
    matches!(
        limits.get(keys::DISK_BYTES),
        Some(ProfileConstraint::Fix { .. })
    )
}

impl EffectiveLimitsView {
    /// Resolves everything at once. `user_disk` is the value the user has chosen.
    pub fn resolve(user_disk: u64, limits: &ProfileLimits, debug_build: bool) -> Self {
        Self {
            plan: PlanLimits::resolve(limits),
            disk_limit_bytes: effective_disk_limit(user_disk, limits, debug_build),
            disk_min_bytes: if debug_build { 1 } else { DISK_LIMIT_BYTES.min },
            disk_max_bytes: disk_limit_max(limits),
            disk_locked: disk_limit_locked(limits),
            debug_build,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_inside_bounds() {
        assert!(DISK_LIMIT_BYTES.default >= DISK_LIMIT_BYTES.min);
        assert!(DISK_LIMIT_BYTES.default <= DISK_LIMIT_BYTES.max);
        assert!(SIZE_CLEANUP_INTERVAL_SECS.default >= SIZE_CLEANUP_INTERVAL_SECS.min);
        assert!(SIZE_CLEANUP_INTERVAL_SECS.default <= SIZE_CLEANUP_INTERVAL_SECS.max);
        assert!(KEEP_COUNT_PER_PARTITION.default <= KEEP_COUNT_PER_PARTITION.max);
        assert!(INDEXING_BATCH_SIZE.default >= INDEXING_BATCH_SIZE.min);
        assert!(AUTO_SYNC_SLICE_SIZE.default <= AUTO_SYNC_SLICE_SIZE.max);
    }

    #[test]
    fn clamp_pins_to_range() {
        assert_eq!(DISK_LIMIT_BYTES.clamp(0), GB); // floor
        assert_eq!(DISK_LIMIT_BYTES.clamp(200 * GB), 100 * GB); // ceiling
        assert_eq!(DISK_LIMIT_BYTES.clamp(5 * GB), 5 * GB);
        assert_eq!(SIZE_CLEANUP_INTERVAL_SECS.clamp(1), 60);
        assert_eq!(SIZE_CLEANUP_INTERVAL_SECS.clamp(86_400), 3_600);
    }

    /// The floor may be relaxed; the ceiling may not.
    #[test]
    fn debug_bypasses_floor_but_not_ceiling() {
        assert_eq!(
            DISK_LIMIT_BYTES.clamp_no_floor(100 * 1_048_576),
            104_857_600
        ); // allowed
        assert_eq!(DISK_LIMIT_BYTES.clamp_no_floor(0), 1); // zero is still guarded
        assert_eq!(DISK_LIMIT_BYTES.clamp_no_floor(500 * GB), 100 * GB); // ceiling holds
    }

    /// The per-partition ceiling must agree with the per-topic total — one axis, one
    /// number.
    #[test]
    fn keep_count_ceiling_tracks_topic_cap() {
        assert_eq!(KEEP_COUNT_PER_PARTITION.max, TOPIC_INDEX_COUNT_CAP);
    }

    // ── Resolution ──────────────────────────────────────────────────────────

    fn limits_of(pairs: &[(&str, ProfileConstraint)]) -> ProfileLimits {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    /// No profile means the shipped defaults, which is safe for limits that only refuse.
    #[test]
    fn no_profile_yields_launch_defaults() {
        let plan = PlanLimits::resolve(&ProfileLimits::new());
        assert_eq!(plan, PlanLimits::default());
        assert_eq!(plan.cluster_profile_cap, 2);
        assert_eq!(plan.indexed_topic_cap, 5);
        assert_eq!(plan.topic_index_count_cap, TOPIC_INDEX_COUNT_CAP);
    }

    /// A profile may raise a limit, but only inside the hard bounds.
    #[test]
    fn profile_fix_raises_within_bounds() {
        let plan = PlanLimits::resolve(&limits_of(&[
            (keys::CLUSTER_COUNT, ProfileConstraint::Fix { value: 5 }),
            (
                keys::INDEXED_TOPIC_COUNT,
                ProfileConstraint::Fix { value: 20 },
            ),
        ]));
        assert_eq!(plan.cluster_profile_cap, 5);
        assert_eq!(plan.indexed_topic_cap, 20);
    }

    /// Remote data is not trusted: a value beyond the ceiling stops at the ceiling.
    #[test]
    fn profile_cannot_exceed_hard_ceiling() {
        let plan = PlanLimits::resolve(&limits_of(&[
            (keys::CLUSTER_COUNT, ProfileConstraint::Fix { value: 9_999 }),
            (
                keys::TOPIC_INDEX_COUNT,
                ProfileConstraint::Fix { value: 50_000_000 },
            ),
        ]));
        assert_eq!(plan.cluster_profile_cap, CLUSTER_PROFILE_CAP_B.max);
        assert_eq!(plan.topic_index_count_cap, TOPIC_INDEX_COUNT_CAP); // cannot be raised
    }

    /// A constraint that does not apply to an item is ignored.
    #[test]
    fn wrong_constraint_type_is_ignored() {
        let plan = PlanLimits::resolve(&limits_of(&[
            (keys::CLUSTER_COUNT, ProfileConstraint::Max { value: 7 }),
            (
                keys::INDEXED_TOPIC_COUNT,
                ProfileConstraint::Allow {
                    values: vec!["x".into()],
                },
            ),
        ]));
        assert_eq!(plan.cluster_profile_cap, 2); // default kept
        assert_eq!(plan.indexed_topic_cap, 5);
    }

    /// Unknown keys are ignored, so an older build meeting a newer profile stays safe.
    #[test]
    fn unknown_keys_are_ignored() {
        let plan = PlanLimits::resolve(&limits_of(&[(
            "someLimitAddedLater",
            ProfileConstraint::Fix { value: 3 },
        )]));
        assert_eq!(plan, PlanLimits::default());
    }

    /// With no profile the disk ceiling stays at its maximum — tightening it because a
    /// server was unreachable would delete someone's index.
    #[test]
    fn disk_limit_fails_open_without_profile() {
        let none = ProfileLimits::new();
        assert_eq!(effective_disk_limit(50 * GB, &none, false), 50 * GB); // user's value kept
        assert_eq!(effective_disk_limit(200 * GB, &none, false), 100 * GB); // ceiling clamps
    }

    /// A value below the floor returns to the default, not to the floor. Pinning it to
    /// the floor would fail in the direction that deletes the most, and a value that low
    /// was never chosen through the interface in the first place.
    #[test]
    fn sub_floor_persisted_value_falls_back_to_default_not_floor() {
        let none = ProfileLimits::new();
        assert_eq!(effective_disk_limit(100 * 1_048_576, &none, false), 10 * GB);
        assert_eq!(effective_disk_limit(0, &none, false), 10 * GB);
        // A legitimately chosen minimum is respected as it stands.
        assert_eq!(effective_disk_limit(GB, &none, false), GB);
    }

    /// A profile that lowers the ceiling pulls the user's value down with it.
    #[test]
    fn disk_limit_tightened_by_profile_max() {
        let free = limits_of(&[(keys::DISK_BYTES, ProfileConstraint::Max { value: 10 * GB })]);
        assert_eq!(effective_disk_limit(50 * GB, &free, false), 10 * GB);
        assert_eq!(effective_disk_limit(3 * GB, &free, false), 3 * GB); // free below that
    }

    /// A pinned value overrides whatever the user chose.
    #[test]
    fn disk_limit_locked_by_profile_fix() {
        let locked = limits_of(&[(keys::DISK_BYTES, ProfileConstraint::Fix { value: 5 * GB })]);
        assert_eq!(effective_disk_limit(50 * GB, &locked, false), 5 * GB);
        assert_eq!(effective_disk_limit(1 * GB, &locked, false), 5 * GB);
    }

    /// Pins the JSON key names. This type is read as-is by callers, and a nested struct
    /// that loses its rename goes out in a different shape from the keys beside it —
    /// which a consumer sees as missing values rather than as an error. Nothing on the
    /// consuming side can catch that, so it is fixed here.
    #[test]
    fn effective_limits_view_serializes_camel_case_for_fe() {
        let v = EffectiveLimitsView::resolve(10 * GB, &ProfileLimits::new(), false);
        let json = serde_json::to_value(&v).unwrap();

        // Top level.
        for key in [
            "plan",
            "diskLimitBytes",
            "diskMinBytes",
            "diskMaxBytes",
            "diskLocked",
            "debugBuild",
        ] {
            assert!(
                json.get(key).is_some(),
                "missing or renamed top-level key: {key}"
            );
        }
        // The parent's rename does not reach the nested struct.
        let plan = json.get("plan").unwrap();
        for key in [
            "clusterProfileCap",
            "indexedTopicCap",
            "topicIndexCountCap",
            "tokenizeFieldCap",
            "searchMaxGroups",
            "searchMaxLeavesPerGroup",
            "queryMaxLeaves",
        ] {
            assert!(
                plan.get(key).is_some(),
                "missing or renamed plan key: {key}"
            );
        }
        // A stray snake_case key reads as absent on the other side.
        assert!(plan.get("cluster_profile_cap").is_none());
    }

    /// Only the floor is relaxed; the ceiling and the profile's cap still hold.
    #[test]
    fn debug_bypasses_floor_only() {
        let none = ProfileLimits::new();
        assert_eq!(
            effective_disk_limit(100 * 1_048_576, &none, true),
            100 * 1_048_576
        );
        assert_eq!(effective_disk_limit(500 * GB, &none, true), 100 * GB);
    }
}
