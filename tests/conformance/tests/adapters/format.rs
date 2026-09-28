//! The adapter for `tessera-fmt`: the `format` tag (canonical form, SPEC §8.3).
//!
//! A case's `input.md` is formatted under the case's content model, loaded by
//! `tessera-model`, and the result is compared with the case's `formatted`
//! file. The runner formats the result again to check that formatting is
//! idempotent.

use tessera_conformance::{AdapterError, AdapterResult, Case, ConformanceAdapter};
use tessera_core::FileId;
use tessera_fmt::{format_source, options_from_model};

/// Handles the `format` tag.
pub struct FormatAdapter;

impl ConformanceAdapter for FormatAdapter {
    fn name(&self) -> &str {
        "format"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "format"
    }

    fn format(&self, case: &Case, source: &str) -> AdapterResult<String> {
        let text = std::fs::read_to_string(&case.model)
            .map_err(|e| AdapterError(format!("couldn't read {}: {e}", case.model.display())))?;
        let model = tessera_model::load_str(&text, FileId::new(0)).map_err(|issues| {
            let slugs: Vec<&str> = issues.iter().map(|i| i.slug.as_str()).collect();
            AdapterError(format!(
                "{} isn't a valid content model: {}",
                case.model.display(),
                slugs.join(", ")
            ))
        })?;
        Ok(Some(format_source(
            source,
            &options_from_model(&model),
            &model,
        )))
    }
}
