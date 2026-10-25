//! The addresses the base published, against the working tree's (SPEC
//! §7.10), with the base given as data: what's lost, what an entry of
//! `ascribe.addresses` accounts for, and what a mark or a page's own move
//! decides. The commands that read the base from git are in `ascribe-cli`'s
//! tests.

#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use ascribe_check::addresses::{Base, Here, compare, entries};
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

/// What comparing `now`, with `entries` in `ascribe.addresses`, with the
/// base's builds `base` reports.
fn compared(
    entries: &str,
    now: &Project,
    base: &[BuildAddresses],
    builds: &[BuildAddresses],
) -> Vec<Diagnostic> {
    let file = file(entries);
    let text = file.to_toml();
    let here = Here {
        file: &file,
        text: &text,
        index: now,
        builds,
    };
    let base = Base {
        builds: base.to_vec(),
        ..Base::default()
    };
    compare(&here, &base)
}

/// What the base of `base_files` published that `now_files` doesn't, with
/// `entries` in `ascribe.addresses`.
fn lost(base_files: &[(&str, &str)], now_files: &[(&str, &str)], entries: &str) -> Vec<Diagnostic> {
    let now = project(now_files);
    compared(
        entries,
        &now,
        &every_build(&project(base_files)),
        &every_build(&now),
    )
}

/// What comparing, and the file's checks, report for `now_files` against
/// the base of `base_files`, with `renames` git sees, and `entries` in the
/// file: each with the file's text after its fixes, in order.
fn checked(
    base_files: &[(&str, &str)],
    now_files: &[(&str, &str)],
    renames: &[(&str, &str)],
    entries_text: &str,
) -> (Vec<Diagnostic>, String) {
    let now = project(now_files);
    let builds = every_build(&now);
    // Read from the file as Ascribe writes it, so each entry's place is in
    // the text the fixes edit.
    let text = file(entries_text).to_toml();
    let file = AddressFile::parse(&text, FileId::new(1)).expect("a valid file");
    let here = Here {
        file: &file,
        text: &text,
        index: &now,
        builds: &builds,
    };
    let mut base = Base::of(&project(base_files));
    base.renames = renames
        .iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect();
    let mut found = compare(&here, &base);
    found.extend(entries(&here, Some(&base)));
    (found, text)
}

/// `text` with the fix of `diagnostic` titled starting `title` applied.
fn fixed(text: &str, diagnostic: &Diagnostic, title: &str) -> String {
    let fix = diagnostic
        .fixes
        .iter()
        .find(|f| f.title.starts_with(title))
        .unwrap_or_else(|| panic!("no fix `{title}` in {:?}", diagnostic.fixes));
    ascribe_core::apply_edits(text, &fix.edits).expect("the fix applies")
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
    let found = compared("", &now, &[cloud_base, base], &[cloud, builds[1].clone()]);
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
    let found = compared("", &now, &base, &every_build(&now));
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
    let found = compared("", &quill, &builds, &every_build(&quill));
    assert!(found.is_empty(), "{:?}", messages(&found));
}

fn page(title: &str, body: &str) -> String {
    format!("---\ntitle: {title}\n---\n\n{body}\n")
}

fn slugs(found: &[Diagnostic]) -> Vec<&str> {
    found.iter().map(|d| d.slug.as_str()).collect()
}

#[test]
fn a_page_git_sees_renamed_is_fixed_safely_by_recording_git_s_renames() {
    let setup = page("Set up", "## Install\n\nInstall the agent.");
    let (found, text) = checked(
        &[
            ("guides/setup.md", &setup),
            ("guides/sync.md", &page("Sync", "Sync.")),
        ],
        &[
            ("start/setup.md", &setup),
            ("start/sync.md", &page("Sync", "Sync.")),
        ],
        &[
            ("guides/setup.md", "start/setup.md"),
            ("guides/sync.md", "start/sync.md"),
        ],
        "",
    );
    assert_eq!(slugs(&found), ["address-lost", "address-lost"]);
    let fix = &found[0].fixes[0];
    assert_eq!(
        fix.title,
        "git sees it renamed to start/setup.md: record git's renames, as `ascribe addresses move --from-git` does"
    );
    assert_eq!(fix.applicability, ascribe_core::Applicability::Safe);
    // One fix records every rename, the same for each page, so it's taken
    // once.
    assert_eq!(found[1].fixes[0].edits, fix.edits);
    let after = fixed(&text, &found[0], "git sees");
    assert!(
        after.contains(
            "[moved]\n\"guides/setup.md\" = \"start/setup.md\"\n\"guides/sync.md\" = \"start/sync.md\"\n"
        ),
        "{after}"
    );
}

#[test]
fn a_deleted_page_is_offered_the_pages_most_like_it_and_retiring() {
    let (found, text) = checked(
        &[
            (
                "reference/options.md",
                &page(
                    "Options",
                    "## Timeout\n\nSet the timeout option to wait longer.",
                ),
            ),
            (
                "guides/install.md",
                &page("Install", "Install it with npm."),
            ),
        ],
        &[
            (
                "guides/configure.md",
                &page(
                    "Configure",
                    "## Timeout\n\nThe timeout option, and the others.",
                ),
            ),
            (
                "guides/install.md",
                &page("Install", "Install it with npm."),
            ),
        ],
        &[],
        "",
    );
    assert_eq!(slugs(&found), ["address-lost"]);
    let titles: Vec<&str> = found[0].fixes.iter().map(|f| f.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "Forward it to the page most like it, guides/configure.md: `ascribe addresses move reference/options.md guides/configure.md`",
            "Retire it, and write why: `ascribe addresses retire reference/options.md --reason \"…\"`",
        ]
    );
    assert!(
        found[0]
            .fixes
            .iter()
            .all(|f| f.applicability == ascribe_core::Applicability::Unsafe)
    );
    let forwarded = fixed(&text, &found[0], "Forward");
    assert!(forwarded.contains("\"reference/options.md\" = \"guides/configure.md\""));
    let retired = fixed(&text, &found[0], "Retire");
    assert!(retired.contains("\"reference/options.md\" = \"why this address was retired\""));
}

#[test]
fn a_lost_id_is_offered_the_same_id_on_another_page() {
    let (found, text) = checked(
        &[(
            "keys.md",
            &page("Keys", "## Rotate\n@id: rotate\n\nRotate."),
        )],
        &[
            ("keys.md", &page("Keys", "## Rotate\n\nRotate.")),
            (
                "security.md",
                &page("Security", "## Rotate\n@id: rotate\n\nRotate."),
            ),
        ],
        &[],
        "",
    );
    assert_eq!(slugs(&found), ["anchor-lost"]);
    let after = fixed(&text, &found[0], "Forward it to the same id");
    assert!(
        after.contains("\"keys.md#rotate\" = \"security.md#rotate\""),
        "{after}"
    );
}

#[test]
fn an_entry_whose_source_is_back_is_advice_whose_fix_removes_it() {
    let (found, text) = checked(
        &[("setup.md", PAGE)],
        &[("setup.md", PAGE), ("install.md", PAGE)],
        &[],
        "[moved]\n\"setup.md\" = \"install.md\"\n",
    );
    assert_eq!(slugs(&found), ["addresses-entry-live"]);
    assert_eq!(found[0].severity, Severity::Advice);
    assert_eq!(
        found[0].location.span.start(),
        text.find("\"setup.md\"").expect("the key")
    );
    assert!(!fixed(&text, &found[0], "Remove").contains("setup.md"));
}

#[test]
fn an_address_another_page_takes_is_an_error() {
    let model_case = r#"[moved]
"/docs/setup" = "install.md"
"#;
    let (found, _) = checked(
        &[("install.md", PAGE)],
        &[("setup.md", PAGE), ("install.md", PAGE)],
        &[],
        model_case,
    );
    assert_eq!(
        messages(&found),
        [(
            "addresses-entry-taken",
            Severity::Error,
            "/docs/setup, an address of `/docs/setup`, is published by setup.md now, so a reader who follows an old link to `/docs/setup` lands on another page"
        )]
    );
}

#[test]
fn an_entry_for_what_the_base_never_published_is_pruned_unless_the_base_has_it() {
    let (found, text) = checked(
        &[("index.md", PAGE)],
        &[("index.md", PAGE), ("new.md", PAGE)],
        &[],
        "[moved]\n\"draft.md\" = \"new.md\"\n",
    );
    assert_eq!(slugs(&found), ["addresses-entry-unpublished"]);
    assert!(!fixed(&text, &found[0], "Remove").contains("draft.md"));
}

#[test]
fn a_value_that_isn_t_a_page_or_a_heading_is_an_error() {
    let (found, _) = checked(
        &[("old.md", PAGE), ("index.md", PAGE)],
        &[("index.md", PAGE), ("_part.md", "Text.\n")],
        &[],
        "[moved]\n\"old.md\" = \"_part.md\"\n\"/docs/older\" = \"index.md#nothing\"\n",
    );
    assert_eq!(
        messages(&found)
            .iter()
            .filter(|m| m.0 == "addresses-entry-not-a-page")
            .map(|m| m.2)
            .collect::<Vec<_>>(),
        [
            "`/docs/older` is forwarded to `index.md#nothing`, and index.md has no heading with the id `nothing`",
            "`old.md` is forwarded to `_part.md`, which isn't a page of the project, so no build writes its redirect",
        ]
    );
}

#[test]
fn a_chain_is_flattened_and_a_cycle_is_said() {
    let (found, text) = checked(
        &[("a.md", PAGE), ("b.md", PAGE), ("c.md", PAGE)],
        &[("c.md", PAGE)],
        &[],
        "[moved]\n\"a.md\" = \"b.md\"\n\"b.md\" = \"c.md\"\n",
    );
    // The chain ends at a page that's published, so the build writes the
    // redirect, and it's only advice.
    assert_eq!(slugs(&found), ["addresses-chain"]);
    assert_eq!(
        found[0].message,
        "`a.md` is forwarded to `b.md`, which is forwarded to `c.md`"
    );
    assert!(fixed(&text, &found[0], "Forward it to where").contains("\"a.md\" = \"c.md\""));
    let (cycle, _) = checked(
        &[("a.md", PAGE), ("b.md", PAGE), ("c.md", PAGE)],
        &[("c.md", PAGE)],
        &[],
        "[moved]\n\"a.md\" = \"b.md\"\n\"b.md\" = \"a.md\"\n",
    );
    assert!(cycle.iter().any(|d| d.slug.as_str() == "addresses-chain"
        && d.message.contains("lead back")
        && d.fixes.is_empty()));
}

#[test]
fn an_old_id_live_where_its_page_leads_collides() {
    let (found, _) = checked(
        &[("old.md", &page("Old", "## Rotate\n@id: rotate\n\nRotate."))],
        &[
            ("new.md", &page("New", "## Rotate\n@id: rotate\n\nAnother.")),
            (
                "keys.md",
                &page("Keys", "## Rotating\n@id: rotating\n\nRotate."),
            ),
        ],
        &[],
        "[moved]\n\"old.md\" = \"new.md\"\n\"old.md#rotate\" = \"keys.md#rotating\"\n",
    );
    assert_eq!(slugs(&found), ["addresses-anchor-collision"]);
}

#[test]
fn many_addresses_forwarded_to_an_index_are_a_soft_404() {
    let (found, _) = checked(
        &[
            ("a.md", PAGE),
            ("b.md", PAGE),
            ("c.md", PAGE),
            ("index.md", PAGE),
        ],
        &[("index.md", PAGE)],
        &[],
        "[moved]\n\"a.md\" = \"index.md\"\n\"b.md\" = \"index.md\"\n\"c.md\" = \"index.md\"\n",
    );
    assert_eq!(
        messages(&found),
        [(
            "addresses-soft-404",
            Severity::Advice,
            "3 addresses are forwarded to index.md, the docs' home page, so a reader who followed a link to one of them lands on a page without what they were looking for"
        )]
    );
}
