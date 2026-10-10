//! The adapter for `ascribe-check`: file-level diagnostics.
//!
//! It handles the `check` tag. Its diagnostics are `ascribe_check::check_files`
//! on a project built from the case: the case's content model, and every
//! `.md` file under its content root. That's the same entry point
//! `ascribe check`, the build, and the language server call.
//!
//! A `check` case that also carries `structure` or `parser` gets its
//! diagnostics here, from the whole check, not from the parser alone.

use ascribe_check::{Acknowledgements, Project, apply_levels, check_files};
use ascribe_conformance::{
    AdapterError, AdapterResult, Case, CaseKind, ConformanceAdapter, Diagnostic,
};
use ascribe_core::{FileId, RelPath, WideEncoding};

/// Handles the cases that expect file-level diagnostics.
pub struct CheckAdapter;

impl ConformanceAdapter for CheckAdapter {
    fn name(&self) -> &str {
        "check"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "check"
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        file_level_diagnostics(case).map(Some)
    }
}

/// The project a case describes: its model, its sources, its roots, and the
/// day it's checked on. A single-file case's content root is the case
/// directory; a project case's is `files/`. Either way the case directory is
/// the project root.
pub fn project(case: &Case) -> Result<Project, AdapterError> {
    let text = std::fs::read_to_string(&case.model)
        .map_err(|e| AdapterError(format!("couldn't read {}: {e}", case.model.display())))?;
    let model = ascribe_model::load_str(&text, FileId::new(0)).map_err(|issues| {
        let slugs: Vec<_> = issues.iter().map(|i| i.slug.to_string()).collect();
        AdapterError(format!(
            "{} doesn't load: {}",
            case.model.display(),
            slugs.join(", ")
        ))
    })?;
    let content_root = match case.kind {
        CaseKind::SingleFile => RelPath::root(),
        CaseKind::Project => RelPath::parse("files").map_err(|e| AdapterError(e.to_string()))?,
    };
    let sources =
        Project::read_sources(&case.dir, &content_root).map_err(|e| AdapterError(e.to_string()))?;
    let today = match &case.expect.today {
        Some(day) => Some(
            ascribe_core::Date::parse(day)
                .ok_or_else(|| AdapterError(format!("`today` isn't a date: {day}")))?,
        ),
        None => None,
    };
    Ok(Project::from_parts(case.dir.clone(), content_root, model, text, sources).with_today(today))
}

/// `check_files` on the case's project, in the harness's format.
pub fn file_level_diagnostics(case: &Case) -> Result<Vec<Diagnostic>, AdapterError> {
    let project = project(case)?;
    let reported = reported(&project, check_files(&project), false);
    to_conformance(&project, reported)
}

/// The page-level diagnostics of `build`, in the harness's format. The
/// file-level diagnostics are checked with them, as `ascribe check` does, so
/// an acknowledgement of either is applied; they're left out after. With the
/// case's only build, an acknowledgement nothing needs is reported.
pub fn page_level_diagnostics(
    project: &Project,
    build: &ascribe_model::Build,
) -> Result<Vec<Diagnostic>, AdapterError> {
    let file_level = check_files(project);
    let mut all = file_level.clone();
    all.extend(ascribe_check::check_pages(project, build));
    let only_build = project.model().builds.len() == 1;
    let page_level = reported(project, all, only_build)
        .into_iter()
        .filter(|d| {
            !file_level
                .iter()
                .any(|f| f.slug == d.slug && f.location == d.location)
        })
        .collect();
    to_conformance(project, page_level)
}

/// What every tool reports of `diagnostics`: the content model's `[checks]`
/// levels applied, then the acknowledgements; with `unused`, an
/// acknowledgement that covers nothing is reported.
fn reported(
    project: &Project,
    diagnostics: Vec<ascribe_check::Diagnostic>,
    unused: bool,
) -> Vec<ascribe_check::Diagnostic> {
    let leveled = apply_levels(&project.model().checks, diagnostics);
    Acknowledgements::of(project)
        .apply(project.model(), leveled, unused)
        .diagnostics
}

/// Diagnostics in the harness's format: the file relative to the content
/// root, a 1-based line, a column in Unicode scalar values, and the severity.
pub fn to_conformance(
    project: &Project,
    diagnostics: Vec<ascribe_check::Diagnostic>,
) -> Result<Vec<Diagnostic>, AdapterError> {
    let mut out = Vec::new();
    for d in diagnostics {
        let file = project
            .file(d.location.file)
            .ok_or_else(|| AdapterError(format!("{} has no file", d.slug)))?;
        let index = ascribe_core::LineIndex::new(file.text);
        let pos = index
            .wide_line_col(WideEncoding::Utf32, d.location.span.start())
            .ok_or_else(|| AdapterError(format!("bad location for {}", d.slug)))?;
        let path = file
            .content_path
            .map_or_else(|| file.display_path.clone(), |p| p.to_string());
        out.push(Diagnostic {
            slug: d.slug.as_str().to_owned(),
            file: path,
            line: pos.line + 1,
            column: pos.col + 1,
            severity: Some(d.severity.as_str().to_owned()),
        });
    }
    Ok(out)
}
