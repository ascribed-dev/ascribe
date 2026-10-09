//! `ascribe/buildView`, over an in-memory connection.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::path::Path;

use ascribe_core::{LineIndex, WideEncoding, WideLineCol};
use serde_json::{Value, json};
use support::{Client, Fixture};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.pm]
values = ["npm", "pnpm", "yarn"]

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]
labels = { cloud = "Quill Cloud", self-managed = "Self-managed" }

[versions]
scheme = "numeric"

[builds.site]
variants = "switch"
availability = "badge"

[builds.npm]
variants = { pm = "npm" }
availability = "badge"

[builds.sm]
variants = { deployment = "self-managed" }
availability = { filter = "self-managed 3.3" }

[editor]
build = "site"
"#;

fn view(client: &mut Client, path: &Path, build: Option<&str>) -> Value {
    let mut params = json!({ "textDocument": { "uri": support::uri(path).as_str() } });
    if let Some(build) = build {
        params["build"] = json!(build);
    }
    client
        .request("ascribe/buildView", params)
        .response_result
        .expect("the request succeeds")
}

/// Each excluded range as the text it covers, with its reason and detail.
fn excluded(text: &str, result: &Value) -> Vec<(String, String, String)> {
    let lines = LineIndex::new(text);
    let at = |p: &Value| {
        lines
            .wide_offset(
                WideEncoding::Utf16,
                WideLineCol {
                    line: p["line"].as_u64().unwrap() as u32,
                    col: p["character"].as_u64().unwrap() as u32,
                },
            )
            .expect("a position in the text")
    };
    result["excluded"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|e| {
            (
                text[at(&e["range"]["start"])..at(&e["range"]["end"])].to_owned(),
                e["reason"].as_str().unwrap().to_owned(),
                e["detail"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn copy(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).expect("a directory") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            std::fs::create_dir_all(&target).expect("mkdir");
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy");
        }
    }
}

fn lantern() -> Fixture {
    let fixture = Fixture::new("", &[]);
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/monorepo/docs");
    copy(&source, &fixture.root());
    fixture
}

#[test]
fn self_hosted_leaves_out_the_cloud_arm_and_the_scheduled_rollouts_section() {
    let f = lantern();
    let mut client = Client::start(&f.root());
    client.settle();

    let page = f.path("content/getting-started.md");
    let text = std::fs::read_to_string(&page).unwrap();
    let result = view(&mut client, &page, Some("self-hosted"));
    assert_eq!(result["build"], "self-hosted");
    assert_eq!(result["pageIncluded"], true);
    let gone = excluded(&text, &result);
    assert_eq!(gone.len(), 1, "{gone:?}");
    let (cloud, reason, detail) = &gone[0];
    assert!(cloud.starts_with("@variant {edition=cloud}:\n"), "{cloud}");
    assert!(cloud.trim_end().ends_with("lantern login\n```"), "{cloud}");
    assert_eq!(reason, "variant");
    assert_eq!(detail, "Shows only edition=self-hosted");

    let page = f.path("content/guides/rollouts.md");
    let text = std::fs::read_to_string(&page).unwrap();
    let gone = excluded(&text, &view(&mut client, &page, Some("self-hosted")));
    assert_eq!(gone.len(), 1, "{gone:?}");
    let (section, reason, detail) = &gone[0];
    // The heading through the section's last paragraph, as one range.
    assert!(section.starts_with("## Scheduled rollouts\n"), "{section}");
    assert!(section.ends_with("`--max-error-rate`."), "{section}");
    assert_eq!(reason, "availability");
    assert_eq!(
        detail,
        "Scheduled rollouts: available on Lantern Cloud (preview), not Self-hosted 2.5"
    );

    // The site build switches and badges: nothing is left out.
    let result = view(&mut client, &page, Some("site"));
    assert_eq!(result["excluded"], json!([]));
    // Without a build, the editor's.
    assert_eq!(view(&mut client, &page, None)["build"], "site");
    client.shutdown();
}

#[test]
fn arms_in_nested_groups_inside_steps_are_left_out() {
    let page = "@steps
1. Install it:

   @variant {pm=npm}:
   ```shell
   npm i quill
   ```
   @variant {pm=yarn}:
   ```shell
   yarn add quill
   ```
   @end

2. Run it.
";
    let f = Fixture::new(MODEL, &[("docs/index.md", page)]);
    let mut client = Client::start(&f.root());
    client.settle();
    let gone = excluded(
        page,
        &view(&mut client, &f.path("docs/index.md"), Some("npm")),
    );
    assert_eq!(
        gone,
        [(
            "@variant {pm=yarn}:\n   ```shell\n   yarn add quill\n   ```".to_owned(),
            "variant".to_owned(),
            "Shows only pm=npm".to_owned()
        )]
    );
    client.shutdown();
}

#[test]
fn availability_filters_sections_blocks_and_table_rows() {
    let page = "# Page

## Cloud
@available: cloud

Cloud text.

## Everywhere

Kept.

@available: self-managed 3.4
From 3.4.

| Key | Note |
|---|---|
| `a` | Everywhere. |
| `b` {available=cloud} | Cloud only. |
";
    let f = Fixture::new(MODEL, &[("docs/index.md", page)]);
    let mut client = Client::start(&f.root());
    client.settle();
    let gone = excluded(
        page,
        &view(&mut client, &f.path("docs/index.md"), Some("sm")),
    );
    assert_eq!(
        gone,
        [
            (
                "## Cloud\n@available: cloud\n\nCloud text.".to_owned(),
                "availability".to_owned(),
                "Available on Quill Cloud (GA), not Self-managed 3.3".to_owned()
            ),
            (
                "@available: self-managed 3.4\nFrom 3.4.".to_owned(),
                "availability".to_owned(),
                "Available on Self-managed (GA, 3.4+), not Self-managed 3.3".to_owned()
            ),
            (
                "| `b` {available=cloud} | Cloud only. |".to_owned(),
                "availability".to_owned(),
                "Available on Quill Cloud (GA), not Self-managed 3.3".to_owned()
            ),
        ]
    );
    client.shutdown();
}

#[test]
fn a_page_the_build_drops_is_left_out_whole() {
    let page = "---\ntitle: Cloud\navailable: cloud\n---\n\nText.\n";
    let f = Fixture::new(MODEL, &[("docs/cloud.md", page)]);
    let mut client = Client::start(&f.root());
    client.settle();
    let result = view(&mut client, &f.path("docs/cloud.md"), Some("sm"));
    assert_eq!(result["pageIncluded"], false);
    assert_eq!(result["excluded"], json!([]));
    let detail = result["pageDetail"].as_str().unwrap();
    assert!(
        detail.starts_with("The build sm doesn't publish this page: "),
        "{detail}"
    );

    let result = view(&mut client, &f.path("docs/cloud.md"), Some("site"));
    assert_eq!(result["pageIncluded"], true);
    assert_eq!(result["pageDetail"], Value::Null);
    client.shutdown();
}

#[test]
fn a_switch_build_leaves_out_nothing_and_an_unknown_build_answers_empty() {
    let page = "@variant {deployment=cloud}:\nCloud.\n@variant {deployment=self-managed}:\nServer.\n@end\n";
    let f = Fixture::new(MODEL, &[("docs/index.md", page)]);
    let mut client = Client::start(&f.root());
    client.settle();
    let result = view(&mut client, &f.path("docs/index.md"), Some("site"));
    assert_eq!(result["excluded"], json!([]));
    assert_eq!(result["pageIncluded"], true);
    let result = view(&mut client, &f.path("docs/index.md"), Some("nope"));
    assert_eq!(result["build"], "");
    assert_eq!(result["excluded"], json!([]));
    client.shutdown();
}

#[test]
fn unsaved_edits_count() {
    let page = "Intro.\n";
    let f = Fixture::new(MODEL, &[("docs/index.md", page)]);
    let path = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(view(&mut client, &path, Some("npm"))["excluded"], json!([]));

    let edited = "Intro.\n\n@variant {pm=npm}:\nnpm.\n@variant {pm=yarn}:\nyarn.\n@end\n";
    client.open(&path, 1, page);
    client.replace(&path, 2, edited);
    client.settle();
    let result = view(&mut client, &path, Some("npm"));
    assert_eq!(result["documentVersion"], 2);
    assert_eq!(
        excluded(edited, &result),
        [(
            "@variant {pm=yarn}:\nyarn.".to_owned(),
            "variant".to_owned(),
            "Shows only pm=npm".to_owned()
        )]
    );
    client.shutdown();
}
