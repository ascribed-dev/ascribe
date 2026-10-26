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

[types.page.frontmatter]
title = "string"

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
type: page
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
type: page
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
        Server::start_with(dir, &[])
    }

    /// A server in `dir`, for a 2026-07-28 client, with these environment
    /// variables set.
    fn start_with(dir: &Path, env: &[(&str, std::ffi::OsString)]) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ascribe"))
            .current_dir(dir)
            .arg("mcp")
            .env("NO_COLOR", "1")
            .envs(env.iter().map(|(k, v)| (k, v)))
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

    fn read_resource(&mut self, uri: &str) -> String {
        let result = self.result("resources/read", json!({ "uri": uri }));
        result["contents"][0]["text"].as_str().unwrap().to_owned()
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
            "ascribe_model",
            json!({ "path": "site", "section": "consumer" }),
            &["model", "site", "--section", "consumer", "--format", "json"],
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
                "--config",
                "site",
                "--format",
                "json",
            ],
        ),
        (
            "ascribe_resolve",
            json!({ "page": "site/docs/install.md", "build": "self-managed", "frontmatter": true }),
            &[
                "resolve",
                "site/docs/install.md",
                "--build",
                "self-managed",
                "--frontmatter",
                "--format",
                "json",
            ],
        ),
        (
            "ascribe_fmt",
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
            "ascribe_diff",
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
            "ascribe_diff",
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
                "--config",
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

/// `ascribe_resolve` with `go:<name>` returns what `ascribe resolve
/// go:<name> --format json` writes, and the command's message for a name the
/// build doesn't publish.
#[test]
fn resolve_shows_a_permalinks_help() {
    let repo = repository();
    let root = repo.path();
    write(
        &root.join("site/docs/rotate.md"),
        "---\ntype: page\ntitle: Rotating keys\n---\n\n## Rotate keys\n@id {permalink=true}: rotate-keys\n\nRotate them.\n",
    );
    let mut server = Server::start(root);
    let (text, structured) = server.call_ok(
        "ascribe_resolve",
        json!({ "page": "go:rotate-keys", "build": "cloud", "project": "site" }),
    );
    let expected = stdout(
        root,
        &[
            "resolve",
            "go:rotate-keys",
            "--build",
            "cloud",
            "--config",
            "site",
            "--format",
            "json",
        ],
    );
    assert_eq!(text, expected);
    assert_eq!(
        structured,
        serde_json::from_str::<Value>(&expected).unwrap()
    );
    assert_eq!(structured["title"], "Rotate keys", "{structured}");
    assert_eq!(structured["text"], "Rotate them.", "{structured}");

    let message = server.call_error(
        "ascribe_resolve",
        json!({ "page": "go:rotate-key", "build": "cloud", "project": "site" }),
    );
    assert!(
        message.starts_with(
            "build `cloud` publishes no permalink `rotate-key`; did you mean `rotate-keys`?"
        ),
        "{message}"
    );
    let message = server.call_error(
        "ascribe_resolve",
        json!({ "page": "site/docs/keys.md", "build": "cloud", "project": "site" }),
    );
    assert!(message.contains("`project` goes with"), "{message}");
}

/// `ascribe_resolve` with `emit` returns what `ascribe resolve --emit`
/// writes: `site` the site output's Markdown, with its elements, and
/// `plain` the default. With `go:<name>`, `emit` is the command's usage
/// error.
#[test]
fn resolve_takes_emit() {
    let repo = repository();
    let root = repo.path();
    let mut server = Server::start(root);
    let page = "site/docs/install.md";
    let (text, structured) = server.call_ok(
        "ascribe_resolve",
        json!({ "page": page, "build": "cloud", "emit": "site" }),
    );
    let expected = stdout(
        root,
        &[
            "resolve", page, "--build", "cloud", "--emit", "site", "--format", "json",
        ],
    );
    assert_eq!(text, expected);
    assert_eq!(
        structured,
        serde_json::from_str::<Value>(&expected).unwrap()
    );
    assert_eq!(structured["emit"], "site", "{structured}");
    let site = structured["text"].as_str().unwrap();
    assert_eq!(
        site,
        stdout(
            root,
            &["resolve", page, "--build", "cloud", "--emit", "site"]
        ),
    );
    assert!(site.contains("<ascribe-note"), "{site}");

    // Left out, and `plain`, are the plain form.
    let (_, default) = server.call_ok("ascribe_resolve", json!({ "page": page, "build": "cloud" }));
    let (_, plain) = server.call_ok(
        "ascribe_resolve",
        json!({ "page": page, "build": "cloud", "emit": "plain" }),
    );
    assert_eq!(default, plain);
    assert_eq!(default["emit"], "plain", "{default}");
    assert!(
        !default["text"].as_str().unwrap().contains("<ascribe-"),
        "{default}"
    );

    let message = server.call_error(
        "ascribe_resolve",
        json!({ "page": "go:rotate-keys", "build": "cloud", "project": "site", "emit": "plain" }),
    );
    assert!(
        message.starts_with("a permalink's help has one form, help.json's; leave out --emit"),
        "{message}"
    );
    let message = server.call_error(
        "ascribe_resolve",
        json!({ "page": page, "build": "cloud", "emit": "html" }),
    );
    assert!(message.contains("`plain` or `site`"), "{message}");
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
fn code_narrows_the_check_as_the_command_does() {
    let repo = repository();
    let root = repo.path();
    let mut server = Server::start(root);
    let (text, _) = server.call_ok(
        "ascribe_check",
        json!({ "paths": ["site"], "code": ["link-id-missing"] }),
    );
    let expected = stdout(
        root,
        &[
            "check",
            "site",
            "--code",
            "link-id-missing",
            "--format",
            "concise",
        ],
    );
    assert_eq!(text, expected);
    assert!(text.contains("[ASC037]"), "{text}");
    assert!(text.contains("for code `ASC037`"), "{text}");

    let message = server.call_error("ascribe_check", json!({ "code": ["link-targt"] }));
    assert!(message.contains("ASC036 link-target-missing"), "{message}");
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

/// Both revisions give the version `ascribe --version` prints, `+dev` and
/// commit included, as the server's.
#[test]
fn the_server_info_has_the_full_version() {
    let repo = repository();
    let printed = stdout(repo.path(), &["--version"]);
    let version = printed
        .trim()
        .strip_prefix("ascribe ")
        .expect("the name, then the version");
    assert!(version.starts_with(env!("CARGO_PKG_VERSION")), "{version}");

    let mut modern = Server::start(repo.path());
    let result = modern.result("tools/list", json!({}));
    let info = &result["_meta"]["io.modelcontextprotocol/serverInfo"];
    assert_eq!(info["name"], "ascribe", "{result}");
    assert_eq!(info["version"], version, "{result}");

    let mut legacy = Server::start(repo.path());
    legacy.modern = false;
    let result = legacy.result(
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "tests", "version": "0" },
        }),
    );
    assert_eq!(result["serverInfo"]["version"], version, "{result}");
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
            "ascribe_search",
            "ascribe_resolve",
            "ascribe_fmt",
            "ascribe_diff",
            "ascribe_rename",
            "ascribe_addresses_move",
            "ascribe_addresses_retire",
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

    let resources = server.result("resources/list", json!({}));
    let uris: Vec<&str> = resources["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    assert_eq!(
        uris,
        [
            "ascribe://directives",
            "ascribe://model/api",
            "ascribe://instructions/api",
            "ascribe://model/guides",
            "ascribe://instructions/guides",
        ]
    );
}

/// Rewrites `path` with `text`, the same length as what's there, and gives
/// it back its modification time: a change the server's listing can't
/// see, so the answers show whether it loaded the project again.
fn change_unseen(path: &Path, text: &str) {
    let modified = fs::metadata(path).unwrap().modified().unwrap();
    assert_eq!(fs::read(path).unwrap().len(), text.len());
    fs::write(path, text).unwrap();
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(modified)
        .unwrap();
}

#[test]
fn files_outside_the_content_root_dont_load_the_project_again() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_project(root);
    let mut server = Server::start(root);
    let heading = |server: &mut Server| {
        let (_, outline) = server.call_ok("ascribe_outline", json!({ "page": "docs/keys.md" }));
        outline["headings"][0]["id"].as_str().unwrap().to_owned()
    };
    assert_eq!(heading(&mut server), "rotate-keys");
    change_unseen(
        &root.join("docs/keys.md"),
        &KEYS.replace("Rotate keys", "Rotate keyz"),
    );

    // A build's output, and anything else beside ascribe.toml, isn't the
    // project's.
    write(&root.join("target/x.txt"), "built\n");
    write(&root.join("target/debug/y.txt"), "built\n");
    write(&root.join("notes.md"), "# Notes\n");
    assert_eq!(heading(&mut server), "rotate-keys");

    // A file in the content root is.
    write(
        &root.join("docs/tokens.md"),
        "---\ntype: page\ntitle: Tokens\n---\n",
    );
    assert_eq!(heading(&mut server), "rotate-keyz");
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
    write(
        &root.join("docs/tokens.md"),
        "---\ntype: page\ntitle: Tokens\n---\n",
    );
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

#[test]
fn a_second_search_doesnt_load_the_project_again() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_project(root);
    let mut server = Server::start(root);
    let search = |server: &mut Server, query: &str| {
        let (text, _) = server.call_ok("ascribe_search", json!({ "query": query }));
        text
    };
    assert!(search(&mut server, "rotate").contains("docs/keys.md:8: Rotate them."));
    change_unseen(
        &root.join("docs/keys.md"),
        &KEYS.replace("Rotate them.", "Rotate thez."),
    );
    // Another query, and the project as it was loaded.
    assert!(search(&mut server, "them").contains("docs/keys.md:8: Rotate them."));
    write(
        &root.join("docs/tokens.md"),
        "---\ntype: page\ntitle: Tokens\n---\n",
    );
    assert!(search(&mut server, "rotate").contains("docs/keys.md:8: Rotate thez."));
}

/// `ascribe_search` in the project in `site/`: the command's JSON, and its
/// text, the summary's too.
#[test]
fn search_returns_its_commands_json_and_text() {
    let repo = repository();
    let root = repo.path();
    let mut server = Server::start(root);
    let detailed: &[(Value, &[&str])] = &[
        (
            json!({ "query": "product", "project": "site", "response_format": "detailed" }),
            &["search", "product", "--config", "site", "--format", "json"],
        ),
        (
            json!({ "query": "rotate", "paths": ["site/docs/keys.md"], "summary": true, "response_format": "detailed" }),
            &[
                "search",
                "rotate",
                "site/docs/keys.md",
                "--summary",
                "--format",
                "json",
            ],
        ),
    ];
    for (arguments, command) in detailed {
        let (text, structured) = server.call_ok("ascribe_search", arguments.clone());
        let expected = stdout(root, command);
        assert_eq!(text, expected, "{arguments}");
        assert_eq!(
            structured,
            serde_json::from_str::<Value>(&expected).unwrap()
        );
    }
    let concise: &[(Value, &[&str])] = &[
        (
            json!({ "query": "\"you need\"", "project": "site", "build": ["cloud"] }),
            &[
                "search",
                "\"you need\"",
                "--config",
                "site",
                "--build",
                "cloud",
            ],
        ),
        (
            json!({ "query": "product", "project": "site", "summary": true }),
            &["search", "product", "--config", "site", "--summary"],
        ),
    ];
    for (arguments, command) in concise {
        let (text, structured) = server.call_ok("ascribe_search", arguments.clone());
        assert_eq!(text, stdout(root, command), "{arguments}");
        assert_eq!(structured, json!({ "concise": text }));
    }
}

/// The thread's query on `examples/quill`, through the tool: the command's
/// text, and with `detailed`, its JSON.
#[test]
fn search_answers_what_the_command_writes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut server = Server::start(&root);
    let query = "\"sign in to quill cloud\"";
    let command = ["search", query, "--config", "examples/quill"];
    let (text, structured) = server.call_ok(
        "ascribe_search",
        json!({ "query": query, "project": "examples/quill" }),
    );
    assert_eq!(text, stdout(&root, &command));
    assert!(text.starts_with("docs/install-agent.md:62: "), "{text}");
    assert_eq!(structured, json!({ "concise": text }));
    let (text, structured) = server.call_ok(
        "ascribe_search",
        json!({ "query": query, "project": "examples/quill", "response_format": "detailed" }),
    );
    let expected = stdout(&root, &[&command[..], &["--format", "json"]].concat());
    assert_eq!(text, expected);
    assert_eq!(
        structured,
        serde_json::from_str::<Value>(&expected).unwrap()
    );
    assert_eq!(structured["places"][0]["line"], 62);
    let message = server.call_error(
        "ascribe_search",
        json!({ "query": query, "project": "examples/quill", "paths": ["examples/quill/docs"] }),
    );
    assert!(message.contains("not both"), "{message}");
}

#[test]
fn resources_and_prompts_are_what_their_commands_print() {
    let repo = repository();
    let root = repo.path();
    let before = contents(root);
    let mut server = Server::start(root);

    assert_eq!(
        server.read_resource("ascribe://directives"),
        stdout(root, &["agents", "skill", "references/directives.md"])
    );
    assert_eq!(
        server.read_resource("ascribe://model/site"),
        stdout(root, &["model", "site"])
    );
    assert_eq!(
        server.read_resource("ascribe://instructions/site"),
        stdout(root, &["agents", "rules", "site"])
    );
    let reply = server.request("resources/read", json!({ "uri": "ascribe://nothing" }));
    assert_eq!(reply["error"]["code"], -32602, "{reply}");

    let prompts = server.result("prompts/list", json!({}));
    let names: Vec<&str> = prompts["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["new-page", "fix", "review"]);

    let calls: &[(&str, Value, &[&str])] = &[
        (
            "new-page",
            json!({ "type": "guide", "title": "Rotate your keys", "path": "site" }),
            &[
                "agents",
                "prompt",
                "new-page",
                "--arg",
                "type=guide",
                "--arg",
                "title=Rotate your keys",
                "--arg",
                "path=site",
            ],
        ),
        (
            "fix",
            json!({ "path": "site/docs/install.md" }),
            &[
                "agents",
                "prompt",
                "fix",
                "--arg",
                "path=site/docs/install.md",
            ],
        ),
        (
            "fix",
            json!({ "path": "site/docs/keys.md" }),
            &["agents", "prompt", "fix", "--arg", "path=site/docs/keys.md"],
        ),
        (
            "review",
            json!({ "path": "site", "base": "main" }),
            &[
                "agents",
                "prompt",
                "review",
                "--arg",
                "path=site",
                "--arg",
                "base=main",
            ],
        ),
    ];
    for (name, arguments, command) in calls {
        let result = server.result(
            "prompts/get",
            json!({ "name": name, "arguments": arguments }),
        );
        let text = result["messages"][0]["content"]["text"].as_str().unwrap();
        assert_eq!(text, stdout(root, command), "{name} {arguments}");
    }
    assert_eq!(contents(root), before);

    let reply = server.request(
        "prompts/get",
        json!({ "name": "new-page", "arguments": { "type": "guide" } }),
    );
    assert!(
        reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("needs `title`"),
        "{reply}"
    );
}

#[test]
fn the_new_page_prompt_says_where_the_page_goes_and_what_it_needs() {
    let repo = repository();
    let text = stdout(
        repo.path(),
        &[
            "agents",
            "prompt",
            "new-page",
            "--arg",
            "type=guide",
            "--arg",
            "title=Rotate your keys",
            "--arg",
            "path=site",
        ],
    );
    assert!(text.contains("Write a new page of type `guide`"), "{text}");
    assert!(
        text.contains("First, search for the subject in the pages: `ascribe search 'Rotate your keys' site/docs`"),
        "{text}"
    );
    assert!(
        text.contains("when it doesn't say, `site/docs/rotate-your-keys.md`"),
        "{text}"
    );
    assert!(
        text.contains("type: guide\ntitle: \"Rotate your keys\""),
        "{text}"
    );
    assert!(text.contains("\naudience:\n---\n"), "{text}");
}

#[test]
fn agents_prompt_lists_its_prompts_and_refuses_what_it_doesnt_know() {
    let repo = repository();
    let root = repo.path().join("site");
    let list = stdout(&root, &["agents", "prompt", "--list"]);
    for name in ["new-page", "fix", "review"] {
        assert!(list.contains(name), "{list}");
    }
    for args in [
        &["agents", "prompt", "nothing"][..],
        &["agents", "prompt", "new-page", "--arg", "title=X"],
        &[
            "agents",
            "prompt",
            "new-page",
            "--arg",
            "type=nope",
            "--arg",
            "title=X",
        ],
        &["agents", "prompt", "fix", "--arg", "depth=2"],
    ] {
        let out = ascribe(&root, args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
    }
}

/// The command a description names, as words: "The command: `...`."
fn named_command(description: &str) -> Vec<String> {
    let (_, rest) = description
        .rsplit_once("The command: `")
        .unwrap_or_else(|| panic!("names no command: {description}"));
    let command = rest.trim_end_matches("`.");
    command
        .split(' ')
        .filter(|word| !word.starts_with('<'))
        .map(str::to_owned)
        .collect()
}

#[test]
fn every_tool_resource_and_prompt_names_a_command_that_exists() {
    let repo = repository();
    let root = repo.path();
    let mut server = Server::start(root);
    let mut descriptions = Vec::new();
    let mut collect = |result: Value, list: &str| {
        for item in result[list].as_array().unwrap() {
            descriptions.push(item["description"].as_str().unwrap().to_owned());
        }
    };
    collect(server.result("tools/list", json!({})), "tools");
    collect(server.result("resources/list", json!({})), "resources");
    collect(
        server.result("resources/templates/list", json!({})),
        "resourceTemplates",
    );
    collect(server.result("prompts/list", json!({})), "prompts");
    assert_eq!(descriptions.len(), 13 + 2 + 3 + 2 + 3);
    for description in descriptions {
        let words = named_command(&description);
        assert_eq!(words[0], "ascribe", "{description}");
        let mut args: Vec<&str> = words[1..].iter().map(String::as_str).collect();
        args.push("--help");
        // A subcommand that doesn't exist is an error, even with `--help`.
        let out = ascribe(root, &args);
        assert_eq!(out.status.code(), Some(0), "{description}: {args:?}");
    }
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

/// A call of each tool, in a project at the server's working directory.
fn call_in_project(tool: &str) -> Value {
    match tool {
        "ascribe_check" => json!({}),
        "ascribe_explain" => json!({ "code": "ASC037" }),
        "ascribe_model" => json!({ "section": "phrases" }),
        "ascribe_outline" => json!({ "page": "docs/install.md", "build": "cloud" }),
        "ascribe_link" => json!({ "target": "keys.md", "from": "docs/install.md" }),
        "ascribe_refs" => json!({ "target": "phrase:product" }),
        "ascribe_search" => json!({ "query": "product" }),
        "ascribe_resolve" => json!({ "page": "docs/install.md", "build": "cloud" }),
        "ascribe_fmt" | "ascribe_format" => json!({ "paths": ["docs"] }),
        "ascribe_diff" | "ascribe_changes" => json!({ "base": "HEAD" }),
        "ascribe_rename" => json!({ "target": "keys.md", "new": "api-keys.md", "project": "." }),
        "ascribe_addresses_move" => json!({ "old": "keys.md", "new": "api-keys.md" }),
        "ascribe_addresses_retire" => json!({ "old": "keys.md", "reason": "Removed." }),
        other => panic!("{other} is a new tool: give it a call here"),
    }
}

/// Every tool `tools/list` gives says it only reads, so none starts a
/// program the project names: not Vale, with `[checks.vale] in-check`,
/// whether `command` is a path in the project or a name on the `PATH`, and
/// not `git` taking a committed folder shaped like a repository for the
/// repository. The `fix` prompt runs the same check. Vale's stand-ins are
/// shell scripts, so they're left out on Windows; the `git` half isn't.
#[test]
fn no_tool_starts_a_program_the_project_names() {
    let marks = tempfile::tempdir().unwrap();
    let vale = |name: &str| -> String {
        let mark = marks.path().join(name).display().to_string();
        format!("#!/bin/sh\ntouch '{mark}'\n")
    };
    let model = |command: &str| {
        format!(
            "{MODEL}\n[checks.vale]\npreset = \"quiet\"\nin-check = true\ncommand = \"{command}\"\n"
        )
    };
    let planted = crate::planted::Planted::new(&[
        ("ascribe.toml", &model("bin/vale")),
        ("docs/install.md", INSTALL),
        ("docs/_fragments/before.md", BEFORE),
        ("docs/keys.md", KEYS),
        ("bin/vale", &vale("project")),
        ("path/vale", &vale("path")),
    ]);
    let project = planted.project();
    #[cfg(unix)]
    for script in ["bin/vale", "path/vale"] {
        use std::os::unix::fs::PermissionsExt;
        let file = project.join(script);
        fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut search = vec![project.join("path")];
    search.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(search).unwrap();

    for command in ["bin/vale", "vale"] {
        write(&project.join("ascribe.toml"), &model(command));
        let mut server = Server::start_with(&project, &[("PATH", path.clone())]);
        let tools = server.result("tools/list", json!({}));
        for tool in tools["tools"].as_array().unwrap() {
            assert_eq!(tool["annotations"]["readOnlyHint"], true, "{tool}");
            let name = tool["name"].as_str().unwrap();
            let result = server.call(name, call_in_project(name));
            if name == "ascribe_check" {
                assert!(
                    result["content"][0]["text"]
                        .as_str()
                        .unwrap()
                        .contains("don't start Vale"),
                    "{command}: {result}"
                );
            }
        }
        server.result("prompts/get", json!({ "name": "fix", "arguments": {} }));
        for mark in ["project", "path"] {
            assert!(
                !marks.path().join(mark).exists(),
                "a tool ran the {mark} Vale, with `command = \"{command}\"`"
            );
        }
        assert!(!planted.ran(), "a tool ran the planted folder's command");
        // Nothing written for Vale either: no tool writes a file.
        assert!(!project.join(".ascribe").exists(), "{command}");
    }
}

/// A project with no repository around it, three folders below the code its
/// source reads: `path = "../../.."`, and a page with a snippet from there.
fn project_below_its_source() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("secret.rs"),
        "// :snippet-start: key\nconst KEY: &str = \"hunter2\";\n// :snippet-end:\n",
    );
    write(
        &dir.path().join("a/b/project/ascribe.toml"),
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[sources.code]\npath = \"../../..\"\ninclude = [\"**/*.rs\"]\n",
    );
    write(
        &dir.path().join("a/b/project/docs/index.md"),
        "---\ntype: page\ntitle: Home\n---\n\n@snippet: code:secret.rs#key\n",
    );
    dir
}

/// With no repository, `ascribe mcp` reads a source only inside the folder
/// it was started in (decision 77); the commands read it wherever the model
/// says.
#[test]
fn with_no_repository_a_source_outside_the_servers_folder_isnt_read() {
    let dir = project_below_its_source();
    let project = dir.path().join("a/b/project");

    let mut server = Server::start(&project);
    let error = server.call_error(
        "ascribe_resolve",
        json!({ "page": "docs/index.md", "build": "site" }),
    );
    assert!(error.contains("ascribe.toml has errors"), "{error}");
    assert!(!error.contains("hunter2"), "{error}");

    // Started in the folder that holds the source, it's read.
    let mut server = Server::start(dir.path());
    let (text, _) = server.call_ok(
        "ascribe_resolve",
        json!({ "page": "a/b/project/docs/index.md", "build": "site" }),
    );
    assert!(text.contains("hunter2"), "{text}");

    // And a command reads it, as before.
    let out = ascribe(&project, &["resolve", "docs/index.md", "--build", "site"]);
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("hunter2"));
}

#[test]
fn check_holds_back_what_the_baseline_lists_and_counts_it() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        &root.join("ascribe.toml"),
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n",
    );
    write(
        &root.join("docs/index.md"),
        "---\ntitle: Home\n---\n\nSee [sync](sync.md).\n",
    );
    write(
        &root.join("docs/sync.md"),
        "---\ntitle: Sync\nnav-label: Synchronize two copies\n---\n\nText.\n",
    );
    assert!(ascribe(root, &["baseline", "write"]).status.success());
    let mut server = Server::start(root);

    // The project's backlog isn't the agent's: it's left out, and counted,
    // as `ascribe check` does.
    let (text, _) = server.call_ok("ascribe_check", json!({}));
    assert_eq!(text, stdout(root, &["check", "--format", "concise"]));
    assert!(!text.contains("nav-label"), "{text}");
    assert!(text.contains(", 1 in the baseline"), "{text}");
    let (_, structured) = server.call_ok("ascribe_check", json!({ "response_format": "detailed" }));
    assert_eq!(structured["summary"]["baselined"], 1, "{structured}");
    assert_eq!(structured["summary"]["advice"], 0, "{structured}");
}

/// A project written for a newer Ascribe has keys this one doesn't know: a
/// tool's reply quotes the first, whose message names this Ascribe's version
/// (#849), and so does the command's.
#[test]
fn an_unknown_key_says_which_ascribe_read_it() {
    let dir = tempfile::tempdir().unwrap();
    write_project(dir.path());
    write(
        &dir.path().join("ascribe.toml"),
        &MODEL.replacen(
            "spec = \"0.1\"\n",
            "spec = \"0.1\"\nlayouts = \"wide\"\n",
            1,
        ),
    );
    let note = format!(
        "unknown key `layouts` in `[ascribe.toml]`; this is ascribe {}, and the project may need a newer one",
        env!("CARGO_PKG_VERSION")
    );

    let mut server = Server::start(dir.path());
    let error = server.call_error("ascribe_outline", json!({ "page": "docs/install.md" }));
    assert!(error.contains("ascribe.toml has errors"), "{error}");
    assert!(error.contains(&note), "{error}");

    let out = ascribe(dir.path(), &["outline", "docs/install.md"]);
    assert!(!out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(&note), "{stderr}");
    let out = ascribe(dir.path(), &["check"]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(text.contains(&note), "{text}");
}

/// The byte offset of `pos`, a report's line and column, from 1, in
/// characters, in `text`.
fn offset(text: &str, pos: &Value) -> usize {
    let (line, column) = (
        pos["line"].as_u64().unwrap() as usize,
        pos["column"].as_u64().unwrap() as usize,
    );
    let start: usize = text
        .split_inclusive('\n')
        .take(line - 1)
        .map(str::len)
        .sum();
    start
        + text[start..]
            .chars()
            .take(column - 1)
            .map(char::len_utf8)
            .sum::<usize>()
}

/// `text` with `edits`, a report's, applied: each replaces its range of the
/// text as it was.
fn applied(text: &str, edits: &Value) -> String {
    let mut edits: Vec<(usize, usize, &str)> = edits
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                offset(text, &e["range"]["start"]),
                offset(text, &e["range"]["end"]),
                e["new_text"].as_str().unwrap(),
            )
        })
        .collect();
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut out = text.to_owned();
    for (start, end, new_text) in edits {
        out.replace_range(start..end, new_text);
    }
    out
}

/// An agent's session: it moves a page with `git mv`, the check finds the
/// page's addresses lost, `ascribe_addresses_move` with `from_git` gives the
/// edit that records the move, and once the agent applies it the check
/// passes. The tool writes nothing itself.
#[test]
fn an_agent_records_a_move_git_sees_with_the_tool_s_edit() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_project(root);
    git(root, &["init", "-q"]);
    let out = ascribe(root, &["addresses", "init"]);
    assert!(out.status.success(), "{out:?}");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "Start"]);
    git(root, &["mv", "docs/keys.md", "docs/api-keys.md"]);
    write(
        &root.join("docs/install.md"),
        &INSTALL.replace("(keys.md#rotat)", "(api-keys.md#rotate-keys)"),
    );

    let mut server = Server::start(root);
    let (_, report) = server.call_ok("ascribe_check", json!({ "response_format": "detailed" }));
    let codes: Vec<&str> = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["slug"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"address-lost"), "{report}");

    let file = root.join("ascribe.addresses");
    let before = fs::read_to_string(&file).unwrap();
    let (_, moved) = server.call_ok("ascribe_addresses_move", json!({ "from_git": true }));
    assert_eq!(
        moved["moves"],
        json!([{ "key": "keys.md", "value": "api-keys.md" }])
    );
    assert_eq!(moved["changed"], true, "{moved}");
    assert_eq!(moved["written"], false, "{moved}");
    assert_eq!(fs::read_to_string(&file).unwrap(), before, "the tool wrote");
    let after = applied(&before, &moved["edits"]);
    assert!(after.contains("\"keys.md\" = \"api-keys.md\""), "{after}");
    fs::write(&file, after).unwrap();

    let (_, report) = server.call_ok("ascribe_check", json!({ "response_format": "detailed" }));
    assert!(
        report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["severity"] != "error"),
        "{report}"
    );
}

/// An agent with no shell moves a page with `ascribe_rename`: the tool
/// writes nothing, and gives what `ascribe rename --format json` writes in a
/// copy of the project, but `written`. Once the agent writes each file's
/// text and moves the page, the project is what the command leaves, and the
/// check passes.
#[test]
fn an_agent_moves_a_page_with_the_rename_tool() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("agent");
    write_project(&root);
    let out = ascribe(&root, &["addresses", "init"]);
    assert!(out.status.success(), "{out:?}");
    let copy = dir.path().join("command");
    fs::create_dir_all(&copy).unwrap();
    copy_tree(&root, &copy);
    let before = contents(&root);

    let mut server = Server::start(&root);
    let arguments = json!({ "target": "docs/keys.md", "new": "docs/security/keys.md" });
    let (_, renamed) = server.call_ok("ascribe_rename", arguments);
    assert_eq!(contents(&root), before, "the tool wrote");
    assert_eq!(renamed["written"], false, "{renamed}");
    assert_eq!(
        renamed["moved"],
        json!({ "from": "docs/keys.md", "to": "docs/security/keys.md" })
    );
    assert_eq!(
        renamed["recorded"],
        json!([{ "key": "keys.md", "value": "security/keys.md" }])
    );

    let out = ascribe(
        &copy,
        &[
            "rename",
            "docs/keys.md",
            "docs/security/keys.md",
            "--format",
            "json",
        ],
    );
    assert!(out.status.success(), "{out:?}");
    let mut command: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(command["written"], true);
    command["written"] = json!(false);
    assert_eq!(renamed, command);

    for file in renamed["files"].as_array().unwrap() {
        fs::write(
            root.join(file["file"].as_str().unwrap()),
            file["text"].as_str().unwrap(),
        )
        .unwrap();
    }
    fs::create_dir_all(root.join("docs/security")).unwrap();
    fs::rename(
        root.join("docs/keys.md"),
        root.join("docs/security/keys.md"),
    )
    .unwrap();
    let relative = |files: BTreeMap<PathBuf, Vec<u8>>, from: &Path| -> BTreeMap<PathBuf, Vec<u8>> {
        files
            .into_iter()
            .map(|(path, text)| (path.strip_prefix(from).unwrap().to_owned(), text))
            .collect()
    };
    assert_eq!(
        relative(contents(&root), &root),
        relative(contents(&copy), &copy)
    );
    let link = fs::read_to_string(root.join("docs/install.md")).unwrap();
    assert!(link.contains("(security/keys.md#rotat)"), "{link}");

    let (_, report) = server.call_ok("ascribe_check", json!({ "response_format": "detailed" }));
    let codes: Vec<&str> = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["slug"].as_str().unwrap())
        .collect();
    assert!(!codes.contains(&"address-lost"), "{report}");
}

/// `ascribe_rename` renames a phrase in the project `project` names, from a
/// server started above it, as `ascribe rename --config` does; without
/// `project`, there's no project at or above the server's folder.
#[test]
fn rename_finds_the_project_an_entry_is_in_from_project() {
    let repo = repository();
    let root = repo.path();
    let before = contents(root);
    let mut server = Server::start(root);
    let message = server.call_error(
        "ascribe_rename",
        json!({ "target": "phrase:product", "new": "brand" }),
    );
    assert!(message.contains("ascribe.toml"), "{message}");

    let (_, renamed) = server.call_ok(
        "ascribe_rename",
        json!({ "target": "phrase:product", "new": "brand", "project": "site" }),
    );
    assert_eq!(contents(root), before, "the tool wrote");
    let files: Vec<&str> = renamed["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["file"].as_str().unwrap())
        .collect();
    assert!(files.contains(&"site/ascribe.toml"), "{renamed}");
    assert!(files.contains(&"site/docs/install.md"), "{renamed}");

    let out = ascribe(
        root,
        &[
            "rename",
            "phrase:product",
            "brand",
            "--config",
            "site",
            "--format",
            "json",
        ],
    );
    assert!(out.status.success(), "{out:?}");
    let mut command: Value = serde_json::from_slice(&out.stdout).unwrap();
    command["written"] = json!(false);
    assert_eq!(renamed, command);
}

/// A tool's old name calls the tool, and `tools/list` gives it as the tool
/// with a sentence saying so first.
#[test]
fn an_old_name_calls_its_tool() {
    let repo = repository();
    let mut server = Server::start(repo.path());
    let arguments = json!({ "project": "site", "base": "HEAD" });
    assert_eq!(
        server.call_ok("ascribe_changes", arguments.clone()),
        server.call_ok("ascribe_diff", arguments)
    );
    let tools = server.result("tools/list", json!({}));
    let tool = |name: &str| {
        tools["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == name)
            .unwrap()
            .clone()
    };
    let (old, new) = (tool("ascribe_format"), tool("ascribe_fmt"));
    assert_eq!(old["inputSchema"], new["inputSchema"]);
    let description = old["description"].as_str().unwrap();
    assert!(
        description.starts_with("The old name of `ascribe_fmt`")
            && description.ends_with(new["description"].as_str().unwrap()),
        "{description}"
    );
}

/// Copies the files under `from` into `to`.
fn copy_tree(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            fs::create_dir_all(&target).unwrap();
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn link_answers_with_the_operation_a_link_to_a_spec_names() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/astro-site");
    fs::copy(example.join("ascribe.toml"), root.join("ascribe.toml")).unwrap();
    for folder in ["api", "content"] {
        fs::create_dir_all(root.join(folder)).unwrap();
        copy_tree(&example.join(folder), &root.join(folder));
    }
    let guide = "content/Guides/Teams/Share a loom.md";
    let mut server = Server::start(root);
    let target = "../../../api/openapi.yaml#delete/looms/{id}";
    let (_, answer) = server.call_ok("ascribe_link", json!({ "target": target, "from": guide }));
    assert_eq!(answer["kind"], "operation");
    assert_eq!(
        answer["operation"],
        json!({
            "api": "loom",
            "operation_id": null,
            "method": "delete",
            "path": "/looms/{id}",
            "summary": "Delete a loom",
            "tag": "Looms",
            "deprecated": false,
            "address": "/docs/api#paths/~1looms~1%7Bid%7D/delete",
        })
    );
    let command: Value = serde_json::from_str(&stdout(
        root,
        &["link", target, "--from", guide, "--format", "json"],
    ))
    .unwrap();
    assert_eq!(answer, command);
}
