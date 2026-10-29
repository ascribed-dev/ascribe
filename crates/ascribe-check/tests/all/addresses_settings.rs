//! A routing setting changed between the base and the working tree (SPEC
//! §7.10): `base-path`, `permalink-path`, `agents`, and a build's name. The
//! addresses it moved are one `address-lost` warning, whose safe fix
//! records the old value in `[former]`, which then accounts for them.

#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use ascribe_check::addresses::{Base, Here, compare};
use ascribe_check::{Diagnostic, Severity};
use ascribe_core::{Applicability, FileId, diagnostics};
use ascribe_model::AddressFile;
use ascribe_resolve::addresses::every_build;
use ascribe_resolve::{Layout, MemoryFs, Project};

const MODEL: &str = r#"spec = "0.1"
[project]
content-root = "docs"
[consumer]
profile = "astro"
site = "https://docs.example.com"
base-path = "/docs/"
trailing-slash = "never"
[builds.site]
"#;

const FILES: &[(&str, &str)] = &[
    ("index.md", "---\ntitle: Home\n---\n\nText.\n"),
    (
        "setup.md",
        "---\ntitle: Setup\npermalink: setup\n---\n\nText.\n",
    ),
    ("keys.md", "---\ntitle: Keys\n---\n\nText.\n"),
];

fn project(model: &str, files: &[(&str, &str)]) -> Project {
    let model = ascribe_model::load_str(model, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    Project::load(Arc::new(model), layout, &fs)
}

/// What comparing the project under `now_model` with the base under
/// `base_model`, the same pages, reports, with `entries` in
/// `ascribe.addresses`; and the file's text.
fn compared(base_model: &str, now_model: &str, entries: &str) -> (Vec<Diagnostic>, String) {
    compared_files(base_model, FILES, now_model, FILES, entries)
}

fn compared_files(
    base_model: &str,
    base_files: &[(&str, &str)],
    now_model: &str,
    now_files: &[(&str, &str)],
    entries: &str,
) -> (Vec<Diagnostic>, String) {
    let text = AddressFile::parse(&format!("version = 1\n{entries}"), FileId::new(1))
        .expect("a valid file")
        .to_toml();
    let file = AddressFile::parse(&text, FileId::new(1)).expect("a valid file");
    let now = project(now_model, now_files);
    let builds = every_build(&now);
    let here = Here {
        file: &file,
        text: &text,
        index: &now,
        builds: &builds,
    };
    let base = Base::of(&project(base_model, base_files));
    (compare(&here, &base), text)
}

fn messages(diagnostics: &[Diagnostic]) -> Vec<(&str, Severity, &str)> {
    diagnostics
        .iter()
        .map(|d| (d.slug.as_str(), d.severity, d.message.as_str()))
        .collect()
}

/// The file after the one fix of the one diagnostic, which is safe.
fn fixed(found: &[Diagnostic], text: &str) -> String {
    assert_eq!(found.len(), 1, "{:?}", messages(found));
    let fix = &found[0].fixes[0];
    assert_eq!(
        fix.title,
        "Record the settings change: `ascribe check --fix`"
    );
    assert_eq!(fix.applicability, Applicability::Safe);
    ascribe_core::apply_edits(text, &fix.edits).expect("the fix applies")
}

fn entries_of(text: &str) -> String {
    text.split_once("version = 1\n")
        .map(|(_, rest)| rest.to_owned())
        .expect("the version")
}

#[test]
fn a_base_path_changed_is_one_warning_whose_fix_records_it() {
    let now = MODEL.replace("/docs/", "/handbook/");
    let (found, text) = compared(MODEL, &now, "");
    // Each page and its Markdown version, the permalink, and llms.txt.
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "base-path changed from \"/docs/\" to \"/handbook/\", so 8 addresses moved, 1 of them a permalink"
        )]
    );
    let after = fixed(&found, &text);
    assert!(
        after.contains("[former]\nbase-path = [\"/docs/\"]\n"),
        "{after}"
    );
    // Recorded, it accounts for every address it moved.
    let (found, _) = compared(MODEL, &now, &entries_of(&after));
    assert!(found.is_empty(), "{:?}", messages(&found));
}

#[test]
fn a_second_change_is_recorded_after_the_first() {
    let now = MODEL.replace("/docs/", "/manual/");
    let base = MODEL.replace("/docs/", "/handbook/");
    let (found, text) = compared(&base, &now, "[former]\nbase-path = [\"/docs/\"]\n");
    assert!(
        fixed(&found, &text).contains("base-path = [\"/docs/\", \"/handbook/\"]\n"),
        "{text}"
    );
}

#[test]
fn a_page_lost_with_the_change_is_still_its_own_warning() {
    let now = MODEL.replace("/docs/", "/handbook/");
    let (found, _) = compared_files(MODEL, FILES, &now, &FILES[..2], "");
    let found = messages(&found);
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(
        found[0].2,
        "base-path changed from \"/docs/\" to \"/handbook/\", so 6 addresses moved, 1 of them a permalink"
    );
    assert_eq!(
        found[1].2,
        "keys.md was published at /docs/keys and /docs/keys.md, and nothing publishes them now"
    );
}

#[test]
fn a_permalink_path_changed_moves_only_the_permalinks() {
    let now = MODEL.replace(
        "trailing-slash = \"never\"",
        "trailing-slash = \"never\"\npermalink-path = \"/go/\"",
    );
    let (found, text) = compared(MODEL, &now, "");
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "permalink-path changed from \"/docs/go/\" to \"/go/\", so 1 address moved, a permalink's"
        )]
    );
    assert!(
        fixed(&found, &text).contains("permalink-path = [\"/docs/go/\"]\n"),
        "{text}"
    );
    // Set apart from the base path, it doesn't move with it.
    let (found, _) = compared(&now, &now.replace("/docs/", "/handbook/"), "");
    assert_eq!(
        found[0].message,
        "base-path changed from \"/docs/\" to \"/handbook/\", so 7 addresses moved"
    );
}

#[test]
fn agents_turned_off_records_true() {
    let now = MODEL.replace(
        "trailing-slash = \"never\"",
        "trailing-slash = \"never\"\nagents = false",
    );
    let (found, text) = compared(MODEL, &now, "");
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "agents was turned off, so 4 addresses moved"
        )]
    );
    let after = fixed(&found, &text);
    assert!(after.contains("[former]\nagents = [true]\n"), "{after}");
    let (found, _) = compared(MODEL, &now, &entries_of(&after));
    assert!(found.is_empty(), "{:?}", messages(&found));
    // Turned on, it publishes more, and moves nothing.
    let (found, _) = compared(&now, MODEL, "");
    assert!(found.is_empty(), "{:?}", messages(&found));
}

#[test]
fn two_changes_at_once_are_one_warning() {
    let now = MODEL.replace("/docs/", "/handbook/").replace(
        "trailing-slash = \"never\"",
        "trailing-slash = \"never\"\nagents = false",
    );
    let (found, text) = compared(MODEL, &now, "");
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "base-path changed from \"/docs/\" to \"/handbook/\" and agents was turned off, so 8 addresses moved, 1 of them a permalink"
        )]
    );
    let after = fixed(&found, &text);
    assert!(
        after.contains("[former]\nagents = [true]\nbase-path = [\"/docs/\"]\n"),
        "{after}"
    );
}

#[test]
fn a_renamed_build_is_compared_by_its_new_name_once_recorded() {
    let now = MODEL.replace("[builds.site]", "[builds.web]");
    let (found, text) = compared(MODEL, &now, "");
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "the build `site` was renamed `web`, so no address moved"
        )]
    );
    let after = fixed(&found, &text);
    assert!(
        after.contains("[former.builds]\n\"site\" = \"web\"\n"),
        "{after}"
    );
    let (found, _) = compared(MODEL, &now, &entries_of(&after));
    assert!(found.is_empty(), "{:?}", messages(&found));
    // Compared with it by name, a page the renamed build lost is lost.
    let (found, _) = compared_files(MODEL, FILES, &now, &FILES[..2], &entries_of(&after));
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "keys.md was published at /docs/keys and /docs/keys.md, and nothing publishes them now"
        )]
    );
}

#[test]
fn a_build_with_none_of_the_pages_is_gone_not_renamed() {
    let now = MODEL.replace("[builds.site]", "[builds.web]");
    let others = [("other.md", "---\ntitle: Other\n---\n\nText.\n")];
    let (found, _) = compared_files(MODEL, FILES, &now, &others, "");
    assert_eq!(found.len(), 1, "{:?}", messages(&found));
    assert_eq!(found[0].slug, diagnostics::ADDRESSES_BUILD_GONE);
}

/// `MODEL` with its build versioned: `site`, the current line `1.1`, under
/// `/docs/`, and `site@1.0`, the older line, under `/docs/1.0/`.
fn with_lines() -> String {
    MODEL.replace(
        "[builds.site]\n",
        "[dimensions.release]\nlabel = \"Product\"\nvalues = [\"prod\"]\n\
         [releases.prod]\nlive = [\"1.0\", \"1.1\"]\n\
         [builds.site]\nversions = \"prod\"\n",
    )
}

#[test]
fn a_rename_that_ties_prefers_the_build_that_isn_t_an_older_line() {
    // `site` and `site@1.0` publish every page `old` did.
    let before = MODEL.replace("[builds.site]", "[builds.old]");
    let (found, _) = compared(&before, &with_lines(), "");
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "the build `old` was renamed `site`, so no address moved"
        )]
    );
}

#[test]
fn an_older_line_s_own_folder_is_each_page_s_move() {
    // `site` keeps its name, so `old` can only be `site@1.0`, whose pages
    // moved under `1.0/` with no setting changed.
    let before = format!("{MODEL}[builds.old]\n");
    let (found, _) = compared(&before, &with_lines(), "");
    let lost = |page: &str| format!("{page}, and nothing publishes them now in the build `old`");
    let (index, keys, setup) = (
        lost("index.md was published at /docs and /docs/index.md"),
        lost("keys.md was published at /docs/keys and /docs/keys.md"),
        lost("setup.md was published at /docs/setup and /docs/setup.md"),
    );
    let lost = diagnostics::ADDRESS_LOST.as_str();
    assert_eq!(
        messages(&found),
        [
            (
                lost,
                Severity::Warning,
                "the build `old` was renamed `site@1.0`, so no address moved"
            ),
            (lost, Severity::Warning, index.as_str()),
            (lost, Severity::Warning, keys.as_str()),
            (lost, Severity::Warning, setup.as_str()),
        ]
    );
}
