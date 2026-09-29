//! The adapter for page-level checks (phase 14).
//!
//! It handles the `page-check` tag. A case with it expects, per build, the
//! diagnostics `tessera_check::check_pages` reports for that build: the same
//! entry point `tessera check`, the build, and the language server call. The
//! build's pages and assets come from the resolve adapter's code, so a case
//! that expects both (the Quill project, the selection cases) runs whole; and
//! the file-level diagnostics a case lists at the top level are
//! `check_files`, as for the `check` tag.

use tessera_conformance::{AdapterResult, BuildResult, Case, ConformanceAdapter, Diagnostic};

/// Handles the cases that expect page-level diagnostics.
pub struct PageCheckAdapter;

impl ConformanceAdapter for PageCheckAdapter {
    fn name(&self) -> &str {
        "page-check"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "page-check"
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        super::file_level_diagnostics(case).map(Some)
    }

    fn build(&self, case: &Case, build: &str) -> AdapterResult<BuildResult> {
        super::resolve::resolve_build(case, build).map(Some)
    }
}
