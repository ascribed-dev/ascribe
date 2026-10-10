//! File-level validation of Ascribe documents, producing diagnostics.
//!
//! [`check_files`] is the single file-level entry point: the command line
//! (`ascribe check`), the build, and the language server all call it, which
//! is what makes their results identical. It takes a [`Project`] (the content
//! model and the source files) and returns [`Diagnostic`]s.
//!
//! What it reports, and where each row of SPEC §8.2 comes from:
//!
//! - **The parser's issues**, from `ascribe_syntax::parse`: attribute blocks,
//!   primaries, unknown directives, containers, end lines, nesting, binding,
//!   titles, `@variant` group rules, `@steps`, `@details`, and the list
//!   warnings. They're turned into diagnostics here and never reported twice.
//! - **Checks that need the content model**: attribute keys and value types
//!   (directives, widgets, images), `@variant` dimensions and values,
//!   `@available` specs, frontmatter (content type, fields, reserved keys,
//!   `available` and `variant`), undeclared phrases, headings that contain
//!   a phrase and have no `@id`, and `@id` values.
//! - **Checks that need the file system**: `@include` targets, link
//!   destinations (files, fragments, routes), and image sources, all with the
//!   boundary and exact-case rules for local files.
//! - **Sources in other repositories**: `ascribe.lock`
//!   against the content model, and each source's copies against the lock.
//!
//! Messages, codes, and severities come from the diagnostics registry
//! (`tests/conformance/diagnostics.toml`), through [`Registry`].
//!
//! **Page-level checks** are in [`page`]: [`check_project`] is the one function
//! every tool calls for a build (the file-level diagnostics, then the build's
//! page-level ones), and [`check_all_builds`] runs every build and reports
//! each distinct problem once, naming the builds it appears in
//! ([`Diagnostic::builds`]).

mod builds;
mod checks;
mod diagnostic;
pub mod intended;
mod levels;
pub mod page;
mod project;
pub mod prompt;
pub mod prose;
pub mod registry;
mod scope;
mod yaml;

pub use builds::{
    Diagnosed, UnknownBuild, count_errors, diagnose, diagnose_editor_build,
    diagnose_editor_build_in, select_builds,
};
pub use checks::check_file;
use checks::check_sources;
pub use diagnostic::{Diagnostic, EVIDENCE, Evidence, RelatedInfo, Severity};
pub use intended::{Acknowledged, Acknowledgement, Acknowledgements, Applied};
pub use levels::apply_levels;
pub use page::{
    PageChecker, PageIndex, check_all_builds, check_builds, check_pages, check_project,
};
pub use project::{
    FileEntry, LOCK_FILE_ID, LoadError, LocateError, MODEL_FILE, ModelFile, Project, ReadFailure,
    SourceFile,
};
pub use registry::{Entry, Example, Level, Next, Registry};
pub use scope::{Reported, Scope, ScopeError, locate_for};

/// Checks every file of the project at file level (SPEC §8.1): the content
/// model's warnings, then each source file's diagnostics, in file order and,
/// within a file, in source order.
///
/// Never panics on user input. Sorting is stable, so two tools that call it
/// on the same project get the same list.
pub fn check_files(project: &Project) -> Vec<Diagnostic> {
    // The content model's warnings are part of the list.
    let mut out: Vec<Diagnostic> = project.model_warnings().to_vec();
    let model = out.len();
    for file in project.sources() {
        out.extend(check_file(project, file));
    }
    // The lock and the copies come next to the content model; whether a copy
    // is used is known once every file's snippets have been read.
    let sources = check_sources(project);
    out.splice(model..model, sources);
    out
}
