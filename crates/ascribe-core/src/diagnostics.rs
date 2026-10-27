//! Slugs for every diagnostic in the registry.
//!
//! One constant per entry of the registry, `data/diagnostics.toml`, in the
//! same order, named after the slug. `build.rs` generates them from the
//! registry, so adding a diagnostic is one edit, to the registry. A crate
//! that finds a problem reports it with one of these
//! ([`Issue::new`](crate::Issue::new)); since [`DiagnosticSlug`] can't be
//! constructed outside this crate, a misspelled or unregistered slug is a
//! compile error.
//!
//! The registry holds each diagnostic's code, severity, level, SPEC section,
//! and message templates; the checks read them from [`REGISTRY`].

use crate::DiagnosticSlug;

/// The diagnostics registry's text, embedded: the file these constants are
/// generated from.
pub const REGISTRY: &str = include_str!("../data/diagnostics.toml");

include!(concat!(env!("OUT_DIR"), "/diagnostics.rs"));

/// The checks renamed, each by the name it had before, which still names
/// it in `[checks]`, in an acknowledgement, and to `ascribe explain`, for a
/// release (decision #953).
pub const OLD_NAMES: &[(&str, DiagnosticSlug)] = &[("review-overdue", RECHECK_OVERDUE)];
