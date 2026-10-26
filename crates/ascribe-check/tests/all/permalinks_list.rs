//! The permalinks list, `ascribe.permalinks`, against what the content
//! declares (SPEC §7.5), beyond the conformance cases in
//! `tests/conformance/cases/permalinks/`: a name declared in several places,
//! a heading in a fragment no page includes, a name that isn't a valid id,
//! and that every entry point reports the same, whether or not it already
//! has the source index. What the commands show is in `ascribe-cli`'s
//! tests.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::sync::Arc;

use ascribe_check::{Diagnostic, Project, check_all_builds, check_files};
use ascribe_core::{FileId, RelPath};
use ascribe_resolve::{Layout, MemoryFs};

/// Two builds, each publishing the pages of one deployment.
const MODEL: &str = r#"spec = "0.1"
[project]
content-root = "docs"
[dimensions.deployment]
values = ["cloud", "self-managed"]
[builds.cloud]
variants = { deployment = "cloud" }
[builds.self-managed]
variants = { deployment = "self-managed" }
[editor]
build = "cloud"
"#;

/// A project over `files`, with `list` as its `ascribe.permalinks`.
fn project(files: &[(&str, &str)], list: Option<&str>) -> Project {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    if let Some(list) = list {
        fs = fs.with_file("ascribe.permalinks", list);
    }
    let sources = Project::from_sources(files.iter().map(|(path, text)| {
        (
            RelPath::parse(path).expect("a relative path"),
            (*text).to_owned(),
        )
    }));
    Project::from_parts_with_fs(
        PathBuf::from("/nonexistent-ascribe-project"),
        RelPath::parse("docs").expect("a relative path"),
        model,
        MODEL.to_owned(),
        sources,
        Arc::new(fs),
    )
}

/// The list's diagnostics: each slug, with the file and line it's on.
fn found(project: &Project, diagnostics: &[Diagnostic]) -> Vec<(String, String, u32)> {
    diagnostics
        .iter()
        .filter(|d| d.slug.as_str().starts_with("permalink"))
        .map(|d| {
            let line = project
                .line_index(d.location.file)
                .and_then(|index| index.line_col(d.location.span.start()))
                .map_or(0, |at| at.line + 1);
            (
                d.slug.as_str().to_owned(),
                project.display_path(d.location.file).unwrap_or_default(),
                line,
            )
        })
        .collect()
}

const LIST_HEAD: &str = "version = 1\n[permalinks]\n";

#[test]
fn a_name_on_two_pages_no_build_shares_is_listed_once_at_either() {
    let files = [
        (
            "a.md",
            "---\ntitle: A\nvariant:\n  deployment: cloud\npermalink: setup\n---\n",
        ),
        (
            "b.md",
            "---\ntitle: B\nvariant:\n  deployment: self-managed\npermalink: setup\n---\n",
        ),
    ];
    for place in ["a.md", "b.md"] {
        let p = project(&files, Some(&format!("{LIST_HEAD}setup = \"{place}\"\n")));
        assert_eq!(found(&p, &check_files(&p)), [], "listed at {place}");
    }
    let p = project(&files, Some(&format!("{LIST_HEAD}setup = \"c.md\"\n")));
    assert_eq!(
        found(&p, &check_files(&p)),
        [(
            "permalinks-list-behind".to_owned(),
            "docs/a.md".to_owned(),
            5
        )],
        "reported once, at the first place it's declared"
    );
}

#[test]
fn a_heading_in_a_fragment_no_page_includes_is_still_declared() {
    let files = [("_setup.md", "## Setup\n@id {permalink=true}: setup\n")];
    let p = project(&files, Some(LIST_HEAD));
    assert_eq!(
        found(&p, &check_files(&p)),
        [(
            "permalinks-list-behind".to_owned(),
            "docs/_setup.md".to_owned(),
            2
        )]
    );
}

#[test]
fn a_name_that_isnt_an_id_is_id_invalids_not_the_lists() {
    let files = [("a.md", "---\ntitle: A\npermalink: a/b\n---\n")];
    let p = project(&files, Some(LIST_HEAD));
    let diagnostics = check_files(&p);
    assert_eq!(found(&p, &diagnostics), []);
    assert!(
        diagnostics.iter().any(|d| d.slug.as_str() == "id-invalid"),
        "{diagnostics:?}"
    );
}

#[test]
fn every_entry_point_reports_the_list_the_same() {
    let files = [
        (
            "a.md",
            "---\ntitle: A\n---\n\n## Keys\n@id {permalink=true}: keys\n",
        ),
        ("b.md", "---\ntitle: B\n---\n"),
    ];
    let p = project(&files, Some(&format!("{LIST_HEAD}gone = \"b.md\"\n")));
    let expected = [
        (
            "permalink-lost".to_owned(),
            "ascribe.permalinks".to_owned(),
            3,
        ),
        (
            "permalinks-list-behind".to_owned(),
            "docs/a.md".to_owned(),
            6,
        ),
    ];
    let mut files_only = found(&p, &check_files(&p));
    files_only.sort();
    assert_eq!(files_only, expected);
    let mut every_build = found(&p, &check_all_builds(&p));
    every_build.sort();
    assert_eq!(
        every_build, expected,
        "the list's diagnostics are once, not per build"
    );
}

#[test]
fn a_project_without_the_list_has_no_list_diagnostics() {
    let files = [("a.md", "---\ntitle: A\npermalink: a\n---\n")];
    let p = project(&files, None);
    assert_eq!(found(&p, &check_all_builds(&p)), []);
}
