//! `status` in a project in a temporary folder that took templates with
//! `eject`, compared with this binary's set and with a newer one made here:
//! unchanged, changed upstream, changed by the project, both, a file deleted,
//! and a copy with nothing remembered.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;

use ascribe_check::Project;
use ascribe_core::RelPath;
use ascribe_resolve::{DiskFs, Layout};
use ascribe_templates::{
    Change, Eject, FileKind, Record, SectionChangeKind, Status, StatusAnswer, TemplateStatus,
    Upstream, UpstreamChange, diff, eject, status,
};

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n";

/// A project that took `names` from this binary's set.
fn project(names: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary folder");
    std::fs::write(dir.path().join("ascribe.toml"), MODEL).expect("written");
    std::fs::create_dir(dir.path().join("docs")).expect("made");
    let file = Project::load_model(&dir.path().join("ascribe.toml")).expect("the model loads");
    let names: Vec<String> = names.iter().map(|n| (*n).to_owned()).collect();
    let fs = DiskFs::new(dir.path(), &Layout::from_model(&file.model));
    let answer = eject(
        &Eject {
            root: dir.path(),
            model: &file.model,
            model_text: &file.text,
            names: &names,
            to: &RelPath::parse("templates").expect("a path"),
            ascribe_version: "9.9.9",
        },
        &fs,
    )
    .expect("ejected");
    assert!(answer.refused.is_empty(), "{answer:?}");
    dir
}

/// This binary's set, a release later, with the how-to's "Before you start"
/// renamed and a section added before "See also".
fn newer() -> Upstream {
    let mut upstream = Upstream::carried();
    upstream.set.tag = "v9.0.0".to_owned();
    let how_to = upstream.texts.get_mut("how-to.md").expect("the how-to");
    assert!(how_to.contains("\n## Before you start\n"));
    *how_to = how_to.replace("\n## Before you start\n", "\n## Before you begin\n");
    *how_to = how_to.replace(
        "\n## See also\n",
        "\n## Questions\n\n\\{Where to ask.}\n\n## See also\n",
    );
    upstream
}

/// What status says of the project in `dir`, compared with `upstream`. The
/// model is read as written, so a file it names may be gone.
fn run(dir: &Path, upstream: &Upstream) -> StatusAnswer {
    let text = std::fs::read_to_string(dir.join("ascribe.toml")).expect("read");
    let model =
        ascribe_model::load_str(&text, ascribe_core::FileId::new(0)).expect("the model loads");
    let fs = DiskFs::new(dir, &Layout::from_model(&model));
    status(
        &Status {
            model: &model,
            upstream,
        },
        &fs,
    )
}

fn template<'a>(answer: &'a StatusAnswer, name: &str) -> &'a TemplateStatus {
    answer
        .templates
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("no {name} in {answer:#?}"))
}

fn edit(dir: &Path, path: &str, f: impl FnOnce(String) -> String) {
    let file = dir.join(path);
    let text = std::fs::read_to_string(&file).expect("read");
    std::fs::write(&file, f(text)).expect("written");
}

#[test]
fn an_untouched_copy_of_this_set_is_unchanged_on_both_sides() {
    let dir = project(&["how-to", "concept"]);
    let answer = run(dir.path(), &Upstream::carried());
    assert!(!answer.upstream_changed);
    assert_eq!(answer.set.tag, ascribe_templates::set().tag);
    let names: Vec<&str> = answer.templates.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["how-to", "concept"]);
    for t in &answer.templates {
        assert_eq!(t.record, Record::Entry);
        assert_eq!(t.lock, "templates/templates.lock");
        assert_eq!(t.copied_from.as_deref(), Some(answer.set.tag.as_str()));
        assert_eq!(t.upstream, UpstreamChange::Unchanged);
        assert_eq!(t.project, Change::Unchanged);
        let kinds: Vec<FileKind> = t.files.iter().map(|f| f.kind).collect();
        assert_eq!(
            kinds,
            [FileKind::Template, FileKind::Guide, FileKind::Notes]
        );
        assert!(t.files.iter().all(|f| f.matches_set == Some(true)));
    }
    let not_taken: Vec<&str> = answer.not_taken.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        not_taken,
        ["tutorial", "reference", "troubleshooting", "release-notes"]
    );
}

/// The acceptance criterion: against a newer set, untouched copies get
/// exactly the templates that differ.
#[test]
fn against_a_newer_set_untouched_copies_list_exactly_what_changed_upstream() {
    let dir = project(&[]);
    let answer = run(dir.path(), &newer());
    assert!(answer.upstream_changed);
    assert_eq!(answer.set.tag, "v9.0.0");
    let changed: Vec<&str> = answer
        .templates
        .iter()
        .filter(|t| t.upstream == UpstreamChange::Changed)
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(changed, ["how-to"]);
    assert!(
        answer
            .templates
            .iter()
            .all(|t| t.project == Change::Unchanged)
    );
    let how_to = template(&answer, "how-to");
    let file = &how_to.files[0];
    assert_eq!(file.path, "templates/how-to.md");
    assert_eq!(file.upstream, UpstreamChange::Changed);
    assert_eq!(file.project, Change::Unchanged);
    assert_eq!(file.matches_set, Some(false));
    let sections: Vec<(Option<&str>, SectionChangeKind)> = file
        .sections
        .as_ref()
        .expect("sections, since the copy is as written")
        .iter()
        .map(|s| (s.heading.as_deref(), s.change))
        .collect();
    assert_eq!(
        sections,
        [
            (Some("Before you begin"), SectionChangeKind::Added),
            (Some("Questions"), SectionChangeKind::Added),
            (Some("Before you start"), SectionChangeKind::Removed),
        ]
    );
    assert!(
        how_to.files[1..]
            .iter()
            .all(|f| f.upstream == UpstreamChange::Unchanged)
    );
}

#[test]
fn a_copy_the_project_changed_is_changed_here_and_not_upstream() {
    let dir = project(&["how-to"]);
    edit(dir.path(), "templates/how-to.md", |t| t + "\nOurs.\n");
    let answer = run(dir.path(), &Upstream::carried());
    assert!(!answer.upstream_changed);
    let how_to = template(&answer, "how-to");
    assert_eq!(how_to.upstream, UpstreamChange::Unchanged);
    assert_eq!(how_to.project, Change::Changed);
    assert_eq!(how_to.files[0].project, Change::Changed);
    assert_eq!(how_to.files[0].sections, None);
    assert_eq!(how_to.files[1].project, Change::Unchanged);
}

#[test]
fn a_copy_changed_on_both_sides_says_both_and_no_sections() {
    let dir = project(&["how-to"]);
    edit(dir.path(), "templates/how-to.md", |t| t + "\nOurs.\n");
    let answer = run(dir.path(), &newer());
    assert!(answer.upstream_changed);
    let file = &template(&answer, "how-to").files[0];
    assert_eq!(file.upstream, UpstreamChange::Changed);
    assert_eq!(file.project, Change::Changed);
    assert_eq!(file.sections, None);
}

#[test]
fn a_deleted_file_is_deleted() {
    let dir = project(&["how-to"]);
    std::fs::remove_file(dir.path().join("templates/how-to.guide.md")).expect("removed");
    let answer = run(dir.path(), &Upstream::carried());
    let how_to = template(&answer, "how-to");
    assert_eq!(how_to.project, Change::Changed);
    assert_eq!(how_to.files[1].project, Change::Deleted);
    assert_eq!(how_to.files[1].matches_set, None);
}

#[test]
fn crlf_line_endings_from_a_checkout_are_not_a_change() {
    let dir = project(&["how-to"]);
    edit(dir.path(), "templates/how-to.md", |t| {
        t.replace('\n', "\r\n")
    });
    let answer = run(dir.path(), &Upstream::carried());
    assert_eq!(template(&answer, "how-to").project, Change::Unchanged);
}

#[test]
fn with_no_lock_it_says_so_and_compares_the_files_with_the_set() {
    let dir = project(&["how-to"]);
    std::fs::remove_file(dir.path().join("templates/templates.lock")).expect("removed");
    edit(dir.path(), "templates/how-to.md", |t| t + "\nOurs.\n");
    let answer = run(dir.path(), &newer());
    assert!(!answer.upstream_changed, "nothing remembered says it did");
    let how_to = template(&answer, "how-to");
    assert_eq!(how_to.record, Record::NoLock);
    assert_eq!(how_to.lock, "templates/templates.lock");
    assert_eq!(how_to.copied_from, None);
    assert_eq!(how_to.upstream, UpstreamChange::Unknown);
    assert_eq!(how_to.project, Change::Unknown);
    let matches: Vec<(&str, Option<bool>)> = how_to
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.matches_set))
        .collect();
    assert_eq!(
        matches,
        [
            ("templates/how-to.md", Some(false)),
            ("templates/how-to.guide.md", Some(true)),
            ("templates/how-to.ascribe.md", Some(true)),
        ]
    );
    assert!(how_to.files.iter().all(|f| f.project == Change::Unknown));
}

#[test]
fn a_template_with_no_entry_is_told_so() {
    let dir = project(&["how-to", "concept"]);
    edit(dir.path(), "templates/templates.lock", |t| {
        let at = t
            .find("[[template]]\nname = \"concept\"")
            .expect("concept's entry");
        t[..at].to_owned()
    });
    let answer = run(dir.path(), &Upstream::carried());
    assert_eq!(template(&answer, "how-to").record, Record::Entry);
    let concept = template(&answer, "concept");
    assert_eq!(concept.record, Record::NoEntry);
    assert!(concept.files.iter().all(|f| f.matches_set == Some(true)));
}

#[test]
fn a_lock_eject_didn_t_write_is_told_so() {
    let dir = project(&["how-to"]);
    std::fs::write(dir.path().join("templates/templates.lock"), "version = 7\n").expect("written");
    let answer = run(dir.path(), &Upstream::carried());
    let how_to = template(&answer, "how-to");
    assert_eq!(how_to.record, Record::InvalidLock);
    assert!(
        how_to
            .lock_problem
            .as_deref()
            .is_some_and(|p| p.contains("version is 7")),
        "{how_to:?}"
    );
}

#[test]
fn diff_is_the_upstream_change_to_an_untouched_copy() {
    let dir = project(&["how-to", "concept"]);
    let upstream = newer();
    let answer = run(dir.path(), &upstream);
    let fs = DiskFs::new(
        dir.path(),
        &Layout {
            content_root: RelPath::parse("docs").expect("a path"),
            output_dir: RelPath::parse("dist").expect("a path"),
        },
    );
    let diffs = diff(&answer, "how-to", &upstream, &fs).expect("taken");
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert_eq!(diffs[0].path, "templates/how-to.md");
    assert_eq!(diffs[0].project, Change::Unchanged);
    let text = &diffs[0].diff;
    assert!(
        text.starts_with("--- a/templates/how-to.md\n+++ b/templates/how-to.md\n"),
        "{text}"
    );
    assert!(
        text.contains("\n-## Before you start\n+## Before you begin\n"),
        "{text}"
    );
    assert!(text.contains("\n+## Questions\n"), "{text}");
    assert!(
        diff(&answer, "concept", &upstream, &fs)
            .expect("taken")
            .is_empty()
    );
    assert!(diff(&answer, "tutorial", &upstream, &fs).is_none());
}
