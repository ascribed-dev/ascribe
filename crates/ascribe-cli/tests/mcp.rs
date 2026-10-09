//! `ascribe mcp`, driven over pipes as a host drives it: each tool's result
//! is its command's JSON, no call changes a file, a project is loaded again
//! when its files change, and each resource and prompt is what its command
//! prints. `crates/ascribe-mcp/tests/protocol.rs` has the protocol itself.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use serde_json::{Map, Value, json};
use tempfile::TempDir;

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[types.page]
default = true

[types.page.frontmatter]
title = "string"

[types.guide]
files = ["guides/**"]

[types.guide.frontmatter]
title = "string"
audience = "string"
summary = "string?"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud", "self-managed"]

[phrases]
product = "Quill"

[editor]
build = "cloud"

[builds.cloud]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }

[builds.self-managed]
variants = { deployment = "self-managed" }
availability = { filter = "self-managed" }
"#;

const INSTALL: &str = "---
title: Install Quill
---

@include: _fragments/before.md

## Configure
@id: configure

See [keys](keys.md#rotat) and [{product}](https://quill.dev).

@note{ type = tip }:Careful.

## Upgrade
@available: self-managed

Upgrade {product}.
";

const BEFORE: &str = "## Before you start

You need {product}.
";

const KEYS: &str = "---
title: API keys
---

## Rotate keys

Rotate them.
";

/// The 2026-07-28 revision's `_meta`, which every request carries.
fn modern_meta() -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": {},
        "io.modelcontextprotocol/clientInfo": { "name": "tests", "version": "0" },
    })
}

/// A running `ascribe mcp`, one request at a time.
struct Server {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next_id: u64,
    /// Whether requests carry the 2026-07-28 `_meta`.
    modern: bool,
}

impl Server {
    /// A server in `dir`, for a 2026-07-28 client.
    fn start(dir: &Path) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ascribe"))
            .current_dir(dir)
            .arg("mcp")
            .env("NO_COLOR", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("start ascribe mcp");
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Server {
            child,
            input,
            output,
            next_id: 0,
            modern: true,
        }
    }

    /// A server in `dir`, for a client on an earlier revision, after its
    /// handshake.
    fn start_legacy(dir: &Path, version: &str) -> Server {
        let mut server = Server::start(dir);
        server.modern = false;
        let result = server.result(
            "initialize",
            json!({
                "protocolVersion": version,
                "capabilities": {},
                "clientInfo": { "name": "tests", "version": "0" },
            }),
        );
        assert_eq!(result["protocolVersion"], version);
        server.send_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
        server
    }

    fn send_line(&mut self, line: &str) {
        self.input.write_all(line.as_bytes()).unwrap();
        self.input.write_all(b"\n").unwrap();
        self.input.flush().unwrap();
    }

    fn read_reply(&mut self) -> Value {
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        assert!(line.ends_with('\n'), "the server stopped: {line:?}");
        serde_json::from_str(&line).expect("a JSON line")
    }

    /// Sends a request and returns the whole response.
    fn request(&mut self, method: &str, mut params: Value) -> Value {
        if self.modern {
            params["_meta"] = modern_meta();
        }
        self.next_id += 1;
        let id = self.next_id;
        let request = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        self.send_line(&request.to_string());
        let reply = self.read_reply();
        assert_eq!(reply["id"], id, "{reply}");
        reply
    }

    /// Sends a request and returns its result, which must be one.
    fn result(&mut self, method: &str, params: Value) -> Value {
        let reply = self.request(method, params);
        assert!(reply.get("error").is_none(), "{method}: {reply}");
        reply["result"].clone()
    }

    /// Calls a tool and returns its result.
    fn call(&mut self, tool: &str, arguments: Value) -> Value {
        self.result(
            "tools/call",
            json!({ "name": tool, "arguments": arguments }),
        )
    }

    /// Calls a tool that must succeed, and returns its text and structured
    /// content.
    fn call_ok(&mut self, tool: &str, arguments: Value) -> (String, Value) {
        let result = self.call(tool, arguments.clone());
        assert_eq!(result["isError"], false, "{tool} {arguments}: {result}");
        let text = result["content"][0]["text"].as_str().unwrap().to_owned();
        (text, result["structuredContent"].clone())
    }

    /// Calls a tool that must fail, and returns its message.
    fn call_error(&mut self, tool: &str, arguments: Value) -> String {
        let result = self.call(tool, arguments.clone());
        assert_eq!(result["isError"], true, "{tool} {arguments}: {result}");
        assert!(result.get("structuredContent").is_none(), "{result}");
        result["content"][0]["text"].as_str().unwrap().to_owned()
    }

}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

/// What `ascribe <args>` writes to standard output.
fn stdout(dir: &Path, args: &[&str]) -> String {
    String::from_utf8(ascribe(dir, args).stdout).unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args([
            "-c",
            "user.name=Tests",
            "-c",
            "user.email=tests@example.com",
        ])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Writes the project into `dir`.
fn write_project(dir: &Path) {
    write(&dir.join("ascribe.toml"), MODEL);
    write(&dir.join("docs/install.md"), INSTALL);
    write(&dir.join("docs/_fragments/before.md"), BEFORE);
    write(&dir.join("docs/keys.md"), KEYS);
}

/// A git repository with the project in `site/`, committed, and then a
/// change to a page, so there's something for `ascribe diff` to report.
fn repository() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write_project(&dir.path().join("site"));
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "Start"]);
    write(
        &dir.path().join("site/docs/keys.md"),
        &KEYS.replace("Rotate them.", "Rotate them every month."),
    );
    dir
}

/// Every file under `dir` but `.git`, with its contents.
fn contents(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_name() == ".git" {
                continue;
            }
            if entry.file_type().unwrap().is_dir() {
                walk(&entry.path(), out);
            } else {
                out.insert(entry.path(), fs::read(entry.path()).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, &mut out);
    out
}

#[test]
fn every_tool_returns_its_commands_json_and_changes_nothing() {
    let repo = repository();
    let root = repo.path();
    let before = contents(root);
    let mut server = Server::start(root);

    // Each call, with the command that answers the same, run where the
    // server runs.
    let detailed: &[(&str, Value, &[&str])] = &[
        (
            "ascribe_check",
            json!({ "paths": ["site/docs/install.md"], "response_format": "detailed" }),
            &["check", "site/docs/install.md", "--format", "json"],
        ),
        (
            "ascribe_explain",
            json!({ "code": "ASC037" }),
            &["explain", "ASC037", "--format", "json"],
        ),
        (
            "ascribe_model",
            json!({ "path": "site", "section": "phrases" }),
            &["model", "site", "--section", "phrases", "--format", "json"],
        ),
        (
            "ascribe_outline",
            json!({ "page": "site/docs/install.md", "build": "cloud" }),
            &[
                "outline",
                "site/docs/install.md",
                "--build",
                "cloud",
                "--format",
                "json",
            ],
        ),
        (
            "ascribe_link",
            json!({ "target": "keys.md#rotat", "from": "site/docs/install.md" }),
            &[
                "link",
                "keys.md#rotat",
                "--from",
                "site/docs/install.md",
                "--format",
                "json",
            ],
        ),
        (
            "ascribe_refs",
            json!({ "target": "phrase:product", "project": "site", "response_format": "detailed" }),
            &[
                "refs",
                "phrase:product",
                "--project",
                "site",
                "--format",
                "json",
            ],
        ),
        (
            "ascribe_render",
            json!({ "page": "site/docs/install.md", "build": "self-managed", "frontmatter": true }),
            &[
                "render",
                "site/docs/install.md",
                "--build",
                "self-managed",
                "--frontmatter",
                "--format",
                "json",
            ],
        ),
        (
            "ascribe_format",
            json!({ "paths": ["site/docs"] }),
            &[
                "fmt",
                "--config",
                "site",
                "--check",
                "--format",
                "json",
                "site/docs",
            ],
        ),
        (
            "ascribe_changes",
            json!({ "project": "site", "base": "HEAD" }),
            &[
                "diff",
                "--config",
                "site",
                "--base",
                "HEAD",
                "--format",
                "json",
                "--pages-only",
            ],
        ),
        (
            "ascribe_changes",
            json!({ "project": "site", "base": "HEAD", "blocks": true }),
            &[
                "diff", "--config", "site", "--base", "HEAD", "--format", "json",
            ],
        ),
    ];
    for (tool, arguments, command) in detailed {
        let (text, structured) = server.call_ok(tool, arguments.clone());
        let expected = stdout(root, command);
        assert_eq!(text, expected, "{tool} {arguments}");
        let expected: Value = serde_json::from_str(&expected).unwrap();
        assert_eq!(structured, expected, "{tool} {arguments}");
        assert_eq!(contents(root), before, "{tool} changed a file");
    }

    // The concise forms: the command's text.
    let concise: &[(&str, Value, &[&str])] = &[
        (
            "ascribe_check",
            json!({ "paths": ["site/docs/install.md"] }),
            &["check", "site/docs/install.md", "--format", "concise"],
        ),
        (
            "ascribe_refs",
            json!({ "target": "phrase:product", "project": "site" }),
            &[
                "refs",
                "phrase:product",
                "--project",
                "site",
                "--limit",
                "50",
            ],
        ),
    ];
    for (tool, arguments, command) in concise {
        let (text, structured) = server.call_ok(tool, arguments.clone());
        assert_eq!(text, stdout(root, command), "{tool} {arguments}");
        assert_eq!(structured, json!({ "concise": text }));
    }
}

#[test]
fn unsaved_text_is_checked_as_the_file_it_names() {
    let repo = repository();
    let root = repo.path();
    let before = contents(root);
    let mut server = Server::start(root);
    // The file on disk has other text.
    let unsaved = INSTALL.replace("keys.md#rotat", "keys.md#rotate-keys");
    let (text, _) = server.call_ok(
        "ascribe_check",
        json!({ "text": unsaved, "path": "site/docs/install.md" }),
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(root)
        .args(["check", "--stdin", "--path", "site/docs/install.md"])
        .args(["--format", "concise"])
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    command
        .stdin
        .take()
        .unwrap()
        .write_all(unsaved.as_bytes())
        .unwrap();
    let expected = String::from_utf8(command.wait_with_output().unwrap().stdout).unwrap();
    assert_eq!(text, expected);
    assert!(!text.contains("ASC037"), "{text}");
    assert_eq!(contents(root), before);
}

#[test]
fn the_commands_json_has_what_the_tools_need() {
    let repo = repository();
    let root = repo.path();
    let before = contents(root);

    // `fmt --check --format json`: the edits, and nothing written.
    let out = ascribe(
        root,
        &["fmt", "--check", "--format", "json", "--config", "site"],
    );
    assert_eq!(out.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["written"], false);
    assert_eq!(report["files"][0]["file"], "docs/install.md");
    assert!(
        !report["files"][0]["edits"].as_array().unwrap().is_empty(),
        "{report}"
    );
    assert_eq!(contents(root), before);

    // `diff --pages-only`: the pages, with no blocks.
    let report: Value = serde_json::from_str(&stdout(
        root,
        &[
            "diff",
            "--config",
            "site",
            "--base",
            "HEAD",
            "--format",
            "json",
            "--pages-only",
        ],
    ))
    .unwrap();
    assert_eq!(report["blocks_omitted"], true);
    let page = &report["builds"][0]["pages"][0];
    assert_eq!(page["path"], "keys.md");
    assert_eq!(page["changes"], json!([]), "{page}");
    assert_eq!(page["counts"]["added"], 1, "{page}");
    let full: Value = serde_json::from_str(&stdout(
        root,
        &[
            "diff", "--config", "site", "--base", "HEAD", "--format", "json",
        ],
    ))
    .unwrap();
    assert!(full.get("blocks_omitted").is_none(), "{full}");

    // `fmt --format json` without `--check` writes the files, and says so.
    let out = ascribe(root, &["fmt", "--format", "json", "--config", "site"]);
    assert_eq!(out.status.code(), Some(0));
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["written"], true);
    assert!(
        fs::read_to_string(root.join("site/docs/install.md"))
            .unwrap()
            .contains("@note {type=tip}: Careful.")
    );
}

#[test]
fn both_revisions_list_the_same_tools_in_the_same_order() {
    let repo = repository();
    let mut modern = Server::start(repo.path());
    let discover = modern.result("server/discover", json!({}));
    assert!(
        discover["supportedVersions"]
            .as_array()
            .unwrap()
            .contains(&json!("2026-07-28"))
    );
    assert!(discover["capabilities"]["tools"].is_object());
    assert!(discover["capabilities"]["resources"].is_object());
    assert!(discover["capabilities"]["prompts"].is_object());

    let tools = modern.result("tools/list", json!({}));
    assert_eq!(tools, modern.result("tools/list", json!({})));
    let names: Vec<&str> = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "ascribe_check",
            "ascribe_explain",
            "ascribe_model",
            "ascribe_outline",
            "ascribe_link",
            "ascribe_refs",
            "ascribe_render",
            "ascribe_format",
            "ascribe_changes",
        ]
    );
    for tool in tools["tools"].as_array().unwrap() {
        let annotations = &tool["annotations"];
        assert_eq!(annotations["readOnlyHint"], true, "{tool}");
        assert_eq!(annotations["destructiveHint"], false, "{tool}");
        assert_eq!(annotations["idempotentHint"], true, "{tool}");
        assert_eq!(annotations["openWorldHint"], false, "{tool}");
        assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
        assert_eq!(tool["outputSchema"]["type"], "object", "{tool}");
        assert!(tool["description"].as_str().unwrap().contains("Example: "));
    }

    for version in ["2025-11-25", "2025-06-18"] {
        let mut legacy = Server::start_legacy(repo.path(), version);
        let listed = legacy.result("tools/list", json!({}));
        assert_eq!(listed["tools"], tools["tools"], "{version}");
        let (text, _) = legacy.call_ok("ascribe_explain", json!({ "code": "ASC037" }));
        assert!(text.contains("link-id-missing"), "{text}");
    }
}

#[test]
fn bad_calls_are_errors_with_what_to_do() {
    let repo = repository();
    let root = repo.path();
    let mut server = Server::start(root);

    let reply = server.request(
        "tools/call",
        json!({ "name": "ascribe_build", "arguments": {} }),
    );
    assert_eq!(reply["error"]["code"], -32602, "{reply}");

    let message = server.call_error("ascribe_outline", json!({}));
    assert!(message.contains("needs `page`"), "{message}");
    let message = server.call_error("ascribe_outline", json!({ "page": 3 }));
    assert!(message.contains("`page` is a string"), "{message}");
    let message = server.call_error("ascribe_outline", json!({ "page": "a.md", "depth": 2 }));
    assert!(message.contains("no argument `depth`"), "{message}");
    let message = server.call_error(
        "ascribe_check",
        json!({ "paths": ["site"], "response_format": "short" }),
    );
    assert!(message.contains("`concise` or `detailed`"), "{message}");
    let message = server.call_error("ascribe_explain", json!({ "code": "ASC999" }));
    assert!(message.contains("ascribe explain --list"), "{message}");

    // A malformed line, then the server goes on.
    server.send_line("{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\"");
    let reply = server.read_reply();
    assert_eq!(reply["error"]["code"], -32700, "{reply}");
    server.call_ok("ascribe_explain", json!({ "code": "ASC037" }));
}

#[test]
fn one_server_serves_two_projects_and_says_when_a_path_is_in_neither() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_project(&root.join("guides"));
    write_project(&root.join("api"));
    write(
        &root.join("api/docs/keys.md"),
        &KEYS.replace("## Rotate keys", "## Revoke keys"),
    );
    write(&root.join("notes/todo.md"), "# To do\n");
    let mut server = Server::start(root);

    let (_, guides) = server.call_ok("ascribe_outline", json!({ "page": "guides/docs/keys.md" }));
    assert_eq!(guides["headings"][0]["id"], "rotate-keys", "{guides}");
    let (_, api) = server.call_ok("ascribe_outline", json!({ "page": "api/docs/keys.md" }));
    assert_eq!(api["headings"][0]["id"], "revoke-keys", "{api}");

    let message = server.call_error("ascribe_outline", json!({ "page": "notes/todo.md" }));
    assert!(message.contains("ascribe.toml"), "{message}");
    assert!(
        message.contains("Paths are read from the server's working directory"),
        "{message}"
    );
}

#[test]
fn a_project_is_loaded_again_when_its_files_change() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_project(root);
    let mut server = Server::start(root);
    let link = |server: &mut Server, target: &str| {
        let (_, answer) = server.call_ok(
            "ascribe_link",
            json!({ "target": target, "from": "docs/install.md" }),
        );
        answer["exists"].as_bool().unwrap()
    };
    assert!(link(&mut server, "keys.md#rotate-keys"));
    let check = |server: &mut Server| {
        let (text, _) = server.call_ok("ascribe_check", json!({ "paths": ["docs/install.md"] }));
        text
    };
    assert!(check(&mut server).contains("[ASC037]"));
    // The second check is kept from the first.
    assert!(check(&mut server).contains("[ASC037]"));
    write(
        &root.join("docs/install.md"),
        &INSTALL.replace("keys.md#rotat", "keys.md#rotate-keys"),
    );
    assert!(!check(&mut server).contains("[ASC037]"));

    // A file changed.
    write(
        &root.join("docs/keys.md"),
        &format!("{KEYS}\n## Revoke keys\n\nRevoke them.\n"),
    );
    assert!(link(&mut server, "keys.md#revoke-keys"));

    // A file added.
    assert!(!link(&mut server, "tokens.md"));
    write(&root.join("docs/tokens.md"), "---\ntitle: Tokens\n---\n");
    assert!(link(&mut server, "tokens.md"));

    // A file deleted.
    fs::remove_file(root.join("docs/tokens.md")).unwrap();
    assert!(!link(&mut server, "tokens.md"));

    // ascribe.toml changed.
    let (_, model) = server.call_ok("ascribe_model", json!({ "section": "phrases" }));
    assert_eq!(model["phrases"].as_array().unwrap().len(), 1, "{model}");
    write(
        &root.join("ascribe.toml"),
        &MODEL.replace(
            "product = \"Quill\"",
            "product = \"Quill\"\ncompany = \"Inkwell\"",
        ),
    );
    let (_, model) = server.call_ok("ascribe_model", json!({ "section": "phrases" }));
    assert_eq!(model["phrases"].as_array().unwrap().len(), 2, "{model}");
    let (text, _) = server.call_ok("ascribe_refs", json!({ "target": "phrase:company" }));
    assert!(text.contains("0 places") || text.contains("No "), "{text}");
}

/// The arguments a tool's description gives as its example are ones its
/// input schema has.
#[test]
fn each_tools_example_uses_its_arguments() {
    let repo = repository();
    let mut server = Server::start(repo.path());
    let tools = server.result("tools/list", json!({}));
    for tool in tools["tools"].as_array().unwrap() {
        let description = tool["description"].as_str().unwrap();
        let example = description
            .split("Example: ")
            .nth(1)
            .and_then(|rest| rest.split(" returns").next())
            .unwrap();
        let example: Map<String, Value> = serde_json::from_str(example)
            .unwrap_or_else(|e| panic!("{}: {example}: {e}", tool["name"]));
        let properties = tool["inputSchema"]["properties"].as_object().unwrap();
        for key in example.keys() {
            assert!(properties.contains_key(key), "{}: {key}", tool["name"]);
        }
    }
}
