//! What `available-same-in-every-line`'s fix writes, on the conformance
//! cases `tests/conformance/cases/checks/available-inert-only-target`,
//! `available-inert-two-targets`, and `available-inert-nested`, which hold
//! where the check reports: a case can't hold a fix. A mark of the only
//! value of its dimension that would be its target alone, generally
//! available, is removed in each place it's written; with a second value in
//! the dimension, or inside a mark that says something, it's rewritten as
//! before.

#![allow(clippy::expect_used)]

use std::path::PathBuf;

use ascribe_check::{Project, check_files};
use ascribe_core::{Applicability, apply_edits};

/// Each file with the check's fixes applied, by its path, with each fix's
/// title. `safe` is what each fix's applicability is expected to be.
fn fixed(case: &str, safe: bool) -> Vec<(String, String, Vec<String>)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/conformance/cases/checks")
        .join(case);
    let project = Project::load(&root.join("ascribe.toml")).expect("the project loads");
    let diagnostics: Vec<_> = check_files(&project)
        .into_iter()
        .filter(|d| d.slug.as_str() == "available-same-in-every-line")
        .collect();
    let mut out = Vec::new();
    for source in project.sources() {
        let fixes: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.location.file == source.id)
            .map(|d| {
                assert_eq!(d.fixes.len(), 1, "{}", d.message);
                // A badge build shows the mark, so no fix there is safe.
                let expected = if safe {
                    Applicability::Safe
                } else {
                    Applicability::Unsafe
                };
                assert_eq!(d.fixes[0].applicability, expected);
                &d.fixes[0]
            })
            .collect();
        let edits: Vec<_> = fixes.iter().flat_map(|f| f.edits.clone()).collect();
        let text = apply_edits(&source.text, &edits).expect("the edits apply");
        let titles = fixes.iter().map(|f| f.title.clone()).collect();
        out.push((source.path.to_string(), text, titles));
    }
    out.sort();
    out
}

fn file<'a>(all: &'a [(String, String, Vec<String>)], path: &str) -> (&'a str, &'a [String]) {
    let (_, text, titles) = all
        .iter()
        .find(|(p, ..)| p == path)
        .expect("the file is in the case");
    (text, titles)
}

#[test]
fn a_mark_of_the_only_target_is_removed_in_each_place_it_is_written() {
    let all = fixed("available-inert-only-target", false);
    let (page, titles) = file(&all, "page.md");
    assert_eq!(
        page,
        "---\ntype: page\ntitle: Page\n---\n\nThe whole page is marked.\n"
    );
    assert_eq!(titles, ["Remove the mark"]);
    let (section, titles) = file(&all, "section.md");
    assert_eq!(
        section,
        "---\ntype: page\ntitle: Section\n---\n\n## Marked\n\nThe section is marked.\n\n\
         ## Marked in preview\n\n@available: ascribe preview\n\n\
         The state stays, so the mark does too.\n"
    );
    assert_eq!(titles, ["Remove the mark", "Write `ascribe preview`"]);
    let (table, titles) = file(&all, "table.md");
    assert_eq!(
        table,
        "---\ntype: page\ntitle: Table\n---\n\n| Option | Meaning |\n|---|---|\n\
         | `anchors` | The row is marked. |\n| `plain` | No mark. |\n"
    );
    assert_eq!(titles, ["Remove the mark"]);
}

#[test]
fn a_mark_of_one_of_two_values_is_rewritten_as_before() {
    let all = fixed("available-inert-two-targets", false);
    let (page, titles) = file(&all, "page.md");
    assert!(page.contains("\navailable: ascribe\n"), "{page}");
    assert_eq!(titles, ["Write `ascribe`"]);
    let (section, titles) = file(&all, "section.md");
    assert!(section.contains("\n@available: ascribe\n"), "{section}");
    assert!(
        section.contains("\n@available: ascribe preview\n"),
        "{section}"
    );
    assert_eq!(titles, ["Write `ascribe`", "Write `ascribe preview`"]);
    let (table, titles) = file(&all, "table.md");
    assert!(table.contains("{available=\"ascribe\"}"), "{table}");
    assert_eq!(titles, ["Write `ascribe`"]);
}

#[test]
fn a_mark_inside_one_that_says_something_is_rewritten() {
    let all = fixed("available-inert-nested", true);
    let (nest, titles) = file(&all, "nest.md");
    assert_eq!(
        nest,
        "---\ntype: page\ntitle: Nest\navailable: ascribe deprecated\n---\n\n\
         ## Went GA\n\n@available: ascribe\n\n| Option | Meaning |\n|---|---|\n\
         | `row` {available=\"ascribe\"} | GA row. |\n"
    );
    assert_eq!(
        titles,
        [
            "Write `ascribe deprecated`",
            "Write `ascribe`",
            "Write `ascribe`"
        ]
    );
    let (sections, titles) = file(&all, "sections.md");
    assert!(
        sections.contains("### Went GA\n\n@available: ascribe\n"),
        "{sections}"
    );
    assert!(
        sections.contains("## Plain\n\nNo mark around it says anything"),
        "{sections}"
    );
    assert!(sections.contains("{available=\"ascribe\"}"), "{sections}");
    assert_eq!(
        titles,
        [
            "Write `ascribe preview`",
            "Write `ascribe`",
            "Remove the mark",
            "Write `ascribe preview`",
            "Write `ascribe`"
        ]
    );
}
