//! Runs the official CommonMark spec examples against a markdown renderer.
//!
//! The suite is renderer-agnostic: [`run`] takes any `&str -> String` HTML
//! renderer. `baselines/comrak.toml` records unmodified comrak from
//! crates.io; the `comrak-ascribe` fork's baselines, with the Ascribe option
//! off and on, are next to it.
//!
//! A baseline pins the exact set of failing examples, not just a count, so a
//! change that fixes one example and breaks another is still caught.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The CommonMark spec version whose examples are in `spec.json`. It's the
/// version comrak targets.
pub const SPEC_VERSION: &str = "0.31.2";

/// Errors loading the spec examples or a baseline.
#[derive(Debug, thiserror::Error)]
pub enum SuiteError {
    /// A file couldn't be read.
    #[error("couldn't read {path}: {source}")]
    Io {
        /// The file.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },
    /// `spec.json` isn't valid.
    #[error("couldn't parse {path}: {source}")]
    Json {
        /// The file.
        path: PathBuf,
        /// The underlying error.
        source: serde_json::Error,
    },
    /// A baseline file isn't valid.
    #[error("couldn't parse {path}: {source}")]
    Toml {
        /// The file.
        path: PathBuf,
        /// The underlying error.
        source: toml::de::Error,
    },
}

/// One example from the CommonMark spec, as it appears in `spec.json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Example {
    /// The markdown input.
    pub markdown: String,
    /// The expected HTML output.
    pub html: String,
    /// The example's number in the spec, starting at 1.
    pub example: u32,
    /// The first line of the example in the spec's source text.
    pub start_line: u32,
    /// The last line of the example in the spec's source text.
    pub end_line: u32,
    /// The spec section the example belongs to.
    pub section: String,
}

/// The directory holding `spec.json` and `baselines/`.
pub fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Loads the examples from a `spec.json` file.
pub fn load_examples(path: &Path) -> Result<Vec<Example>, SuiteError> {
    let text = std::fs::read_to_string(path).map_err(|source| SuiteError::Io {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| SuiteError::Json {
        path: path.to_owned(),
        source,
    })
}

/// Loads the examples bundled with this crate, for [`SPEC_VERSION`].
pub fn load_bundled_examples() -> Result<Vec<Example>, SuiteError> {
    load_examples(&suite_dir().join("spec.json"))
}

/// An example whose rendered HTML didn't match.
#[derive(Debug, Clone)]
pub struct Failure {
    /// The example's number.
    pub example: u32,
    /// The example's section.
    pub section: String,
    /// The markdown input.
    pub markdown: String,
    /// The HTML the spec expects.
    pub expected: String,
    /// The HTML the renderer produced.
    pub actual: String,
}

/// The result of running a renderer over the examples.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// The number of examples run.
    pub total: usize,
    /// The number of examples whose HTML matched exactly.
    pub passed: usize,
    /// Every example that didn't match, in spec order.
    pub failures: Vec<Failure>,
}

/// Renders every example and compares the HTML with the spec's, byte for byte.
pub fn run(examples: &[Example], mut render: impl FnMut(&str) -> String) -> Report {
    let mut report = Report {
        total: examples.len(),
        ..Report::default()
    };
    for ex in examples {
        let actual = render(&ex.markdown);
        if actual == ex.html {
            report.passed += 1;
        } else {
            report.failures.push(Failure {
                example: ex.example,
                section: ex.section.clone(),
                markdown: ex.markdown.clone(),
                expected: ex.html.clone(),
                actual,
            });
        }
    }
    report
}

impl Report {
    /// The numbers of the failing examples.
    pub fn failing(&self) -> Vec<u32> {
        self.failures.iter().map(|f| f.example).collect()
    }

    /// A one-line pass count, for example `650/652 passed`.
    pub fn pass_count(&self) -> String {
        format!("{}/{} passed", self.passed, self.total)
    }

    /// Failure counts per spec section, for sections with any failures.
    pub fn failures_by_section(&self) -> BTreeMap<&str, usize> {
        let mut map = BTreeMap::new();
        for f in &self.failures {
            *map.entry(f.section.as_str()).or_insert(0) += 1;
        }
        map
    }

    /// A readable listing of every failure, with input, expected, and actual HTML.
    pub fn describe_failures(&self) -> String {
        let mut out = String::new();
        for f in &self.failures {
            let _ = writeln!(
                out,
                "--- example {} ({})\ninput:\n{}\nexpected:\n{}\nactual:\n{}",
                f.example, f.section, f.markdown, f.expected, f.actual
            );
        }
        out
    }
}

/// A recorded result that later runs are compared against.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    /// The CommonMark spec version the baseline was recorded for.
    pub spec_version: String,
    /// A description of the renderer, for people reading the file.
    pub renderer: String,
    /// The number of examples.
    pub total: usize,
    /// The number of passing examples.
    pub passed: usize,
    /// The numbers of the failing examples, in spec order.
    pub failing: Vec<u32>,
}

impl Baseline {
    /// Loads a baseline file.
    pub fn load(path: &Path) -> Result<Self, SuiteError> {
        let text = std::fs::read_to_string(path).map_err(|source| SuiteError::Io {
            path: path.to_owned(),
            source,
        })?;
        toml::from_str(&text).map_err(|source| SuiteError::Toml {
            path: path.to_owned(),
            source,
        })
    }

    /// Compares a report with this baseline. Returns a description of every
    /// difference, or an empty list when the report matches exactly.
    pub fn compare(&self, report: &Report) -> Vec<String> {
        let mut problems = Vec::new();
        if self.spec_version != SPEC_VERSION {
            problems.push(format!(
                "baseline is for spec {}, but the suite is {SPEC_VERSION}",
                self.spec_version
            ));
        }
        if self.total != report.total {
            problems.push(format!(
                "baseline has {} examples, but the run had {}",
                self.total, report.total
            ));
        }
        if self.passed != report.passed {
            problems.push(format!(
                "baseline has {} passing, but the run had {}",
                self.passed, report.passed
            ));
        }
        let actual = report.failing();
        let newly_failing: Vec<u32> = actual
            .iter()
            .copied()
            .filter(|n| !self.failing.contains(n))
            .collect();
        let newly_passing: Vec<u32> = self
            .failing
            .iter()
            .copied()
            .filter(|n| !actual.contains(n))
            .collect();
        if !newly_failing.is_empty() {
            problems.push(format!("newly failing examples: {newly_failing:?}"));
        }
        if !newly_passing.is_empty() {
            problems.push(format!(
                "newly passing examples (update the baseline): {newly_passing:?}"
            ));
        }
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example(n: u32, md: &str, html: &str) -> Example {
        Example {
            markdown: md.into(),
            html: html.into(),
            example: n,
            start_line: 0,
            end_line: 0,
            section: "Test".into(),
        }
    }

    #[test]
    fn run_counts_exact_matches() {
        let examples = [example(1, "a", "A"), example(2, "b", "x")];
        let report = run(&examples, |s| s.to_uppercase());
        assert_eq!(report.passed, 1);
        assert_eq!(report.failing(), vec![2]);
        assert_eq!(report.pass_count(), "1/2 passed");
    }

    #[test]
    fn baseline_reports_changes_in_either_direction() {
        let examples = [example(1, "a", "A"), example(2, "b", "x")];
        let report = run(&examples, |s| s.to_uppercase());
        let baseline = Baseline {
            spec_version: SPEC_VERSION.into(),
            renderer: "test".into(),
            total: 2,
            passed: 1,
            failing: vec![1],
        };
        let problems = baseline.compare(&report);
        assert!(
            problems
                .iter()
                .any(|p| p.contains("newly failing examples: [2]"))
        );
        assert!(
            problems
                .iter()
                .any(|p| p.contains("newly passing examples"))
        );
    }

    #[test]
    fn bundled_examples_load() {
        let examples = load_bundled_examples().unwrap();
        assert_eq!(examples.len(), 652);
        assert_eq!(examples[0].example, 1);
    }
}
