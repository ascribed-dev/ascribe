//! The adapter for the baseline, `ascribe.baseline` beside the case's
//! `ascribe.toml` (tag `baseline`).
//!
//! Its diagnostics are what `ascribe check`, `ascribe build`, and `ascribe
//! report` report of the whole project for every build, with the baseline
//! read: `ascribe_check::diagnose_with`, the call `build` and `report` make,
//! which applies the baseline as `check` does. A case expects them in its
//! top-level `diagnostics`, so it uses file-level checks only. A baseline
//! that can't be read stops the commands before they report anything, so
//! it's tested with them, in `crates/ascribe-cli/tests/all/baseline.rs`, and
//! not here.

use ascribe_check::Request;
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
        let request = Request {
            baseline: true,
            ..Request::default()
        };
        let diagnostics = ascribe_check::diagnose_with(&project, &request)
            .map_err(|e| AdapterError(e.to_string()))?
            .into_diagnostics();
        to_conformance(&project, diagnostics).map(Some)
    }
}
