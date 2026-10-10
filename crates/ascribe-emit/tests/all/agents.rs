//! The outputs for agents (SPEC §9.4, `[consumer] agents = true`): the plain
//! output laid out by URL, `llms.txt`, and the pointer to it on each page of
//! both outputs. How the index is written line by line is
//! `ascribe_resolve::llms`'s; the delivery spec's checker runs on the built
//! example in `examples/astro-site/test/e2e/agent-docs.test.ts`. The checks
//! that measure the index are in `ascribe-check`, and the conformance cases
//! under `tests/conformance/cases/builds/agents/`.

#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::Path;

use ascribe_emit::PlainEmitter;
use ascribe_resolve::Project;

use crate::support::{FULL_MODEL, emit_build, memory_project, memory_project_with_files, site};

/// The full model with the outputs for agents on. Its site is
/// `https://docs.quill.dev`, under the base path `/docs/`.
fn agents_model() -> String {
    let model = FULL_MODEL.replacen("html = true\n", "html = true\nagents = true\n", 1);
    assert_ne!(model, FULL_MODEL, "the full model's [consumer] moved");
    model
}

/// Every file the plain output for agents writes for a build.
fn plain_files(project: &Project, build: &str) -> BTreeMap<String, String> {
    emit_build(
        Path::new("/nowhere"),
        project,
        build,
        &PlainEmitter::new(project.model()),
    )
}

/// The URLs `llms.txt` links to, in order.
fn listed(llms: &str) -> Vec<&str> {
    llms.lines()
        .filter_map(|line| line.strip_prefix("- ["))
        .filter_map(|line| line.split_once("](").map(|(_, rest)| rest))
        .filter_map(|rest| rest.split_once(')').map(|(url, _)| url))
        .collect()
}

#[test]
fn each_page_is_published_at_its_route_with_md_and_points_at_the_index() {
    let project = memory_project(
        &agents_model(),
        &[
            ("index.md", "---\ntitle: Home\n---\n\nWelcome.\n"),
            ("Guides/My Setup.md", "---\ntitle: Set up\n---\n\nSteps.\n"),
        ],
    );
    let files = plain_files(&project, "site");
    let pages: Vec<&str> = files
        .keys()
        .map(String::as_str)
        .filter(|p| p.ends_with(".md"))
        .collect();
    assert_eq!(pages, ["guides/my-setup.md", "index.md"]);
    assert_eq!(
        files["guides/my-setup.md"],
        "> For the complete documentation index, see [llms.txt](https://docs.quill.dev/docs/llms.txt).\n\n# Set up\n\nSteps.\n"
    );
}

#[test]
fn llms_txt_lists_exactly_the_pages_a_build_publishes() {
    let project = memory_project(
        &agents_model(),
        &[
            (
                "index.md",
                "---\ntitle: Home\ndescription: Quill's docs.\n---\n",
            ),
            ("install.md", "---\ntitle: Install\n---\n"),
            (
                "streaming.md",
                "---\ntitle: Streaming\navailable: self-managed 3.4\n---\n",
            ),
        ],
    );
    for (build, expected) in [
        ("site", vec!["index.md", "install.md", "streaming.md"]),
        ("cloud", vec!["index.md", "install.md"]),
    ] {
        let files = plain_files(&project, build);
        let urls: Vec<String> = expected
            .iter()
            .map(|p| format!("https://docs.quill.dev/docs/{p}"))
            .collect();
        assert_eq!(listed(&files["llms.txt"]), urls, "in build {build}");
        // Each URL is a page the build wrote.
        for page in expected {
            assert!(files.contains_key(page), "{build} doesn't write {page}");
        }
    }
}

#[test]
fn llms_txt_is_named_and_summarized_by_the_home_page_and_sectioned_by_folder() {
    let project = memory_project(
        &agents_model(),
        &[
            (
                "index.md",
                "---\ntitle: Quill\ndescription: Docs for *Quill*, the [editor](https://quill.dev).\n---\n",
            ),
            ("install.md", "---\ntitle: Install\n---\n"),
            ("guides/index.md", "---\ntitle: How-to guides\n---\n"),
            (
                "guides/sync.md",
                "---\ntitle: Sync\ndescription: Keep two copies the same.\n---\n",
            ),
        ],
    );
    let files = plain_files(&project, "site");
    assert_eq!(
        files["llms.txt"],
        "# Quill\n\n> Docs for Quill, the editor.\n\n\
         ## Pages\n\n\
         - [Quill](https://docs.quill.dev/docs/index.md): Docs for Quill, the editor.\n\
         - [Install](https://docs.quill.dev/docs/install.md)\n\n\
         ## How-to guides\n\n\
         - [How-to guides](https://docs.quill.dev/docs/guides.md)\n\
         - [Sync](https://docs.quill.dev/docs/guides/sync.md): Keep two copies the same.\n"
    );
}

#[test]
fn an_index_over_the_limit_is_split_into_a_file_per_section() {
    let long = "x".repeat(1_000);
    let mut sources = vec![("index.md".to_owned(), "---\ntitle: Home\n---\n".to_owned())];
    for n in 0..60 {
        sources.push((
            format!("reference/page-{n:02}.md"),
            format!("---\ntitle: Page {n} {long}\n---\n"),
        ));
    }
    let refs: Vec<(&str, &str)> = sources
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    let project = memory_project(&agents_model(), &refs);
    let files = plain_files(&project, "site");
    let root = &files["llms.txt"];
    assert!(
        root.contains("## Sections\n\n- [Reference](https://docs.quill.dev/docs/reference/llms.txt): 60 pages\n"),
        "{root}"
    );
    assert_eq!(listed(root).len(), 2, "the home page and the section");
    let section = &files["reference/llms.txt"];
    assert!(section.starts_with("# Reference\n\n"), "{section}");
    assert_eq!(listed(section).len(), 60);
}

#[test]
fn an_asset_is_published_under_files_at_its_absolute_url() {
    let project = memory_project_with_files(
        &agents_model(),
        &[(
            "guides/sync.md",
            "---\ntitle: Sync\n---\n\n![Diagram](diagram.png)\n",
        )],
        &[("docs/guides/diagram.png", "PNG")],
    );
    let files = plain_files(&project, "site");
    assert!(files.contains_key("_ascribe/files/guides/diagram.png"));
    assert!(
        files["guides/sync.md"]
            .contains("![Diagram](https://docs.quill.dev/docs/_ascribe/files/guides/diagram.png)"),
        "{}",
        files["guides/sync.md"]
    );
}

#[test]
fn without_agents_the_plain_output_is_laid_out_by_source() {
    let project = memory_project(
        FULL_MODEL,
        &[("Guides/My Setup.md", "---\ntitle: Set up\n---\n")],
    );
    let files = plain_files(&project, "site");
    assert_eq!(
        files.keys().collect::<Vec<_>>(),
        ["Guides/My Setup.md"],
        "no llms.txt, and the source's path"
    );
    assert_eq!(files["Guides/My Setup.md"], "# Set up\n");
}

#[test]
fn a_site_page_opens_with_the_pointer_for_agents() {
    let source = "---\ntitle: Set up\n---\n\nSteps.\n";
    let project = memory_project(&agents_model(), &[("Guides/My Setup.md", source)]);
    let page = site(&project, "site", "Guides/My Setup.md");
    let body = page
        .split_once("---\n\n")
        .map(|(_, b)| b)
        .unwrap_or_default();
    assert_eq!(
        body,
        "<ascribe-for-agents>\n\nFor AI agents: the documentation index is at [llms.txt](/docs/llms.txt), and this page is available as [Markdown](/docs/guides/my-setup.md).\n\n</ascribe-for-agents>\n\nSteps.\n",
        "{page}"
    );
    let off = memory_project(FULL_MODEL, &[("Guides/My Setup.md", source)]);
    assert!(!site(&off, "site", "Guides/My Setup.md").contains("ascribe-for-agents"));
}
