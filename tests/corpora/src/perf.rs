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
    "lsp/completion-3000",
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

/// Compares `results` with the baseline at `baseline`; with `record`, writes
/// the results as the new baseline's metrics (keeping its margin, floor, and
/// targets) instead, unless a [`REQUIRED`] metric is missing from them, which
/// fails and leaves the file as it was.
///
/// # Errors
///
/// A message when a file can't be read, parsed, or written.
pub fn compare(
    results: &Path,
    baseline: &Path,
    record: bool,
) -> std::result::Result<Comparison, String> {
    let results = read_results(results)?;
    let mut base: Baseline = serde_json::from_str(
        &std::fs::read_to_string(baseline).map_err(|e| format!("{}: {e}", baseline.display()))?,
    )
    .map_err(|e| format!("{}: {e}", baseline.display()))?;
    let missing: Vec<&str> = REQUIRED
        .iter()
        .copied()
        .filter(|r| !results.contains_key(*r))
        .collect();
    if record {
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
    let mut text = format!(
        "baseline recorded on: {}\nfails over baseline x {} + {} ms, or over a target\n\n",
        base.machine, base.margin, base.floor_ms
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
        let limit = recorded.map(|b| b * base.margin + base.floor_ms);
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
        text.push_str(&format!(
            "{name:<44} {:>10.1} {:>10} {:>10} {:>10}  {status}\n",
            r.median_ms,
            show(recorded),
            show(limit),
            show(target)
        ));
    }
    Ok(Comparison { text, ok })
}
