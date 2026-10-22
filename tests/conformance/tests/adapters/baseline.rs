//! The adapter for the baseline, `ascribe.baseline` beside the case's
//! `ascribe.toml` (tag `baseline`).
//!
//! Its diagnostics are what `ascribe check` reports of the whole project for
//! every build, with the baseline read: `ascribe_check::diagnose`, then
//! `ascribe_check::baseline::apply`, the calls the command makes. A case
//! expects them in its top-level `diagnostics`, so it uses file-level checks
//! only. A baseline that can't be read stops `ascribe check` before it
//! reports anything, so it's tested with the command, in
//! `crates/ascribe-cli/tests/all/baseline.rs`, and not here.

use ascribe_check::baseline;
use ascribe_conformance::{AdapterError, AdapterResult, Case, ConformanceAdapter, Diagnostic};

use super::check::{project, to_conformance};

/// Handles the cases about the baseline.
pub struct BaselineAdapter;

impl ConformanceAdapter for BaselineAdapter {
    fn name(&self) -> &str {
        "baseline"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "baseline"
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        let project = project(case)?;
        let found = ascribe_check::diagnose(&project, &[])
            .map_err(|e| AdapterError(e.to_string()))?
            .diagnostics;
        let diagnostics = match project.baseline() {
            None => found,
            Some(Err(e)) => return Err(AdapterError(e.to_string())),
            Some(Ok(b)) => baseline::apply(&project, b, found, true, &[]).diagnostics,
        };
        to_conformance(&project, diagnostics).map(Some)
    }
}
