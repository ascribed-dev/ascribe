//! The CommonMark spec examples against the `comrak-tessera` fork (phase 04),
//! twice: with the Ascribe option off and with it on.
//!
//! - Off, the fork must match unmodified comrak exactly: its baseline,
//!   `baselines/comrak-tessera-off.toml`, must equal `baselines/comrak.toml`
//!   apart from the renderer description.
//! - On, every example that fails and passes with the option off must involve
//!   a line that is a valid directive line, and is justified in this suite's
//!   README. The baseline is `baselines/comrak-tessera-on.toml`.
//!
//! Prints both pass counts and fails if either result differs from its
//! baseline in either direction.
//!
//! - `COMMONMARK_VERBOSE=1` prints every failing example.
//! - `COMMONMARK_WRITE_BASELINE=1` rewrites both baselines from this run.

use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use comrak_tessera::tessera::TesseraOptions;
use tessera_commonmark_suite::{
    Baseline, Example, Report, SPEC_VERSION, load_bundled_examples, run, suite_dir,
};

const RENDERER_OFF: &str =
    "comrak-tessera (the fork), Ascribe option off, CommonMark options only, render.unsafe = true";
const RENDERER_ON: &str = "comrak-tessera (the fork), Ascribe option on with the built-in keywords, \
     `end`, and a project widget, CommonMark options only, render.unsafe = true";

/// The built-in keywords (SPEC §4), `end`, and one project widget, as in the
/// fork's spike tests. Only `@note` and the widget take a text primary.
fn keywords() -> TesseraOptions {
    TesseraOptions::new()
        .keyword("id", false)
        .keyword("include", false)
        .keyword("variant", false)
        .keyword("available", false)
        .keyword("note", true)
        .keyword("steps", false)
        .keyword("details", false)
        .keyword("end", false)
        .keyword("quill-demo", true)
}

fn options(tessera: bool) -> comrak_tessera::Options<'static> {
    let mut options = comrak_tessera::Options::default();
    // The spec examples expect raw HTML to pass through.
    options.render.r#unsafe = true;
    if tessera {
        options.extension.tessera = Some(Arc::new(keywords()));
    }
    options
}

fn render_with(options: &comrak_tessera::Options) -> impl FnMut(&str) -> String {
    move |markdown| comrak_tessera::markdown_to_html(markdown, options)
}

/// Compares a report with its baseline, or rewrites the baseline. Returns
/// whether the check passed.
fn check(report: &Report, renderer: &str, file: &str) -> bool {
    println!(
        "CommonMark {SPEC_VERSION}, {renderer}: {}",
        report.pass_count()
    );
    for (section, count) in report.failures_by_section() {
        println!("  {count} failing in {section}");
    }
    if std::env::var_os("COMMONMARK_VERBOSE").is_some() {
        print!("{}", report.describe_failures());
    }

    let path = suite_dir().join("baselines").join(file);
    if std::env::var_os("COMMONMARK_WRITE_BASELINE").is_some() {
        return write_baseline(report, renderer, &path);
    }

    let baseline = match Baseline::load(&path) {
        Ok(baseline) => baseline,
        Err(err) => {
            eprintln!("error: {err}");
            return false;
        }
    };
    let problems = baseline.compare(report);
    if problems.is_empty() {
        println!("matches {}", path.display());
        true
    } else {
        eprintln!("result differs from {}:", path.display());
        for problem in problems {
            eprintln!("  {problem}");
        }
        false
    }
}

fn write_baseline(report: &Report, renderer: &str, path: &Path) -> bool {
    let baseline = Baseline {
        spec_version: SPEC_VERSION.into(),
        renderer: renderer.into(),
        total: report.total,
        passed: report.passed,
        failing: report.failing(),
    };
    let text = match toml::to_string(&baseline) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("error: {err}");
            return false;
        }
    };
    let header = "# Written by `COMMONMARK_WRITE_BASELINE=1 cargo test -p tessera-commonmark-suite --test fork`.\n\
                  # Every example that fails only with the Ascribe option on is justified in README.md.\n";
    match std::fs::write(path, format!("{header}{text}")) {
        Ok(()) => {
            println!("wrote {}", path.display());
            true
        }
        Err(err) => {
            eprintln!("error: couldn't write {}: {err}", path.display());
            false
        }
    }
}

/// The examples whose HTML changes when the Ascribe option is turned on,
/// whether or not they pass either way.
fn changed_by_option(examples: &[Example]) -> Vec<u32> {
    let off = options(false);
    let on = options(true);
    examples
        .iter()
        .filter(|ex| {
            comrak_tessera::markdown_to_html(&ex.markdown, &off)
                != comrak_tessera::markdown_to_html(&ex.markdown, &on)
        })
        .map(|ex| ex.example)
        .collect()
}

/// The directive line [`check_after_directive_line`] puts before every
/// example, and the HTML the fork renders for it.
const LEADING_LINE: &str = "@end\n";
const LEADING_HTML: &str = "<div data-tessera-line=\"@end\">\n</div>\n";

/// A directive line is a leaf block that closes when the next line starts,
/// so putting one before an example must leave the example's own blocks
/// exactly as they were. This checks that for every example, which puts a
/// directive line next to every kind of block the spec covers. Every example
/// must pass; there's no baseline.
fn check_after_directive_line(examples: &[Example]) -> bool {
    let options = options(true);
    let report = run(examples, |markdown| {
        let html = comrak_tessera::markdown_to_html(&format!("{LEADING_LINE}{markdown}"), &options);
        match html.strip_prefix(LEADING_HTML) {
            Some(rest) => rest.to_string(),
            None => html,
        }
    });
    println!(
        "CommonMark {SPEC_VERSION}, each example after a directive line, Ascribe option on: {}",
        report.pass_count()
    );
    if report.failures.is_empty() {
        true
    } else {
        eprint!("{}", report.describe_failures());
        false
    }
}

fn main() -> ExitCode {
    let examples = match load_bundled_examples() {
        Ok(examples) => examples,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let off = run(&examples, render_with(&options(false)));
    let on = run(&examples, render_with(&options(true)));
    let mut ok = check(&off, RENDERER_OFF, "comrak-tessera-off.toml");
    ok &= check(&on, RENDERER_ON, "comrak-tessera-on.toml");

    // With the option off, the fork must behave exactly as upstream does.
    match Baseline::load(&suite_dir().join("baselines").join("comrak.toml")) {
        Ok(upstream) => {
            let problems = upstream.compare(&off);
            if !problems.is_empty() {
                eprintln!("with the Ascribe option off, the fork differs from unmodified comrak:");
                for problem in problems {
                    eprintln!("  {problem}");
                }
                ok = false;
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            ok = false;
        }
    }

    ok &= check_after_directive_line(&examples);

    let changed = changed_by_option(&examples);
    println!(
        "{} of {} examples render differently with the Ascribe option on: {changed:?}",
        changed.len(),
        examples.len()
    );

    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
