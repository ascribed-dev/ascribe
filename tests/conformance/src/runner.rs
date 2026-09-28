//! Runs cases through adapters and reports every case as passed, failed, or skipped.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use crate::adapter::{AdapterResult, BuildResult, ConformanceAdapter, Diagnostic, Registry};
use crate::case::{Case, CaseError, CaseKind, discover};
use crate::expect::{BuildExpect, ExpectedDiagnostic, OutputKind, OutputSlot};
use crate::outline::{compare, outline_to_yaml};
use crate::registry::{DiagnosticsRegistry, Level, RegistryError};
use crate::skips::{Check, SkipTarget, Skips, SkipsError};

/// The conformance suite's directory layout.
#[derive(Debug, Clone)]
pub struct Suite {
    /// The suite's root, `tests/conformance/`.
    pub root: PathBuf,
}

/// A problem that stops the whole run.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// `SKIPS.toml` is invalid.
    #[error(transparent)]
    Skips(#[from] SkipsError),
    /// The cases directory couldn't be read.
    #[error(transparent)]
    Discovery(#[from] CaseError),
    /// `diagnostics.toml` couldn't be read.
    #[error(transparent)]
    Registry(#[from] RegistryError),
}

/// Which cases to run.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// Run only cases carrying at least one of these tags. Empty means all.
    pub tags: Vec<String>,
    /// Run only cases whose id contains this text.
    pub case: Option<String>,
}

impl Filter {
    fn selects(&self, id: &str, tags: &[String]) -> bool {
        let tag_ok = self.tags.is_empty() || tags.iter().any(|t| self.tags.contains(t));
        let case_ok = self.case.as_deref().is_none_or(|c| id.contains(c));
        tag_ok && case_ok
    }
}

/// How one case ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Every check that ran passed. Lists checks skipped by a partial skip
    /// entry, with their reasons.
    Passed {
        /// `(check, reason)` for each skipped check.
        skipped_checks: Vec<(Check, String)>,
    },
    /// The case was skipped, for these reasons from `SKIPS.toml`.
    Skipped {
        /// One line per skip entry that applied.
        reasons: Vec<String>,
    },
    /// The case failed, or couldn't be run.
    Failed {
        /// One line per problem.
        problems: Vec<String>,
    },
}

/// One case's result.
#[derive(Debug, Clone)]
pub struct CaseReport {
    /// The case id.
    pub id: String,
    /// The outcome.
    pub outcome: Outcome,
}

/// The result of a run.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// Every selected case, in path order.
    pub cases: Vec<CaseReport>,
    /// Problems with the suite itself, such as stale skip entries.
    pub errors: Vec<String>,
}

impl Suite {
    /// The suite in this crate's directory.
    pub fn bundled() -> Suite {
        Suite::new(env!("CARGO_MANIFEST_DIR"))
    }

    /// A suite rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Suite {
        Suite { root: root.into() }
    }

    /// `cases/`, where cases live.
    pub fn cases_dir(&self) -> PathBuf {
        self.root.join("cases")
    }

    /// `SKIPS.toml`.
    pub fn skips_path(&self) -> PathBuf {
        self.root.join("SKIPS.toml")
    }

    /// `diagnostics.toml`, the diagnostics registry.
    pub fn diagnostics_path(&self) -> PathBuf {
        self.root.join("diagnostics.toml")
    }

    /// `_model/tessera.toml`, the shared fixture model.
    pub fn shared_model_path(&self) -> PathBuf {
        self.root.join("_model").join("tessera.toml")
    }

    /// `snapshots/`, where `insta` output snapshots live.
    pub fn snapshots_dir(&self) -> PathBuf {
        self.root.join("snapshots")
    }

    /// Discovers the cases, applies the skips, and runs the rest.
    pub fn run(&self, registry: &Registry, filter: &Filter) -> Result<Report, RunError> {
        let skips = Skips::load(&self.skips_path())?;
        let registry_file = DiagnosticsRegistry::load(&self.diagnostics_path())?;
        let discovered = discover(&self.cases_dir(), &self.shared_model_path())?;
        let mut report = Report::default();

        // Skip entries must stay truthful: no full skip for a tag an adapter
        // handles, and no skip for a case that doesn't exist.
        for entry in &skips.entries {
            match &entry.target {
                SkipTarget::Tag(tag) if entry.is_full() && registry.handles_tag(tag) => {
                    report.errors.push(format!(
                        "SKIPS.toml: tag `{tag}` is skipped, but an adapter handles it; remove the entry"
                    ));
                }
                SkipTarget::Case(id) if !discovered.iter().any(|d| &d.id == id) => {
                    report
                        .errors
                        .push(format!("SKIPS.toml: case `{id}` doesn't exist"));
                }
                _ => {}
            }
        }

        for d in discovered {
            // A case that fails to load has no tags, so a tag filter drops it;
            // an unfiltered run always reports it.
            let tags = d
                .case
                .as_ref()
                .map(|c| c.expect.tags.as_slice())
                .unwrap_or(&[]);
            if !filter.selects(&d.id, tags) {
                continue;
            }
            let outcome = match d.case {
                Err(e) => Outcome::Failed {
                    problems: vec![format!("couldn't load case: {e}")],
                },
                Ok(case) => {
                    let problems = check_slugs(&case, &registry_file);
                    if problems.is_empty() {
                        self.run_case(&case, registry, &skips)
                    } else {
                        Outcome::Failed { problems }
                    }
                }
            };
            report.cases.push(CaseReport { id: d.id, outcome });
        }
        Ok(report)
    }

    fn run_case(&self, case: &Case, registry: &Registry, skips: &Skips) -> Outcome {
        let mut skipped_checks: Vec<(Check, String)> = Vec::new();
        let mut note_partial = |entry: &crate::skips::SkipEntry| {
            for check in entry.checks.iter().flatten() {
                skipped_checks.push((*check, format!("{}: {}", entry.target, entry.reason)));
            }
        };

        if let Some(entry) = skips.for_case(&case.id) {
            if entry.is_full() {
                return Outcome::Skipped {
                    reasons: vec![format!("{}: {}", entry.target, entry.reason)],
                };
            }
            note_partial(entry);
        }

        let tags: Vec<&str> = case.expect.area_tags().collect();
        let mut problems = Vec::new();
        let mut reasons = Vec::new();
        for tag in &tags {
            let skip = skips.for_tag(tag);
            match skip {
                Some(entry) if entry.is_full() => {
                    reasons.push(format!("{}: {}", entry.target, entry.reason));
                }
                _ if !registry.handles_tag(tag) => problems.push(format!(
                    "tag `{tag}` has no adapter and no skip entry in SKIPS.toml"
                )),
                Some(entry) => note_partial(entry),
                None => {}
            }
        }
        if !problems.is_empty() {
            return Outcome::Failed { problems };
        }
        if !reasons.is_empty() {
            return Outcome::Skipped { reasons };
        }

        let skipped: BTreeSet<Check> = skipped_checks.iter().map(|(c, _)| *c).collect();
        let adapters: Vec<&dyn ConformanceAdapter> = registry.for_tags(&tags).collect();
        let e = &case.expect;

        if let Some(expected) = &e.outline
            && !skipped.contains(&Check::Outline)
        {
            match first(&adapters, "outline", |a| a.outline(case)) {
                Err(p) => problems.push(p),
                Ok(actual) => {
                    let diffs = compare(expected, &actual);
                    if !diffs.is_empty() {
                        problems.extend(diffs);
                        problems.push(format!("actual outline:\n{}", outline_to_yaml(&actual)));
                    }
                }
            }
        }

        if let Some(expected) = &e.diagnostics
            && !skipped.contains(&Check::Diagnostics)
        {
            match first(&adapters, "diagnostics", |a| a.diagnostics(case)) {
                Err(p) => problems.push(p),
                Ok(actual) => problems.extend(compare_diagnostics(case, "", expected, &actual)),
            }
        }

        if let Some(file) = &e.formatted
            && !skipped.contains(&Check::Format)
        {
            problems.extend(check_format(case, file, &adapters));
        }

        if !skipped.contains(&Check::Builds) {
            for (name, expected) in &e.builds {
                match first(&adapters, &format!("build `{name}`"), |a| {
                    a.build(case, name)
                }) {
                    Err(p) => problems.push(p),
                    Ok(actual) => {
                        problems.extend(self.compare_build(case, name, expected, &actual));
                    }
                }
            }
        }

        if problems.is_empty() {
            Outcome::Passed { skipped_checks }
        } else {
            Outcome::Failed { problems }
        }
    }

    fn compare_build(
        &self,
        case: &Case,
        name: &str,
        expected: &BuildExpect,
        actual: &BuildResult,
    ) -> Vec<String> {
        let mut problems = Vec::new();
        let at = format!("builds.{name}");
        if let Some(pages) = &expected.pages {
            let want: BTreeSet<&String> = pages.keys().collect();
            let got: BTreeSet<&String> = actual.pages.keys().collect();
            for missing in want.difference(&got) {
                problems.push(format!("{at}.pages: page `{missing}` wasn't published"));
            }
            for extra in got.difference(&want) {
                problems.push(format!("{at}.pages: unexpected page `{extra}`"));
            }
            for (page, expect) in pages {
                let (Some(expect), Some(result)) = (expect, actual.pages.get(page)) else {
                    continue;
                };
                let pat = format!("{at}.pages.{page}");
                if let Some(outline) = &expect.outline {
                    match &result.outline {
                        None => problems.push(format!("{pat}.outline: the adapter produced none")),
                        Some(actual) => {
                            let diffs = compare(outline, actual);
                            if !diffs.is_empty() {
                                problems.extend(diffs.into_iter().map(|d| format!("{pat}: {d}")));
                                problems.push(format!(
                                    "{pat}: actual outline:\n{}",
                                    outline_to_yaml(actual)
                                ));
                            }
                        }
                    }
                }
                for (kind, slot) in &expect.outputs {
                    let Some(output) = result.outputs.get(kind) else {
                        problems.push(format!("{pat}.outputs.{kind}: the adapter produced none"));
                        continue;
                    };
                    if let Err(p) = self.check_output(case, name, page, *kind, slot, output) {
                        problems.push(format!("{pat}.outputs.{kind}: {p}"));
                    }
                }
            }
        }
        if let Some(assets) = &expected.assets {
            let want: BTreeSet<&String> = assets.iter().collect();
            let got: BTreeSet<&String> = actual.assets.iter().collect();
            if want != got {
                problems.push(format!("{at}.assets: expected {want:?}, got {got:?}"));
            }
        }
        if let Some(diags) = &expected.diagnostics {
            problems.extend(compare_diagnostics(
                case,
                &format!("{at}."),
                diags,
                &actual.diagnostics,
            ));
        }
        problems
    }

    fn check_output(
        &self,
        case: &Case,
        build: &str,
        page: &str,
        kind: OutputKind,
        slot: &OutputSlot,
        output: &str,
    ) -> Result<(), String> {
        match slot {
            OutputSlot::File { file } => {
                let path = case.dir.join(file);
                let expected = std::fs::read_to_string(&path)
                    .map_err(|e| format!("couldn't read {}: {e}", path.display()))?;
                if expected == output {
                    Ok(())
                } else {
                    Err(format!(
                        "differs from {file}\n--- expected\n{expected}\n--- actual\n{output}"
                    ))
                }
            }
            OutputSlot::Snapshot(_) => {
                let name = snapshot_name(&case.id, build, page, kind);
                let mut settings = insta::Settings::clone_current();
                settings.set_snapshot_path(self.snapshots_dir());
                settings.set_prepend_module_to_snapshot(false);
                settings.set_input_file(case.dir.join("expect.yaml"));
                let result = catch_unwind(AssertUnwindSafe(|| {
                    settings.bind(|| insta::assert_snapshot!(name.as_str(), output));
                }));
                result.map_err(|_| format!("snapshot `{name}` doesn't match (see above)"))
            }
        }
    }
}

/// Checks the slugs a case expects against the registry, whether or not the
/// case runs: each must be registered, at the level where the case expects
/// it, and a provisional diagnostic needs a provisional case that lists its
/// questions.
fn check_slugs(case: &Case, registry: &DiagnosticsRegistry) -> Vec<String> {
    let e = &case.expect;
    let file_level = e
        .diagnostics
        .iter()
        .flatten()
        .map(|d| ("diagnostics", Level::File, d));
    let page_level = e.builds.iter().flat_map(|(name, b)| {
        b.diagnostics
            .iter()
            .flatten()
            .map(move |d| (name.as_str(), Level::Page, d))
    });
    let mut problems = Vec::new();
    for (place, level, d) in file_level.chain(page_level) {
        let at = if level == Level::File {
            place.to_owned()
        } else {
            format!("builds.{place}.diagnostics")
        };
        let Some(entry) = registry.get(&d.slug) else {
            problems.push(format!(
                "{at}: `{}` isn't a slug in diagnostics.toml",
                d.slug
            ));
            continue;
        };
        if entry.level != level {
            let hint = match entry.level {
                Level::File => "expect it in the top-level `diagnostics`",
                Level::Page => "expect it under `builds.<name>.diagnostics`",
            };
            problems.push(format!(
                "{at}: `{}` is a {}-level diagnostic; {hint}",
                d.slug, entry.level
            ));
        }
        if !entry.provisional.is_empty() {
            let missing: Vec<&String> = entry
                .provisional
                .iter()
                .filter(|q| !e.questions.contains(q))
                .collect();
            if !e.is_provisional() || !missing.is_empty() {
                problems.push(format!(
                    "{at}: `{}` is provisional ({}); tag the case `provisional` and list those questions",
                    d.slug,
                    entry.provisional.join(", ")
                ));
            }
        }
    }
    problems
}

/// The `insta` snapshot name for an output: the case id, build, page, and
/// emitter, with path separators and dots replaced.
pub fn snapshot_name(case: &str, build: &str, page: &str, kind: OutputKind) -> String {
    let clean = |s: &str| s.replace(['/', '.'], "_");
    format!("{}__{}__{}__{kind}", clean(case), clean(build), clean(page))
}

/// Formats `input.md` through the adapters, and compares the result with the
/// expected file. The result formatted again must not change: canonical form
/// is a fixed point.
fn check_format(case: &Case, file: &str, adapters: &[&dyn ConformanceAdapter]) -> Vec<String> {
    let input = match case.input() {
        Ok(Some(input)) => input,
        Ok(None) => return vec!["formatted: the case has no input.md".into()],
        Err(e) => return vec![format!("formatted: couldn't read input.md: {e}")],
    };
    let path = case.dir.join(file);
    let expected = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => return vec![format!("formatted: couldn't read {}: {e}", path.display())],
    };
    let actual = match first(adapters, "format", |a| a.format(case, &input)) {
        Ok(actual) => actual,
        Err(p) => return vec![p],
    };
    let mut problems = Vec::new();
    if actual != expected {
        problems.push(format!(
            "formatted: differs from {file}\n--- expected\n{expected}\n--- actual\n{actual}"
        ));
    }
    match first(adapters, "format (the result again)", |a| {
        a.format(case, &actual)
    }) {
        Ok(again) if again != actual => problems.push(format!(
            "formatted: formatting the result again changes it\n--- once\n{actual}\n--- twice\n{again}"
        )),
        Ok(_) => {}
        Err(p) => problems.push(p),
    }
    problems
}

/// Asks each adapter in turn, returning the first result produced.
fn first<T>(
    adapters: &[&dyn ConformanceAdapter],
    what: &str,
    mut call: impl FnMut(&dyn ConformanceAdapter) -> AdapterResult<T>,
) -> Result<T, String> {
    for adapter in adapters {
        let result = catch_unwind(AssertUnwindSafe(|| call(*adapter)));
        match result {
            Err(panic) => {
                let msg = panic
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| panic.downcast_ref::<&str>().copied())
                    .unwrap_or("unknown panic");
                return Err(format!(
                    "adapter `{}` panicked on {what}: {msg}",
                    adapter.name()
                ));
            }
            Ok(Err(e)) => {
                return Err(format!(
                    "adapter `{}` failed on {what}: {e}",
                    adapter.name()
                ));
            }
            Ok(Ok(Some(value))) => return Ok(value),
            Ok(Ok(None)) => {}
        }
    }
    Err(format!(
        "the case expects {what}, but no adapter for its tags produces it; add a skip entry with `checks`, or implement it"
    ))
}

fn compare_diagnostics(
    case: &Case,
    prefix: &str,
    expected: &[ExpectedDiagnostic],
    actual: &[Diagnostic],
) -> Vec<String> {
    let default_file = match case.kind {
        CaseKind::SingleFile => "input.md",
        CaseKind::Project => "",
    };
    let mut used = vec![false; actual.len()];
    let mut missing = Vec::new();
    // Match expectations that pin a column first, so a looser expectation
    // can't take the diagnostic a stricter one needs.
    let mut order: Vec<&ExpectedDiagnostic> = expected.iter().collect();
    order.sort_by_key(|d| d.column.is_none());
    for exp in order {
        let file = exp.file.as_deref().unwrap_or(default_file);
        let found = actual.iter().enumerate().position(|(i, a)| {
            !used[i]
                && a.slug == exp.slug
                && a.file == file
                && a.line == exp.line
                && exp.column.is_none_or(|c| c == a.column)
        });
        match found {
            Some(i) => used[i] = true,
            None => missing.push(exp),
        }
    }
    let mut problems: Vec<String> = missing
        .into_iter()
        .map(|d| format!("{prefix}diagnostics: missing {d}"))
        .collect();
    for (a, used) in actual.iter().zip(used) {
        if !used {
            problems.push(format!("{prefix}diagnostics: unexpected {a}"));
        }
    }
    problems
}

impl Report {
    /// The number of passed cases.
    pub fn passed(&self) -> usize {
        self.count(|o| matches!(o, Outcome::Passed { .. }))
    }

    /// The number of skipped cases.
    pub fn skipped(&self) -> usize {
        self.count(|o| matches!(o, Outcome::Skipped { .. }))
    }

    /// The number of failed cases.
    pub fn failed(&self) -> usize {
        self.count(|o| matches!(o, Outcome::Failed { .. }))
    }

    fn count(&self, f: impl Fn(&Outcome) -> bool) -> usize {
        self.cases.iter().filter(|c| f(&c.outcome)).count()
    }

    /// Whether the run succeeded: no failed cases and no suite errors.
    pub fn success(&self) -> bool {
        self.failed() == 0 && self.errors.is_empty()
    }

    /// The outcome of a case, by id.
    pub fn outcome(&self, id: &str) -> Option<&Outcome> {
        self.cases.iter().find(|c| c.id == id).map(|c| &c.outcome)
    }

    /// A readable summary: every failure with its problems, every skip with
    /// its reason, every partially skipped check, and the totals.
    pub fn summary(&self) -> String {
        let mut out = String::new();
        for c in &self.cases {
            if let Outcome::Failed { problems } = &c.outcome {
                let _ = writeln!(out, "FAILED  {}", c.id);
                for p in problems {
                    for (i, line) in p.lines().enumerate() {
                        let indent = if i == 0 { "  - " } else { "    " };
                        let _ = writeln!(out, "{indent}{line}");
                    }
                }
            }
        }
        for c in &self.cases {
            match &c.outcome {
                Outcome::Skipped { reasons } => {
                    let _ = writeln!(out, "skipped {}", c.id);
                    for r in reasons {
                        let _ = writeln!(out, "  - {r}");
                    }
                }
                Outcome::Passed { skipped_checks } if !skipped_checks.is_empty() => {
                    let _ = writeln!(out, "passed  {} (some checks skipped)", c.id);
                    for (check, r) in skipped_checks {
                        let _ = writeln!(out, "  - {check} skipped by {r}");
                    }
                }
                _ => {}
            }
        }
        for e in &self.errors {
            let _ = writeln!(out, "ERROR   {e}");
        }
        let _ = writeln!(
            out,
            "conformance: {} passed, {} failed, {} skipped, {} suite error(s)",
            self.passed(),
            self.failed(),
            self.skipped(),
            self.errors.len()
        );
        out
    }
}

/// Parses runner arguments: `--tag <tag>` (repeatable) and a positional case
/// filter. Other flags, such as those `cargo test` passes, are ignored.
pub fn filter_from_args(args: impl IntoIterator<Item = String>) -> Filter {
    let mut filter = Filter::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--tag" {
            if let Some(tag) = args.next() {
                filter.tags.push(tag);
            }
        } else if let Some(tag) = arg.strip_prefix("--tag=") {
            filter.tags.push(tag.to_owned());
        } else if !arg.starts_with('-') && filter.case.is_none() {
            filter.case = Some(arg);
        }
    }
    filter
}
