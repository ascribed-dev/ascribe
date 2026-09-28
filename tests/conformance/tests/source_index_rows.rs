//! The rows of SPEC §8.2 that the source index answers (phase 11), run before
//! phases 10 and 14 exist.
//!
//! The conformance runner skips a case while any of its area tags has no
//! adapter, and every case that expects one of these rows also carries `check`
//! or `page-check`. This test runs them now, for what the source index
//! reports: in every case whose expectations say anything about diagnostics,
//! the file-level rows about referenced files (`include-target-missing`,
//! `link-target-missing`, `image-source-missing`, `link-to-fragment`,
//! `link-route`) and the page-level rows that expansion and the source ids
//! decide (`include-cycle`, `include-id-missing`, `link-id-missing`,
//! `link-id-in-fragment`), compared with what the case expects, by slug, file,
//! and line. Everything else a case expects is left out of both sides.
//!
//! When phase 10 adds an adapter for `check` and phase 14 one for `page-check`,
//! the runner covers these cases whole; delete this test then (or fold it into
//! those adapters), so two runners don't drift.

#![allow(clippy::expect_used, clippy::panic)]

#[path = "adapters/include.rs"]
mod include;

use std::collections::BTreeSet;

use include::{FILE_LEVEL, PAGE_LEVEL, source_problems};
use tessera_conformance::{Case, CaseKind, ExpectedDiagnostic, Suite, discover};

type Row = (String, String, u32);

fn expected(case: &Case, list: &[ExpectedDiagnostic], slugs: &[&str]) -> Vec<Row> {
    let default_file = match case.kind {
        CaseKind::SingleFile => "input.md",
        CaseKind::Project => "",
    };
    let mut rows: Vec<Row> = list
        .iter()
        .filter(|d| slugs.contains(&d.slug.as_str()))
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
fn source_index_rows() {
    let suite = Suite::bundled();
    let discovered =
        discover(&suite.cases_dir(), &suite.shared_model_path()).expect("cases are discoverable");

    let mut problems = Vec::new();
    let mut covered: BTreeSet<String> = BTreeSet::new();
    let mut cases_run = 0;
    for d in discovered {
        let case = d.case.expect("case loads");
        // A case that tests the model itself has a model that may not load.
        if case.expect.area_tags().any(|t| t == "model") {
            continue;
        }
        let has_page_level = case.expect.builds.values().any(|b| b.diagnostics.is_some());
        if case.expect.diagnostics.is_none() && !has_page_level {
            continue;
        }
        cases_run += 1;
        let actual = source_problems(&case).unwrap_or_else(|e| panic!("{}: {e}", d.id));

        if let Some(list) = &case.expect.diagnostics {
            let want = expected(&case, list, FILE_LEVEL);
            covered.extend(want.iter().map(|r| r.0.clone()));
            let got: Vec<Row> = actual
                .file_level
                .iter()
                .map(|a| (a.slug.clone(), a.file.clone(), a.line))
                .collect();
            let mut got = got;
            got.sort();
            if want != got {
                problems.push(format!(
                    "{}: file-level rows: expected {want:?}, got {got:?}",
                    d.id
                ));
            }
        }

        // Page-level rows are per build; the ones the source index reports
        // don't depend on the build, so every build that lists diagnostics
        // must agree with the source, and a case with no builds says nothing.
        let mut got: Vec<Row> = actual
            .page_level
            .iter()
            .map(|a| (a.slug.clone(), a.file.clone(), a.line))
            .collect();
        got.sort();
        got.dedup();
        for (name, build) in &case.expect.builds {
            let Some(list) = &build.diagnostics else {
                continue;
            };
            let mut want = expected(&case, list, PAGE_LEVEL);
            want.dedup();
            covered.extend(want.iter().map(|r| r.0.clone()));
            if want != got {
                problems.push(format!(
                    "{} (build {name}): page-level rows: expected {want:?}, got {got:?}",
                    d.id
                ));
            }
        }
    }

    eprintln!("source index rows: {cases_run} cases, rows {covered:?}");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert!(cases_run >= 60, "only {cases_run} cases ran");
    for slug in FILE_LEVEL.iter().chain(PAGE_LEVEL) {
        assert!(
            covered.contains(*slug),
            "no case expects the source-index row `{slug}`"
        );
    }
}
