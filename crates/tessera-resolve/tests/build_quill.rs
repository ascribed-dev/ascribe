//! The Quill project (`examples/quill`) resolves under all three of its
//! builds with no problems.

#![allow(clippy::expect_used, clippy::panic)]

mod build_support;

use std::path::PathBuf;
use std::sync::Arc;

use build_support::{plain, summary};
use tessera_resolve::{DefaultRouter, DiskFs, Layout, LinkTarget, Project, ResolvedPage};

fn quill() -> Project {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let model = tessera_model::load(root.join("ascribe.toml")).expect("the Quill model loads");
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
        if let tessera_resolve::ResolvedKind::Leaf(leaf) = &b.kind
            && let tessera_syntax::BlockKind::Paragraph(p) = &leaf.kind
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
