//! The adapter for `tessera-resolve`'s source index: what phase 11 produces.
//!
//! It handles the `include` tag (includes and the source index, SPEC §4.2) and
//! the `slug` tag (heading slugs and source ids, SPEC §5.5).
//!
//! **Every case with these tags also carries another area tag** (`check`,
//! `page-check`, or `resolve`): the source index has no outline of its own, and
//! what the cases expect (diagnostics, resolved pages, page ids) is decided by
//! the checks and by build resolution. So the adapter produces no outline,
//! diagnostics, or build result, and doesn't get in the way of the adapters
//! that do (the runner asks the first adapter that answers). It offers
//! [`load_project`], which the `resolve` adapter uses to index a case.

use std::sync::Arc;

use tessera_conformance::{AdapterError, Case, ConformanceAdapter};
use tessera_core::{FileId, RelPath};
use tessera_resolve::{DiskFs, Layout, Project};

/// Handles the tags whose cases the source index helps answer.
pub struct IncludeAdapter;

impl ConformanceAdapter for IncludeAdapter {
    fn name(&self) -> &str {
        "include"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        matches!(tag, "include" | "slug")
    }
}

fn err(e: impl std::fmt::Display) -> AdapterError {
    AdapterError(e.to_string())
}

/// Indexes a case as a project: the case directory is the project root, and
/// `Case::content_root` the content root, whatever the model says.
///
/// The model is read with `load_str`, as phase 10's adapter reads it, which
/// skips the file-system rules (the shared model's `content-root = "files"`
/// doesn't exist for a single-file case).
pub fn load_project(case: &Case) -> Result<Project, AdapterError> {
    let text = std::fs::read_to_string(&case.model).map_err(err)?;
    let model = tessera_model::load_str(&text, FileId::new(0))
        .map_err(|issues| AdapterError(format!("the case's model doesn't load: {issues:?}")))?;
    let content_root = case
        .content_root()
        .strip_prefix(&case.dir)
        .map_err(err)?
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    let mut layout = Layout::from_model(&model);
    layout.content_root = RelPath::parse(&content_root).map_err(err)?;
    let fs = DiskFs::new(&case.dir, &layout);
    Ok(Project::load(Arc::new(model), layout, &fs))
}
