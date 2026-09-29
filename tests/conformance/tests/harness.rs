//! Tests of the harness itself: discovery, skips, routing, and comparison,
//! using the fixture suite in `tests/fixtures/selftest/` and a fake adapter.

// Test helpers outside `#[test]` functions may unwrap and panic too.
#![allow(clippy::unwrap_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tessera_conformance::{
    AdapterResult, BuildResult, Case, ConformanceAdapter, Diagnostic, Directive, Filter, Form,
    Node, Outcome, OutputKind, PageResult, Registry, Suite,
};

/// A toy "parser": blank-line-separated chunks; lines starting with `@` are
/// directives, other lines are paragraph text. Every line containing `BAD` gets
/// a `bad-line` diagnostic.
struct FakeAdapter {
    tags: Vec<&'static str>,
}

fn fake_outline(text: &str) -> Vec<Node> {
    let mut out = Vec::new();
    for chunk in text.split("\n\n") {
        let mut para: Vec<&str> = Vec::new();
        for line in chunk.lines() {
            if let Some(rest) = line.strip_prefix('@') {
                if !para.is_empty() {
                    out.push(Node::Paragraph {
                        text: Some(para.join("\n")),
                    });
                    para.clear();
                }
                out.push(Node::Directive(Directive {
                    name: rest.split_whitespace().next().unwrap_or("").to_owned(),
                    form: Form::Line,
                    attributes: BTreeMap::new(),
                    primary: None,
                    title: None,
                    binding: None,
                    children: Vec::new(),
                }));
            } else if !line.trim().is_empty() {
                para.push(line);
            }
        }
        if !para.is_empty() {
            out.push(Node::Paragraph {
                text: Some(para.join("\n")),
            });
        }
    }
    out
}

fn fake_diagnostics(file: &str, text: &str) -> Vec<Diagnostic> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            line.find("BAD").map(|byte| Diagnostic {
                slug: "bad-line".into(),
                file: file.into(),
                line: i as u32 + 1,
                column: line[..byte].chars().count() as u32 + 1,
            })
        })
        .collect()
}

impl ConformanceAdapter for FakeAdapter {
    fn name(&self) -> &str {
        "fake"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag)
    }

    fn outline(&self, case: &Case) -> AdapterResult<Vec<Node>> {
        Ok(case.input().unwrap().map(|text| fake_outline(&text)))
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        let root = case.content_root();
        let mut out = Vec::new();
        for file in case.source_files().unwrap() {
            let text = std::fs::read_to_string(root.join(&file)).unwrap();
            out.extend(fake_diagnostics(&file, &text));
        }
        Ok(Some(out))
    }

    /// A toy formatter: trailing spaces go, and `@grow` lines gain a `!` each
    /// time (so formatting them is never a fixed point).
    fn format(&self, _case: &Case, source: &str) -> AdapterResult<String> {
        let lines: Vec<String> = source
            .lines()
            .map(|l| {
                let l = l.trim_end();
                if l.starts_with("@grow") {
                    format!("{l}!")
                } else {
                    l.to_owned()
                }
            })
            .collect();
        Ok(Some(lines.join("\n") + "\n"))
    }

    fn build(&self, case: &Case, _build: &str) -> AdapterResult<BuildResult> {
        let root = case.content_root();
        let mut result = BuildResult::default();
        for file in case.source_files().unwrap() {
            if !file.ends_with(".md") {
                result.assets.push(file);
                continue;
            }
            if file.starts_with('_') {
                continue;
            }
            let text = std::fs::read_to_string(root.join(&file)).unwrap();
            let mut outputs = BTreeMap::new();
            outputs.insert(OutputKind::Plain, text.to_uppercase());
            outputs.insert(OutputKind::Site, format!("<site>\n{text}"));
            result.pages.insert(
                file,
                PageResult {
                    outline: Some(fake_outline(&text)),
                    outputs,
                },
            );
        }
        Ok(Some(result))
    }
}

fn selftest() -> Suite {
    Suite::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/selftest"))
}

fn fake_registry(tags: Vec<&'static str>) -> Registry {
    let mut registry = Registry::new();
    registry.register(FakeAdapter { tags });
    registry
}

fn problems(outcome: Option<&Outcome>) -> Vec<String> {
    match outcome {
        Some(Outcome::Failed { problems }) => problems.clone(),
        other => panic!("expected a failure, got {other:?}"),
    }
}

#[test]
fn bundled_cases_are_discovered_and_skipped_with_recorded_reasons() {
    // Every phase that lands removes its tag's skip entry, so the bundled
    // cases stop being skipped with an empty registry. This runs one case
    // with stand-in adapters for the tags whose phases have landed, so it's
    // skipped for the one that hasn't. `projects/quill` carries `check`,
    // `include`, and `resolve` (phases 10, 11, and 12 handle them) and
    // `page-check` (phase 14): when phase 14 lands, pick a case that is
    // skipped for a tag that's still unimplemented (`output`, phases 18 and 20).
    let filter = Filter {
        case: Some("projects/quill".into()),
        ..Filter::default()
    };
    let report = Suite::bundled()
        .run(&fake_registry(vec!["check", "include", "resolve"]), &filter)
        .unwrap();
    println!("{}", report.summary());
    assert!(report.success(), "{}", report.summary());

    for id in ["projects/quill"] {
        match report.outcome(id) {
            Some(Outcome::Skipped { reasons }) => {
                assert!(!reasons.is_empty());
                assert!(
                    reasons.iter().all(|r| r.contains("not yet implemented")),
                    "{reasons:?}"
                );
            }
            other => panic!("{id}: expected skipped, got {other:?}"),
        }
    }
    let summary = report.summary();
    assert!(summary.contains("skipped projects/quill"));
    assert!(summary.contains("tag `page-check`: Adapter for tag `page-check`"));
}

#[test]
fn readme_worked_example_matches_the_sample_case() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(dir.join("README.md")).unwrap();
    let section = readme.split("## Worked example").nth(1).unwrap();
    let block = section
        .split("```yaml\n")
        .nth(1)
        .and_then(|rest| rest.split("\n```").next())
        .unwrap();
    let expect = std::fs::read_to_string(dir.join("cases/samples/appendix-b/expect.yaml")).unwrap();
    // The file starts with a comment the README leaves out.
    let body: String = expect
        .lines()
        .skip_while(|l| l.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(block.trim(), body.trim());
}

#[test]
fn every_outcome_in_the_fixture_suite() {
    let report = selftest()
        .run(&fake_registry(vec!["fake"]), &Filter::default())
        .unwrap();
    let summary = report.summary();
    println!("{summary}");

    let ids: Vec<&str> = report.cases.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "broken/no-expect",
            "broken/unknown-key",
            "fake/format-mismatch",
            "fake/format-not-idempotent",
            "fake/format-pass",
            "fake/mismatch",
            "fake/partial",
            "fake/pass",
            "fake/project",
            "fake/provisional",
            "skipped/whole",
            "slugs/provisional-untagged",
            "slugs/unknown",
            "slugs/wrong-level",
            "unhandled/mixed",
            "unhandled/no-adapter",
        ],
        "`_ignored` must not be discovered"
    );

    // Skipped with the reason recorded in SKIPS.toml.
    assert_eq!(
        report.outcome("skipped/whole"),
        Some(&Outcome::Skipped {
            reasons: vec!["tag `skipped-area`: Recorded reason for skipped-area.".into()]
        })
    );

    // A tag with neither an adapter nor a skip entry fails the case, even when
    // another of its tags is skipped.
    let msg = "tag `nobody` has no adapter and no skip entry in SKIPS.toml";
    assert_eq!(problems(report.outcome("unhandled/no-adapter")), vec![msg]);
    assert_eq!(problems(report.outcome("unhandled/mixed")), vec![msg]);

    // Passing cases, including a project case with outputs and a provisional case.
    for id in [
        "fake/pass",
        "fake/project",
        "fake/provisional",
        "fake/format-pass",
    ] {
        assert_eq!(
            report.outcome(id),
            Some(&Outcome::Passed {
                skipped_checks: vec![]
            }),
            "{id}\n{summary}"
        );
    }

    // A partial skip runs the other checks and records the skipped one.
    match report.outcome("fake/partial") {
        Some(Outcome::Passed { skipped_checks }) => assert_eq!(skipped_checks.len(), 1),
        other => panic!("fake/partial: {other:?}"),
    }
    assert!(summary.contains("diagnostics skipped by case `fake/partial`"));

    // Mismatches are reported with paths.
    let p = problems(report.outcome("fake/mismatch"));
    assert_eq!(p[0], "outline[0]: expected heading 1, got paragraph");
    assert!(p[1].starts_with("actual outline:"));
    assert_eq!(p[2], "diagnostics: missing bad-line at input.md:1");

    // Formatting: the result must equal the expected file, and formatting it
    // again must change nothing.
    let p = problems(report.outcome("fake/format-mismatch"));
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(p[0].starts_with("formatted: differs from formatted.md\n--- expected\na   \n"));
    assert!(p[0].ends_with("--- actual\na\n"), "{}", p[0]);
    let p = problems(report.outcome("fake/format-not-idempotent"));
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(
        p[0].starts_with("formatted: formatting the result again changes it\n--- once\n@grow!\n")
    );
    assert!(p[0].ends_with("--- twice\n@grow!!\n"), "{}", p[0]);

    // Broken cases fail with a reason.
    assert!(problems(report.outcome("broken/no-expect"))[0].contains("no expect.yaml"));
    assert!(problems(report.outcome("broken/unknown-key"))[0].contains("unknown field `outlines`"));

    // Expected slugs are checked against diagnostics.toml before a case runs.
    assert_eq!(
        problems(report.outcome("slugs/unknown")),
        vec!["diagnostics: `no-such-slug` isn't a slug in diagnostics.toml"]
    );
    assert_eq!(
        problems(report.outcome("slugs/wrong-level")),
        vec![
            "diagnostics: `bad-page` is a page-level diagnostic; expect it under `builds.<name>.diagnostics`",
            "builds.site.diagnostics: `bad-line` is a file-level diagnostic; expect it in the top-level `diagnostics`",
        ]
    );
    assert_eq!(
        problems(report.outcome("slugs/provisional-untagged")),
        vec![
            "diagnostics: `maybe-bad` is provisional (Q1); tag the case `provisional` and list those questions"
        ]
    );

    assert_eq!(
        (report.passed(), report.failed(), report.skipped()),
        (5, 10, 1)
    );
    assert!(!report.success());
    assert!(summary.contains("conformance: 5 passed, 10 failed, 1 skipped, 0 suite error(s)"));
}

#[test]
fn a_full_skip_for_a_handled_tag_is_a_suite_error() {
    let report = selftest()
        .run(
            &fake_registry(vec!["fake", "skipped-area"]),
            &Filter::default(),
        )
        .unwrap();
    assert_eq!(
        report.errors,
        vec![
            "SKIPS.toml: tag `skipped-area` is skipped, but an adapter handles it; remove the entry"
        ]
    );
}

#[test]
fn filters_select_by_tag_and_id() {
    let registry = fake_registry(vec!["fake"]);
    let by_tag = selftest()
        .run(
            &registry,
            &Filter {
                tags: vec!["skipped-area".into()],
                case: None,
            },
        )
        .unwrap();
    let ids: Vec<&str> = by_tag.cases.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["skipped/whole", "unhandled/mixed"]);

    let by_id = selftest()
        .run(
            &registry,
            &Filter {
                tags: vec![],
                case: Some("fake/pa".into()),
            },
        )
        .unwrap();
    let ids: Vec<&str> = by_id.cases.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["fake/partial", "fake/pass"]);
}
