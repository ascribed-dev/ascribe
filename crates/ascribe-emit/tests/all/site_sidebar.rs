//! The docs site's sidebar, without a browser: `docs/`'s `site` build's
//! share of the navigation, the `_ascribe/nav.json` the site draws its
//! sidebar from, grouped as `@ascribed/astro`'s `sidebar()` groups it
//! (`packages/astro/src/nav.ts`) and listed as `site/src/components/Sidebar.astro`
//! lists it, each page at its route under the docs' `[consumer]`. It's held
//! to `site/test/fixtures/sidebar.json`, which `site/test/e2e.test.ts` holds
//! the published sidebar to in Chromium, so a page added to `docs/content/`,
//! moved, or retitled fails here, in `cargo test`, before CI's browser does.
//! Run `ASCRIBE_BLESS=1 cargo test -p ascribe-emit --test all site_sidebar`
//! to rewrite the fixture, and read its diff: it's the sidebar's change.

#![allow(clippy::expect_used, clippy::panic)]

use crate::support::{load, repository};

use std::collections::BTreeSet;
use std::fs;

use ascribe_core::Router;
use ascribe_resolve::{AstroRouter, NavPage, SharedSection};
use serde_json::{Value, json};

const FIXTURE: &str = "site/test/fixtures/sidebar.json";
const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-emit --test all site_sidebar";

/// A page of a group as the sidebar lists it: its address, its label, and
/// whether it's in a nested section.
fn entry(router: &AstroRouter, page: &NavPage, nested: bool) -> Value {
    let mut out = json!({ "href": router.route(&page.page.path), "label": page.label });
    if nested {
        out["nested"] = json!(true);
    }
    out
}

/// Whether `page` is the own page of one of `section`'s nested sections,
/// beside its folder (`checks/diagnostics.md` for `checks/diagnostics/`).
fn beside(section: &SharedSection, page: &NavPage) -> bool {
    section.sections.iter().any(|nested| {
        nested
            .page
            .as_ref()
            .is_some_and(|own| own.page.path == page.page.path)
    })
}

/// A section's pages, as `navigationGroups` gives them: its own page first,
/// unless it's beside the folder and so among the parent's pages.
fn pages<'a>(
    section: &'a SharedSection<'a>,
    parent: Option<&SharedSection>,
) -> Vec<&'a NavPage<'a>> {
    let own = section.page.as_ref().filter(|own| {
        !parent.is_some_and(|parent| parent.pages.iter().any(|p| p.page.path == own.page.path))
    });
    own.into_iter().chain(&section.pages).collect()
}

/// A group's pages in order, each nested section's right after its own page,
/// or after the group's pages when its own page is its `index.md`, as
/// `Sidebar.astro`'s `items` lists them.
fn items(
    router: &AstroRouter,
    section: &SharedSection,
    parent: Option<&SharedSection>,
    nested: bool,
    out: &mut Vec<Value>,
) {
    let mut placed = BTreeSet::new();
    for page in pages(section, parent) {
        out.push(entry(router, page, nested));
        for (at, sub) in section.sections.iter().enumerate() {
            if sub
                .page
                .as_ref()
                .is_some_and(|own| own.page.path == page.page.path)
            {
                placed.insert(at);
                items(router, sub, Some(section), true, out);
            }
        }
    }
    for (at, sub) in section.sections.iter().enumerate() {
        if !placed.contains(&at) {
            items(router, sub, Some(section), true, out);
        }
    }
}

/// The sidebar's groups: the root's own page and its pages that aren't a
/// top section's, under its label, when it has any, then each top section.
fn sidebar(router: &AstroRouter, root: &SharedSection) -> Value {
    let mut groups = Vec::new();
    let start: Vec<Value> = root
        .page
        .iter()
        .chain(root.pages.iter().filter(|page| !beside(root, page)))
        .map(|page| entry(router, page, false))
        .collect();
    if !start.is_empty() {
        groups.push(json!({ "label": root.label, "pages": start }));
    }
    for section in &root.sections {
        let mut out = Vec::new();
        items(router, section, None, false, &mut out);
        groups.push(json!({ "label": section.label, "pages": out }));
    }
    Value::Array(groups)
}

#[test]
fn the_docs_sidebar_is_the_fixture_the_site_is_held_to() {
    let root = repository().join("docs");
    let project = load(&root);
    let build = project
        .model()
        .build("site")
        .expect("the docs have a site build");
    let project = project.for_build(build);
    let router = AstroRouter::from_consumer(&project.model().consumer);
    let resolved = project.resolve_build(build, &router);
    let navigation = project.navigation();
    let share = navigation.share(&project, &resolved);

    let mut text = serde_json::to_string_pretty(&sidebar(&router, &share.root))
        .expect("JSON values serialize");
    text.push('\n');
    let path = repository().join(FIXTURE);
    if std::env::var_os("ASCRIBE_BLESS").is_some() {
        fs::write(&path, &text).expect("writes the fixture");
        return;
    }
    let fixture = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        fixture == text,
        "{FIXTURE} isn't the docs' sidebar: a page under docs/content/ was added, moved, or retitled, or docs/ascribe.nav changed. Run `{BLESS}` and read the diff.\n--- the docs' sidebar\n{text}"
    );
}
