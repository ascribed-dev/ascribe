//! A link to an operation of a declared API's spec in the editor: its
//! completion after `#`, its hover, going to it in the spec, the spec read
//! again when it changes on disk, inside the project's folder or outside it,
//! and the safe fix for a link to an operation that was renamed, from what
//! the server read before or from the review's base.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::support;

use std::path::Path;
use std::process::Command;

use crate::support::{Client, Fixture, MODEL, uri};
use lsp_types::FileChangeType;
use serde_json::{Value, json};

const APIS: &str = "\n[apis.public]\nspec = \"api/openapi.yaml\"\nreference = { renderer = \"redoc\", at = \"/api/\" }\n";

const SPEC: &str = "openapi: 3.1.0
info: { title: Users, version: \"1\" }
paths:
  /users:
    get:
      operationId: listUsers
      summary: List users
      tags: [Users]
  /users/{id}:
    get:
      operationId: getUser
      summary: Get a user
      tags: [Users, Admin]
      deprecated: true
    delete:
      summary: Delete a user
  /teams:
    get:
      operationId: listTeams
      summary: List teams
";

/// `SPEC` once `getUser` is `fetchUser`, as a regenerated spec has it.
fn renamed() -> String {
    SPEC.replace("operationId: getUser", "operationId: fetchUser")
}

const PAGE: &str =
    "---\ntitle: Users\n---\n\nFetch one with [Get a user](../api/openapi.yaml#getUser).\n";

fn project() -> Fixture {
    Fixture::new(
        &format!("{MODEL}{APIS}"),
        &[("api/openapi.yaml", SPEC), ("docs/users.md", PAGE)],
    )
}

fn text_doc(path: &Path) -> Value {
    json!({ "uri": uri(path).as_str() })
}

/// The LSP position of the end of the first `needle` in `text`.
fn after(text: &str, needle: &str) -> Value {
    let end = text.find(needle).expect("the needle") + needle.len();
    let before = &text[..end];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    json!({ "line": line, "character": text[start..end].encode_utf16().count() })
}

fn complete(client: &mut Client, path: &Path, text: &str, needle: &str) -> Vec<Value> {
    client.replace(path, 2, text);
    let result = client
        .request(
            "textDocument/completion",
            json!({ "textDocument": text_doc(path), "position": after(text, needle) }),
        )
        .response_result
        .expect("a result");
    result["items"].as_array().cloned().unwrap_or_default()
}

fn labels(items: &[Value]) -> Vec<&str> {
    items.iter().filter_map(|i| i["label"].as_str()).collect()
}

fn diagnostic(client: &Client, path: &Path, slug: &str) -> Option<Value> {
    client
        .diagnostics(path)
        .iter()
        .find(|d| support::slug(d) == slug)
        .map(|d| serde_json::to_value(d).expect("a diagnostic"))
}

/// The titles of the quick fixes of a diagnostic, and whether the page
/// has a safe fix, which "Apply the safe fixes in this page" applies, as
/// `ascribe check --fix` does.
fn fixes(client: &mut Client, path: &Path, diagnostic: &Value) -> (Vec<String>, bool) {
    let actions = client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": text_doc(path),
                "range": diagnostic["range"],
                "context": { "diagnostics": [diagnostic] }
            }),
        )
        .response_result
        .expect("actions");
    let titles: Vec<String> = actions
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|a| a.get("edit").is_some())
        .filter_map(|a| a["title"].as_str().map(str::to_owned))
        .collect();
    let safe = titles.iter().any(|t| t == SAFE_FIXES);
    (
        titles.into_iter().filter(|t| t != SAFE_FIXES).collect(),
        safe,
    )
}

const SAFE_FIXES: &str = "Apply the safe fixes in this page";

#[test]
fn openapi_operations_are_completed_after_the_spec_s_hash_by_id_method_path_or_summary() {
    let f = project();
    let path = f.path("docs/users.md");
    let mut client = Client::start(&f.root());
    client.open(&path, 1, PAGE);
    client.settle();

    let typed = "---\ntitle: Users\n---\n\nSee [them](../api/openapi.yaml#";
    let all = complete(&mut client, &path, typed, "#");
    assert_eq!(
        labels(&all),
        ["listUsers", "getUser", "delete/users/{id}", "listTeams"]
    );
    let get = &all[1];
    assert_eq!(get["detail"], "GET /users/{id}");
    assert_eq!(get["labelDetails"]["description"], "Get a user");
    assert_eq!(get["tags"], json!([1]), "deprecated");
    let docs = get["documentation"]["value"].as_str().expect("docs");
    assert!(
        docs.contains("Get a user") && docs.contains("Deprecated"),
        "{docs}"
    );
    // An operation with no id is written by its method and path.
    assert_eq!(all[2]["textEdit"]["newText"], "delete/users/{id}");

    let by_id = complete(&mut client, &path, &format!("{typed}us"), "#us");
    assert_eq!(
        labels(&by_id),
        ["listUsers", "getUser", "delete/users/{id}"]
    );
    let by_summary = complete(&mut client, &path, &format!("{typed}team"), "#team");
    assert_eq!(labels(&by_summary), ["listTeams"]);
    let by_method = complete(&mut client, &path, &format!("{typed}delete"), "#delete");
    assert_eq!(labels(&by_method), ["delete/users/{id}"]);
    let edit = &by_summary[0]["textEdit"];
    assert_eq!(edit["newText"], "listTeams");
    assert_eq!(edit["range"]["start"], after(typed, "#"));
}

#[test]
fn openapi_a_link_to_an_operation_shows_it_on_hover_and_opens_it_in_the_spec() {
    let f = project();
    let path = f.path("docs/users.md");
    let mut client = Client::start(&f.root());
    client.open(&path, 1, PAGE);
    client.settle();
    let on_link = after(PAGE, "openapi.yaml#get");

    let hover = client
        .request(
            "textDocument/hover",
            json!({ "textDocument": text_doc(&path), "position": on_link }),
        )
        .response_result
        .expect("a hover");
    let text = hover["contents"]["value"].as_str().expect("markup");
    assert!(text.contains("**GET** `/users/{id}`"), "{text}");
    assert!(text.contains("Get a user"), "{text}");
    assert!(text.contains("**Deprecated**"), "{text}");
    assert!(text.contains("tagged Users"), "{text}");

    let location = client
        .request(
            "textDocument/definition",
            json!({ "textDocument": text_doc(&path), "position": on_link }),
        )
        .response_result
        .expect("a location");
    assert_eq!(location["uri"], uri(&f.path("api/openapi.yaml")).as_str());
    let line = SPEC
        .lines()
        .position(|l| l == "      operationId: getUser")
        .expect("getUser")
        - 1;
    assert_eq!(
        location["range"]["start"],
        json!({ "line": line, "character": 0 })
    );
    // The operation is deprecated, which the page is told.
    assert!(diagnostic(&client, &path, "link-operation-deprecated").is_some());
}

#[test]
fn openapi_a_spec_changed_on_disk_is_read_again_and_a_renamed_operation_is_fixed() {
    let f = project();
    let path = f.path("docs/users.md");
    let mut client = Client::start(&f.root());
    client.settle();
    assert!(diagnostic(&client, &path, "link-operation-missing").is_none());

    f.write("api/openapi.yaml", &renamed());
    client.watched(&[(&f.path("api/openapi.yaml"), FileChangeType::CHANGED)]);
    client.settle();
    let missing = diagnostic(&client, &path, "link-operation-missing")
        .expect("the link to getUser is missing now");
    let (titles, safe) = fixes(&mut client, &path, &missing);
    assert_eq!(
        titles[0], "Link to `../api/openapi.yaml#fetchUser`",
        "{titles:?}"
    );
    assert!(safe, "the rename is safe");
}

#[test]
fn openapi_a_spec_outside_the_folder_is_watched_and_read_again() {
    let f = Fixture::new(
        &format!(
            "{MODEL}{}",
            APIS.replace("api/openapi.yaml", "../api/openapi.yaml")
        ),
        &[("api/openapi.yaml", SPEC), ("docs/users.md", PAGE)],
    );
    // The project is `site/`, in a repository whose spec is beside it.
    for file in ["ascribe.toml", "docs/users.md"] {
        f.write(
            &format!("site/{file}"),
            &std::fs::read_to_string(f.path(file)).unwrap(),
        );
        f.remove(file);
    }
    f.write("site/docs/users.md", &PAGE.replace("../api/", "../../api/"));
    std::fs::create_dir(f.path(".git")).expect("a repository");
    let site = f.path("site");
    let path = f.path("site/docs/users.md");
    let mut client = Client::start(&site);
    client.settle();
    let spec = f.path("api/openapi.yaml");
    let watched = client.requests_from_server.iter().any(|r| {
        r.method == "client/registerCapability"
            && r.params.to_string().contains("ascribe-spec-files")
            && r.params.to_string().contains("openapi.yaml")
    });
    assert!(watched, "{:?}", client.requests_from_server);
    assert!(diagnostic(&client, &path, "link-operation-missing").is_none());

    f.write("api/openapi.yaml", &renamed());
    client.watched(&[(&spec, FileChangeType::CHANGED)]);
    client.settle();
    let missing = diagnostic(&client, &path, "link-operation-missing").expect("missing now");
    let (titles, safe) = fixes(&mut client, &path, &missing);
    assert_eq!(
        titles[0], "Link to `../../api/openapi.yaml#fetchUser`",
        "{titles:?}"
    );
    assert!(safe, "the rename is safe");
}

fn git(root: &Path, args: &[&str]) {
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
}

#[test]
fn openapi_a_renamed_operation_is_fixed_from_the_review_s_base() {
    let f = project();
    git(&f.root(), &["init", "-q", "-b", "main"]);
    git(&f.root(), &["add", "-A"]);
    git(&f.root(), &["commit", "-q", "-m", "First"]);
    f.write("api/openapi.yaml", &renamed());
    let path = f.path("docs/users.md");
    let mut client = Client::start(&f.root());
    client.settle();
    // The server never read `getUser`: which operation took its place is a
    // guess.
    let missing = diagnostic(&client, &path, "link-operation-missing").expect("missing");
    assert!(!fixes(&mut client, &path, &missing).1);

    let set = client
        .request("ascribe/review/setBase", json!({ "base": "main" }))
        .response_result
        .expect("the base is set");
    assert!(set["problem"].is_null(), "{set}");
    client.settle();
    let missing = diagnostic(&client, &path, "link-operation-missing").expect("missing");
    let (titles, safe) = fixes(&mut client, &path, &missing);
    assert_eq!(
        titles[0], "Link to `../api/openapi.yaml#fetchUser`",
        "{titles:?}"
    );
    assert!(safe, "the rename is safe");
}

/// `PAGE`, putting `getUser` in the page with `@operation` instead.
const EMBED: &str = "---\ntitle: Users\n---\n\n@operation: ../api/openapi.yaml#getUser\n";

fn embedding() -> Fixture {
    Fixture::new(
        &format!("{MODEL}{APIS}"),
        &[("api/openapi.yaml", SPEC), ("docs/users.md", EMBED)],
    )
}

#[test]
fn openapi_an_operation_directive_completes_its_operations_after_the_spec_s_hash() {
    let f = embedding();
    let path = f.path("docs/users.md");
    let mut client = Client::start(&f.root());
    client.open(&path, 1, EMBED);
    client.settle();

    let typed = "---\ntitle: Users\n---\n\n@operation: ../api/openapi.yaml#";
    let all = complete(&mut client, &path, typed, "#");
    assert_eq!(
        labels(&all),
        ["listUsers", "getUser", "delete/users/{id}", "listTeams"]
    );
    assert_eq!(all[1]["detail"], "GET /users/{id}");
    assert_eq!(all[1]["tags"], json!([1]), "deprecated");
    let by_summary = complete(&mut client, &path, &format!("{typed}team"), "#team");
    assert_eq!(labels(&by_summary), ["listTeams"]);
    let edit = &by_summary[0]["textEdit"];
    assert_eq!(edit["newText"], "listTeams");
    assert_eq!(edit["range"]["start"], after(typed, "#"));
    // A file that isn't a declared spec has no operations to offer.
    let page = "---\ntitle: Users\n---\n\n@operation: users.md#";
    assert!(complete(&mut client, &path, page, "#").is_empty());
}

#[test]
fn openapi_an_operation_directive_shows_its_operation_on_hover_and_opens_it_in_the_spec() {
    let f = embedding();
    let path = f.path("docs/users.md");
    let mut client = Client::start(&f.root());
    client.open(&path, 1, EMBED);
    client.settle();
    let on_link = after(EMBED, "openapi.yaml#get");

    let hover = client
        .request(
            "textDocument/hover",
            json!({ "textDocument": text_doc(&path), "position": on_link }),
        )
        .response_result
        .expect("a hover");
    let text = hover["contents"]["value"].as_str().expect("markup");
    assert!(text.contains("**GET** `/users/{id}`"), "{text}");
    assert!(text.contains("Get a user"), "{text}");
    assert!(text.contains("**Deprecated**"), "{text}");
    assert!(text.contains("tagged Users"), "{text}");
    assert_eq!(hover["range"]["start"], after(EMBED, "@operation: "));

    let location = client
        .request(
            "textDocument/definition",
            json!({ "textDocument": text_doc(&path), "position": on_link }),
        )
        .response_result
        .expect("a location");
    assert_eq!(location["uri"], uri(&f.path("api/openapi.yaml")).as_str());
    let line = SPEC
        .lines()
        .position(|l| l == "      operationId: getUser")
        .expect("getUser")
        - 1;
    assert_eq!(
        location["range"]["start"],
        json!({ "line": line, "character": 0 })
    );
    // The operation is deprecated, which the page is told.
    assert!(diagnostic(&client, &path, "link-operation-deprecated").is_some());
}

#[test]
fn openapi_an_operation_directive_to_a_renamed_operation_is_fixed() {
    let f = embedding();
    let path = f.path("docs/users.md");
    let mut client = Client::start(&f.root());
    client.settle();
    assert!(diagnostic(&client, &path, "link-operation-missing").is_none());

    f.write("api/openapi.yaml", &renamed());
    client.watched(&[(&f.path("api/openapi.yaml"), FileChangeType::CHANGED)]);
    client.settle();
    let missing = diagnostic(&client, &path, "link-operation-missing")
        .expect("the @operation's getUser is missing now");
    assert_eq!(missing["range"]["start"], after(EMBED, "@operation: "));
    let (titles, safe) = fixes(&mut client, &path, &missing);
    assert_eq!(
        titles[0], "Link to `../api/openapi.yaml#fetchUser`",
        "{titles:?}"
    );
    assert!(safe, "the rename is safe");
}
