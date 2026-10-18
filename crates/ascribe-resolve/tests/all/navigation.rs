//! The navigation (SPEC §7.8): the default order, `ascribe.nav`'s order over
//! it, and a build's share.

#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use ascribe_core::FileId;
use ascribe_resolve::{
    BuildNavigation, DefaultRouter, Layout, MemoryFs, Project, Section, SharedSection,
};

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"

[types.guide.frontmatter]
title = { type = "string", inline = "code" }

[types.reference.frontmatter]
title = "string"

[types.concept.frontmatter]
title = { type = "string", inline = "code", phrases = true }

[types.note.frontmatter]
title = { type = "string", phrases = true }

[phrases]
product = "Loom"

[dimensions.deployment]
values = ["cloud", "self-managed"]

[builds.site]

[builds.cloud]
variants = { deployment = "cloud" }
"#;

fn page(kind: &str, title: &str) -> String {
    format!("---\ntype: {kind}\ntitle: {title}\n---\n\nText.\n")
}

fn project(files: &[(&str, String)], nav: Option<&str>) -> Project {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("the model loads");
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in files {
        fs = fs.with_source(path, text);
    }
    if let Some(nav) = nav {
        fs = fs.with_file("ascribe.nav", nav);
    }
    Project::load(Arc::new(model), layout, &fs)
}

/// A section as lines: its folder and own page, then each page and nested
/// section in the order they're placed, indented.
fn outline(section: &Section) -> Vec<String> {
    let mut out = Vec::new();
    lines(section, 0, &mut out);
    out
}

fn lines(section: &Section, depth: usize, out: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    let own = section.page.as_ref().map_or("-", |p| p.as_str());
    out.push(format!("{pad}[{}] {own}", section.folder));
    for page in &section.pages {
        out.push(format!("{pad}  {page}"));
        for nested in section
            .sections
            .iter()
            .filter(|s| s.page.as_ref() == Some(page))
        {
            lines(nested, depth + 1, out);
        }
    }
    for nested in &section.sections {
        let is_placed = nested
            .page
            .as_ref()
            .is_some_and(|p| section.pages.contains(p));
        if !is_placed {
            lines(nested, depth + 1, out);
        }
    }
}

/// A build's share as lines: each section's label and own page, then its
/// pages and nested sections.
fn shared(navigation: &BuildNavigation<'_>) -> Vec<String> {
    let mut out = Vec::new();
    shared_lines(&navigation.root, 0, &mut out);
    out
}

fn shared_lines(section: &SharedSection<'_>, depth: usize, out: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    let own = section.page.as_ref().map_or("-", |p| p.page.path.as_str());
    out.push(format!("{pad}{} ({own})", section.label));
    for page in &section.pages {
        out.push(format!(
            "{pad}  {} {} {}",
            page.page.path,
            page.label,
            page.type_name.unwrap_or("-")
        ));
    }
    for nested in &section.sections {
        shared_lines(nested, depth + 1, out);
    }
}

fn files() -> Vec<(&'static str, String)> {
    vec![
        ("index.md", page("guide", "Home")),
        ("Guides/My Setup.md", page("guide", "My setup")),
        ("Guides/Advanced.md", page("guide", "Advanced")),
        ("Guides/api.md", page("reference", "API")),
        ("Reference/Options.md", page("reference", "Options")),
        ("_fragments/intro.md", "Included.\n".to_owned()),
    ]
}

#[test]
fn the_default_order() {
    let p = project(&files(), None);
    assert_eq!(
        outline(&p.navigation().root),
        [
            "[] index.md",
            "  [Guides/] -",
            "    Guides/Advanced.md",
            "    Guides/My Setup.md",
            // A type declared after `guide` comes after its pages.
            "    Guides/api.md",
            "  [Reference/] -",
            "    Reference/Options.md",
        ]
    );
}

#[test]
fn the_files_order_over_the_default() {
    let nav = "version = 1\n\n[[sections]]\nfolder = \"\"\n\n[[sections]]\nfolder = \"Reference/\"\n\n[[sections]]\nfolder = \"Guides/\"\npages = [\"My Setup.md\", \"Nowhere.md\", \"sub/x.md\"]\nlabel = \"How-to\"\n";
    let p = project(&files(), Some(nav));
    let navigation = p.navigation();
    assert_eq!(
        outline(&navigation.root),
        [
            "[] index.md",
            "  [Reference/] -",
            "    Reference/Options.md",
            "  [Guides/] -",
            // Listed first; the rest by title, whatever their type. Names
            // that match nothing are passed over.
            "    Guides/My Setup.md",
            "    Guides/Advanced.md",
            "    Guides/api.md",
        ]
    );
    let guides = &navigation.root.sections[1];
    assert!(guides.listed && navigation.root.listed);
    assert_eq!(guides.label.as_deref(), Some("How-to"));
}

#[test]
fn a_file_that_doesnt_read_is_the_default_order() {
    let p = project(&files(), Some("version = 2\n"));
    assert!(p.nav_file().is_none());
    assert_eq!(p.navigation(), project(&files(), None).navigation());
}

#[test]
fn a_section_not_listed_comes_after_the_listed_ones() {
    let mut files = files();
    files.push(("Concepts/Model.md", page("guide", "Model")));
    let nav = "version = 1\n[[sections]]\nfolder = \"Reference/\"\n";
    let p = project(&files, Some(nav));
    let folders: Vec<String> = p
        .navigation()
        .root
        .sections
        .iter()
        .map(|s| s.folder.clone())
        .collect();
    assert_eq!(folders, ["Reference/", "Concepts/", "Guides/"]);
}

#[test]
fn a_nested_section_with_its_page_beside_its_folder_sits_where_the_page_is() {
    let files = vec![
        ("checks/check.md", page("guide", "Check")),
        ("checks/diagnostics.md", page("guide", "Diagnostics")),
        ("checks/diagnostics/pages.md", page("guide", "Pages")),
        ("checks/diagnostics/content.md", page("guide", "Content")),
        ("checks/vale.md", page("guide", "Vale")),
        ("checks/more/index.md", page("guide", "More")),
        ("checks/more/one.md", page("guide", "One")),
    ];
    let p = project(&files, None);
    assert_eq!(
        outline(&p.navigation().root),
        [
            "[] -",
            "  [checks/] -",
            "    checks/check.md",
            "    checks/diagnostics.md",
            "    [checks/diagnostics/] checks/diagnostics.md",
            "      checks/diagnostics/content.md",
            "      checks/diagnostics/pages.md",
            "    checks/vale.md",
            // One with only an `index.md` comes after the pages.
            "    [checks/more/] checks/more/index.md",
            "      checks/more/one.md",
        ]
    );
    // Named in the parent's `pages`, the page places the section.
    let nav = "version = 1\n[[sections]]\nfolder = \"checks/\"\npages = [\"vale.md\", \"diagnostics.md\"]\n";
    let p = project(&files, Some(nav));
    assert_eq!(
        outline(&p.navigation().root),
        [
            "[] -",
            "  [checks/] -",
            "    checks/vale.md",
            "    checks/diagnostics.md",
            "    [checks/diagnostics/] checks/diagnostics.md",
            "      checks/diagnostics/content.md",
            "      checks/diagnostics/pages.md",
            "    checks/check.md",
            "    [checks/more/] checks/more/index.md",
            "      checks/more/one.md",
        ]
    );
}

#[test]
fn titles_sort_as_a_reader_sees_them() {
    let p = project(
        &[
            ("Guides/zeta.md", page("concept", "\"`zeta.yaml` options\"")),
            ("Guides/apply.md", page("concept", "Apply")),
            (
                "Guides/overview.md",
                page("concept", "\"{product} overview\""),
            ),
            ("Guides/b-plain.md", page("concept", "b")),
            ("Guides/b-code.md", page("concept", "\"`b`\"")),
            // Markup a title doesn't read is its text.
            ("Notes/zeta.md", page("note", "\"`zeta`\"")),
            ("Notes/overview.md", page("note", "\"{product} overview\"")),
            ("Notes/apply.md", page("note", "Apply")),
        ],
        None,
    );
    assert_eq!(
        outline(&p.navigation().root),
        [
            "[] -",
            "  [Guides/] -",
            "    Guides/apply.md",
            // The same plain text, then as written.
            "    Guides/b-code.md",
            "    Guides/b-plain.md",
            "    Guides/overview.md",
            "    Guides/zeta.md",
            "  [Notes/] -",
            "    Notes/zeta.md",
            "    Notes/apply.md",
            "    Notes/overview.md",
        ]
    );
}

#[test]
fn a_top_section_with_its_page_beside_its_folder_follows_the_files_sections() {
    // `guides.md` sorts between the root's other pages, and its section
    // would be first in path order; it comes after them, in path order or
    // the file's, as any top section does (decision #610).
    let files = vec![
        ("index.md", page("guide", "Home")),
        ("a.md", page("guide", "Alpha")),
        ("guides.md", page("guide", "Guides")),
        ("b.md", page("guide", "Beta")),
        ("guides/one.md", page("guide", "One")),
        ("ref/api.md", page("guide", "API")),
    ];
    let folders = |p: &Project| -> Vec<String> {
        p.navigation()
            .root
            .sections
            .iter()
            .map(|s| s.folder.clone())
            .collect()
    };
    let p = project(&files, None);
    assert_eq!(folders(&p), ["guides/", "ref/"]);
    let root = &p.navigation().root;
    assert!(!root.places(&root.sections[0]));
    // Its page is still one of the root's, as the navigation contract says.
    assert!(root.pages.iter().any(|page| page.as_str() == "guides.md"));
    let nav = "version = 1\n[[sections]]\nfolder = \"ref/\"\n[[sections]]\nfolder = \"guides/\"\n";
    assert_eq!(folders(&project(&files, Some(nav))), ["ref/", "guides/"]);
    // Naming the page in the root's `pages` doesn't move the section.
    let nav = "version = 1\n[[sections]]\nfolder = \"\"\npages = [\"guides.md\", \"b.md\"]\n[[sections]]\nfolder = \"ref/\"\n";
    assert_eq!(folders(&project(&files, Some(nav))), ["ref/", "guides/"]);
    // Below the root, the page still places it.
    let nested = project(
        &[
            ("checks/check.md", page("guide", "Check")),
            ("checks/diagnostics.md", page("guide", "Diagnostics")),
            ("checks/diagnostics/pages.md", page("guide", "Pages")),
        ],
        None,
    );
    let checks = &nested.navigation().root.sections[0];
    assert!(checks.places(&checks.sections[0]));
}

#[test]
fn a_build_shows_its_share() {
    let mut files = files();
    files.push((
        "Guides/Hidden.md",
        "---\ntype: guide\ntitle: Hidden\nlisting: unlisted\n---\n\nText.\n".to_owned(),
    ));
    files.push((
        "Self-managed/Install.md",
        "---\ntype: guide\ntitle: Install `loom`\nvariant:\n  deployment: self-managed\n---\n\nText.\n"
            .to_owned(),
    ));
    files.push(("Reference/index.md", page("reference", "The reference")));
    let p = project(&files, None);
    let navigation = p.navigation();
    let model = p.model();
    let share = |name: &str| {
        let build = model.build(name).expect("a build");
        let resolved = p.resolve_build(build, &DefaultRouter::new());
        shared(&navigation.share(&p, &resolved))
    };
    assert_eq!(
        share("site"),
        [
            "Start (index.md)",
            // An unlisted page is left out.
            "  Guides (-)",
            "    Guides/Advanced.md Advanced guide",
            "    Guides/My Setup.md My setup guide",
            "    Guides/api.md API reference",
            // A section's label is its own page's title.
            "  The reference (Reference/index.md)",
            "    Reference/Options.md Options reference",
            // A folder's name has `-` as a space.
            "  Self managed (-)",
            "    Self-managed/Install.md Install loom guide",
        ]
    );
    // A build that publishes none of a section's pages leaves it out.
    assert!(!share("cloud").iter().any(|l| l.contains("Self-managed")));
}

#[test]
fn a_build_with_no_pages_has_an_empty_root() {
    let p = project(&[], None);
    let navigation = p.navigation();
    let build = p.model().build("site").expect("a build");
    let resolved = p.resolve_build(build, &DefaultRouter::new());
    let share = navigation.share(&p, &resolved);
    assert_eq!(shared(&share), ["Start (-)"]);
}

#[test]
fn a_folder_s_name_has_its_hyphens_and_underscores_as_spaces() {
    let p = project(
        &[
            ("getting-started/install.md", page("guide", "Install")),
            ("api_reference/options.md", page("reference", "Options")),
        ],
        None,
    );
    let navigation = p.navigation();
    let labels: Vec<_> = navigation
        .root
        .sections
        .iter()
        .map(|s| s.label(&p))
        .collect();
    assert_eq!(labels, ["Api reference", "Getting started"]);
    let build = p.model().build("site").expect("a build");
    let resolved = p.resolve_build(build, &DefaultRouter::new());
    let share = navigation.share(&p, &resolved);
    let shared: Vec<_> = share.root.sections.iter().map(|s| &s.label).collect();
    assert_eq!(shared, ["Api reference", "Getting started"]);
}

#[test]
fn labels_are_formatted_where_the_title_is() {
    let p = project(
        &[("Guides/index.md", page("guide", "The `loom` guides"))],
        None,
    );
    let navigation = p.navigation();
    let build = p.model().build("site").expect("a build");
    let resolved = p.resolve_build(build, &DefaultRouter::new());
    let share = navigation.share(&p, &resolved);
    let guides = &share.root.sections[0];
    assert_eq!(guides.label, "The loom guides");
    assert!(guides.formatted.is_some());
    let own = guides.page.as_ref().expect("its own page");
    assert!(own.formatted.is_some());
}

#[test]
fn a_page_s_nav_label_is_its_label_formatted_as_its_title_is() {
    let labeled = |kind: &str, title: &str, label: &str| {
        format!("---\ntype: {kind}\ntitle: {title}\nnav-label: \"{label}\"\n---\n\nText.\n")
    };
    let p = project(
        &[
            ("index.md", page("guide", "Home")),
            (
                "Guides/keys.md",
                labeled("guide", "The API keys of `loom`", "`loom` keys"),
            ),
            (
                "Guides/options.md",
                labeled("reference", "Every option there is", "Options"),
            ),
        ],
        None,
    );
    let navigation = p.navigation();
    let build = p.model().build("site").expect("a build");
    let resolved = p.resolve_build(build, &DefaultRouter::new());
    let share = navigation.share(&p, &resolved);
    let pages = &share.root.sections[0].pages;
    let keys = pages
        .iter()
        .find(|n| n.page.path.as_str() == "Guides/keys.md")
        .expect("the keys");
    assert_eq!(keys.label, "loom keys");
    assert_eq!(
        keys.formatted.map(ascribe_model::inline::plain_text),
        Some("loom keys".to_owned())
    );
    let options = pages
        .iter()
        .find(|n| n.page.path.as_str() == "Guides/options.md")
        .expect("the options");
    assert_eq!(options.label, "Options");
    assert!(
        options.formatted.is_none(),
        "its title isn't read with inline markup"
    );
    // The order is still by title.
    assert_eq!(pages[0].page.path.as_str(), "Guides/keys.md");
    // The project's labels, before a build, as `ascribe model` reports them.
    let file = p.file(&ascribe_core::RelPath::parse("Guides/keys.md").expect("a path"));
    assert_eq!(
        ascribe_resolve::file_label(file.expect("the file"), p.model()),
        "loom keys"
    );
}

#[test]
fn a_page_with_no_nav_label_is_labeled_with_its_title_s_plain_text() {
    let p = project(
        &[
            ("index.md", page("guide", "Home")),
            ("Guides/index.md", page("guide", "The `loom` guides")),
            ("Guides/keys.md", page("guide", "The `loom` keys")),
            (
                "Guides/options.md",
                page("reference", "Every `loom` option"),
            ),
        ],
        None,
    );
    let label = |path: &str| {
        let file = p.file(&ascribe_core::RelPath::parse(path).expect("a path"));
        ascribe_resolve::file_label(file.expect("the file"), p.model())
    };
    // The same label the navigation gives it (decision #594).
    assert_eq!(label("Guides/keys.md"), "The loom keys");
    let build = p.model().build("site").expect("a build");
    let resolved = p.resolve_build(build, &DefaultRouter::new());
    let navigation = p.navigation();
    let share = navigation.share(&p, &resolved);
    let guides = &share.root.sections[0];
    // A section's label from its own page's title, likewise.
    assert_eq!(guides.label, "The loom guides");
    assert_eq!(navigation.root.sections[0].label(&p), guides.label);
    let keys = guides
        .pages
        .iter()
        .find(|n| n.page.path.as_str() == "Guides/keys.md")
        .expect("the keys");
    assert_eq!(keys.label, label("Guides/keys.md"));
    // A title that isn't read with inline markup stays as written.
    assert_eq!(label("Guides/options.md"), "Every `loom` option");
}

#[test]
fn a_long_lived_project_follows_the_file_as_it_changes() {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("the model loads");
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (path, text) in files() {
        fs = fs.with_source(path, &text);
    }
    let mut project = ascribe_resolve::IncrementalProject::load(Arc::new(model), layout, fs);
    let folders = |p: &ascribe_resolve::IncrementalProject| -> Vec<String> {
        p.snapshot()
            .navigation()
            .root
            .sections
            .iter()
            .map(|s| s.folder.clone())
            .collect()
    };
    assert_eq!(folders(&project), ["Guides/", "Reference/"]);
    let nav = ascribe_model::NavFile::parse(
        "version = 1\n[[sections]]\nfolder = \"Reference/\"\n",
        FileId::new(0),
    )
    .expect("it reads");
    let before = project.version();
    let affected = project
        .apply([ascribe_resolve::Change::Nav(Some(Arc::new(nav.clone())))])
        .expect("it applies");
    assert!(!affected.is_empty(), "a new snapshot");
    assert!(affected.recheck.is_empty() && affected.re_resolve.is_empty());
    assert_eq!(folders(&project), ["Reference/", "Guides/"]);
    // The same file again changes nothing.
    let again = project
        .apply([ascribe_resolve::Change::Nav(Some(Arc::new(nav)))])
        .expect("it applies");
    assert!(again.is_empty());
    assert_ne!(project.version(), before);
    project
        .apply([ascribe_resolve::Change::Nav(None)])
        .expect("it applies");
    assert_eq!(folders(&project), ["Guides/", "Reference/"]);
}
