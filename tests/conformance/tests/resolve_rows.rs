//! What build resolution answers in the cases the runner can't run yet.
//!
//! The runner skips a case while any of its area tags has no adapter, and
//! every case that expects a page-level diagnostic carries `page-check`, whose
//! adapter is phase 14's. Several of them also carry `resolve` (a group with
//! no surviving arm, a link to an id or a page a build removes, the Quill
//! project), and a few, such as `directives/available/scope-*`, exercise only
//! the scope rule. So this test runs them now, through the `resolve` adapter:
//! for each build a case expects, the published pages, their resolved
//! outlines, the assets, and the page-level rows that resolution records
//! (`variant-no-arm-survives`, `available-exceeds-scope`, `link-id-removed`,
//! `link-page-dropped`, `include-cycle`, `include-id-missing`), by slug, file,
//! and line. Everything else a case expects is left out of both sides.
//!
//! When phase 14 adds an adapter for `page-check`, the runner covers these
//! cases whole; delete this test then.

#![allow(clippy::expect_used, clippy::panic)]
#![allow(dead_code)]

mod adapters;

use std::collections::BTreeSet;

use adapters::resolve::resolve_build;
use tessera_conformance::outline::compare;
use tessera_conformance::{Case, CaseKind, ExpectedDiagnostic, Suite, discover};

/// The rows resolution records, for phase 14 to report.
const ROWS: &[&str] = &[
    "variant-no-arm-survives",
    "available-exceeds-scope",
    "link-id-removed",
    "link-page-dropped",
    "include-cycle",
    "include-id-missing",
];

type Row = (String, String, u32);

fn rows(case: &Case, list: &[ExpectedDiagnostic]) -> Vec<Row> {
    let default_file = match case.kind {
        CaseKind::SingleFile => "input.md",
        CaseKind::Project => "",
    };
    let mut rows: Vec<Row> = list
        .iter()
        .filter(|d| ROWS.contains(&d.slug.as_str()))
        .map(|d| {
            (
                d.slug.clone(),
                d.file.clone().unwrap_or_else(|| default_file.to_owned()),
                d.line,
            )
        })
        .collect();
    rows.sort();
    rows
}

#[test]
fn resolution_of_cases_that_also_need_page_checks() {
    let suite = Suite::bundled();
    let discovered =
        discover(&suite.cases_dir(), &suite.shared_model_path()).expect("cases are discoverable");

    let mut problems = Vec::new();
    let mut covered: BTreeSet<String> = BTreeSet::new();
    let mut cases_run = 0;
    for d in discovered {
        let case = d.case.expect("case loads");
        let tags: Vec<&str> = case.expect.area_tags().collect();
        // Only cases the runner skips for `page-check`: the rest it runs.
        if !tags.contains(&"page-check") || tags.contains(&"model") || case.expect.builds.is_empty()
        {
            continue;
        }
        cases_run += 1;
        for (name, expect) in &case.expect.builds {
            let actual = resolve_build(&case, name)
                .unwrap_or_else(|e| panic!("{} (build {name}): {e}", d.id));
            let at = format!("{} (build {name})", d.id);
            if let Some(pages) = &expect.pages {
                let want: BTreeSet<&String> = pages.keys().collect();
                let got: BTreeSet<&String> = actual.pages.keys().collect();
                if want != got {
                    problems.push(format!("{at}: pages: expected {want:?}, got {got:?}"));
                }
                for (page, expected) in pages {
                    let Some(outline) = expected.as_ref().and_then(|e| e.outline.as_ref()) else {
                        continue;
                    };
                    let Some(result) = actual.pages.get(page) else {
                        continue;
                    };
                    let diffs = compare(outline, result.outline.as_ref().expect("an outline"));
                    problems.extend(
                        diffs
                            .into_iter()
                            .map(|diff| format!("{at}: {page}: {diff}")),
                    );
                }
            }
            if let Some(assets) = &expect.assets {
                let want: BTreeSet<&String> = assets.iter().collect();
                let got: BTreeSet<&String> = actual.assets.iter().collect();
                if want != got {
                    problems.push(format!("{at}: assets: expected {want:?}, got {got:?}"));
                }
            }
            if let Some(list) = &expect.diagnostics {
                let want = rows(&case, list);
                covered.extend(want.iter().map(|r| r.0.clone()));
                let mut got: Vec<Row> = actual
                    .diagnostics
                    .iter()
                    .filter(|a| ROWS.contains(&a.slug.as_str()))
                    .map(|a| (a.slug.clone(), a.file.clone(), a.line))
                    .collect();
                got.sort();
                if want != got {
                    problems.push(format!("{at}: rows: expected {want:?}, got {got:?}"));
                }
            }
        }
    }

    eprintln!("resolution rows: {cases_run} cases, rows {covered:?}");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert!(cases_run >= 30, "only {cases_run} cases ran");
    for slug in [
        "variant-no-arm-survives",
        "available-exceeds-scope",
        "link-id-removed",
        "link-page-dropped",
    ] {
        assert!(
            covered.contains(slug),
            "no case expects the resolution row `{slug}`"
        );
    }
}
