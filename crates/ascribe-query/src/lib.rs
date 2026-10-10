//! Answers to the questions an author or an agent asks about a project,
//! each from what the checks, the source index, and the outputs already
//! compute:
//!
//! - [`explain`]: what a diagnostic means, how to fix it, and an example;
//! - [`model`]: what the content model allows, in full or as a short summary;
//! - [`outline`]: a page's title, type, and headings with their ids;
//! - [`link`]: whether a link target exists as seen from a page, and the
//!   link to write;
//! - [`render`]: a page as a reader of one build sees it, as plain Markdown;
//! - [`refs`]: where a page, a heading, or a content model entry is used;
//! - [`rules`]: a project's rules, for the instruction files an agent reads;
//! - [`report`]: what state a project is in, section by section.
//!
//! The `ascribe` commands of the same names call these and print what they
//! return; each answer is the command's JSON. Every answer carries
//! `schema_version`, which changes only when a field is removed or changes
//! meaning.

mod error;
pub mod explain;
mod lines;
pub mod link;
pub mod model;
pub mod outline;
pub mod refs;
pub mod render;
pub mod report;
pub mod rules;

pub use error::QueryError;
pub use explain::{DiagnosticList, ExampleText, Explanation, ListedDiagnostic, explain, list};
pub use link::{LinkAnswer, LinkKind, LinkSuggestion, link};
pub use model::{ModelReport, Section, model, summary};
pub use outline::{Outline, OutlineHeading, outline};
pub use refs::{Asked, Place, Refs, TargetKind, refs};
pub use render::{Rendered, render};
pub use report::{Report, report};
pub use rules::{pointer, rules};

/// The version of every answer's JSON schema.
pub const SCHEMA_VERSION: u32 = 1;

/// The version of Ascribe, which every answer carries.
pub(crate) const ASCRIBE_VERSION: &str = env!("CARGO_PKG_VERSION");
