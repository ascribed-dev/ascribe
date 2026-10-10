//! Diagnostic parity: the diagnostics `ascribe lsp` publishes for a build are
//! the diagnostics `ascribe check --build <name> --format json` reports for it:
//! same codes, files, spans, severities, messages, and related places.
//!
//! The server runs as a real process over stdio, as an editor starts it, so this
//! also checks that nothing but LSP messages reaches standard output. The
//! spans are compared as byte offsets: the JSON carries them, and the test
//! converts the server's positions back with `ascribe_core::LineIndex` in the
//! encoding it negotiated.
//!
//! The Windows and macOS runs of this test wait for a manual CI run (CI is
//! manual-only); the test uses no platform-specific behavior.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ascribe_core::{LineIndex, WideEncoding, WideLineCol};
use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(30);

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A diagnostic in the form both sides are reduced to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Entry {
    file: String,
    code: String,
    slug: String,
    severity: String,
    start: usize,
    end: usize,
    message: String,
    builds: Vec<String>,
    unpublished: bool,
    related: Vec<(String, usize, usize, String)>,
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("mkdir");
    for entry in fs::read_dir(from).expect("read_dir") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy");
        }
    }
}

/// `ascribe.toml` with `[editor] build` set to `build`.
fn set_editor_build(root: &Path, build: &str) {
    let path = root.join("ascribe.toml");
    let text = fs::read_to_string(&path).expect("model");
    let mut out = String::new();
    let mut skipping = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[editor]" {
            skipping = true;
            continue;
        }
        if skipping && trimmed.starts_with('[') {
            skipping = false;
        }
        if !skipping {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str(&format!("\n[editor]\nbuild = \"{build}\"\n"));
    fs::write(path, out).expect("write model");
}

fn builds_of(root: &Path) -> Vec<String> {
    ascribe_model::load(root.join("ascribe.toml"))
        .expect("the model loads")
        .builds
        .iter()
        .map(|b| b.name.clone())
        .collect()
}

/// What `ascribe check --build <name> --format json` reports.
fn cli_entries(root: &Path, build: &str) -> Vec<Entry> {
    let output = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(root)
        .args(["check", "--build", build, "--format", "json"])
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe check");
    let report: Value = serde_json::from_slice(&output.stdout).expect("JSON on stdout");
    let offset = |range: &Value, end: &str| range[end]["offset"].as_u64().expect("offset") as usize;
    let mut entries: Vec<Entry> = report["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .map(|d| Entry {
            file: d["file"].as_str().expect("file").to_owned(),
            code: d["code"].as_str().expect("code").to_owned(),
            slug: d["slug"].as_str().expect("slug").to_owned(),
            severity: d["severity"].as_str().expect("severity").to_owned(),
            start: offset(&d["range"], "start"),
            end: offset(&d["range"], "end"),
            message: d["message"].as_str().expect("message").to_owned(),
            builds: d["builds"]
                .as_array()
                .expect("builds")
                .iter()
                .map(|b| b.as_str().expect("build").to_owned())
                .collect(),
            unpublished: d["unpublished"].as_bool().expect("unpublished"),
            related: d["related"]
                .as_array()
                .expect("related")
                .iter()
                .map(|r| {
                    (
                        r["file"].as_str().expect("file").to_owned(),
                        offset(&r["range"], "start"),
                        offset(&r["range"], "end"),
                        r["message"].as_str().expect("message").to_owned(),
                    )
                })
                .collect(),
        })
        .collect();
    entries.sort();
    entries
}

// -- A stdio client -----------------------------------------------------------

struct Server {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    messages: mpsc::Receiver<Value>,
    /// `uri -> diagnostics`, the last publication for each.
    published: BTreeMap<String, Vec<Value>>,
    encoding: String,
}

fn read_frame(reader: &mut impl BufRead) -> Option<Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        // Anything on stdout that isn't a header is a bug in the server.
        let value = line
            .strip_prefix("Content-Length: ")
            .unwrap_or_else(|| panic!("stdout carried something that isn't LSP: {line:?}"));
        length = Some(value.parse::<usize>().expect("a length"));
    }
    let mut body = vec![0; length.expect("a Content-Length header")];
    reader.read_exact(&mut body).ok()?;
    Some(serde_json::from_slice(&body).expect("JSON body"))
}

impl Server {
    fn start(root: &Path, offer_utf8: bool) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ascribe"))
            .arg("lsp")
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("start ascribe lsp");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let (tx, messages) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Some(value) = read_frame(&mut reader) {
                if tx.send(value).is_err() {
                    break;
                }
            }
        });
        let mut server = Server {
            child,
            stdin,
            messages,
            published: BTreeMap::new(),
            encoding: String::new(),
        };
        let mut general = json!({});
        if offer_utf8 {
            general["positionEncodings"] = json!(["utf-16", "utf-8"]);
        }
        let root_uri = file_uri(root);
        server.send(&json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "processId": null,
                "workspaceFolders": [{ "uri": root_uri, "name": "w" }],
                "capabilities": {
                    "general": general,
                    "workspace": { "didChangeWatchedFiles": { "dynamicRegistration": true } },
                },
            },
        }));
        let response = server.wait_response(1);
        server.encoding = response["result"]["capabilities"]["positionEncoding"]
            .as_str()
            .expect("an encoding")
            .to_owned();
        server.send(&json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
        server
    }

    fn send(&mut self, message: &Value) {
        let body = serde_json::to_vec(message).expect("json");
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len()).expect("write");
        self.stdin.write_all(&body).expect("write");
        self.stdin.flush().expect("flush");
    }

    fn handle(&mut self, message: Value) -> Option<Value> {
        if let Some(method) = message["method"].as_str() {
            if let Some(id) = message.get("id") {
                // A request from the server (registerCapability): accept it.
                let reply = json!({ "jsonrpc": "2.0", "id": id, "result": null });
                self.send(&reply);
            } else if method == "textDocument/publishDiagnostics" {
                let params = &message["params"];
                self.published.insert(
                    params["uri"].as_str().expect("uri").to_owned(),
                    params["diagnostics"].as_array().expect("list").clone(),
                );
            }
            return None;
        }
        Some(message)
    }

    fn wait_response(&mut self, id: u64) -> Value {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let message = self.messages.recv_timeout(left).expect("a response");
            if let Some(response) = self.handle(message)
                && response["id"] == id
            {
                return response;
            }
        }
    }

    /// Reads messages until `done` says the published state is what is wanted,
    /// or the time is up. Returns whether it got there.
    fn pump_until(&mut self, timeout: Duration, mut done: impl FnMut(&Server) -> bool) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if done(self) {
                return true;
            }
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            if let Ok(message) = self
                .messages
                .recv_timeout(left.min(Duration::from_millis(50)))
            {
                self.handle(message);
            }
        }
    }

    fn entries(&self, root: &Path) -> Vec<Entry> {
        let mut out = Vec::new();
        for (uri, diagnostics) in &self.published {
            let file = rel(root, uri);
            for d in diagnostics {
                out.push(self.entry(root, &file, d));
            }
        }
        out.sort();
        out
    }

    fn offsets(&self, root: &Path, file: &str, range: &Value) -> (usize, usize) {
        let text = fs::read(root.join(file)).map(|b| String::from_utf8(b).unwrap_or_default());
        let text = text.unwrap_or_default();
        let index = LineIndex::new(&text);
        let at = |p: &Value| {
            let pos = WideLineCol {
                line: p["line"].as_u64().expect("line") as u32,
                col: p["character"].as_u64().expect("character") as u32,
            };
            match self.encoding.as_str() {
                "utf-8" => index
                    .offset(ascribe_core::LineCol {
                        line: pos.line,
                        col: pos.col,
                    })
                    .expect("a byte position"),
                _ => index
                    .wide_offset(WideEncoding::Utf16, pos)
                    .expect("a UTF-16 position"),
            }
        };
        (at(&range["start"]), at(&range["end"]))
    }

    fn entry(&self, root: &Path, file: &str, d: &Value) -> Entry {
        let (start, end) = self.offsets(root, file, &d["range"]);
        Entry {
            file: file.to_owned(),
            code: d["code"].as_str().expect("code").to_owned(),
            slug: d["data"]["slug"].as_str().expect("slug").to_owned(),
            severity: match d["severity"].as_u64().expect("severity") {
                1 => "error",
                2 => "warning",
                3 => "advice",
                other => panic!("severity {other}"),
            }
            .to_owned(),
            start,
            end,
            message: d["message"].as_str().expect("message").to_owned(),
            builds: d["data"]["builds"]
                .as_array()
                .expect("builds")
                .iter()
                .map(|b| b.as_str().expect("build").to_owned())
                .collect(),
            unpublished: d["data"]["unpublished"].as_bool().expect("unpublished"),
            related: d["relatedInformation"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .map(|r| {
                            let related_file =
                                rel(root, r["location"]["uri"].as_str().expect("uri"));
                            let (s, e) = self.offsets(root, &related_file, &r["location"]["range"]);
                            (
                                related_file,
                                s,
                                e,
                                r["message"].as_str().expect("message").to_owned(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    fn shutdown(mut self) {
        self.send(&json!({ "jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": null }));
        let _ = self.wait_response(2);
        self.send(&json!({ "jsonrpc": "2.0", "method": "exit" }));
        let status = self.child.wait().expect("the server exits");
        assert!(
            status.success(),
            "the server exits with 0 after shutdown: {status:?}"
        );
    }
}

/// The `file:` URI of an absolute path: `file:///C:/dir` for a drive path.
fn file_uri(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let text = text.strip_prefix("//?/").unwrap_or(&text);
    if text.starts_with('/') {
        format!("file://{text}")
    } else {
        format!("file:///{text}")
    }
}

/// A `file:` URI's path relative to the project root.
fn rel(root: &Path, uri: &str) -> String {
    let path = uri.strip_prefix("file://").expect("a file URI");
    let mut path = percent_decode(path);
    // `/c:/dir` names a drive; the project root's drive letter may differ in case.
    let bytes = path.as_bytes();
    if bytes.len() > 2 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic() && bytes[2] == b':' {
        path.remove(0);
    }
    // `canonicalize` gives a Windows path the `\\?\` prefix; URIs don't carry it.
    let root_text = root.to_string_lossy().replace('\\', "/");
    let root_text = root_text
        .strip_prefix("//?/")
        .unwrap_or(&root_text)
        .to_string();
    if path.as_bytes().get(1) == Some(&b':') && root_text.as_bytes().get(1) == Some(&b':') {
        path.replace_range(0..1, &root_text[0..1]);
    }
    let root = Path::new(&root_text);
    Path::new(&path)
        .strip_prefix(root)
        .unwrap_or_else(|_| panic!("{path} is outside {}", root.display()))
        .to_string_lossy()
        .replace('\\', "/")
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            out.push(u8::from_str_radix(&text[i + 1..i + 3], 16).expect("hex"));
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).expect("UTF-8")
}

// -- The test -------------------------------------------------------------------

/// Runs the parity check for one build of one project.
fn check_parity(source: &Path, build: &str, offer_utf8: bool) {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().canonicalize().expect("real path");
    copy_dir(source, &root);
    set_editor_build(&root, build);
    let expected = cli_entries(&root, build);

    let mut server = Server::start(&root, offer_utf8);
    assert_eq!(server.encoding, if offer_utf8 { "utf-8" } else { "utf-16" });
    let reached = server.pump_until(TIMEOUT, |s| s.entries(&root) == expected);
    let got = server.entries(&root);
    assert!(
        reached,
        "build `{build}` of {}: the server's diagnostics differ from `ascribe check`.\nonly in the CLI:\n{:#?}\nonly in the server:\n{:#?}",
        source.display(),
        expected
            .iter()
            .filter(|e| !got.contains(e))
            .collect::<Vec<_>>(),
        got.iter()
            .filter(|e| !expected.contains(e))
            .collect::<Vec<_>>(),
    );
    // And it stays that way: nothing more arrives that changes it.
    let changed = server.pump_until(Duration::from_millis(300), |s| s.entries(&root) != expected);
    assert!(
        !changed,
        "build `{build}`: the published diagnostics changed after they matched"
    );
    server.shutdown();
}

fn parity_over(source: &Path, min_slugs: usize) {
    let builds = builds_of(source);
    assert!(builds.len() >= 2, "a project with several builds");
    for build in &builds {
        check_parity(source, build, false);
    }
    // With UTF-8 columns too, on the first build.
    check_parity(source, &builds[0], true);
    let expected = cli_entries(source, &builds[0]);
    let slugs: std::collections::BTreeSet<&str> =
        expected.iter().map(|e| e.slug.as_str()).collect();
    assert!(
        slugs.len() >= min_slugs,
        "the fixture exercises enough rows: {slugs:?}"
    );
}

#[test]
fn quill_parity_for_every_build() {
    let quill = manifest_dir().join("../../examples/quill");
    let builds = builds_of(&quill);
    assert_eq!(builds, ["site", "cloud", "self-managed-3.3"]);
    for build in &builds {
        check_parity(&quill, build, false);
    }
}

#[test]
fn problem_fixture_parity_for_every_build() {
    let fixture = manifest_dir().join("tests/fixtures/lsp/problems");
    parity_over(&fixture, 25);
}

#[test]
fn the_fixture_differs_between_builds() {
    // Parity per build is only meaningful if the builds differ: `cloud` finds a
    // problem `site` doesn't.
    let fixture = manifest_dir().join("tests/fixtures/lsp/problems");
    let site = cli_entries(&fixture, "site");
    let cloud = cli_entries(&fixture, "cloud");
    assert_ne!(site, cloud);
}

/// `ascribe check --editor-build --format prompt` with `args`, in `root`.
fn cli_prompt(root: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(root)
        .arg("check")
        .args(args)
        .args(["--editor-build", "--format", "prompt"])
        .output()
        .expect("run ascribe check");
    String::from_utf8(out.stdout).expect("UTF-8")
}

#[test]
fn agent_prompts_are_the_command_lines_for_the_editors_build() {
    let fixture = manifest_dir().join("tests/fixtures/lsp/problems");
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().canonicalize().expect("real path");
    copy_dir(&fixture, &root);
    let build = builds_of(&root).remove(0);
    set_editor_build(&root, &build);
    let expected = cli_entries(&root, &build);

    let mut server = Server::start(&root, false);
    assert!(server.pump_until(TIMEOUT, |s| s.entries(&root) == expected));
    let files: Vec<String> = server
        .published
        .iter()
        .filter(|(_, list)| !list.is_empty())
        .map(|(uri, _)| uri.clone())
        .collect();
    assert!(files.len() >= 3, "files with problems: {files:?}");
    let mut id = 100;
    let mut ask = |server: &mut Server, params: Value| {
        id += 1;
        server.send(&json!({
            "jsonrpc": "2.0", "id": id, "method": "ascribe/agentPrompt", "params": params,
        }));
        server.wait_response(id)["result"].clone()
    };
    for uri in &files {
        let file = rel(&root, uri);
        let answer = ask(
            &mut server,
            json!({ "kind": "file", "textDocument": { "uri": uri } }),
        );
        let prompt = answer["prompt"].as_str().unwrap_or_default();
        assert_eq!(
            prompt,
            cli_prompt(&root, &[&file]),
            "the prompt about {file}"
        );
    }
    let answer = ask(&mut server, json!({ "kind": "project" }));
    assert_eq!(
        answer["prompt"].as_str().unwrap_or_default(),
        cli_prompt(&root, &[]),
        "the project's prompt"
    );
    server.shutdown();
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
        .expect("run git");
    assert!(out.status.success(), "git {args:?} failed");
}

/// `ascribe diff --format prompt` about `page`, for `build`, in `root`.
fn cli_review_prompt(root: &Path, page: &str, build: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(root)
        .args(["diff", "--format", "prompt", page, "--build", build])
        .output()
        .expect("run ascribe diff");
    String::from_utf8(out.stdout).expect("UTF-8")
}

#[test]
fn review_prompts_are_the_command_lines() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().canonicalize().expect("real path");
    let write = |rel: &str, text: &str| {
        let path = rel.split('/').fold(root.clone(), |p, s| p.join(s));
        fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    };
    write(
        "ascribe.toml",
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[builds.site]\n\n[builds.cloud]\n\n[editor]\nbuild = \"cloud\"\n",
    );
    write(
        "docs/install.md",
        "---\ntitle: Install\n---\n\nRun the installer.\n\n@include: _fragments/check.md\n\nThe end.\n",
    );
    write(
        "docs/upgrade.md",
        "---\ntitle: Upgrade\n---\n\n@include: _fragments/check.md\n",
    );
    write("docs/_fragments/check.md", "Check the version.\n");
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "First"]);
    write(
        "docs/install.md",
        "---\ntitle: Install\n---\n\nRun the new installer.\n\n@include: _fragments/check.md\n\nThe end.\n\nOne more thing.\n",
    );
    write("docs/_fragments/check.md", "Check the version first.\n");

    let mut server = Server::start(&root, false);
    server.send(&json!({
        "jsonrpc": "2.0", "id": 2, "method": "ascribe/review/setBase", "params": {},
    }));
    assert_eq!(server.wait_response(2)["result"]["problem"], Value::Null);
    let mut id = 100;
    let mut ask = |server: &mut Server, params: Value| {
        id += 1;
        server.send(&json!({
            "jsonrpc": "2.0", "id": id, "method": "ascribe/agentPrompt", "params": params,
        }));
        server.wait_response(id)["result"].clone()
    };
    for page in ["install.md", "upgrade.md"] {
        let uri = file_uri(&root.join("docs").join(page));
        // Without a build, the editor's.
        let answer = ask(
            &mut server,
            json!({ "kind": "pageChanges", "textDocument": { "uri": uri } }),
        );
        let expected = cli_review_prompt(&root, &format!("docs/{page}"), "cloud");
        assert!(
            expected.starts_with("Review what this change does to"),
            "{expected}"
        );
        assert_eq!(
            answer["prompt"].as_str().unwrap_or_default(),
            expected,
            "{page}"
        );
        let answer = ask(
            &mut server,
            json!({ "kind": "pageChanges", "textDocument": { "uri": uri }, "build": "site" }),
        );
        assert_eq!(
            answer["prompt"].as_str().unwrap_or_default(),
            cli_review_prompt(&root, &format!("docs/{page}"), "site"),
            "{page} in site"
        );
    }
    let answer = ask(
        &mut server,
        json!({ "kind": "fragmentReach", "fragment": "_fragments/check.md", "build": "site" }),
    );
    let expected = cli_review_prompt(&root, "docs/_fragments/check.md", "site");
    assert!(expected.contains("and 2 pages show it"), "{expected}");
    assert_eq!(answer["prompt"].as_str().unwrap_or_default(), expected);
    server.shutdown();
}
