//! The addresses each build publishes (SPEC §7.10): each page's route, its
//! Markdown version's with `agents`, and each explicit `@id`'s
//! `route#id`, with the source `ascribe.addresses` records it by; each
//! permalink's address, and each `llms.txt` file's; and the settings they
//! were made with.

#![allow(clippy::expect_used, clippy::panic)]

use crate::build_support::{path, project_with};

use std::path::PathBuf;
use std::sync::Arc;

use ascribe_core::FileId;
use ascribe_resolve::addresses::{
    AddressKind, BuildAddresses, Settings, build_addresses, every_build, rebase,
};
use ascribe_resolve::{DiskFs, Layout, MemoryFs, Project};

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"
output-dir = ".ascribe/build"

[consumer]
profile = "astro"
site = "https://docs.example.com"
base-path = "/docs/"
trailing-slash = "never"

[builds.site]
variants = "switch"
availability = "badge"
"#;

const PAGE: &str = "---\ntype: page\ntitle: Page\n---\n\n";

fn listed(addresses: &BuildAddresses) -> Vec<(String, AddressKind, String)> {
    addresses
        .addresses
        .iter()
        .map(|a| (a.address.clone(), a.kind, a.source.clone()))
        .collect()
}

fn site(project: &Project) -> BuildAddresses {
    build_addresses(project, project.model().build("site").expect("the build"))
}

#[test]
fn a_page_has_its_route_its_markdown_and_its_explicit_ids() {
    let project = project_with(
        MODEL,
        &[(
            "guides/setup.md",
            &format!("{PAGE}## Install\n@id: install\n\nText.\n\n## Generated\n\nText.\n"),
        )],
    );
    let site = site(&project);
    assert_eq!(
        listed(&site),
        [
            (
                "/docs/guides/setup".to_owned(),
                AddressKind::Page,
                "guides/setup.md".to_owned()
            ),
            (
                "/docs/guides/setup.md".to_owned(),
                AddressKind::Markdown,
                "guides/setup.md".to_owned()
            ),
            (
                "/docs/guides/setup#install".to_owned(),
                AddressKind::Anchor,
                "guides/setup.md#install".to_owned()
            ),
            (
                "/docs/llms.txt".to_owned(),
                AddressKind::Llms,
                "llms.txt".to_owned()
            ),
        ]
    );
    assert!(site.has("/docs/guides/setup#install"));
    assert!(!site.has("/docs/guides/setup#generated"));
    assert!(site.publishes(&path("guides/setup.md")));
}

#[test]
fn without_agents_there_is_no_markdown_version() {
    let model = MODEL.replace(
        "trailing-slash = \"never\"",
        "trailing-slash = \"never\"\nagents = false",
    );
    let project = project_with(&model, &[("index.md", PAGE)]);
    assert_eq!(
        listed(&site(&project)),
        [("/docs".to_owned(), AddressKind::Page, "index.md".to_owned())]
    );
}

#[test]
fn a_fragments_id_is_one_source_on_every_page_that_includes_it() {
    let project = project_with(
        MODEL,
        &[
            ("a.md", &format!("{PAGE}@include: _shared.md\n")),
            ("b.md", &format!("{PAGE}@include: _shared.md\n")),
            (
                "_shared.md",
                "## Requirements\n@id: requirements\n\nText.\n",
            ),
        ],
    );
    let anchors: Vec<(String, String)> = site(&project)
        .addresses
        .into_iter()
        .filter(|a| a.kind == AddressKind::Anchor)
        .map(|a| (a.address, a.source))
        .collect();
    assert_eq!(
        anchors,
        [
            (
                "/docs/a#requirements".to_owned(),
                "_shared.md#requirements".to_owned()
            ),
            (
                "/docs/b#requirements".to_owned(),
                "_shared.md#requirements".to_owned()
            ),
        ]
    );
}

/// Quill's builds, each by its name, a line's as `site@3.4`.
#[test]
fn every_build_of_quill_is_listed_by_name() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let model = ascribe_model::load(root.join("ascribe.toml")).expect("the Quill model loads");
    let layout = Layout::from_model(&model);
    let fs = DiskFs::new(&root, &layout);
    let project = Project::load(Arc::new(model), layout, &fs);
    let builds = every_build(&project);
    let names: Vec<&str> = builds.iter().map(|b| b.build.as_str()).collect();
    assert_eq!(names, ["site", "site@3.4", "cloud", "self-managed-3.3"]);
    let pages = |build: &BuildAddresses| -> Vec<String> {
        build
            .addresses
            .iter()
            .filter(|a| a.kind == AddressKind::Page)
            .map(|a| a.address.clone())
            .collect()
    };
    // An older line's addresses are under its folder.
    assert_eq!(
        pages(&builds[1]),
        [
            "/3.4/configure-legacy/",
            "/3.4/install-agent/",
            "/3.4/keys/",
            "/3.4/quickstart/",
            "/3.4/upgrade/",
        ]
    );
    assert_eq!(
        pages(&builds[2]),
        ["/install-agent/", "/keys/", "/quickstart/"]
    );
}

#[test]
fn each_name_a_build_answers_to_has_an_address_under_the_permalinks_path() {
    let model = Arc::new(ascribe_model::load_str(MODEL, FileId::new(0)).expect("loads"));
    let layout = Layout::from_model(&model);
    let fs = MemoryFs::new(&layout)
        .with_source(
            "guides/setup.md",
            "---\ntype: page\ntitle: Page\npermalink: setup\n---\n\nText.\n",
        )
        .with_file(
            "ascribe.addresses",
            "version = 1\n\n[moved]\n\"go:setting-up\" = \"go:setup\"\n\n[retired]\n\"go:old-name\" = \"Gone.\"\n",
        );
    let project = Project::load(model, layout, &fs);
    let permalinks: Vec<(String, String, String)> = site(&project)
        .addresses
        .into_iter()
        .filter(|a| a.kind == AddressKind::Permalink)
        .map(|a| (a.address, a.source, a.page.to_string()))
        .collect();
    assert_eq!(
        permalinks,
        [
            (
                "/docs/go/old-name".to_owned(),
                "old-name".to_owned(),
                "index.md".to_owned()
            ),
            (
                "/docs/go/setting-up".to_owned(),
                "setting-up".to_owned(),
                "guides/setup.md".to_owned()
            ),
            (
                "/docs/go/setup".to_owned(),
                "setup".to_owned(),
                "guides/setup.md".to_owned()
            ),
        ]
    );
}

#[test]
fn a_build_s_settings_are_the_model_s_and_its_line_s_base() {
    let project = project_with(MODEL, &[("index.md", PAGE)]);
    assert_eq!(
        site(&project).settings,
        Settings {
            base_path: "/docs/".to_owned(),
            base: "/docs/".to_owned(),
            permalink_path: None,
            permalinks: "/docs/go/".to_owned(),
            agents: true,
            trailing_slash: false,
        }
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let model = ascribe_model::load(root.join("ascribe.toml")).expect("the Quill model loads");
    let layout = Layout::from_model(&model);
    let fs = DiskFs::new(&root, &layout);
    let quill = Project::load(Arc::new(model), layout, &fs);
    let line = every_build(&quill)
        .into_iter()
        .find(|b| b.build == "site@3.4")
        .expect("the line");
    assert_eq!(
        line.settings.base,
        format!("{}3.4/", line.settings.base_path)
    );
}

#[test]
fn an_address_is_rebased_from_one_path_to_another() {
    assert_eq!(
        rebase("/docs/guides/setup", "/docs/", "/handbook/").as_deref(),
        Some("/handbook/guides/setup")
    );
    assert_eq!(
        rebase("/docs", "/docs/", "/handbook/").as_deref(),
        Some("/handbook")
    );
    assert_eq!(rebase("/docs", "/docs/", "/").as_deref(), Some("/"));
    assert_eq!(rebase("/other/a", "/docs/", "/handbook/"), None);
}

#[test]
fn an_incremental_project_follows_ascribe_addresses_for_what_it_records_of_permalinks() {
    let model = Arc::new(ascribe_model::load_str(MODEL, FileId::new(0)).expect("loads"));
    let layout = Layout::from_model(&model);
    let fs = MemoryFs::new(&layout).with_source(
        "guides/setup.md",
        "---\ntype: page\ntitle: Page\npermalink: setup\n---\n\nText.\n",
    );
    let mut project = ascribe_resolve::IncrementalProject::load(model, layout, fs);
    let names = |p: &ascribe_resolve::IncrementalProject| -> Vec<String> {
        p.snapshot()
            .recorded_permalinks()
            .names
            .into_iter()
            .map(|n| n.name)
            .collect()
    };
    assert!(names(&project).is_empty());
    let file = ascribe_model::AddressFile::parse(
        "version = 1\n\n[moved]\n\"go:setting-up\" = \"go:setup\"\n",
        FileId::new(0),
    )
    .expect("it reads");
    let affected = project
        .apply([ascribe_resolve::Change::Addresses(Some(Arc::new(
            file.clone(),
        )))])
        .expect("it applies");
    assert!(!affected.is_empty(), "a new snapshot");
    assert!(affected.recheck.is_empty() && affected.re_resolve.is_empty());
    assert_eq!(names(&project), ["setting-up"]);
    // The same file again changes nothing.
    let again = project
        .apply([ascribe_resolve::Change::Addresses(Some(Arc::new(file)))])
        .expect("it applies");
    assert!(again.is_empty());
    project
        .apply([ascribe_resolve::Change::Addresses(None)])
        .expect("it applies");
    assert!(names(&project).is_empty());
}
