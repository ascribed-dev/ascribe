//! The adapter for `tessera-resolve`'s source index: what phase 11 produces.
//!
//! It handles the `include` tag (includes and the source index, SPEC §4.2) and
//! the `slug` tag (heading slugs and source ids, SPEC §5.5).
//!
//! **Every case with these tags also carries another area tag** (`check`,
//! `page-check`, or `resolve`): the source index has no outline of its own, and
//! what the cases expect (diagnostics, resolved pages, page ids) is decided by
//! the checks and by build resolution, which are phases 10, 12, and 14. So the
//! adapter produces no outline, diagnostics, or build result, and doesn't get
//! in the way of the adapters that will (the runner asks the first adapter that
//! answers). What it offers is [`source_problems`]: the problems that follow
//! from the source alone, with the locations SPEC §8.1 and Q20 give them, for
//! `tests/source_index_rows.rs` to compare with the cases' expectations now.
//! Phases 10 and 14 read the same problems from `Project::problems` and
//! `ExpandedPage::problems`.

// Only `source_index_rows.rs` calls the helpers below; the conformance runner
// links this module too and doesn't.
#![allow(dead_code)]

use std::sync::Arc;

use tessera_conformance::{AdapterError, Case, ConformanceAdapter, Diagnostic};
use tessera_core::{FileId, Issue, LineIndex, RelPath, WideEncoding};
use tessera_resolve::{DiskFs, FileKind, Layout, Project};

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

/// The slugs of the file-level rows the source index answers: a referenced
/// file that doesn't exist, and what a link names.
pub const FILE_LEVEL: &[&str] = &[
    "include-target-missing",
    "link-target-missing",
    "image-source-missing",
    "link-to-fragment",
    "link-route",
];

/// The slugs of the page-level rows the source index answers: what expansion
/// finds, and links to ids.
pub const PAGE_LEVEL: &[&str] = &[
    "include-cycle",
    "include-id-missing",
    "link-id-missing",
    "link-id-in-fragment",
];

/// The source problems of a case.
#[derive(Debug, Default)]
pub struct SourceProblems {
    /// Problems of the file-level rows, from every file.
    pub file_level: Vec<Diagnostic>,
    /// Problems of the page-level rows, from every page, once each.
    pub page_level: Vec<Diagnostic>,
}

fn err(e: impl std::fmt::Display) -> AdapterError {
    AdapterError(e.to_string())
}

/// Indexes a case as a project: the case directory is the project root, and
/// `Case::content_root` the content root, whatever the model says.
///
/// The model is read with `load_str`, which skips the file-system rules (the
/// shared model's `content-root = "files"` doesn't exist for a single-file
/// case). The shared model declares a `role` attribute on `quill-audience`,
/// which phase 08's loader reserves (Q27), so the key is read as `audience`;
/// only the model's phrases and fragment patterns matter to the source index.
/// Phase 03's cases still use `role`, so the model itself is left alone here.
pub fn load_project(case: &Case) -> Result<Project, AdapterError> {
    let text = std::fs::read_to_string(&case.model)
        .map_err(err)?
        .replace("attributes = { role =", "attributes = { audience =");
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

/// The source problems of every file in the case.
pub fn source_problems(case: &Case) -> Result<SourceProblems, AdapterError> {
    let project = load_project(case)?;
    let mut out = SourceProblems::default();
    for file in project.files() {
        for issue in project.problems(&file.path) {
            let slug = issue.slug.as_str();
            if FILE_LEVEL.contains(&slug) {
                out.file_level.push(diagnostic(&project, &issue, None)?);
            } else if PAGE_LEVEL.contains(&slug) && file.kind == FileKind::Page {
                // A link to an id: reported at the link, in the page.
                out.page_level.push(diagnostic(&project, &issue, None)?);
            }
        }
    }
    for page in project.pages() {
        let expanded = project
            .expand(&page.path)
            .ok_or_else(|| AdapterError(format!("{} isn't in the project", page.path)))?;
        for problem in &expanded.problems {
            // A cycle is reported where it closes, in the file containing that
            // include (Q20); everything else at the outermost include site.
            let at = match (problem.issue.slug.as_str(), problem.via.first()) {
                ("include-cycle", _) | (_, None) => None,
                (_, Some(site)) => Some(*site),
            };
            out.page_level
                .push(diagnostic(&project, &problem.issue, at)?);
        }
    }
    out.file_level.sort_by(key);
    out.page_level.sort_by(key);
    out.page_level.dedup();
    Ok(out)
}

fn key(a: &Diagnostic, b: &Diagnostic) -> std::cmp::Ordering {
    (&a.file, a.line, a.column, &a.slug).cmp(&(&b.file, b.line, b.column, &b.slug))
}

/// An issue as a conformance diagnostic, at its own location or at an include
/// site.
fn diagnostic(
    project: &Project,
    issue: &Issue,
    site: Option<tessera_resolve::IncludeSite>,
) -> Result<Diagnostic, AdapterError> {
    let (file, span) = match site {
        Some(site) => (site.file, site.span),
        None => (issue.location.file, issue.location.span),
    };
    let index = project
        .file_by_id(file)
        .ok_or_else(|| AdapterError(format!("no file {file}")))?;
    let pos = LineIndex::new(&index.source)
        .wide_line_col(WideEncoding::Utf32, span.start())
        .ok_or_else(|| AdapterError(format!("bad location in {}", index.path)))?;
    Ok(Diagnostic {
        slug: issue.slug.as_str().to_owned(),
        file: index.path.to_string(),
        line: pos.line + 1,
        column: pos.col + 1,
    })
}
