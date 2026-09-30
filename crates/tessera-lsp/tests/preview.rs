//! `ascribe/preview`, over an in-memory connection.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::path::Path;
use std::time::Instant;

use serde_json::{Value, json};
use support::{Client, Fixture, MODEL, edit};

fn quill() -> Fixture {
    let fixture = Fixture::new("", &[]);
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    copy(&source, &fixture.root());
    fixture
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

fn preview(client: &mut Client, path: &Path, build: Option<&str>) -> Value {
    let mut params = json!({ "textDocument": { "uri": support::uri(path).as_str() } });
    if let Some(build) = build {
        params["build"] = json!(build);
    }
    client
        .request("ascribe/preview", params)
        .response_result
        .expect("the request succeeds")
}

fn html(result: &Value) -> &str {
    result["page"]["html"].as_str().expect("html")
}

#[test]
fn renders_the_quill_page_as_the_site_does() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let result = preview(&mut client, &page, None);

    assert_eq!(result["build"], "site");
    assert_eq!(result["problems"], json!([]));
    assert_eq!(result["page"]["path"], "install-agent.md");
    assert_eq!(result["page"]["title"], "Install the Quill agent");
    assert_eq!(result["page"]["route"], "/install-agent/");
    let html = html(&result);
    // Tabs, a note, steps, badges: the element contract's markup.
    assert!(html.contains("<ascribe-tabs"), "{html}");
    assert!(html.contains("<ascribe-note"), "{html}");
    assert!(html.contains("<ascribe-steps>"), "{html}");
    assert!(html.contains("<ascribe-availability"), "{html}");
    // Heading ids come from Ascribe, through the marker.
    assert!(html.contains("<h2 id=\"prerequisites\">"), "{html}");
    assert!(html.contains("<h2 id=\"streaming-sync\">"), "{html}");
    assert!(!html.contains("ascribe-attributes"), "{html}");
    // The frontmatter is the site output's: `available` is a list of targets.
    let frontmatter = &result["page"]["frontmatter"];
    assert_eq!(frontmatter["title"], "Install the Quill agent");
    assert_eq!(frontmatter["available"][0]["target"], "cloud");
    // Not a layout: no frontmatter block in the HTML.
    assert!(!html.contains("title: Install"), "{html}");
    client.shutdown();
}

#[test]
fn a_fragments_image_resolves_from_the_fragment() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let result = preview(&mut client, &page, None);
    // `prerequisites.png` is written in `_fragments/prerequisites.md`, and the
    // page includes it, so the reference is relative to the page's output
    // location and names the file beside the fragment.
    let assets = result["page"]["assets"].as_array().expect("assets");
    let image = assets
        .iter()
        .find(|a| {
            a["path"]
                .as_str()
                .unwrap()
                .ends_with("docs/_fragments/prerequisites.png")
        })
        .unwrap_or_else(|| panic!("{assets:?}"));
    assert_eq!(image["reference"], "./_fragments/prerequisites.png");
    assert_eq!(image["kind"], "image");
    assert_eq!(image["servable"], true);
    assert!(
        html(&result).contains("src=\"./_fragments/prerequisites.png\""),
        "{}",
        html(&result)
    );
    assert_eq!(result["contentRoot"], f.path("docs").to_str().unwrap());
    client.shutdown();
}

#[test]
fn unsaved_edits_are_in_the_answer() {
    let f = quill();
    let page = f.path("docs/quickstart.md");
    let text = std::fs::read_to_string(&page).unwrap();
    let mut client = Client::start(&f.root());
    client.open(&page, 1, &text);
    let first = preview(&mut client, &page, None);
    assert_eq!(first["documentVersion"], 1);
    assert!(html(&first).contains("Open the playground"));

    // Nothing is saved: the file on disk still says "Open the playground".
    client.change(&page, 2, vec![edit((8, 0), (8, 4), "Launch")]);
    let second = preview(&mut client, &page, None);
    assert_eq!(second["documentVersion"], 2);
    assert!(
        html(&second).contains("Launch the playground"),
        "{}",
        html(&second)
    );
    assert!(
        std::fs::read_to_string(&page)
            .unwrap()
            .contains("Open the playground")
    );

    // Adding an image beside the page, in an unsaved edit.
    client.change(
        &page,
        3,
        vec![edit(
            (8, 0),
            (8, 0),
            "![Logo](playground.png){width=32}\n\n",
        )],
    );
    let third = preview(&mut client, &page, None);
    assert!(html(&third).contains("width=\"32\""), "{}", html(&third));
    client.shutdown();
}

#[test]
fn the_builds_are_listed_and_change_the_content() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let mut client = Client::start(&f.root());
    client.settle();

    let site = preview(&mut client, &page, None);
    let names: Vec<&str> = site["builds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["site", "cloud", "self-managed-3.3"]);
    assert_eq!(site["builds"][0]["editor"], true);
    assert_eq!(site["builds"][1]["editor"], false);
    assert_eq!(
        site["builds"][1]["description"],
        "variants: deployment=cloud; availability: filter cloud"
    );

    // `site` keeps both deployment arms as tabs; `cloud` keeps the cloud arm
    // as plain content.
    assert!(html(&site).contains("Point the agent at your server"));
    let cloud = preview(&mut client, &page, Some("cloud"));
    assert_eq!(cloud["build"], "cloud");
    assert!(
        !html(&cloud).contains("Point the agent at your server"),
        "{}",
        html(&cloud)
    );
    assert!(html(&cloud).contains("Sign in to Quill Cloud"));
    // The self-managed 3.3 build filters out the streaming section (in
    // preview from 3.4).
    let sm = preview(&mut client, &page, Some("self-managed-3.3"));
    assert!(!html(&sm).contains("Streaming sync"), "{}", html(&sm));
    assert!(html(&site).contains("Streaming sync"));
    client.shutdown();
}

#[test]
fn an_unknown_build_is_a_problem_and_still_lists_the_builds() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let result = preview(&mut client, &page, Some("nope"));
    assert!(result["page"].is_null());
    assert_eq!(result["problems"][0]["severity"], "error");
    assert!(
        result["problems"][0]["message"]
            .as_str()
            .unwrap()
            .contains("no build named nope")
    );
    assert_eq!(result["builds"].as_array().unwrap().len(), 3);
    client.shutdown();
}

#[test]
fn links_and_sections_let_the_client_follow_the_page() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let result = preview(&mut client, &page, None);
    let links = result["page"]["links"].as_array().unwrap();
    let keys = links
        .iter()
        .find(|l| l["href"] == "/keys/#rotate-keys")
        .unwrap_or_else(|| panic!("{links:?}"));
    assert_eq!(keys["path"], f.path("docs/keys.md").to_str().unwrap());
    assert_eq!(keys["id"], "rotate-keys");
    assert!(
        html(&result).contains("href=\"/keys/#rotate-keys\""),
        "{}",
        html(&result)
    );

    let sections = result["page"]["sections"].as_array().unwrap();
    let ids: Vec<&str> = sections.iter().map(|s| s["id"].as_str().unwrap()).collect();
    assert_eq!(
        ids,
        [
            "prerequisites",
            "install-agent",
            "connect",
            "streaming-sync",
            "troubleshooting"
        ]
    );
    // The heading of `_fragments/prerequisites.md`, if it had one, isn't a
    // line of this file, so it isn't listed; these are lines of this file.
    let text = std::fs::read_to_string(&page).unwrap();
    let line = sections[0]["line"].as_u64().unwrap() as usize;
    assert_eq!(text.lines().nth(line).unwrap(), "## Prerequisites");
    client.shutdown();
}

#[test]
fn what_cannot_be_previewed_says_why() {
    let f = quill();
    let mut client = Client::start(&f.root());
    client.settle();

    let fragment = preview(
        &mut client,
        &f.path("docs/_fragments/prerequisites.md"),
        None,
    );
    assert!(fragment["page"].is_null());
    let message = fragment["problems"][0]["message"].as_str().unwrap();
    assert!(
        message.contains("is a fragment") && message.contains("install-agent.md"),
        "{message}"
    );

    let outside = preview(&mut client, &f.path("README.md"), None);
    assert!(outside["page"].is_null());
    assert_eq!(outside["problems"][0]["severity"], "info");
    // The builds and roots are there even so, for the client's picker.
    assert_eq!(outside["builds"].as_array().unwrap().len(), 3);
    client.shutdown();
}

#[test]
fn a_page_a_build_drops_says_so() {
    let model = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n\n[builds.site]\nvariants = \"switch\"\navailability = \"badge\"\n\n[builds.cloud]\nvariants = { deployment = \"cloud\" }\navailability = \"badge\"\n\n[editor]\nbuild = \"site\"\n";
    let f = Fixture::new(
        model,
        &[(
            "docs/sm.md",
            "---\ntitle: SM\nvariant:\n  deployment: self-managed\n---\n# SM\n",
        )],
    );
    let mut client = Client::start(&f.root());
    client.settle();
    let page = f.path("docs/sm.md");
    assert!(!preview(&mut client, &page, Some("site"))["page"].is_null());
    let dropped = preview(&mut client, &page, Some("cloud"));
    assert!(dropped["page"].is_null());
    let message = dropped["problems"][0]["message"].as_str().unwrap();
    assert!(message.contains("doesn't publish sm.md"), "{message}");
    client.shutdown();
}

#[test]
fn an_asset_beside_the_content_root_is_served_from_its_own_directory() {
    let f = Fixture::new(
        MODEL,
        &[
            (
                "docs/page.md",
                "# Page\n\n![Shared](../shared/logo.png)\n\n![Again](../shared/other.png)\n\n![Deep](../assets/icons/i.png)\n",
            ),
            ("shared/logo.png", "not really a png"),
            ("shared/other.png", "not really a png"),
            ("assets/icons/i.png", "not really a png"),
        ],
    );
    let mut client = Client::start(&f.root());
    client.settle();
    let result = preview(&mut client, &f.path("docs/page.md"), None);
    let assets = result["page"]["assets"].as_array().unwrap();
    assert_eq!(assets.len(), 3);
    for asset in assets {
        assert_eq!(asset["servable"], true, "{asset}");
    }
    assert_eq!(assets[0]["reference"], "./_ascribe/up/shared/logo.png");
    assert!(
        assets[0]["path"]
            .as_str()
            .unwrap()
            .ends_with("shared/logo.png")
    );
    // The directory of each asset, once, and nothing wider: not the project root.
    let roots: Vec<&str> = result["assetRoots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect();
    assert_eq!(
        roots,
        [
            f.path("shared").to_str().unwrap(),
            f.path("assets/icons").to_str().unwrap()
        ]
    );
    assert_eq!(result["problems"], json!([]));
    client.shutdown();
}

#[test]
fn assets_the_preview_will_not_serve_are_reported() {
    let f = Fixture::new(
        MODEL,
        &[
            (
                "docs/page.md",
                "# Page\n\n![Root](../logo.png)\n\n![Package](../node_modules/pkg/i.png)\n\n![Inside](inside.png)\n",
            ),
            ("logo.png", "x"),
            ("node_modules/pkg/i.png", "x"),
            ("docs/inside.png", "x"),
        ],
    );
    let mut client = Client::start(&f.root());
    client.settle();
    let result = preview(&mut client, &f.path("docs/page.md"), None);
    let assets = result["page"]["assets"].as_array().unwrap();
    let by_name = |name: &str| {
        assets
            .iter()
            .find(|a| a["path"].as_str().unwrap().ends_with(name))
            .unwrap_or_else(|| panic!("{assets:?}"))
    };
    // A file directly in the project root would need the project root served.
    assert_eq!(by_name("/logo.png")["servable"], false);
    assert_eq!(by_name("node_modules/pkg/i.png")["servable"], false);
    assert_eq!(by_name("docs/inside.png")["servable"], true);
    assert_eq!(result["assetRoots"], json!([]));
    let messages: Vec<&str> = result["problems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["message"].as_str().unwrap())
        .collect();
    assert_eq!(messages.len(), 2, "{messages:?}");
    assert!(
        messages
            .iter()
            .any(|m| m.contains("directly in the project root"))
    );
    assert!(messages.iter().any(|m| m.contains("node_modules")));
    client.shutdown();
}

#[test]
fn two_pages_with_one_route_are_reported_for_both() {
    let f = Fixture::new(
        MODEL,
        &[
            ("docs/a.md", "# One\n"),
            ("docs/a/index.md", "# Two\n"),
            ("docs/other.md", "# Other\n"),
        ],
    );
    let mut client = Client::start(&f.root());
    client.settle();
    for name in ["docs/a.md", "docs/a/index.md"] {
        let result = preview(&mut client, &f.path(name), None);
        assert!(!result["page"].is_null());
        let message = result["problems"][0]["message"].as_str().unwrap();
        assert!(message.contains("can't publish them together"), "{message}");
    }
    let other = preview(&mut client, &f.path("docs/other.md"), None);
    assert_eq!(other["problems"], json!([]));
    client.shutdown();
}

#[test]
fn a_workspace_without_a_project_answers_with_a_problem() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.md"), "# A\n").unwrap();
    let root = support::real_path(dir.path());
    let mut client = Client::start(&root);
    client.settle();
    let result = preview(&mut client, &root.join("a.md"), None);
    assert!(result["page"].is_null());
    assert!(result["builds"].as_array().unwrap().is_empty());
    assert_eq!(result["problems"][0]["severity"], "error");
    client.shutdown();
}

#[test]
fn a_bad_request_is_an_error_not_a_crash() {
    let f = quill();
    let mut client = Client::start(&f.root());
    let response = client.request("ascribe/preview", json!({ "nonsense": true }));
    assert!(response.response_result.is_err());
    // The server still answers.
    let page = f.path("docs/quickstart.md");
    assert!(!preview(&mut client, &page, None)["page"].is_null());
    client.shutdown();
}

#[test]
fn an_answer_takes_milliseconds() {
    let f = quill();
    let page = f.path("docs/install-agent.md");
    let text = std::fs::read_to_string(&page).unwrap();
    let mut client = Client::start(&f.root());
    client.open(&page, 1, &text);
    client.settle();
    let mut times = Vec::new();
    for version in 2..22 {
        client.change(&page, version, vec![edit((5, 0), (5, 0), "x")]);
        let start = Instant::now();
        let result = preview(&mut client, &page, None);
        times.push(start.elapsed());
        assert_eq!(result["documentVersion"], version);
    }
    times.sort();
    let median = times[times.len() / 2];
    eprintln!(
        "preview median {median:?}, max {:?}",
        times[times.len() - 1]
    );
    assert!(median.as_millis() < 100, "{times:?}");
    client.shutdown();
}
