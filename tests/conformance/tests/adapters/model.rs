//! The adapter for `ascribe-model`: loading `ascribe.toml` (the `model` tag).
//!
//! A `model` case's diagnostics are what the loader reports for the case's
//! content model, in `ascribe.toml` (a loader rule is `file: ascribe.toml`,
//! README, "The shared model and the content root"), and the problems with the
//! files its types name, looked for from the case directory, the project
//! root, with the content root the model gives. What `[from]` takes is read
//! from the case directory too. The fixtures for the other loader rules are
//! in `crates/ascribe-model/tests/`.

use ascribe_conformance::{AdapterError, AdapterResult, Case, ConformanceAdapter, Diagnostic};
use ascribe_core::{FileId, LineIndex, WideEncoding};
use ascribe_resolve::{DiskFs, Layout};

/// Handles the cases that expect content-model diagnostics.
pub struct ModelAdapter;

impl ConformanceAdapter for ModelAdapter {
    fn name(&self) -> &str {
        "model"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "model"
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        let text = std::fs::read_to_string(&case.model)
            .map_err(|e| AdapterError(format!("couldn't read {}: {e}", case.model.display())))?;
        let issues = match load_model(case, &text) {
            Ok(mut model) => {
                let disk = DiskFs::new(&case.dir, &Layout::from_model(&model));
                ascribe_check::type_files::from_lock(&mut model, &disk);
                let files = ascribe_check::type_files::check(&model, &disk);
                model.warnings.into_iter().chain(files).collect()
            }
            Err(issues) => issues,
        };
        let index = LineIndex::new(&text);
        let mut out = Vec::new();
        for issue in issues {
            let pos = index
                .wide_line_col(WideEncoding::Utf32, issue.location.span.start())
                .ok_or_else(|| AdapterError(format!("bad location for {}", issue.slug)))?;
            out.push(Diagnostic {
                slug: issue.slug.as_str().to_owned(),
                file: "ascribe.toml".to_owned(),
                line: pos.line + 1,
                column: pos.col + 1,
                severity: None,
                related: Vec::new(),
            });
        }
        Ok(Some(out))
    }
}

/// The case's content model in `text`, read as text alone, without the
/// rules about the folders it names; with `[from]`, what it takes is read
/// from the case directory, its project root, as `ascribe check` reads it,
/// and those rules run, so such a case names its content root.
pub fn load_model(
    case: &Case,
    text: &str,
) -> Result<ascribe_model::ContentModel, Vec<ascribe_core::Issue>> {
    if ascribe_model::from_files(text).is_empty() {
        return ascribe_model::load_str(text, FileId::new(0));
    }
    let taking = DiskFs::new(
        &case.dir,
        &Layout {
            content_root: ascribe_core::RelPath::root(),
            output_dir: ascribe_core::RelPath::root(),
        },
    );
    ascribe_model::load_str_taking(
        text,
        FileId::new(0),
        &case.dir,
        ascribe_model::SourceBound::Anywhere,
        &taking,
    )
}
