//! The addresses the base published, against the working tree's (SPEC
//! §7.10), with the base given as data: what's lost, what an entry of
//! `ascribe.addresses` accounts for, and what a mark or a page's own move
//! decides. The commands that read the base from git are in `ascribe-cli`'s
//! tests.

#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use ascribe_check::addresses::compare;
use ascribe_check::{Diagnostic, Severity};
use ascribe_core::{FileId, RelPath, diagnostics};
use ascribe_model::AddressFile;
use ascribe_resolve::addresses::{Address, AddressKind, BuildAddresses, every_build};
use ascribe_resolve::{Layout, MemoryFs, Project};

/// Two builds, each publishing the pages of one deployment, with Markdown
/// versions.
const MODEL: &str = r#"spec = "0.1"
[project]
content-root = "docs"
[dimensions.deployment]
values = ["cloud", "self-managed"]
[consumer]
profile = "astro"
site = "https://docs.example.com"
base-path = "/docs/"
trailing-slash = "never"
[builds.cloud]
variants = { deployment = "cloud" }
[builds.self-managed]
variants = { deployment = "self-managed" }
[editor]
build = "cloud"
"#;

const PAGE: &str = "---\ntitle: Page\n---\n\n";

fn project(files: &[(&str, &str)]) -> Project {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    Project::load(Arc::new(model), layout, &fs)
}

fn file(text: &str) -> AddressFile {
    AddressFile::parse(&format!("version = 1\n{text}"), FileId::new(1)).expect("a valid file")
}

/// What the base of `base_files` published that `now_files` doesn't, with
/// `entries` in `ascribe.addresses`.
fn lost(base_files: &[(&str, &str)], now_files: &[(&str, &str)], entries: &str) -> Vec<Diagnostic> {
    let now = project(now_files);
    compare(
        &file(entries),
        &now,
        &every_build(&project(base_files)),
        &every_build(&now),
    )
}

fn messages(diagnostics: &[Diagnostic]) -> Vec<(&str, Severity, &str)> {
    diagnostics
        .iter()
        .map(|d| (d.slug.as_str(), d.severity, d.message.as_str()))
        .collect()
}

#[test]
fn a_page_that_moved_with_nothing_recorded_is_lost_once_with_its_addresses() {
    let found = lost(&[("setup.md", PAGE)], &[("install.md", PAGE)], "");
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "setup.md was published at /docs/setup and /docs/setup.md, and nothing publishes them now"
        )]
    );
    assert_eq!(found[0].builds, ["cloud", "self-managed"]);
}

#[test]
fn a_recorded_move_or_retirement_accounts_for_the_page() {
    let base = [("setup.md", PAGE)];
    let now = [("install.md", PAGE)];
    assert!(lost(&base, &now, "[moved]\n\"setup.md\" = \"install.md\"\n").is_empty());
    assert!(
        lost(
            &base,
            &now,
            "[retired]\n\"setup.md\" = \"Merged into the install page.\"\n"
        )
        .is_empty()
    );
    // An address of its own is a key too.
    assert_eq!(
        messages(&lost(
            &base,
            &now,
            "[retired]\n\"/docs/setup\" = \"Gone.\"\n"
        )),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "setup.md was published at /docs/setup.md, and nothing publishes it now"
        )]
    );
}

#[test]
fn a_move_to_a_page_nothing_publishes_accounts_for_nothing() {
    let found = lost(
        &[("setup.md", PAGE)],
        &[("install.md", PAGE)],
        "[moved]\n\"setup.md\" = \"elsewhere.md\"\n",
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].slug, diagnostics::ADDRESS_LOST);
}

#[test]
fn a_page_one_build_drops_by_a_mark_while_another_publishes_it_is_the_marks_decision() {
    let cloud = "---\ntitle: Page\nvariant:\n  deployment: cloud\n---\n\n";
    assert!(lost(&[("setup.md", PAGE)], &[("setup.md", cloud)], "").is_empty());
}

#[test]
fn a_page_lost_in_only_some_builds_names_them() {
    let base = BuildAddresses {
        build: "self-managed".to_owned(),
        addresses: vec![Address {
            address: "/docs/old".to_owned(),
            kind: AddressKind::Page,
            source: "old.md".to_owned(),
            page: RelPath::parse("old.md").expect("a path"),
        }],
    };
    let now = project(&[("index.md", PAGE)]);
    let builds = every_build(&now);
    let mut cloud = builds[0].clone();
    cloud.addresses.push(base.addresses[0].clone());
    let cloud_base = BuildAddresses {
        build: "cloud".to_owned(),
        addresses: base.addresses.clone(),
    };
    let found = compare(
        &file(""),
        &now,
        &[cloud_base, base],
        &[cloud, builds[1].clone()],
    );
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESS_LOST.as_str(),
            Severity::Warning,
            "old.md was published at /docs/old, and nothing publishes it now in the build `self-managed`"
        )]
    );
    assert_eq!(found[0].builds, ["self-managed"]);
}

#[test]
fn an_id_taken_off_a_page_still_published_is_advice() {
    let found = lost(
        &[(
            "setup.md",
            &format!("{PAGE}## Install\n@id: install\n\nText.\n"),
        )],
        &[("setup.md", &format!("{PAGE}## Install\n\nText.\n"))],
        "",
    );
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ANCHOR_LOST.as_str(),
            Severity::Advice,
            "the id `install` in setup.md was published at /docs/setup#install, and the page doesn't have it now"
        )]
    );
    // An entry for the id accounts for it.
    let moved = lost(
        &[(
            "setup.md",
            &format!("{PAGE}## Install\n@id: install\n\nText.\n"),
        )],
        &[(
            "setup.md",
            &format!("{PAGE}## Install\n@id: installing\n\nText.\n"),
        )],
        "[moved]\n\"setup.md#install\" = \"setup.md#installing\"\n",
    );
    assert!(moved.is_empty(), "{moved:?}");
}

#[test]
fn an_id_on_a_page_that_moved_goes_with_the_page() {
    let page = format!("{PAGE}## Install\n@id: install\n\nText.\n");
    let found = lost(&[("setup.md", &page)], &[("install.md", &page)], "");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].slug, diagnostics::ADDRESS_LOST);
}

#[test]
fn a_build_only_the_base_has_is_advice() {
    let now = project(&[("index.md", PAGE)]);
    let mut base = every_build(&now);
    base.push(BuildAddresses {
        build: "beta".to_owned(),
        addresses: base[0].addresses.clone(),
    });
    let found = compare(&file(""), &now, &base, &every_build(&now));
    assert_eq!(
        messages(&found),
        [(
            diagnostics::ADDRESSES_BUILD_GONE.as_str(),
            Severity::Advice,
            "the base has the build `beta`, and ascribe.toml doesn't now, so its 2 addresses aren't compared"
        )]
    );
}

/// Quill's builds and its line, each against itself by name: nothing is
/// lost, and no build is gone.
#[test]
fn quill_compared_with_itself_loses_nothing() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let model = ascribe_model::load(root.join("ascribe.toml")).expect("the Quill model loads");
    let layout = Layout::from_model(&model);
    let fs = ascribe_resolve::DiskFs::new(&root, &layout);
    let quill = Project::load(Arc::new(model), layout, &fs);
    let builds = every_build(&quill);
    assert!(builds.iter().any(|b| b.build == "site@3.4"));
    let found = compare(&file(""), &quill, &builds, &every_build(&quill));
    assert!(found.is_empty(), "{:?}", messages(&found));
}
