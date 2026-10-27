//! Comparing benchmark results with recorded baselines.
//!
//! The benchmarks (`benches/perf.rs`, and those of `ascribe-resolve` and
//! `ascribe-lsp`, when
//! `ASCRIBE_BENCH_OUT` is set) append one JSON line per metric. A metric
//! **regresses** when its median exceeds `baseline * margin + floor_ms`, and
//! **misses its target** when it exceeds an absolute limit from the spec's
//! performance targets. Both are read from `baselines/perf.json`.
//!
//! A metric whose name starts with `memory/` is a peak resident memory in
//! megabytes, not a time: its line has the same fields, and it's compared the
//! same way, with the floor read as megabytes.
//!
//! The margin is deliberately generous (see `RESULTS.md`): shared CI runners
//! vary by a factor of two between runs and by more between machines, and the
//! job exists to catch an algorithm that got worse, not a few percent.
//!
//! The pull request's run (`corpora compare --quick`) compares only the
//! handful of metrics the baseline's `quick` section names, against the same
//! recorded medians, with that section's wider margin and floor.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// `baselines/perf.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Baseline {
    /// The machine the numbers were recorded on.
    pub machine: String,
    /// A metric fails when its median is over `baseline * margin + floor_ms`.
    pub margin: f64,
    /// Added to every limit, so a metric measured in microseconds isn't failed
    /// by scheduler noise.
    pub floor_ms: f64,
    /// Absolute limits in milliseconds (the spec's targets), by metric.
    pub targets: BTreeMap<String, f64>,
    /// The recorded median in milliseconds, by metric.
    pub metrics: BTreeMap<String, f64>,
    /// What the pull request's run compares.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quick: Option<Quick>,
}

/// The pull request's run: a few metrics, with a wider margin than the weekly
/// run's, since it measures once on whichever runner it gets.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quick {
    /// A metric fails when its median is over `baseline * margin + floor_ms`.
    pub margin: f64,
    /// Added to every limit.
    pub floor_ms: f64,
    /// The metrics it measures, each of which must be in the results.
    pub metrics: Vec<String>,
}

/// One line of a results file.
#[derive(Clone, Debug, Deserialize)]
pub struct Result {
    /// The metric.
    pub name: String,
    /// Its median, in milliseconds.
    pub median_ms: f64,
    /// Its 95th percentile, in milliseconds.
    pub p95_ms: f64,
}

/// The outcome of a comparison.
#[derive(Clone, Debug)]
pub struct Comparison {
    /// The table, for printing.
    pub text: String,
    /// Whether nothing regressed or missed its target.
    pub ok: bool,
}

/// The baseline file next to this crate.
pub fn default_baseline() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("baselines")
        .join("perf.json")
}

/// Metrics that are tracked (and so must be in a results file), whatever the
/// baseline says.
pub const REQUIRED: &[&str] = &[
    "check/synthetic-3000",
    "build/synthetic-3000-first",
    "check/noisy-1000-text",
    "diff/synthetic-3000-unchanged",
    "drift/synthetic-3000-snippets-unchanged",
    "lsp/keystroke-page-3000",
    "lsp/keystroke-fragment-3000",
    "lsp/watched-unread-3000",
    "lsp/completion-3000",
    "lsp/search-3000",
    "memory/check-synthetic-3000",
    "memory/build-synthetic-3000-first",
    "memory/diff-synthetic-3000-unchanged",
    "memory/lsp-load-3000",
    "memory/lsp-100-edits-3000",
];

/// Reads a results file: the last line for a metric wins.
///
/// # Errors
///
/// A message when the file can't be read or a line isn't a result.
pub fn read_results(path: &Path) -> std::result::Result<BTreeMap<String, Result>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = BTreeMap::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let r: Result = serde_json::from_str(line).map_err(|e| format!("{line}: {e}"))?;
        out.insert(r.name.clone(), r);
    }
    Ok(out)
}

/// The machine, for a baseline recorded here.
fn machine() -> String {
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|t| {
            t.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1).map(|s| s.trim().to_owned()))
        })
        .unwrap_or_else(|| "unknown CPU".into());
    let cores = std::thread::available_parallelism().map_or(0, usize::from);
    format!("{cpu}, {cores} logical cores, {}", std::env::consts::OS)
}

/// What [`compare`] does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The weekly run: every result, against the baseline's margin.
    Full,
    /// The pull request's run: the metrics of the baseline's `quick` section,
    /// against its margin; other results are left out.
    Quick,
    /// Write the results as the new baseline's metrics.
    Record,
}

/// Compares `results` with the baseline at `baseline` as `mode` says. To
/// record, it writes the results as the new baseline's metrics (keeping its
/// margin, floor, targets, and `quick` section) instead, unless a
/// [`REQUIRED`] metric is missing from them, which fails and leaves the file
/// as it was.
///
/// # Errors
///
/// A message when a file can't be read, parsed, or written, or when the
/// baseline has no `quick` section for [`Mode::Quick`].
pub fn compare(
    results: &Path,
    baseline: &Path,
    mode: Mode,
) -> std::result::Result<Comparison, String> {
    let mut results = read_results(results)?;
    let mut base: Baseline = serde_json::from_str(
        &std::fs::read_to_string(baseline).map_err(|e| format!("{}: {e}", baseline.display()))?,
    )
    .map_err(|e| format!("{}: {e}", baseline.display()))?;
    let (required, margin, floor_ms): (Vec<&str>, f64, f64) = if mode == Mode::Quick {
        let quick = base
            .quick
            .as_ref()
            .ok_or_else(|| format!("{}: no `quick` section", baseline.display()))?;
        results.retain(|name, _| quick.metrics.contains(name));
        (
            quick.metrics.iter().map(String::as_str).collect(),
            quick.margin,
            quick.floor_ms,
        )
    } else {
        (REQUIRED.to_vec(), base.margin, base.floor_ms)
    };
    let missing: Vec<String> = required
        .into_iter()
        .filter(|r| !results.contains_key(*r))
        .map(str::to_owned)
        .collect();
    if mode == Mode::Record {
        // A recording without a required metric would drop its baseline
        // unnoticed; keep the old file instead.
        if !missing.is_empty() {
            return Ok(Comparison {
                text: format!(
                    "not recorded: missing from the results: {}\n",
                    missing.join(", ")
                ),
                ok: false,
            });
        }
        base.machine = machine();
        base.metrics = results
            .values()
            .map(|r| (r.name.clone(), (r.median_ms * 100.0).round() / 100.0))
            .collect();
        let mut text = serde_json::to_string_pretty(&base).map_err(|e| e.to_string())?;
        text.push('\n');
        std::fs::write(baseline, text).map_err(|e| e.to_string())?;
        return Ok(Comparison {
            text: format!("recorded {} metrics\n", results.len()),
            ok: true,
        });
    }
    let mut ok = true;
    let mut failures = String::new();
    let mut text = format!(
        "baseline recorded on: {}\nfails over baseline x {margin} + {floor_ms} ms, or over a target\n\n",
        base.machine
    );
    text.push_str(&format!(
        "{:<44} {:>10} {:>10} {:>10} {:>10}  status\n",
        "metric", "median", "baseline", "limit", "target"
    ));
    for required in &missing {
        ok = false;
        text.push_str(&format!("{required:<44} missing from the results\n"));
    }
    for (name, r) in &results {
        let recorded = base.metrics.get(name).copied();
        let limit = recorded.map(|b| b * margin + floor_ms);
        let target = base.targets.get(name).copied();
        let mut status = "ok";
        if limit.is_some_and(|l| r.median_ms > l) {
            status = "REGRESSED";
            ok = false;
        }
        if target.is_some_and(|t| r.median_ms > t) {
            status = "MISSES TARGET";
            ok = false;
        }
        if recorded.is_none() {
            status = "no baseline";
        }
        let show = |v: Option<f64>| v.map_or_else(|| "-".to_owned(), |v| format!("{v:.1}"));
        if status == "REGRESSED" || status == "MISSES TARGET" {
            let unit = if name.starts_with("memory/") {
                "MB"
            } else {
                "ms"
            };
            failures.push_str(&format!(
                "{status}: {name} measured {:.1} {unit}, baseline {} {unit}, limit {} {unit}, target {}\n",
                r.median_ms,
                show(recorded),
                show(limit),
                target.map_or_else(|| "none".to_owned(), |t| format!("{t:.1} {unit}")),
            ));
        }
        text.push_str(&format!(
            "{name:<44} {:>10.1} {:>10} {:>10} {:>10}  {status}\n",
            r.median_ms,
            show(recorded),
            show(limit),
            show(target)
        ));
    }
    if !failures.is_empty() {
        text.push('\n');
        text.push_str(&failures);
    }
    Ok(Comparison { text, ok })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const BASELINE: &str = r#"{
        "machine": "test",
        "margin": 3.0,
        "floor_ms": 25.0,
        "targets": { "lsp/keystroke-page-3000": 50.0 },
        "metrics": { "check/synthetic-3000": 100.0, "lsp/keystroke-page-3000": 2.0 },
        "quick": {
            "margin": 4.0,
            "floor_ms": 50.0,
            "metrics": ["check/synthetic-3000", "lsp/keystroke-page-3000"]
        }
    }"#;

    fn run(results: &str, mode: Mode) -> Comparison {
        let dir = tempfile::tempdir().unwrap();
        let baseline = dir.path().join("perf.json");
        std::fs::write(&baseline, BASELINE).unwrap();
        let out = dir.path().join("results.jsonl");
        std::fs::write(&out, results).unwrap();
        compare(&out, &baseline, mode).unwrap()
    }

    fn line(name: &str, median_ms: f64) -> String {
        format!("{{\"name\":\"{name}\",\"median_ms\":{median_ms},\"p95_ms\":{median_ms}}}\n")
    }

    #[test]
    fn the_quick_run_has_its_own_margin_and_metrics() {
        // 400 ms is over the weekly limit (3 x 100 + 25) and under the quick
        // one (4 x 100 + 50); a result the quick run doesn't name is left out.
        let results = line("check/synthetic-3000", 400.0)
            + &line("lsp/keystroke-page-3000", 2.0)
            + &line("lsp/watched-unread-3000", 1.0);
        let quick = run(&results, Mode::Quick);
        assert!(quick.ok, "{}", quick.text);
        assert!(!quick.text.contains("watched-unread"), "{}", quick.text);
        assert!(!run(&results, Mode::Full).ok);
    }

    #[test]
    fn a_quick_failure_names_the_metric_its_baseline_and_what_was_measured() {
        let results = line("check/synthetic-3000", 451.0) + &line("lsp/keystroke-page-3000", 2.0);
        let quick = run(&results, Mode::Quick);
        assert!(!quick.ok);
        assert!(
            quick.text.contains(
                "REGRESSED: check/synthetic-3000 measured 451.0 ms, baseline 100.0 ms, limit 450.0 ms, target none"
            ),
            "{}",
            quick.text
        );
    }

    #[test]
    fn a_metric_the_quick_run_names_must_be_measured() {
        let quick = run(&line("check/synthetic-3000", 100.0), Mode::Quick);
        assert!(!quick.ok);
        assert!(
            quick.text.contains("lsp/keystroke-page-3000"),
            "{}",
            quick.text
        );
    }

    #[test]
    fn the_quick_section_names_metrics_with_baselines() {
        let base: Baseline =
            serde_json::from_str(&std::fs::read_to_string(default_baseline()).unwrap()).unwrap();
        let quick = base.quick.unwrap();
        assert!(!quick.metrics.is_empty());
        for metric in &quick.metrics {
            assert!(
                base.metrics.contains_key(metric),
                "{metric} has no baseline"
            );
        }
    }
}
