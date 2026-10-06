//! The CommonMark baseline: the official spec examples against unmodified
//! comrak from crates.io.
//!
//! Prints the pass count and fails if the result differs from
//! `baselines/comrak.toml` in either direction.
//!
//! - `COMMONMARK_VERBOSE=1` prints every failing example.
//! - `COMMONMARK_WRITE_BASELINE=1` rewrites the baseline from this run.

// A test harness run with `harness = false`: its report is printed.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::process::ExitCode;

use tessera_commonmark_suite::{Baseline, SPEC_VERSION, load_bundled_examples, run, suite_dir};

const RENDERER: &str =
    "comrak (crates.io, unmodified), CommonMark options only, render.unsafe = true";

fn render(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    // The spec examples expect raw HTML to pass through.
    options.render.r#unsafe = true;
    comrak::markdown_to_html(markdown, &options)
}

fn main() -> ExitCode {
    let examples = match load_bundled_examples() {
        Ok(examples) => examples,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };
    let report = run(&examples, render);

    println!(
        "CommonMark {SPEC_VERSION} baseline, {RENDERER}: {}",
        report.pass_count()
    );
    for (section, count) in report.failures_by_section() {
        println!("  {count} failing in {section}");
    }
    if std::env::var_os("COMMONMARK_VERBOSE").is_some() {
        print!("{}", report.describe_failures());
    }

    let path = suite_dir().join("baselines").join("comrak.toml");
    if std::env::var_os("COMMONMARK_WRITE_BASELINE").is_some() {
        let baseline = Baseline {
            spec_version: SPEC_VERSION.into(),
            renderer: RENDERER.into(),
            total: report.total,
            passed: report.passed,
            failing: report.failing(),
        };
        let text = match toml::to_string(&baseline) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("error: {err}");
                return ExitCode::FAILURE;
            }
        };
        let header =
            "# Written by `COMMONMARK_WRITE_BASELINE=1 cargo test -p tessera-commonmark-suite`.\n";
        if let Err(err) = std::fs::write(&path, format!("{header}{text}")) {
            eprintln!("error: couldn't write {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
        println!("wrote {}", path.display());
        return ExitCode::SUCCESS;
    }

    let baseline = match Baseline::load(&path) {
        Ok(baseline) => baseline,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };
    let problems = baseline.compare(&report);
    if problems.is_empty() {
        println!("matches {}", path.display());
        ExitCode::SUCCESS
    } else {
        eprintln!("result differs from {}:", path.display());
        for problem in problems {
            eprintln!("  {problem}");
        }
        ExitCode::FAILURE
    }
}
