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
    let model = FULL_MODEL.replacen("agents = false\n", "agents = true\n", 1);
    assert_ne!(model, FULL_MODEL, "the full model's [consumer] moved");
    model
}

/// What the full model's builds that aren't versioned say they describe,
/// after `llms.txt`'s summary and each page's pointer (decision #1028).
const RELEASES: &str = " This documentation describes Self-managed 3.5 and 3.4. \
    Anything marked planned isn't in any of those releases yet.";

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
            (
                "index.md",
                "---\ntype: guide\ntitle: Home\n---\n\nWelcome.\n",
            ),
            (
                "Guides/My Setup.md",
                "---\ntype: guide\ntitle: Set up\n---\n\nSteps.\n",
            ),
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
        format!(
            "> For the complete documentation index, see [llms.txt](https://docs.quill.dev/docs/llms.txt).{RELEASES}\n\n\
             # Set up\n\n> Guide\n\nSteps.\n"
        )
    );
}

#[test]
fn llms_txt_lists_exactly_the_pages_a_build_publishes() {
    let project = memory_project(
        &agents_model(),
        &[
            (
                "index.md",
                "---\ntype: guide\ntitle: Home\ndescription: Quill's docs.\n---\n",
            ),
            ("install.md", "---\ntype: guide\ntitle: Install\n---\n"),
            (
                "streaming.md",
                "---\ntype: guide\ntitle: Streaming\navailable: self-managed 3.4\n---\n",
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
fn llms_txt_is_named_and_summarized_by_the_home_page_and_sectioned_by_the_navigation() {
    let project = memory_project(
        &agents_model(),
        &[
            (
                "index.md",
                "---\ntype: guide\ntitle: Quill\ndescription: Docs for *Quill*, the [editor](https://quill.dev).\n---\n",
            ),
            ("install.md", "---\ntype: guide\ntitle: Install\n---\n"),
            (
                "guides/index.md",
                "---\ntype: guide\ntitle: How-to guides\n---\n",
            ),
            (
                "guides/sync.md",
                "---\ntype: guide\ntitle: Sync\ndescription: Keep two copies the same.\n---\n",
            ),
        ],
    );
    let files = plain_files(&project, "site");
    assert_eq!(
        files["llms.txt"],
        format!(
            "# Quill\n\n> Docs for Quill, the editor.{RELEASES}\n\n\
             ## Start\n\n\
             - [Quill](https://docs.quill.dev/docs/index.md): Docs for Quill, the editor.\n\
             - [Install](https://docs.quill.dev/docs/install.md)\n\n\
             ## How-to guides\n\n\
             - [How-to guides](https://docs.quill.dev/docs/guides.md)\n\
             - [Sync](https://docs.quill.dev/docs/guides/sync.md): Keep two copies the same.\n"
        )
    );
}

#[test]
fn llms_txt_follows_ascribe_nav_with_nested_sections_folded_in_reading_order() {
    let page = |title: &str| format!("---\ntype: guide\ntitle: {title}\nnav-label: Short\n---\n");
    let sources = [
        ("index.md", page("Home")),
        ("install.md", page("Install")),
        ("about.md", page("About")),
        ("guides/index.md", page("Guides")),
        ("guides/sync.md", page("Sync")),
        ("guides/backup.md", page("Back up")),
        ("guides/backup/restore.md", page("Restore")),
        ("guides/zebra.md", page("Zebra")),
        ("reference/options.md", page("Options")),
    ];
    let refs: Vec<(&str, &str)> = sources.iter().map(|(p, t)| (*p, t.as_str())).collect();
    let nav = "version = 1\n\n\
        [[sections]]\nfolder = \"\"\npages = [\"install.md\"]\n\n\
        [[sections]]\nfolder = \"reference/\"\nlabel = \"Look it up\"\n\n\
        [[sections]]\nfolder = \"guides/\"\npages = [\"zebra.md\", \"backup.md\"]\n";
    let project = memory_project_with_files(&agents_model(), &refs, &[("ascribe.nav", nav)]);
    let files = plain_files(&project, "site");
    let url = |p: &str| format!("https://docs.quill.dev/docs/{p}");
    assert_eq!(
        files["llms.txt"],
        format!(
            "# Home\n\n> Every page of this documentation, each linked to its Markdown version.{RELEASES}\n\n\
             ## Start\n\n\
             - [Home]({})\n- [Install]({})\n- [About]({})\n\n\
             ## Look it up\n\n\
             - [Options]({})\n\n\
             ## Guides\n\n\
             - [Guides]({})\n- [Zebra]({})\n- [Back up]({})\n- [Restore]({})\n- [Sync]({})\n",
            url("index.md"),
            url("install.md"),
            url("about.md"),
            url("reference/options.md"),
            url("guides.md"),
            url("guides/zebra.md"),
            url("guides/backup.md"),
            url("guides/backup/restore.md"),
            url("guides/sync.md"),
        ),
        "the titles, not the `nav-label`s, in the file's order"
    );
}

#[test]
fn an_index_over_the_limit_is_split_into_a_file_per_section() {
    let long = "x".repeat(1_000);
    let mut sources = vec![(
        "index.md".to_owned(),
        "---\ntype: guide\ntitle: Home\n---\n".to_owned(),
    )];
    for n in 0..60 {
        sources.push((
            format!("reference/page-{n:02}.md"),
            format!("---\ntype: guide\ntitle: Page {n} {long}\n---\n"),
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
            "---\ntype: guide\ntitle: Sync\n---\n\n![Diagram](diagram.png)\n",
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
        &[(
            "Guides/My Setup.md",
            "---\ntype: guide\ntitle: Set up\n---\n",
        )],
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
    let source = "---\ntype: guide\ntitle: Set up\n---\n\nSteps.\n";
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

/// A page whose headings' anchors in its Markdown differ from their page ids
/// in each way they can: an `@id`, the label a heading in a variant arm ends
/// with, and the numbering those labels change.
const SYNC: &str = "---\ntype: guide\ntitle: Sync\n---\n\n\
    ## Set up\n\nFirst.\n\n\
    ## Connect\n@id: link-up\n\nThen.\n\n\
    @variant {pm=npm}:\n## Install\nnpm i\n@variant {pm=yarn}:\n## Install\nyarn add\n@end\n\n\
    ## Set up\n\nAgain.\n";

/// Each heading of a Markdown document, in order, as the anchor a reader
/// gives it (comrak's, which is GitHub's) and its text.
fn anchored_headings(markdown: &str) -> Vec<(String, String)> {
    use comrak_ascribe::nodes::NodeValue;
    let arena = comrak_ascribe::Arena::new();
    let root =
        comrak_ascribe::parse_document(&arena, markdown, &comrak_ascribe::Options::default());
    let mut anchorizer = comrak_ascribe::Anchorizer::new();
    root.descendants()
        .filter(|n| matches!(n.data.borrow().value, NodeValue::Heading(_)))
        .map(|heading| {
            let mut text = String::new();
            for node in heading.descendants() {
                match &node.data.borrow().value {
                    NodeValue::Text(t) => text.push_str(t),
                    NodeValue::Code(c) => text.push_str(&c.literal),
                    _ => {}
                }
            }
            (anchorizer.anchorize(&text), text)
        })
        .collect()
}

#[test]
fn a_link_to_a_page_leads_to_its_markdown_and_its_fragment_to_the_same_heading_there() {
    let project = memory_project(
        &agents_model(),
        &[
            (
                "index.md",
                "---\ntype: guide\ntitle: Home\n---\n\n\
                 [Sync](guides/sync.md), [first](guides/sync.md#set-up), \
                 [connect](guides/sync.md#link-up), [npm](guides/sync.md#install), \
                 [Yarn](guides/sync.md#install-1), [again](guides/sync.md#set-up-1). \
                 Rotate your API key.\n",
            ),
            ("guides/sync.md", SYNC),
            (
                "reference/glossary.md",
                "---\ntype: guide\ntitle: Glossary\n---\n\n## API key\n@term\n\nA secret token.\n",
            ),
        ],
    );
    let files = plain_files(&project, "site");
    let sync = "https://docs.quill.dev/docs/guides/sync.md";
    assert_eq!(
        files["index.md"],
        format!(
            "> For the complete documentation index, see [llms.txt](https://docs.quill.dev/docs/llms.txt).{RELEASES}\n\n\
             # Home\n\n> Guide\n\n\
             [Sync]({sync}), [first]({sync}#set-up), [connect]({sync}#connect), \
             [npm]({sync}#install-npm), [Yarn]({sync}#install-yarn), [again]({sync}#set-up-1). \
             Rotate your [API key](https://docs.quill.dev/docs/reference/glossary.md#api-key).\n"
        )
    );
    // Each fragment is the anchor a Markdown reader gives the heading the
    // link names: the one its page id names on the page.
    let headings = anchored_headings(&files["guides/sync.md"]);
    let text_at = |anchor: &str| {
        let at = headings.iter().position(|(a, _)| a == anchor);
        at.map(|at| (at, headings[at].1.as_str()))
    };
    assert_eq!(text_at("set-up"), Some((1, "Set up")));
    assert_eq!(text_at("connect"), Some((2, "Connect")));
    assert_eq!(text_at("install-npm"), Some((4, "Install (npm)")));
    assert_eq!(text_at("install-yarn"), Some((6, "Install (Yarn)")));
    assert_eq!(text_at("set-up-1"), Some((7, "Set up")));
    let glossary = anchored_headings(&files["reference/glossary.md"]);
    assert_eq!(glossary[1], ("api-key".to_owned(), "API key".to_owned()));
}

#[test]
fn without_agents_a_link_keeps_the_pages_address() {
    let project = memory_project(
        FULL_MODEL,
        &[
            (
                "index.md",
                "---\ntype: guide\ntitle: Home\n---\n\n[connect](guides/sync.md#link-up)\n",
            ),
            ("guides/sync.md", SYNC),
        ],
    );
    let files = plain_files(&project, "site");
    assert!(
        files["index.md"].contains("(https://docs.quill.dev/docs/guides/sync#link-up)"),
        "{}",
        files["index.md"]
    );
}

#[test]
fn a_page_s_markdown_says_its_type_and_description_after_its_title() {
    let project = memory_project(
        &agents_model(),
        &[
            ("index.md", "---\ntype: guide\ntitle: Home\n---\n"),
            (
                "guides/sync.md",
                "---\ntype: guide\ntitle: Sync\navailable: cloud\n\
                 description: Keep *two* copies the same.\n---\n\nSteps.\n",
            ),
            (
                "reference/options.md",
                "---\ntype: reference\ntitle: Options\n---\n\nEach option.\n",
            ),
        ],
    );
    let files = plain_files(&project, "site");
    let sync = &files["guides/sync.md"];
    let after_pointer = |page: &str| page.split_once("\n\n").map(|(_, rest)| rest.to_owned());
    // After the availability line, as the site shows it, and as plain text,
    // as llms.txt lists it.
    assert_eq!(
        after_pointer(sync).as_deref(),
        Some(
            "# Sync\n\nAvailable: Quill Cloud (GA)\n\n> Guide · Keep two copies the same.\n\nSteps.\n"
        ),
        "{sync}"
    );
    // A type with no description field says the type alone.
    assert_eq!(
        after_pointer(&files["reference/options.md"]).as_deref(),
        Some("# Options\n\n> Reference\n\nEach option.\n")
    );
}

#[test]
fn a_build_that_filters_for_a_version_says_it_describes_that_version() {
    let project = memory_project(
        &agents_model(),
        &[("index.md", "---\ntype: guide\ntitle: Home\n---\n")],
    );
    let pointer = |build: &str| {
        let files = plain_files(&project, build);
        let page = files["index.md"].lines().next().map(str::to_owned);
        let summary = files["llms.txt"].lines().nth(2).map(str::to_owned);
        (page, summary)
    };
    let (page, summary) = pointer("self-managed-3.5");
    assert_eq!(
        page.as_deref(),
        Some(
            "> For the complete documentation index, see [llms.txt](https://docs.quill.dev/docs/llms.txt). \
             This documentation describes Self-managed 3.5."
        )
    );
    assert_eq!(
        summary.as_deref(),
        Some(
            "> Every page of this documentation, each linked to its Markdown version. \
             This documentation describes Self-managed 3.5."
        )
    );
    // Filtered for a target with no lines, it names no release: what
    // another target's releases have is gone.
    let (page, summary) = pointer("cloud");
    assert_eq!(
        page.as_deref(),
        Some(
            "> For the complete documentation index, see [llms.txt](https://docs.quill.dev/docs/llms.txt)."
        )
    );
    assert_eq!(
        summary.as_deref(),
        Some("> Every page of this documentation, each linked to its Markdown version.")
    );
}

#[test]
fn a_project_without_types_or_releases_writes_nothing_more() {
    let model = "spec = \"0.1\"\n\
        [consumer]\nsite = \"https://docs.example.com\"\nagents = true\n";
    let project = memory_project(model, &[("index.md", "---\ntitle: Home\n---\n\nHello.\n")]);
    let files = plain_files(&project, "site");
    assert_eq!(
        files["index.md"],
        "> For the complete documentation index, see [llms.txt](https://docs.example.com/llms.txt).\n\n\
         # Home\n\nHello.\n"
    );
    assert!(
        files["llms.txt"].starts_with("# Home\n\n> Every page"),
        "{}",
        files["llms.txt"]
    );
}
