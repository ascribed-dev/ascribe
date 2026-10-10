//! The adapter for `ascribe-model`: loading `ascribe.toml` (the `model` tag).
//!
//! A `model` case's diagnostics are what the loader reports for the case's
//! content model, in `ascribe.toml` (a loader rule is `file: ascribe.toml`,
//! README, "The shared model and the content root"). The fixtures for the
//! other loader rules are in `crates/ascribe-model/tests/`.

use ascribe_conformance::{AdapterError, AdapterResult, Case, ConformanceAdapter, Diagnostic};
use ascribe_core::{FileId, LineIndex, WideEncoding};

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
        let issues = match ascribe_model::load_str(&text, FileId::new(0)) {
            Ok(model) => model.warnings,
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
            });
        }
        Ok(Some(out))
    }
}
