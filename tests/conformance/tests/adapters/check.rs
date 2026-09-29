//! The adapter for `tessera-check`: file-level diagnostics (phase 10).
//!
//! It handles the `check` tag. Its diagnostics are `tessera_check::check_files`
//! on a project built from the case: the case's content model, and every
//! `.md` file under its content root. That's the same entry point
//! `tessera check`, the build, and the language server call.
//!
//! A `check` case that also carries `structure` or `parser` gets its
//! diagnostics here, from the whole check, not from the parser alone.

use tessera_check::{Project, check_files};
use tessera_conformance::{
    AdapterError, AdapterResult, Case, CaseKind, ConformanceAdapter, Diagnostic,
};
use tessera_core::{FileId, RelPath, WideEncoding};

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

/// The project a case describes: its model, its sources, and its roots. A
/// single-file case's content root is the case directory; a project case's
/// is `files/`. Either way the case directory is the project root.
pub fn project(case: &Case) -> Result<Project, AdapterError> {
    let text = std::fs::read_to_string(&case.model)
        .map_err(|e| AdapterError(format!("couldn't read {}: {e}", case.model.display())))?;
    let model = tessera_model::load_str(&text, FileId::new(0)).map_err(|issues| {
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
    Ok(Project::from_parts(
        case.dir.clone(),
        content_root,
        model,
        text,
        sources,
    ))
}

/// `check_files` on the case's project, in the harness's format.
pub fn file_level_diagnostics(case: &Case) -> Result<Vec<Diagnostic>, AdapterError> {
    let project = project(case)?;
    to_conformance(&project, check_files(&project))
}

/// Diagnostics in the harness's format: the file relative to the content root,
/// a 1-based line, and a column in Unicode scalar values.
pub fn to_conformance(
    project: &Project,
    diagnostics: Vec<tessera_check::Diagnostic>,
) -> Result<Vec<Diagnostic>, AdapterError> {
    let mut out = Vec::new();
    for d in diagnostics {
        let file = project
            .file(d.location.file)
            .ok_or_else(|| AdapterError(format!("{} has no file", d.slug)))?;
        let index = tessera_core::LineIndex::new(file.text);
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
        });
    }
    Ok(out)
}
