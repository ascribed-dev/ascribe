//! The Quill project (`examples/quill`) resolves under all three of its
//! builds with no problems (so it indexes and expands with none too), and
//! its index says two things about the install page: it includes the
//! prerequisites fragment, and its source ids are its headings'.

#![allow(clippy::expect_used, clippy::panic)]

mod build_support;

use std::path::PathBuf;
use std::sync::Arc;

use ascribe_resolve::{
    DefaultRouter, DiskFs, FileKind, Layout, LinkTarget, Project, RefKind, ResolvedPage,
};
use build_support::{plain, summary};

fn quill() -> Project {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let model = ascribe_model::load(root.join("ascribe.toml")).expect("the Quill model loads");
    let layout = Layout::from_model(&model);
    let fs = DiskFs::new(&root, &layout);
    Project::load(Arc::new(model), layout, &fs)
}

fn resolve_all(project: &Project, build: &str) -> Vec<ResolvedPage> {
    let b = project.model().build(build).expect("a build");
    let resolved =
        project.resolve_build(b, &DefaultRouter::from_consumer(&project.model().consumer));
    assert!(
        resolved.dropped.is_empty(),
        "{build}: {:?}",
        resolved.dropped
    );
    resolved.pages
}

#[test]
fn quill_resolves_under_every_build_with_no_problems() {
    let project = quill();
    let builds: Vec<String> = project
        .model()
        .builds
        .iter()
        .map(|b| b.name.clone())
        .collect();
    assert_eq!(builds, ["site", "cloud", "self-managed-3.3"]);
    for name in &builds {
        let pages = resolve_all(&project, name);
        let paths: Vec<String> = pages.iter().map(|p| p.path.to_string()).collect();
        assert_eq!(
            paths,
            ["install-agent.md", "keys.md", "quickstart.md"],
            "{name}"
        );
        for page in &pages {
            assert!(
                page.problems.is_empty(),
                "{name}: {}: {:?}",
                page.path,
                page.problems
            );
            // Every link resolved: none is left pointing nowhere.
            let mut unresolved = 0;
            page.visit(&mut |b| {
                unresolved += b
                    .links
                    .iter()
                    .filter(|l| l.target == LinkTarget::Unresolved)
                    .count();
            });
            assert_eq!(unresolved, 0, "{name}: {}", page.path);
        }
        // The same two assets under every build.
        let b = project.model().build(name).expect("a build");
        let resolved =
            project.resolve_build(b, &DefaultRouter::from_consumer(&project.model().consumer));
        assert_eq!(
            resolved
                .assets()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["_fragments/prerequisites.png", "playground.png"],
            "{name}"
        );
    }
    // And no problems at file level.
    for file in project.files() {
        assert!(project.problems(&file.path).is_empty(), "{}", file.path);
    }
}

fn install(build: &str) -> ResolvedPage {
    let project = quill();
    resolve_all(&project, build)
        .into_iter()
        .find(|p| p.path.as_str() == "install-agent.md")
        .expect("the install page")
}

#[test]
fn the_cloud_build_reduces_the_deployment_group_and_keeps_the_pm_switcher() {
    let cloud = summary(&install("cloud")).join("\n");
    assert!(
        cloud.contains("group[pm=npm | pm=pnpm | pm=yarn]"),
        "{cloud}"
    );
    assert!(!cloud.contains("deployment=cloud"), "{cloud}");
    assert!(cloud.contains("Sign in to Quill Cloud"), "{cloud}");
    assert!(!cloud.contains("Point the agent at your server"), "{cloud}");
    // The site build keeps both arms.
    let site = summary(&install("site")).join("\n");
    assert!(
        site.contains("group[deployment=cloud | deployment=self-managed]"),
        "{site}"
    );
}

#[test]
fn the_self_managed_3_3_build_drops_the_streaming_section() {
    let has = |build: &str| {
        install(build)
            .headings()
            .iter()
            .any(|(_, h)| h.text == "Streaming sync")
    };
    assert!(has("site"));
    assert!(has("cloud"));
    assert!(!has("self-managed-3.3"));
}

#[test]
fn included_headings_and_phrases_resolve_on_the_install_page() {
    let page = install("site");
    let ids: Vec<(String, String)> = page
        .headings()
        .iter()
        .map(|(_, h)| (h.text.clone(), h.page_id.clone()))
        .collect();
    assert!(
        ids.iter()
            .any(|(t, id)| t == "Connect to Quill" && id == "connect"),
        "{ids:?}"
    );
    let mut text = String::new();
    page.visit(&mut |b| {
        if let ascribe_resolve::ResolvedKind::Leaf(leaf) = &b.kind
            && let ascribe_syntax::BlockKind::Paragraph(p) = &leaf.kind
        {
            text.push_str(&plain(&p.inlines));
            text.push('\n');
        }
    });
    assert!(
        text.contains("The Quill agent watches your docs repository"),
        "{text}"
    );
    assert!(!text.contains("{product}"), "{text}");
}

#[test]
fn quills_install_page_includes_the_prerequisites_fragment() {
    let project = quill();
    let fragment = ascribe_core::RelPath::parse("_fragments/prerequisites.md").expect("path");
    let page = ascribe_core::RelPath::parse("install-agent.md").expect("path");
    assert_eq!(
        project.file(&fragment).map(|f| f.kind),
        Some(FileKind::Fragment)
    );
    assert_eq!(project.includers(&fragment).len(), 1);
    assert_eq!(
        project.including_pages(&fragment),
        std::slice::from_ref(&page)
    );

    // The fragment's image is the one beside the fragment.
    let assets: Vec<(String, RefKind, String)> = project
        .assets(&page)
        .iter()
        .map(|a| (a.path.to_string(), a.kind, a.written_in.to_string()))
        .collect();
    assert!(
        assets.contains(&(
            "_fragments/prerequisites.png".to_owned(),
            RefKind::Image,
            "_fragments/prerequisites.md".to_owned()
        )),
        "{assets:?}"
    );
    // Across the project, the two assets the conformance case lists.
    let mut all: Vec<String> = project
        .pages()
        .flat_map(|p| project.assets(&p.path))
        .map(|a| a.path.to_string())
        .collect();
    all.sort();
    all.dedup();
    assert_eq!(all, ["_fragments/prerequisites.png", "playground.png"]);
}

#[test]
fn quills_source_ids_match_the_headings() {
    let project = quill();
    let install = ascribe_core::RelPath::parse("install-agent.md").expect("path");
    let ids: Vec<&str> = project
        .file(&install)
        .expect("install-agent.md")
        .headings
        .iter()
        .map(|h| h.source_id.as_str())
        .collect();
    assert!(ids.contains(&"install-agent"), "{ids:?}");
    assert!(ids.contains(&"prerequisites"), "{ids:?}");
}
