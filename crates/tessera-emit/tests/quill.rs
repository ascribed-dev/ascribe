//! The Quill project under each of its builds, with both emitters: snapshots
//! of every page, and what the outputs must contain.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use std::collections::BTreeMap;

use comrak_tessera::nodes::NodeValue;
use comrak_tessera::{Arena, Options, parse_document};
use support::{emit_build, load, quill};
use tessera_emit::{Emitter, JsonEmitter, PlainEmitter};

const BUILDS: [&str; 3] = ["site", "cloud", "self-managed-3.3"];

fn output(build: &str, emitter: &dyn Emitter) -> BTreeMap<String, String> {
    let root = quill();
    let project = load(&root);
    emit_build(&root, &project, build, emitter)
}

/// A snapshot name that is a valid file name.
fn name(emitter: &str, build: &str, path: &str) -> String {
    format!("{emitter}-{build}-{path}").replace(['/', '.'], "-")
}

#[test]
fn plain_snapshots() {
    for build in BUILDS {
        for (path, text) in output(build, &PlainEmitter) {
            if path.ends_with(".md") {
                insta::assert_snapshot!(name("plain", build, &path), text);
            }
        }
    }
}

#[test]
fn json_snapshots() {
    for build in BUILDS {
        for (path, text) in output(build, &JsonEmitter) {
            if path.ends_with(".json") {
                insta::assert_snapshot!(name("json", build, &path), text);
            }
        }
    }
}

#[test]
fn each_output_lists_its_pages_and_the_assets_the_build_keeps() {
    for build in BUILDS {
        for emitter in [&PlainEmitter as &dyn Emitter, &JsonEmitter] {
            let files: Vec<String> = output(build, emitter).into_keys().collect();
            let page = |stem: &str| {
                if emitter.name() == "json" {
                    format!("{stem}.json")
                } else {
                    format!("{stem}.md")
                }
            };
            assert_eq!(
                files,
                [
                    "_fragments/prerequisites.png".to_owned(),
                    page("install-agent"),
                    page("keys"),
                    "playground.png".to_owned(),
                    page("quickstart"),
                ],
                "{build} {}",
                emitter.name()
            );
        }
    }
}

#[test]
fn the_cloud_build_keeps_the_package_manager_group_and_shows_availability() {
    let plain = output("cloud", &PlainEmitter);
    let page = &plain["install-agent.md"];
    // The `pm` group stays a full set of labeled sections...
    // (They're inside a list item, so indented by three spaces.)
    for section in [
        "   **npm**\n\n   ```shell\n   npm install -g @quill/agent\n   ```",
        "   **pnpm**\n\n   ```shell\n   pnpm add -g @quill/agent\n   ```",
        "   **Yarn**\n\n   ```shell\n   yarn global add @quill/agent\n   ```",
    ] {
        assert!(page.contains(section), "{section}\n{page}");
    }
    // ...the `deployment` group is reduced to its cloud arm...
    assert!(
        page.contains("Sign in to Quill Cloud and copy an API key"),
        "{page}"
    );
    assert!(!page.contains("Point the agent at your server"), "{page}");
    assert!(!page.contains("**self-managed**"), "{page}");
    // ...and the availability line is there.
    assert!(
        page.contains("Available: Quill Cloud (GA); self-managed (preview, 3.4+)"),
        "{page}"
    );
}

#[test]
fn the_self_managed_3_3_build_drops_what_isnt_available_yet() {
    let plain = output("self-managed-3.3", &PlainEmitter);
    let page = &plain["install-agent.md"];
    assert!(!page.contains("Streaming sync"), "{page}");
    assert!(page.contains("Point the agent at your server"), "{page}");
    // Both arms of the `deployment` group stay, as labeled sections.
    assert!(
        page.contains("**Quill Cloud**") && page.contains("**self-managed**"),
        "{page}"
    );
}

#[test]
fn plain_output_is_commonmark_with_no_html() {
    for build in BUILDS {
        for (path, text) in output(build, &PlainEmitter) {
            if !path.ends_with(".md") {
                continue;
            }
            let arena = Arena::new();
            let root = parse_document(&arena, &text, &Options::default());
            for node in root.descendants() {
                let value = &node.data.borrow().value;
                assert!(
                    !matches!(value, NodeValue::HtmlBlock(_) | NodeValue::HtmlInline(_)),
                    "{build} {path} has HTML: {value:?}\n{text}"
                );
            }
            assert!(
                !text.contains("<tessera") && !text.contains("</"),
                "{build} {path}\n{text}"
            );
        }
    }
}

#[test]
fn json_output_is_versioned_and_points_back_at_the_source() {
    let json = output("site", &JsonEmitter);
    let doc: serde_json::Value = serde_json::from_str(&json["install-agent.json"]).expect("JSON");
    assert_eq!(doc["schemaVersion"], 1);
    assert_eq!(doc["path"], "install-agent.md");
    let mut nodes = 0;
    visit(&doc["blocks"], &mut |block| {
        nodes += 1;
        assert!(block["source"]["file"].is_string(), "{block}");
        assert!(block["source"]["span"].is_array(), "{block}");
    });
    assert!(nodes > 20, "{nodes}");
    // A block from the included fragment names the fragment, and the include it came through.
    let mut from_fragment = None;
    visit(&doc["blocks"], &mut |block| {
        if block["source"]["file"] == "_fragments/prerequisites.md" && from_fragment.is_none() {
            from_fragment = Some(block.clone());
        }
    });
    let block = from_fragment.expect("a block from the fragment");
    assert_eq!(block["source"]["via"][0]["file"], "install-agent.md");
}

/// Calls `f` on every block in a JSON block list, through containers, lists,
/// quotes, and groups.
fn visit(blocks: &serde_json::Value, f: &mut dyn FnMut(&serde_json::Value)) {
    for block in blocks.as_array().into_iter().flatten() {
        f(block);
        visit(&block["children"], f);
        for item in block["items"].as_array().into_iter().flatten() {
            visit(&item["children"], f);
        }
        for arm in block["arms"].as_array().into_iter().flatten() {
            visit(&arm["children"], f);
        }
    }
}
