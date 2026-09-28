//! The structural rows of SPEC §8.2 (phase 06), run before phase 10 exists.
//!
//! The conformance runner skips a case while any of its area tags has no
//! adapter, so the cases that expect structural diagnostics (tagged `check`)
//! and the `structure` cases that also expect diagnostics wait for phase 10.
//! This test runs them now, for what the structure pass reports: the outline
//! of every case tagged `structure`, and, for every case tagged only with
//! `parser`, `structure`, and `check`, the structural diagnostics, compared
//! with the expectation's structural diagnostics. Diagnostics that need the
//! content model (unknown attribute keys, value types) are phase 10's and are
//! left out of both sides.

#![allow(clippy::expect_used, clippy::panic)]

mod adapters;

use std::collections::BTreeSet;

use tessera_conformance::outline::{compare, outline_to_yaml};
use tessera_conformance::{CaseKind, Diagnostic, Registry, Suite, discover};

/// The slugs of the structural rows of SPEC §8.2: containers, end lines,
/// nesting, binding, titles, groups, `@steps` and `@details`, and lists.
const STRUCTURAL: &[&str] = &[
    "container-unclosed",
    "container-colon-unexpected",
    "container-colon-missing",
    "container-open-at-arm",
    "end-unmatched",
    "end-indent-mismatch",
    "container-nesting-deep",
    "binding-no-block",
    "binding-heading",
    "binding-blank-line",
    "binding-not-section-top",
    "title-not-accepted",
    "title-dot-space",
    "variant-mixed-arms",
    "variant-arm-kind",
    "variant-no-shared-dimension",
    "steps-not-ordered-list",
    "details-title-missing",
    "widget-schema",
    "list-ended-by-directive",
    "directive-indented-code",
    "steps-numbering-continued",
];

/// `widget-schema` is also phase 10's for attributes; only the missing-title
/// case is the structure pass's.
fn is_structural(slug: &str, case_id: &str) -> bool {
    STRUCTURAL.contains(&slug)
        && (slug != "widget-schema" || case_id == "widgets/title-required-missing")
}

fn key(d: &Diagnostic) -> (String, String, u32) {
    (d.slug.clone(), d.file.clone(), d.line)
}

#[test]
fn structure_outlines_and_structural_diagnostics() {
    let mut registry = Registry::new();
    adapters::register(&mut registry);
    let suite = Suite::bundled();
    let discovered =
        discover(&suite.cases_dir(), &suite.shared_model_path()).expect("cases are discoverable");

    let mut problems = Vec::new();
    let mut outlines = 0;
    let mut covered: BTreeSet<String> = BTreeSet::new();
    let mut diagnostic_cases = 0;
    for d in discovered {
        let case = d.case.expect("case loads");
        let tags: Vec<&str> = case.expect.area_tags().collect();
        let mine = tags
            .iter()
            .all(|t| matches!(*t, "parser" | "structure" | "check"));
        if !mine || !tags.contains(&"structure") && case.expect.diagnostics.is_none() {
            continue;
        }
        let adapters: Vec<_> = registry.for_tags(&["parser", "structure"]).collect();
        let adapter = adapters
            .first()
            .expect("the syntax adapter handles the tags");

        // Outlines: every `structure` case.
        if tags.contains(&"structure")
            && let Some(expected) = &case.expect.outline
        {
            outlines += 1;
            let actual = adapter
                .outline(&case)
                .expect("outline works")
                .expect("the adapter gives an outline");
            let diffs = compare(expected, &actual);
            if !diffs.is_empty() {
                problems.push(format!(
                    "{}: {}\nactual outline:\n{}",
                    d.id,
                    diffs.join("\n"),
                    outline_to_yaml(&actual)
                ));
            }
        }

        // Diagnostics: the structural ones, at the expected lines.
        if let Some(expected) = &case.expect.diagnostics {
            diagnostic_cases += 1;
            let default_file = match case.kind {
                CaseKind::SingleFile => "input.md",
                CaseKind::Project => "",
            };
            let mut want: Vec<(String, String, u32)> = expected
                .iter()
                .filter(|e| is_structural(&e.slug, &d.id))
                .map(|e| {
                    covered.insert(e.slug.clone());
                    (
                        e.slug.clone(),
                        e.file.clone().unwrap_or_else(|| default_file.to_owned()),
                        e.line,
                    )
                })
                .collect();
            let actual = adapter
                .diagnostics(&case)
                .expect("diagnostics work")
                .expect("the adapter gives diagnostics");
            let mut got: Vec<_> = actual
                .iter()
                .filter(|a| is_structural(&a.slug, &d.id))
                .map(key)
                .collect();
            want.sort();
            got.sort();
            if want != got {
                problems.push(format!(
                    "{}: expected structural diagnostics {want:?}, got {got:?}",
                    d.id
                ));
            }
        }
    }

    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
    assert!(outlines >= 50, "only {outlines} structure outlines ran");
    assert!(
        diagnostic_cases >= 50,
        "only {diagnostic_cases} diagnostic cases ran"
    );
    for slug in STRUCTURAL {
        assert!(
            covered.contains(*slug),
            "no case expects the structural diagnostic `{slug}`"
        );
    }
}
