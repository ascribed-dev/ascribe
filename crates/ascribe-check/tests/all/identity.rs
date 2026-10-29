//! A finding's identity (`ascribe_check::identity`) on a project's
//! findings: two alike in one file, and one in two builds.

#![allow(clippy::expect_used, clippy::panic)]

use ascribe_check::identity::{Identity, identities};
use ascribe_check::{Project, diagnose};
use ascribe_core::{FileId, RelPath};

/// A project of one page, `index.md`, holding `text`, with `builds`.
fn project(text: &str, builds: &[&str]) -> Project {
    let mut model_text = "spec = \"0.1\"\n".to_owned();
    for build in builds {
        model_text.push_str(&format!("\n[builds.{build}]\n"));
    }
    if let Some(first) = builds.first() {
        model_text.push_str(&format!("\n[editor]\nbuild = \"{first}\"\n"));
    }
    let model = ascribe_model::load_str(&model_text, FileId::new(0)).expect("a model");
    let sources = Vec::<(ascribe_core::RelPath, String)>::from_iter([(
        RelPath::parse("index.md").expect("a path"),
        text.into(),
    )]);
    Project::new(
        ascribe_check::in_memory(
            std::env::temp_dir(),
            RelPath::root(),
            model,
            model_text,
            sources,
        )
        .snapshot(),
    )
}

/// Each finding of `check` in `project`: its builds and its identity.
fn found(project: &Project, check: &str) -> Vec<(Vec<String>, Identity)> {
    let diagnosed = diagnose(project, &[]).expect("diagnosed");
    let ids = identities(project, &diagnosed.diagnostics);
    diagnosed
        .diagnostics
        .iter()
        .zip(ids)
        .filter(|(_, id)| id.check == check)
        .map(|(d, id)| (d.builds.clone(), id))
        .collect()
}

const PAGE: &str = "---\ntitle: Home\n---\n\n# Home\n\nSee [the guide](missing.md).\n\nAnd [the guide](missing.md) again.\n";

#[test]
fn two_alike_findings_in_one_file_are_told_apart_by_their_order() {
    let alike = found(&project(PAGE, &["site"]), "link-target-missing");
    let ids: Vec<&Identity> = alike.iter().map(|(_, id)| id).collect();
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[0].file, "index.md");
    assert_eq!(ids[0].fingerprint, ids[1].fingerprint);
    assert_eq!((ids[0].order, ids[1].order), (0, 1));

    // A line added above both changes neither.
    let moved = PAGE.replace("# Home\n", "# Home\n\nA new paragraph.\n");
    let again = found(&project(&moved, &["site"]), "link-target-missing");
    assert_eq!(again, alike);
}

#[test]
fn a_finding_in_two_builds_is_one() {
    let two = found(&project(PAGE, &["one", "two"]), "heading-level-skipped");
    let [(builds, id)] = two.as_slice() else {
        panic!("one finding: {two:?}");
    };
    assert_eq!(builds, &["one", "two"]);
    let one = found(&project(PAGE, &["one"]), "heading-level-skipped");
    assert_eq!(one.iter().map(|(_, id)| id).collect::<Vec<_>>(), [id]);
}
