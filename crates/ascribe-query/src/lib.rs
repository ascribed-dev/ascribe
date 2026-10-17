//! Answers to the questions an author or an agent asks about a project,
//! each from what the checks, the source index, and the outputs already
//! compute:
//!
//! - [`agents_index`]: a compact index of a build's docs, for a product's
//!   `AGENTS.md`;
//! - [`explain`]: what a diagnostic means, how to fix it, and an example;
//! - [`model`]: what the content model allows, in full or as a short summary;
//! - [`outline`]: a page's title, type, and headings with their ids;
//! - [`link`]: whether a link target exists as seen from a page, and the
//!   link to write;
//! - [`render`]: a page as a reader of one build sees it, as plain Markdown;
//! - [`refs`]: where a page, a heading, or a content model entry is used;
//! - [`rules`]: a project's rules, for the instruction files an agent reads;
//! - [`permalinks`]: the permalinks the content declares, against the
//!   project's list, and names checked against a list;
//! - [`report`]: what state a project is in, section by section;
//! - [`versions`]: a project's release lines, and what differs between two.
//!
//! The `ascribe` commands of the same names call these and print what they
//! return; each answer is the command's JSON. Every answer carries
//! `schema_version`, which changes only when a field is removed or changes
//! meaning.

pub mod agents_index;
mod error;
pub mod explain;
mod lines;
pub mod link;
pub mod model;
pub mod outline;
pub mod permalinks;
pub mod refs;
pub mod render;
pub mod report;
pub mod rules;
pub mod versions;

pub use agents_index::{AgentsIndex, IndexPage, agents_index};
pub use error::QueryError;
pub use explain::{
    DiagnosticList, ExampleText, Explanation, ListedDiagnostic, code_of, explain, list,
};
pub use link::{LinkAnswer, LinkKind, LinkLine, LinkSuggestion, link};
pub use model::{ModelNavPage, ModelNavSection, ModelReport, Section, model, navigation, summary};
pub use outline::{Outline, OutlineHeading, OutlineMemo, OutlineMemoState, outline};
pub use permalinks::{PermalinksAnswer, Retire, RetireAnswer, SyncAnswer, VerifyAnswer};
pub use refs::{Asked, Place, Refs, TargetKind, refs};
pub use render::{Rendered, render};
pub use report::{Report, report};
pub use rules::{pointer, rules};
pub use versions::{VersionChanges, VersionsList};

/// The version of every answer's JSON schema.
pub const SCHEMA_VERSION: u32 = 1;

/// The version of Ascribe, which every answer carries.
pub(crate) const ASCRIBE_VERSION: &str = env!("CARGO_PKG_VERSION");
