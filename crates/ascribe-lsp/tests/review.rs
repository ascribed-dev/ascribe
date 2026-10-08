//! Review: `ascribe/review/setBase`, `ascribe/review/changes`, and
//! `ascribe/preview` with `review: true`, in a temporary git repository.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};
use support::{Client, Fixture, MODEL, edit};

const PAGE: &str = "---\ntitle: Install\n---\n\n# Install\n\nRun the installer.\n\n@include: _fragments/check.md\n\nThen restart the agent so it picks up the new settings.\n";
const FRAGMENT: &str = "Check the version.\n";
const OTHER: &str = "---\ntitle: Other\n---\n\nNothing here.\n";

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

/// A project in a repository with one commit on `main`.
fn repository() -> Fixture {
    let f = Fixture::new(
        MODEL,
        &[
            ("docs/install.md", PAGE),
            ("docs/_fragments/check.md", FRAGMENT),
            ("docs/other.md", OTHER),
        ],
    );
    git(&f.root(), &["init", "-q", "-b", "main"]);
    git(&f.root(), &["add", "-A"]);
    git(&f.root(), &["commit", "-q", "-m", "First"]);
    f
}

fn set_base(client: &mut Client, base: Value) -> Value {
    client
        .request("ascribe/review/setBase", base)
        .response_result
        .expect("the request succeeds")
}

fn preview(client: &mut Client, path: &Path) -> Value {
    client
        .request(
            "ascribe/preview",
            json!({ "textDocument": { "uri": support::uri(path).as_str() }, "review": true }),
        )
        .response_result
        .expect("the request succeeds")
}

fn changes(client: &mut Client) -> Value {
    client
        .request("ascribe/review/changes", json!({}))
        .response_result
        .expect("the request succeeds")
}

fn kinds(review: &Value) -> Vec<String> {
    review["changes"]["changes"]
        .as_array()
        .map(|changes| {
            changes
                .iter()
                .map(|c| c["kind"].as_str().unwrap().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn the_preview_marks_changes_against_the_base_as_the_buffer_changes() {
    let f = repository();
    let page = f.path("docs/install.md");
    // A change on disk, not committed.
    f.write(
        "docs/install.md",
        &PAGE.replace("new settings", "new key and settings"),
    );
    let mut client = Client::start(&f.root());
    client.settle();

    // Without a base, nothing is compared.
    assert_eq!(preview(&mut client, &page)["review"], Value::Null);

    let set = set_base(&mut client, json!({}));
    assert_eq!(set["problem"], Value::Null);
    assert_eq!(set["base"]["requested"], "main");
    let head = git(&f.root(), &["rev-parse", "HEAD"]);
    assert_eq!(set["base"]["commit"], head.as_str());
    assert_eq!(set["base"]["merge_base"], head.as_str());

    let result = preview(&mut client, &page);
    let review = &result["review"];
    assert_eq!(review["base"]["requested"], "main");
    assert_eq!(review["changes"]["status"], "changed");
    assert_eq!(review["changes"]["own_file_changed"], true);
    assert_eq!(kinds(review), ["changed"]);
    let change = &review["changes"]["changes"][0];
    assert_eq!(
        change["words"]["now_text"],
        "Then restart the agent so it picks up the new key and settings."
    );
    // The changed block's anchor is on the page now, and its old one on the
    // page as it was.
    let now = change["now"]["source"].as_str().unwrap();
    let was = change["was"]["source"].as_str().unwrap();
    assert!(result["page"]["html"].as_str().unwrap().contains(now));
    let was_html = review["wasHtml"].as_str().expect("the page as it was");
    assert!(was_html.contains(was), "{was_html}");
    assert!(was_html.contains("new settings."), "{was_html}");

    // An unsaved edit adds a block.
    let text = PAGE.replace("new settings", "new key and settings");
    client.open(&page, 1, &text);
    let end = text.lines().count() as u32;
    client.change(&page, 2, vec![edit((end, 0), (end, 0), "\nYou're done.\n")]);
    let review = preview(&mut client, &page)["review"].clone();
    assert_eq!(kinds(&review), ["changed", "added"]);
    assert_eq!(review["changes"]["counts"]["added"], 1);

    // Back to the base's text: nothing changed.
    client.replace(&page, 3, PAGE);
    let review = preview(&mut client, &page)["review"].clone();
    assert_eq!(review["changes"], Value::Null);
    assert_eq!(review["wasHtml"], Value::Null);
    assert_eq!(review["base"]["requested"], "main");
}

#[test]
fn a_fragment_edit_changes_the_page_that_includes_it() {
    let f = repository();
    let page = f.path("docs/install.md");
    let fragment = f.path("docs/_fragments/check.md");
    let mut client = Client::start(&f.root());
    client.settle();
    set_base(&mut client, json!({ "base": "main" }));

    let listed = changes(&mut client);
    assert_eq!(listed["pages"], json!([]));
    assert_eq!(listed["problem"], Value::Null);

    client.open(&fragment, 1, FRAGMENT);
    client.replace(&fragment, 2, "Check the version first.\n");
    let listed = changes(&mut client);
    assert_eq!(listed["base"]["requested"], "main");
    let pages = listed["pages"].as_array().unwrap();
    assert_eq!(pages.len(), 1, "{listed}");
    assert_eq!(pages[0]["path"], "install.md");
    assert_eq!(pages[0]["title"], "Install");
    assert_eq!(pages[0]["own_file_changed"], false);
    assert_eq!(pages[0]["because"], json!(["_fragments/check.md"]));
    assert_eq!(pages[0]["counts"]["changed"], 1);
    assert!(pages[0].get("changes").is_none(), "{listed}");
    assert!(path_ends_with(&listed["contentRoot"], "/docs"));

    let review = preview(&mut client, &page)["review"].clone();
    assert_eq!(kinds(&review), ["changed"]);
    let change = &review["changes"]["changes"][0];
    assert_eq!(change["now"]["via"], json!(["install.md:9"]));
}

#[test]
fn a_base_that_fails_says_why_and_keeps_the_one_before() {
    let f = repository();
    let page = f.path("docs/install.md");
    let mut client = Client::start(&f.root());
    client.settle();

    let set = set_base(&mut client, json!({ "base": "no-such-branch" }));
    assert_eq!(set["base"], Value::Null);
    assert_eq!(
        set["problem"],
        "`no-such-branch` isn't a branch, tag, or commit of this repository."
    );
    assert_eq!(preview(&mut client, &page)["review"], Value::Null);

    set_base(&mut client, json!({ "base": "main" }));
    let set = set_base(&mut client, json!({ "base": "no-such-branch" }));
    assert!(set["problem"].is_string());
    assert_eq!(
        preview(&mut client, &page)["review"]["base"]["requested"],
        "main"
    );

    // Dropping the base turns review off.
    let set = set_base(&mut client, json!({ "base": null }));
    assert_eq!(set, json!({ "base": null, "problem": null }));
    assert_eq!(preview(&mut client, &page)["review"], Value::Null);
    assert_eq!(
        changes(&mut client)["problem"],
        "Review is off for this project."
    );
}

#[test]
fn a_project_outside_a_repository_gets_a_clear_message_and_a_working_preview() {
    let f = Fixture::new(
        MODEL,
        &[
            ("docs/install.md", PAGE),
            ("docs/_fragments/check.md", FRAGMENT),
        ],
    );
    // Keep git from finding a repository above the temporary directory.
    let page = f.path("docs/install.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let set = set_base(&mut client, json!({}));
    if git_finds_a_repository(&f.root()) {
        return;
    }
    let problem = set["problem"].as_str().expect("a problem");
    assert!(
        problem.starts_with("Review compares the project with a git revision, and "),
        "{problem}"
    );
    assert!(
        problem.ends_with(" isn't in a git repository."),
        "{problem}"
    );
    let result = preview(&mut client, &page);
    assert_eq!(result["review"], Value::Null);
    assert!(
        result["page"]["html"]
            .as_str()
            .unwrap()
            .contains("Run the installer.")
    );
}

#[test]
fn a_new_page_is_added_and_has_no_page_as_it_was() {
    let f = repository();
    f.write("docs/new.md", "---\ntitle: New\n---\n\nFresh.\n");
    let mut client = Client::start(&f.root());
    client.settle();
    set_base(&mut client, json!({}));
    let review = preview(&mut client, &f.path("docs/new.md"))["review"].clone();
    assert_eq!(review["changes"]["status"], "added");
    assert_eq!(review["wasHtml"], Value::Null);
    let listed = changes(&mut client);
    assert_eq!(listed["pages"][0]["path"], "new.md");
    assert_eq!(listed["pages"][0]["title"], "New");
    // Its title's field doesn't set `inline`.
    assert_eq!(listed["pages"][0]["formatted_title"], Value::Null);
}

#[test]
fn a_title_with_code_is_listed_and_previewed_formatted() {
    let f = repository();
    f.write(
        "ascribe.toml",
        &format!(
            "{MODEL}\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = {{ type = \"string\", inline = \"code\" }}\n"
        ),
    );
    git(&f.root(), &["commit", "-q", "-am", "Code in titles"]);
    f.write(
        "docs/keys.md",
        "---\ntitle: \"`ascribe.toml` keys\"\n---\n\nKeys.\n",
    );
    let mut client = Client::start(&f.root());
    client.settle();
    set_base(&mut client, json!({}));
    let formatted = json!([
        { "type": "code", "value": "ascribe.toml" },
        { "type": "text", "value": " keys" },
    ]);
    let listed = changes(&mut client);
    assert_eq!(listed["pages"][0]["path"], "keys.md", "{listed}");
    assert_eq!(listed["pages"][0]["title"], "ascribe.toml keys");
    assert_eq!(listed["pages"][0]["formatted_title"], formatted);
    let page = &preview(&mut client, &f.path("docs/keys.md"))["page"];
    assert_eq!(page["title"], "ascribe.toml keys");
    assert_eq!(page["formattedTitle"], formatted);
    // A title without code spans still has a formatted form.
    let page = &preview(&mut client, &f.path("docs/other.md"))["page"];
    assert_eq!(
        page["formattedTitle"],
        json!([{ "type": "text", "value": "Other" }])
    );
}

#[test]
fn setting_the_base_again_follows_it_when_it_moved_and_keeps_it_when_not() {
    let f = repository();
    let page = f.path("docs/install.md");
    f.write("docs/install.md", &PAGE.replace("new settings", "new key"));
    let mut client = Client::start(&f.root());
    client.settle();
    let first = set_base(&mut client, json!({ "base": "main" }));
    let listed = changes(&mut client);
    assert_eq!(listed["pages"].as_array().unwrap().len(), 1, "{listed}");
    // Asked again with nothing changed, the answer is the same.
    assert_eq!(changes(&mut client), listed);
    let was = preview(&mut client, &page)["review"]["wasHtml"].clone();
    assert!(was.as_str().unwrap().contains("new settings."), "{was}");

    // Nothing moved: the same base.
    assert_eq!(set_base(&mut client, json!({ "base": "main" })), first);
    assert_eq!(changes(&mut client), listed);

    // The change is committed, so the point the branch left main moved.
    git(&f.root(), &["commit", "-q", "-am", "Second"]);
    let moved = set_base(&mut client, json!({ "base": "main" }));
    let head = git(&f.root(), &["rev-parse", "HEAD"]);
    assert_eq!(moved["base"]["merge_base"], head.as_str());
    assert_ne!(moved, first);
    assert_eq!(changes(&mut client)["pages"], json!([]));
    assert_eq!(
        preview(&mut client, &page)["review"]["changes"],
        Value::Null
    );
}

fn git_finds_a_repository(dir: &Path) -> bool {
    Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--git-dir"])
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Whether a path the server reported ends with `suffix`, written with `/`.
fn path_ends_with(path: &Value, suffix: &str) -> bool {
    path.as_str().unwrap().replace('\\', "/").ends_with(suffix)
}
