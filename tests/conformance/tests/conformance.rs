//! Runs the conformance suite in `tests/conformance/cases/`.
//!
//! Prints a summary of every failure and skip, and exits non-zero if any case
//! fails or `SKIPS.toml` is stale.
//!
//! ```text
//! cargo test -p tessera-conformance --test conformance
//! cargo test -p tessera-conformance --test conformance -- --tag structure
//! cargo test -p tessera-conformance --test conformance -- samples/appendix-b
//! ```

mod adapters;

use std::process::ExitCode;

use tessera_conformance::{Registry, Suite, filter_from_args};

fn main() -> ExitCode {
    let mut registry = Registry::new();
    adapters::register(&mut registry);

    let filter = filter_from_args(std::env::args().skip(1));
    match Suite::bundled().run(&registry, &filter) {
        Ok(report) => {
            print!("{}", report.summary());
            if report.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
