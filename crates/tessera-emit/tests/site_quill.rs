//! The Quill project's site output under each of its builds: snapshots of
//! every page and the generated schema, and what the output must contain.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::collections::BTreeMap;

use support::{emit_build, load, quill};
use tessera_emit::{SiteEmitter, render_site_html};

const BUILDS: [&str; 3] = ["site", "cloud", "self-managed-3.3"];

fn output(build: &str) -> BTreeMap<String, String> {
    let root = quill();
    let project = load(&root);
    let emitter = SiteEmitter::new(project.model());
    emit_build(&root, &project, build, &emitter)
}

/// A snapshot name that is a valid file name.
fn name(build: &str, path: &str) -> String {
    format!("site-{build}-{path}").replace(['/', '.'], "-")
}

#[test]
fn site_snapshots() {
    for build in BUILDS {
        for (path, text) in output(build) {
            // The schema is the same in every build: one snapshot.
            if path.ends_with(".md") || (path.ends_with(".ts") && build == "site") {
                insta::assert_snapshot!(name(build, &path), text);
            }
        }
    }
}

#[test]
fn each_build_lists_its_pages_assets_and_schema() {
    for build in BUILDS {
        let files: Vec<String> = output(build).into_keys().collect();
        assert_eq!(
            files,
            [
                "_ascribe/schema.ts",
                "_fragments/prerequisites.png",
                "install-agent.md",
                "keys.md",
                "playground.png",
                "quickstart.md",
            ],
            "{build}"
        );
    }
}

#[test]
fn the_cloud_build_keeps_the_package_manager_group_as_tabs() {
    let out = output("cloud");
    let page = &out["install-agent.md"];
    assert!(page.contains("<ascribe-tabs sync=\"pm\">"), "{page}");
    for value in ["npm", "pnpm", "yarn"] {
        assert!(
            page.contains(&format!("<ascribe-tab value=\"{value}\"")),
            "{page}"
        );
    }
    // The `deployment` group is reduced to its cloud arm, with no element.
    assert!(!page.contains("sync=\"deployment\""), "{page}");
    assert!(
        page.contains("Sign in to Quill Cloud and copy an API key"),
        "{page}"
    );
    assert!(!page.contains("Point the agent at your server"), "{page}");
    // The availability annotations still show.
    assert!(
        page.contains("<ascribe-availability scope=\"section\">"),
        "{page}"
    );
}

#[test]
fn the_switch_build_keeps_both_groups_as_tabs() {
    let out = output("site");
    let page = &out["install-agent.md"];
    assert!(page.contains("<ascribe-tabs sync=\"pm\">"), "{page}");
    assert!(
        page.contains("<ascribe-tabs sync=\"deployment\">"),
        "{page}"
    );
}

#[test]
fn the_self_managed_3_3_build_drops_what_isnt_available_yet() {
    let out = output("self-managed-3.3");
    let page = &out["install-agent.md"];
    assert!(!page.contains("Streaming sync"), "{page}");
    assert!(page.contains("Point the agent at your server"), "{page}");
}

#[test]
fn links_are_astro_routes_and_pages_are_at_their_source_paths() {
    let out = output("site");
    let page = &out["install-agent.md"];
    assert!(page.contains("(/quickstart/#try-in-browser)"), "{page}");
    assert!(page.contains("(/keys/#rotate-keys)"), "{page}");
    assert!(
        page.contains("(https://api.quill.dev/v3/streaming)"),
        "{page}"
    );
    assert!(out["quickstart.md"].contains("(/install-agent/)"));
    assert!(out["quickstart.md"].contains("![The Quill playground](./playground.png)<ascribe-attributes width=\"600\"></ascribe-attributes>"));
    assert!(page.contains("![Checklist of prerequisites](./_fragments/prerequisites.png)"));
}

/// The elements and attributes `packages/elements/CONTRACT.md` defines, in
/// the order it lists each element's attributes.
const CONTRACT: &[(&str, &[&str])] = &[
    ("ascribe-note", &["type", "label", "heading"]),
    ("ascribe-steps", &[]),
    ("ascribe-tabs", &["sync"]),
    ("ascribe-tab", &["value", "label"]),
    ("ascribe-availability", &["scope"]),
    (
        "ascribe-availability-target",
        &["target", "dimension", "states", "versions"],
    ),
    ("ascribe-attributes", &["id", "width", "height"]),
    ("details", &[]),
    ("summary", &[]),
];

/// Every tag in some text: `(name, attribute names in order)`.
fn tags(text: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find('<') {
        rest = &rest[at + 1..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_lowercase()) {
            continue;
        }
        let mut attributes = Vec::new();
        let mut inside = &rest[name.len()..];
        while let Some(stripped) = inside.strip_prefix(' ') {
            let attr: String = stripped
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || *c == '-')
                .collect();
            let after = &stripped[attr.len()..];
            let Some(value) = after.strip_prefix("=\"") else {
                break;
            };
            let end = value.find('"').expect("a closing quote");
            attributes.push(attr);
            inside = &value[end + 1..];
        }
        out.push((name, attributes));
    }
    out
}

#[test]
fn every_element_and_attribute_is_in_the_element_contract() {
    for build in BUILDS {
        for (path, text) in output(build) {
            if !path.ends_with(".md") {
                continue;
            }
            for (name, attributes) in tags(&text) {
                if !name.starts_with("tessera-") && name != "details" && name != "summary" {
                    continue;
                }
                let (_, allowed) = CONTRACT
                    .iter()
                    .find(|(n, _)| *n == name)
                    .unwrap_or_else(|| panic!("{build} {path}: <{name}> isn't in the contract"));
                let mut last = 0;
                for attribute in &attributes {
                    let position =
                        allowed
                            .iter()
                            .position(|a| a == attribute)
                            .unwrap_or_else(|| {
                                panic!("{build} {path}: <{name} {attribute}> isn't in the contract")
                            });
                    assert!(
                        position >= last,
                        "{build} {path}: <{name}> lists {attributes:?} out of order"
                    );
                    last = position;
                }
            }
        }
    }
}

#[test]
fn rendered_pages_have_the_structure_the_elements_expect() {
    for build in BUILDS {
        for (path, text) in output(build) {
            if !path.ends_with(".md") {
                continue;
            }
            let body = text.splitn(3, "---\n").nth(2).expect("frontmatter");
            let html = render_site_html(body);
            // The markdown an element wraps was parsed as markdown, and no
            // element was taken for text.
            assert!(!html.contains("&lt;tessera-"), "{build} {path}\n{html}");
            assert!(!html.contains("<p><tessera-"), "{build} {path}\n{html}");
            assert!(!html.contains("<p></tessera-"), "{build} {path}\n{html}");
            // Every heading has an id, and it's the page id the marker held.
            for level in 1..=6 {
                let mut rest = html.as_str();
                while let Some(at) = rest.find(&format!("<h{level}")) {
                    let tag_end = rest[at..].find('>').expect("a tag end") + at;
                    assert!(
                        rest[at..tag_end].contains(" id=\""),
                        "{build} {path}: {}",
                        &rest[at..tag_end]
                    );
                    rest = &rest[tag_end..];
                }
            }
            assert!(
                !html.contains("<ascribe-attributes"),
                "{build} {path}\n{html}"
            );
        }
    }
    let html = render_site_html(
        output("site")["install-agent.md"]
            .splitn(3, "---\n")
            .nth(2)
            .expect("frontmatter"),
    );
    assert!(
        html.contains("<h2 id=\"prerequisites\">Prerequisites</h2>"),
        "{html}"
    );
    assert!(
        html.contains("<ascribe-tab value=\"npm\" label=\"npm\">"),
        "{html}"
    );
    assert!(
        html.contains("<pre><code class=\"language-shell\">npm install -g @quill/agent"),
        "{html}"
    );
    assert!(html.contains("<li>"), "{html}");
}
