//! File-level diagnostics of every case tagged `check`, including the ones
//! the runner skips because they also carry a tag whose phase isn't built yet
//! (`include`, `resolve`, `slug`, `page-check`).
//!
//! The file-level diagnostics of such a case (its top-level `diagnostics`)
//! don't depend on those phases: `tessera_check::check_files` produces them
//! from the case's files alone. Running them here means every §8.2 row's file-level
//! case is checked now, not when the last of its tags gets an adapter. Cases
//! the runner runs are checked twice, which costs little.

#![allow(clippy::expect_used, clippy::panic)]
// This test uses one function of the adapters; the runner uses the rest.
#![allow(dead_code)]

mod adapters;

use tessera_conformance::{CaseKind, Suite, discover};

#[test]
fn file_level_diagnostics_of_check_cases() {
    let suite = Suite::bundled();
    let discovered =
        discover(&suite.cases_dir(), &suite.shared_model_path()).expect("cases are discoverable");

    let mut problems = Vec::new();
    let mut checked = 0;
    for d in discovered {
        let case = d.case.expect("case loads");
        if !case.expect.area_tags().any(|t| t == "check") {
            continue;
        }
        let Some(expected) = &case.expect.diagnostics else {
            continue;
        };
        checked += 1;
        let default_file = match case.kind {
            CaseKind::SingleFile => "input.md",
            CaseKind::Project => "",
        };
        let mut want: Vec<(String, String, u32, Option<u32>)> = expected
            .iter()
            .map(|e| {
                (
                    e.slug.clone(),
                    e.file.clone().unwrap_or_else(|| default_file.to_owned()),
                    e.line,
                    e.column,
                )
            })
            .collect();
        let actual = adapters::file_level_diagnostics(&case).expect("the project checks");
        let mut got: Vec<(String, String, u32, u32)> = actual
            .iter()
            .map(|a| (a.slug.clone(), a.file.clone(), a.line, a.column))
            .collect();
        want.sort();
        got.sort();
        // Columns are compared only where the case gives one.
        let got_cmp: Vec<_> = got
            .iter()
            .map(|(s, f, l, c)| {
                let column = want
                    .iter()
                    .find(|w| (&w.0, &w.1, w.2) == (s, f, *l))
                    .and_then(|w| w.3.map(|_| *c));
                (s.clone(), f.clone(), *l, column)
            })
            .collect();
        if want != got_cmp {
            problems.push(format!("{}: expected {want:?}, got {got:?}", d.id));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert!(checked >= 150, "only {checked} check cases ran");
}
