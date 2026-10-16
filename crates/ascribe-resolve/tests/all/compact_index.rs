//! The compact index of a build's docs, for a product's `AGENTS.md`
//! (`ascribe_resolve::llms::compact_index`): what it lists, the addresses it
//! gives, and how it stays under its size. `ascribe agents index` writing it
//! is the CLI's test, in `crates/ascribe-cli/tests/all/agents.rs`.

#![allow(clippy::expect_used)]

use ascribe_resolve::llms::{COMPACT_INDEX_LIMIT, CompactIndex, compact_index};
use ascribe_resolve::{AstroRouter, pages};

use crate::build_support::project_with;

const MODEL: &str = "spec = \"0.1\"\n\
    [types.page]\n\
    [types.page.frontmatter]\ntitle = \"string\"\n\
    description = { type = \"string?\", role = \"description\" }\n\
    [consumer]\nsite = \"https://docs.example.com\"\nbase-path = \"/docs/\"\n";

const FILES: &[(&str, &str)] = &[
    (
        "index.md",
        "---\ntype: page\ntitle: Quill\ndescription: Everything about Quill.\n---\n",
    ),
    (
        "guides/install.md",
        "---\ntype: page\ntitle: Install Quill\ndescription: Install it and connect it.\n---\n",
    ),
    (
        "guides/upgrade.md",
        "---\ntype: page\ntitle: Upgrade Quill\ndescription: Move to a new release.\n---\n",
    ),
    (
        "guides/drafts.md",
        "---\ntype: page\ntitle: Drafts\nlisting: unlisted\n---\n",
    ),
    (
        "faq.md",
        "---\ntype: page\ntitle: FAQ\ndescription: Questions people ask.\nlisting: unquoted\n---\n",
    ),
];

/// The index of [`FILES`] under `model`, with `folder` and `limit`.
fn index(model: &str, folder: Option<&str>, limit: usize) -> CompactIndex {
    let project = project_with(model, FILES);
    let build = project.model().build("site").expect("the default build");
    let router = AstroRouter::from_consumer(&project.model().consumer);
    let resolved = project.resolve_build(build, &router);
    let records = pages::records(project.model(), &resolved.pages);
    compact_index(project.model(), build, &records, folder, limit)
}

fn paths(pages: &[ascribe_core::RelPath]) -> Vec<&str> {
    pages.iter().map(|p| p.as_str()).collect()
}

#[test]
fn it_lists_every_offered_page_grouped_as_llms_txt_is() {
    let agents = format!("{MODEL}agents = true\n");
    let index = index(&agents, None, COMPACT_INDEX_LIMIT);
    assert_eq!(
        index.text,
        "## Quill\n\n> Everything about Quill.\n\n\
         ### Pages\n\n\
         - [Quill](https://docs.example.com/docs/index.md): Everything about Quill.\n\
         - [FAQ](https://docs.example.com/docs/faq.md): Questions people ask.\n\n\
         ### Guides\n\n\
         - [Install Quill](https://docs.example.com/docs/guides/install.md): Install it and connect it.\n\
         - [Upgrade Quill](https://docs.example.com/docs/guides/upgrade.md): Move to a new release.\n"
    );
    assert_eq!(
        paths(&index.listed),
        [
            "index.md",
            "faq.md",
            "guides/install.md",
            "guides/upgrade.md"
        ]
    );
    assert!(index.left_out.is_empty() && index.without_description.is_empty());
}

#[test]
fn without_markdown_versions_a_page_s_address_is_its_own() {
    let index = index(MODEL, None, COMPACT_INDEX_LIMIT);
    assert!(
        index
            .text
            .contains("- [Install Quill](https://docs.example.com/docs/guides/install/): "),
        "{}",
        index.text
    );
}

#[test]
fn with_a_folder_each_address_is_a_path_in_it() {
    for (folder, address) in [
        (Some("docs"), "docs/guides/install.md"),
        (Some("../pages/"), "../pages/guides/install.md"),
        (Some("."), "guides/install.md"),
        (Some("my docs"), "<my docs/guides/install.md>"),
    ] {
        let index = index(MODEL, folder, COMPACT_INDEX_LIMIT);
        assert!(
            index
                .text
                .contains(&format!("- [Install Quill]({address}): ")),
            "{folder:?}:\n{}",
            index.text
        );
    }
}

/// Six pages in `guides/`, each with a description of 200 characters.
fn long_files() -> Vec<(String, String)> {
    (1..=6)
        .map(|n| {
            let description = format!("Step {n}. {}", "Read it. ".repeat(22));
            (
                format!("guides/step-{n}.md"),
                format!("---\ntype: page\ntitle: Step {n}\ndescription: {description}\n---\n"),
            )
        })
        .collect()
}

/// The index of [`long_files`] under `model`, with `folder` and `limit`.
fn long_index(model: &str, folder: Option<&str>, limit: usize) -> CompactIndex {
    let files = long_files();
    let files: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    let project = project_with(model, &files);
    let build = project.model().build("site").expect("the default build");
    let router = AstroRouter::from_consumer(&project.model().consumer);
    let resolved = project.resolve_build(build, &router);
    let records = pages::records(project.model(), &resolved.pages);
    compact_index(project.model(), build, &records, folder, limit)
}

#[test]
fn descriptions_are_left_out_from_the_last_page_back_before_any_page() {
    let whole = long_index(MODEL, None, COMPACT_INDEX_LIMIT).text.len();
    // Room for all but about two descriptions, and the sentence saying so.
    let limit = whole - 300;
    let index = long_index(MODEL, None, limit);
    assert!(index.text.len() <= limit, "{}", index.text);
    assert!(index.left_out.is_empty(), "{}", index.text);
    assert_eq!(
        paths(&index.without_description),
        ["guides/step-5.md", "guides/step-6.md"],
        "{}",
        index.text
    );
    assert!(
        index.text.ends_with(&format!(
            "- [Step 6](https://docs.example.com/docs/guides/step-6/)\n\n\
             This index leaves out the descriptions of 2 pages, to stay under {limit} bytes.\n"
        )),
        "{}",
        index.text
    );
}

#[test]
fn pages_are_left_out_from_the_last_back_and_the_index_says_where_they_are() {
    let agents = format!("{MODEL}agents = true\n");
    let index = long_index(&agents, None, 500);
    assert!(index.text.len() <= 500, "{}", index.text);
    assert!(index.without_description.len() == index.listed.len());
    assert_eq!(
        index.listed.len() + index.left_out.len(),
        6,
        "every offered page is listed or left out"
    );
    assert_eq!(
        index.left_out.last().map(|p| p.as_str()),
        Some("guides/step-6.md")
    );
    assert!(
        index.text.contains(&format!(
            "This index leaves out {} pages, and the descriptions of the rest, to stay under 500 bytes. \
             [llms.txt](https://docs.example.com/docs/llms.txt) has every page.",
            index.left_out.len()
        )),
        "{}",
        index.text
    );
    let folder = long_index(&agents, Some("docs"), 350);
    assert!(
        folder.text.contains("`docs/` has every page."),
        "{}",
        folder.text
    );
}
