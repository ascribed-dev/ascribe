//! Real-documentation corpora and performance checks for Ascribe.
//!
//! - [`corpus`]: the three corpora (Astro, Elastic, Docker) and fetching them
//!   at pinned commits. Nothing is committed.
//! - [`recognize`]: what the parser and file-level checks find in unconverted
//!   Markdown, classified.
//! - [`convert`]: converting a corpus's constructs to Ascribe, at volume.
//! - [`perf`]: timing the commands and comparing with recorded baselines.
//!
//! The synthetic 3,000-page project the benchmarks share is in the
//! `ascribe-synthetic` crate.

pub mod convert;
pub mod corpus;
pub mod perf;
pub mod recognize;
