//! The page-level checks, on small projects built in memory.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use tessera_check::{
    Diagnostic, Project, SourceFile, check_all_builds, check_builds, check_files, check_project,
};
use tessera_core::{FileId, RelPath};

/// Three builds: `site` keeps everything, `cloud` and `self-managed` each
/// select one deployment. `edge` is a value that no build selects.
const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"

[types.page]
default = true

[types.page.frontmatter]
title = "string"

[dimensions.deployment]
values = ["cloud", "self-managed", "edge"]
versionless = ["cloud", "self-managed", "edge"]

[builds.site]
variants = "switch"
availability = "badge"

[builds.cloud]
variants = { deployment = "cloud" }
availability = "badge"

[builds.self-managed]
variants = { deployment = "self-managed" }
availability = "badge"
"#;

/// The same, without a build that keeps everything, so `edge` is never
/// published.
const SELECTION_ONLY: &str = r#"
spec = "0.1"

[project]
content-root = "docs"

[types.page]
default = true

[types.page.frontmatter]
title = "string"

[dimensions.deployment]
values = ["cloud", "self-managed", "edge"]
versionless = ["cloud", "self-managed", "edge"]

[builds.cloud]
variants = { deployment = "cloud" }
availability = "badge"

[builds.self-managed]
variants = { deployment = "self-managed" }
availability = "badge"

[editor]
build = "cloud"
"#;

fn project_with(model: &str, files: &[(&str, &str)]) -> Project {
    let sources = Project::from_sources(files.iter().map(|(path, text)| {
        (
            RelPath::parse(path).expect("a relative path"),
            (*text).to_owned(),
        )
    }));
    Project::from_parts(
        PathBuf::from("/nonexistent-ascribe-project"),
        RelPath::parse("docs").expect("a relative path"),
        tessera_model::load_str(model, FileId::new(0)).expect("the model loads"),
        model.to_owned(),
        sources,
    )
}

fn project(files: &[(&str, &str)]) -> Project {
    project_with(MODEL, files)
}

fn page(body: &str) -> String {
    format!("---\ntitle: T\n---\n\n{body}\n")
}

/// `(slug, file, line)` of the diagnostics, in the order reported.
fn at(project: &Project, diagnostics: &[Diagnostic]) -> Vec<(String, String, usize)> {
    diagnostics
        .iter()
        .map(|d| {
            let file = project.file(d.location.file).expect("a known file");
            let line = file.text[..d.location.span.start()].matches('\n').count() + 1;
            (
                d.slug.to_string(),
                file.content_path.expect("a source").to_string(),
                line,
            )
        })
        .collect()
}

fn page_level(project: &Project) -> Vec<Diagnostic> {
    let files = check_files(project).len();
    let mut all = check_all_builds(project);
    // The file-level diagnostics come first, and stay as `check_files` has them.
    assert_eq!(all[..files], check_files(project)[..]);
    all.split_off(files)
}

fn row(slug: &str, file: &str, line: usize) -> (String, String, usize) {
    (slug.to_owned(), file.to_owned(), line)
}

#[test]
fn a_clean_project_has_no_diagnostics_in_any_build() {
    let p = project(&[
        ("index.md", &page("## Setup\n\nSee [it](other.md#setup).")),
        ("other.md", &page("## Setup\n\nText.")),
    ]);
    assert!(
        check_all_builds(&p).is_empty(),
        "{:#?}",
        check_all_builds(&p)
    );
}

#[test]
fn a_problem_in_a_fragment_is_reported_at_the_include_site_and_noted_in_the_fragment() {
    // The link's target id doesn't exist. The link is in `_f.md`, which
    // `index.md` includes at line 6 (SPEC §8.1).
    let p = project(&[
        ("_f.md", "See [it](other.md#gone).\n"),
        ("index.md", &page("@include: _f.md")),
        ("other.md", &page("## Here")),
    ]);
    let found = page_level(&p);
    assert_eq!(at(&p, &found), [row("link-id-missing", "index.md", 5)]);
    let d = &found[0];
    // One diagnostic, with the fragment as related information.
    assert_eq!(d.related.len(), 1);
    let related = p.file(d.related[0].location.file).expect("a file");
    assert_eq!(
        related.content_path.map(ToString::to_string).as_deref(),
        Some("_f.md")
    );
    assert!(d.related[0].message.contains("_f.md"), "{:?}", d.related);
    // The fragment on its own has nothing to report at page level.
    assert!(!at(&p, &found).iter().any(|(_, file, _)| file == "_f.md"));
}

#[test]
fn a_fragment_included_twice_reports_its_duplicate_id_at_the_second_include() {
    let p = project(&[
        ("_f.md", "## Shared\n@id: shared\n\nText.\n"),
        ("index.md", &page("@include: _f.md\n\n@include: _f.md")),
    ]);
    let found = page_level(&p);
    assert_eq!(at(&p, &found), [row("id-duplicate", "index.md", 7)]);
    assert!(
        found[0].message.contains("including `_f.md`"),
        "{}",
        found[0].message
    );
    // The first occurrence is related information.
    assert_eq!(found[0].related.len(), 2, "{:#?}", found[0].related);
}

#[test]
fn a_duplicate_is_reported_once_at_the_later_occurrence() {
    let p = project(&[(
        "index.md",
        &page("## One\n@id: setup\n\n## Two\n@id: setup\n\n## Options\n\n## Options"),
    )]);
    assert_eq!(
        at(&p, &page_level(&p)),
        [
            row("id-duplicate", "index.md", 9),
            row("heading-duplicate-without-id", "index.md", 13),
        ]
    );
}

#[test]
fn a_build_specific_problem_is_reported_once_naming_its_build() {
    // Only `cloud` removes every arm of this group: `site` keeps all arms,
    // and `self-managed` keeps the arm.
    let p = project(&[(
        "index.md",
        &page("@variant {deployment=self-managed}:\nSelf-managed only.\n@end"),
    )]);
    let found = page_level(&p);
    assert_eq!(
        at(&p, &found),
        [row("variant-no-arm-survives", "index.md", 5)]
    );
    assert_eq!(found[0].builds, ["cloud"]);
    assert!(
        found[0].message.contains("build `cloud`"),
        "{}",
        found[0].message
    );
    assert!(!found[0].unpublished);

    // One build's own check says the same, naming it.
    let cloud = p.model().build("cloud").expect("a build");
    let one: Vec<_> = check_project(&p, cloud);
    assert_eq!(one.len(), 1, "{one:#?}");
    assert_eq!(one[0], found[0]);
    let sm = p.model().build("self-managed").expect("a build");
    assert!(check_project(&p, sm).is_empty());
}

#[test]
fn a_problem_in_every_build_is_reported_once_naming_them_all() {
    let p = project(&[("index.md", &page("## A\n@id: same\n\n## B\n@id: same"))]);
    let found = page_level(&p);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].builds, ["site", "cloud", "self-managed"]);
    assert_eq!(found[0].builds_note(3), None);
    assert_eq!(
        found[0].builds_note(4).as_deref(),
        Some("only in builds `site`, `cloud`, `self-managed`")
    );
}

#[test]
fn a_problem_two_builds_share_is_one_diagnostic_naming_both() {
    // Both selecting builds remove the heading the link names... `edge` is
    // selected by neither, and the link is to a heading only that arm has.
    let p = project(&[
        ("index.md", &page("See [it](keys.md#edge-setup).")),
        (
            "keys.md",
            &page("@variant {deployment=edge}:\n## Edge setup\n@id: edge-setup\n@end"),
        ),
    ]);
    let found: Vec<_> = page_level(&p)
        .into_iter()
        .filter(|d| d.slug.as_str() == "link-id-removed")
        .collect();
    assert_eq!(at(&p, &found), [row("link-id-removed", "index.md", 5)]);
    assert_eq!(found[0].builds, ["cloud", "self-managed"]);
    // The message names both builds, with the plural wording.
    assert!(
        found[0]
            .message
            .starts_with("builds `cloud`, `self-managed` remove the heading"),
        "{}",
        found[0].message
    );
}

#[test]
fn a_link_in_a_removed_arm_is_not_that_builds_problem() {
    // The arm holds a link to a missing id. `cloud` removes the arm, so only
    // the builds that keep it report the problem.
    let p = project(&[
        (
            "index.md",
            &page("@variant {deployment=self-managed}:\nSee [it](keys.md#gone).\n@end"),
        ),
        ("keys.md", &page("## Here")),
    ]);
    let found = page_level(&p);
    let missing: Vec<_> = found
        .iter()
        .filter(|d| d.slug.as_str() == "link-id-missing")
        .collect();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].builds, ["site", "self-managed"]);
}

#[test]
fn content_that_no_build_publishes_is_checked_and_marked() {
    // `edge` is selected by neither build, so no build publishes its arm; its
    // duplicate id would otherwise never be reported.
    let p = project_with(
        SELECTION_ONLY,
        &[(
            "index.md",
            &page(
                "## Setup\n@id: setup\n\n@variant {deployment=edge}:\n## Again\n@id: setup\n@end",
            ),
        )],
    );
    // The group itself is in both builds, which remove its only arm; that's
    // theirs to report. The duplicate is in the arm neither publishes.
    let found = page_level(&p);
    assert_eq!(
        at(&p, &found),
        [
            row("variant-no-arm-survives", "index.md", 8),
            row("id-duplicate", "index.md", 10)
        ]
    );
    assert!(!found[0].unpublished);
    assert_eq!(found[0].builds, ["cloud", "self-managed"]);
    assert!(found[1].unpublished);
    assert!(found[1].builds.is_empty());
    assert_eq!(
        found[1].builds_note(2).as_deref(),
        Some("in content that no build publishes")
    );
}

#[test]
fn arms_that_only_clash_when_they_are_all_kept_are_not_a_problem() {
    // Each arm is published by some build; no build has both. The same `@id`
    // in both arms is what a switch would duplicate, but no build here does.
    let p = project_with(
        SELECTION_ONLY,
        &[(
            "index.md",
            &page(
                "@variant {deployment=cloud}:\n## Setup\n@id: setup\n@variant {deployment=self-managed}:\n## Setup\n@id: setup\n@end",
            ),
        )],
    );
    assert!(page_level(&p).is_empty(), "{:#?}", page_level(&p));
}

#[test]
fn a_page_every_build_drops_is_checked_as_unpublished() {
    let p = project_with(
        SELECTION_ONLY,
        &[(
            "index.md",
            "---\ntitle: T\nvariant:\n  deployment: edge\n---\n\n## A\n@id: same\n\n## B\n@id: same\n",
        )],
    );
    let found = page_level(&p);
    assert_eq!(at(&p, &found), [row("id-duplicate", "index.md", 11)]);
    assert!(found[0].unpublished);
}

#[test]
fn a_build_that_keeps_everything_needs_no_extra_pass() {
    // `site` is a switch with badges: nothing is unpublished, so nothing is
    // marked so, even in an arm no selecting build keeps.
    let p = project(&[(
        "index.md",
        &page("## Setup\n@id: setup\n\n@variant {deployment=edge}:\n## Again\n@id: setup\n@end"),
    )]);
    let found: Vec<_> = page_level(&p)
        .into_iter()
        .filter(|d| d.slug.as_str() == "id-duplicate")
        .collect();
    assert_eq!(found.len(), 1);
    assert!(!found[0].unpublished);
    assert_eq!(found[0].builds, ["site"]);
}

#[test]
fn locations_use_the_projects_file_ids_whatever_order_the_sources_came_in() {
    // Sources given in reverse path order, numbered as given.
    let sources = vec![
        SourceFile {
            id: FileId::new(1),
            path: RelPath::parse("index.md").expect("a path"),
            text: page("@include: _f.md"),
            unreadable: None,
        },
        SourceFile {
            id: FileId::new(2),
            path: RelPath::parse("_f.md").expect("a path"),
            text: "See [it](index.md#gone).\n".to_owned(),
            unreadable: None,
        },
    ];
    let p = Project::from_parts(
        PathBuf::from("/nonexistent-ascribe-project"),
        RelPath::parse("docs").expect("a relative path"),
        tessera_model::load_str(MODEL, FileId::new(0)).expect("the model loads"),
        MODEL.to_owned(),
        sources,
    );
    let found = page_level(&p);
    assert_eq!(at(&p, &found), [row("link-id-missing", "index.md", 5)]);
    let related = p.file(found[0].related[0].location.file).expect("a file");
    assert_eq!(
        related.content_path.map(ToString::to_string).as_deref(),
        Some("_f.md")
    );
}

#[test]
fn check_project_is_the_file_level_diagnostics_then_the_builds_page_level_ones() {
    let p = project(&[(
        "index.md",
        &page("[gone](gone.md)\n\n## A\n@id: same\n\n## B\n@id: same"),
    )]);
    let build = p.model().build("cloud").expect("a build");
    let all = check_project(&p, build);
    let files = check_files(&p);
    assert_eq!(all[..files.len()], files[..]);
    let rest: Vec<_> = all[files.len()..].iter().map(|d| d.slug.as_str()).collect();
    assert_eq!(rest, ["id-duplicate"]);
    assert_eq!(all[files.len()].builds, ["cloud"]);
}

#[test]
fn a_message_names_its_builds_when_the_registry_template_has_a_build_placeholder() {
    let registry = tessera_check::Registry::global();
    for (slug, names) in [
        ("variant-no-arm-survives", true),
        ("link-id-removed", true),
        ("link-page-dropped", true),
        ("id-duplicate", false),
        ("include-id-missing", false),
    ] {
        let slug = tessera_core::DiagnosticSlug::from_name(slug).expect("a slug");
        let entry = registry.get(slug).expect("an entry");
        assert_eq!(entry.names_build(), names, "{slug}");
    }
}

#[test]
fn a_subset_of_builds_merges_what_they_share() {
    let p = project(&[
        ("index.md", &page("See [it](keys.md#edge-setup).")),
        (
            "keys.md",
            &page("@variant {deployment=edge}:\n## Edge setup\n@id: edge-setup\n@end"),
        ),
    ]);
    let builds: Vec<_> = p.model().builds.iter().collect();
    let cloud_and_sm: Vec<_> = builds
        .iter()
        .copied()
        .filter(|b| b.name != "site")
        .collect();
    let found: Vec<_> = check_builds(&p, &cloud_and_sm)
        .into_iter()
        .filter(|d| d.slug.as_str() == "link-id-removed")
        .collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].builds, ["cloud", "self-managed"]);
    // Every build is `check_all_builds` less the pass over unpublished content.
    let all = check_builds(&p, &builds);
    let expected: Vec<_> = check_all_builds(&p)
        .into_iter()
        .filter(|d| !d.unpublished)
        .collect();
    assert_eq!(all, expected);
    // One build is `check_project`.
    assert_eq!(check_builds(&p, &builds[..1]), check_project(&p, builds[0]));
}
