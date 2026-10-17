//! This page's parity with the outputs and the checks: what `ascribe/page`
//! says of a page's size is what `page-size` measures in `ascribe check`,
//! its listing is what `pages.json` writes for it in `ascribe build`, its
//! review warns exactly where `ascribe check` reports `review-overdue`, and
//! its description is missing exactly where it reports
//! `page-description-missing`, for projects made here and for
//! `examples/quill`.
//!
//! The server runs as a real process over stdio, as an editor starts it,
//! with `ASCRIBE_TODAY` set as `ascribe check` has it. `page-size`'s limit
//! is lowered to one character, so it reports every page with its size.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(30);

/// The day both the server and `ascribe check` run on.
const TODAY: &str = "2026-10-17";

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

// -- A stdio client -----------------------------------------------------------

struct Server {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    messages: mpsc::Receiver<Value>,
    next_id: u64,
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
        if let Some(value) = line.strip_prefix("Content-Length: ") {
            length = Some(value.parse::<usize>().expect("a length"));
        }
    }
    let mut body = vec![0; length.expect("a Content-Length header")];
    reader.read_exact(&mut body).ok()?;
    Some(serde_json::from_slice(&body).expect("JSON body"))
}

impl Server {
    fn start(root: &Path) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ascribe"))
            .arg("lsp")
            .current_dir(root)
            .env("ASCRIBE_TODAY", TODAY)
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
            next_id: 1,
        };
        server.request(
            "initialize",
            json!({
                "processId": null,
                "workspaceFolders": [{ "uri": file_uri(root), "name": "w" }],
                "capabilities": {},
            }),
        );
        server.send(&json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
        server
    }

    fn send(&mut self, message: &Value) {
        let body = serde_json::to_vec(message).expect("json");
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len()).expect("write");
        self.stdin.write_all(&body).expect("write");
        self.stdin.flush().expect("flush");
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let message = self.messages.recv_timeout(left).expect("a response");
            if message.get("method").is_some() {
                // A request from the server (registerCapability): accept it.
                if let Some(their_id) = message.get("id") {
                    let reply = json!({ "jsonrpc": "2.0", "id": their_id, "result": null });
                    self.send(&reply);
                }
                continue;
            }
            if message["id"] == id {
                return message["result"].clone();
            }
        }
    }

    /// `ascribe/page` for `page`, once the project has loaded: before then
    /// the answer is empty.
    fn page(&mut self, page: &Path) -> Value {
        let params = json!({
            "textDocument": { "uri": file_uri(page) },
            "position": { "line": 0, "character": 0 },
        });
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let result = self.request("ascribe/page", params.clone());
            if !result["about"].is_null() || Instant::now() > deadline {
                return result;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn shutdown(mut self) {
        self.request("shutdown", Value::Null);
        self.send(&json!({ "jsonrpc": "2.0", "method": "exit" }));
        let status = self.child.wait().expect("the server exits");
        assert!(status.success(), "the server exits with 0: {status:?}");
    }
}

// -- The outputs ----------------------------------------------------------------

/// A number with commas between its thousands, as `page-size` writes it.
fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// What `ascribe check` reports, every build: each file's diagnostics, by
/// its path from the project root.
fn check(root: &Path) -> BTreeMap<String, Vec<Value>> {
    let output = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(root)
        .args(["check", "--format", "json"])
        .env("ASCRIBE_TODAY", TODAY)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe check");
    let report: Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    let mut out: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for d in report["diagnostics"].as_array().expect("diagnostics") {
        out.entry(d["file"].as_str().expect("file").to_owned())
            .or_default()
            .push(d.clone());
    }
    out
}

/// The record `pages.json` of `build` has for the page at `source`.
fn record(out: &Path, build: &str, source: &str) -> Value {
    let path = out.join(build).join("site/_ascribe/pages.json");
    let mut text = String::new();
    fs::File::open(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .read_to_string(&mut text)
        .expect("read");
    let list: Value = serde_json::from_str(&text).expect("JSON");
    list["pages"]
        .as_object()
        .expect("pages")
        .values()
        .find(|p| p["source"] == source)
        .cloned()
        .unwrap_or_else(|| panic!("no {source} in {}", path.display()))
}

/// How many pages of a project This page says something of, so a caller can
/// check the project exercises it.
#[derive(Debug, Default, PartialEq)]
struct Said {
    /// Pages whose review date has passed.
    overdue: usize,
    /// Of those, the ones whose review warns.
    reported: usize,
    /// Pages whose description is measured.
    measured: usize,
    /// Pages whose description is missing.
    missing: usize,
}

/// Compares This page with `ascribe check` and `ascribe build` for every
/// page of the project at `root`. A project with errors doesn't build, so
/// for one `builds` is false and its listings aren't compared.
fn parity(root: &Path, builds: bool) -> Said {
    let model_path = root.join("ascribe.toml");
    let mut model_text = fs::read_to_string(&model_path).expect("model");
    model_text.push_str("\n[checks.page-size]\nlimit = 1\n");
    fs::write(&model_path, model_text).expect("write model");
    let model = ascribe_model::load(&model_path).expect("the model loads");
    if builds {
        let status = Command::new(env!("CARGO_BIN_EXE_ascribe"))
            .current_dir(root)
            .args(["build", "--emit", "site"])
            .env("NO_COLOR", "1")
            .output()
            .expect("run ascribe build");
        assert!(status.status.success(), "{status:?}");
    }
    let out = root.join(&model.project.output_dir);
    let content = root.join(&model.project.content_root);
    let checked = check(root);
    let mut server = Server::start(root);
    let mut said = Said::default();
    let mut pages: Vec<PathBuf> = Vec::new();
    collect_pages(&content, &mut pages);
    pages.sort();
    for page in pages {
        let rel = page
            .strip_prefix(&content)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if rel.split('/').any(|s| s.starts_with('_')) {
            continue;
        }
        let file = format!("{}/{rel}", model.project.content_root);
        let file = file.trim_start_matches("./").to_owned();
        let answer = server.page(&page);
        let about = &answer["about"];
        assert!(!about.is_null(), "{rel}: no answer");
        let found = checked.get(&file).cloned().unwrap_or_default();
        let slugged = |slug: &str| found.iter().find(|d| d["slug"] == slug).cloned();
        // Size: page-size's measure, in a build it names.
        match (slugged("page-size"), answer["health"]["size"].clone()) {
            (None, Value::Null) => {}
            (Some(d), size) => {
                let message = d["message"].as_str().unwrap();
                let characters = thousands(size["characters"].as_u64().unwrap_or_else(|| {
                    panic!("{rel}: page-size reports it, and This page has no size")
                }));
                assert!(
                    message.contains(&format!(" {characters} characters in build")),
                    "{rel}: This page says {size}, and page-size says {message}"
                );
                let build = size["build"].as_str().unwrap();
                assert!(
                    message.contains(&format!("`{build}`")),
                    "{rel}: {build} isn't named in {message}"
                );
            }
            (None, size) => panic!("{rel}: This page says {size}, and page-size reports nothing"),
        }
        // Listing: the page's record in the build's pages.json.
        if builds && !about["listing"].is_null() {
            let listing = &about["listing"];
            let build = listing["build"].as_str().unwrap();
            let record = record(&out, build, &rel);
            assert_eq!(listing["listing"], record["listing"], "{rel} in {build}");
            assert_eq!(
                listing["from"], record["listing_from"]["level"],
                "{rel} in {build}"
            );
            assert_eq!(
                listing["rule"],
                record["listing_from"]
                    .get("rule")
                    .cloned()
                    .unwrap_or(Value::Null),
                "{rel} in {build}"
            );
        }
        // Review: a warning where review-overdue is reported.
        let review = &about["review"];
        let reported = review["reported"] == true;
        assert_eq!(
            reported,
            slugged("review-overdue").is_some(),
            "{rel}: This page says {review}, and check reports {found:?}",
        );
        assert!(!reported || review["overdue"] == true, "{rel}: {review}");
        said.overdue += usize::from(review["overdue"] == true);
        said.reported += usize::from(reported);
        // Description: missing where page-description-missing is reported.
        let description = &about["description"];
        let missing = !description.is_null() && description["length"].is_null();
        assert_eq!(
            missing,
            slugged("page-description-missing").is_some(),
            "{rel}: This page says {description}, and check reports {found:?}",
        );
        said.measured += usize::from(description["length"].is_u64());
        said.missing += usize::from(missing);
    }
    server.shutdown();
    said
}

fn collect_pages(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read_dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_pages(&path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
}

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[types.guide.frontmatter]
title = "string"
description = { type = "string?", role = "description" }
review-by = { type = "date?", role = "review-date" }

[dimensions.deployment]
label = "Deployment"
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[consumer]
site = "https://docs.example.com"
agents = true

[listing.folders]
"internal/" = "unlisted"

[builds.site]
variants = "switch"
availability = "badge"

[builds.cloud]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }
"#;

#[test]
fn this_page_agrees_with_check_and_build_for_a_project_with_every_row() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().canonicalize().expect("real path");
    let files = [
        ("ascribe.toml", MODEL),
        (
            "docs/index.md",
            "---\ntype: guide\ntitle: Home\ndescription: The start.\n---\n\nSee [the guide](guide.md).\n",
        ),
        (
            "docs/internal/rotate.md",
            "---\ntype: guide\ntitle: Rotate keys\nreview-by: 2026-10-03\navailable: self-managed\n---\n\nRotate them.\n",
        ),
        (
            "docs/guide.md",
            "---\ntype: guide\ntitle: Guide\nreview-by: 2026-11-01\nlisting: unquoted\n---\n\nIt's done for you.\n\n## On your own servers\n@available: self-managed\n\nRun it yourself, and then run it again to be sure it took.\n",
        ),
    ];
    for (path, text) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
        fs::write(path, text).expect("write");
    }
    let said = parity(&root, true);
    assert_eq!(
        said,
        Said {
            overdue: 1,
            reported: 1,
            measured: 1,
            missing: 2
        }
    );
}

/// A project that turns `review-overdue` off, whose description has a
/// default, and whose pages' descriptions are each a case the check is
/// silent for.
const QUIET_MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[types.guide.frontmatter]
title = "string"
summary = { type = "string", role = "description", default = "A guide to *Lantern*." }
review-by = { type = "date?", role = "review-date" }

[types.note.frontmatter]
title = "string"
description = { type = "string", role = "description" }

[types.tip.frontmatter]
title = "string"
description = { type = "string?", role = "description" }

[consumer]
site = "https://docs.example.com"
agents = true

[checks]
review-overdue = "off"
"#;

#[test]
fn this_page_agrees_with_check_where_the_checks_stay_silent() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().canonicalize().expect("real path");
    let files = [
        ("ascribe.toml", QUIET_MODEL),
        // The default is its description, and its review is overdue with
        // the check off.
        (
            "docs/index.md",
            "---\ntype: guide\ntitle: Home\nreview-by: 2026-10-03\n---\n\nStart here.\n",
        ),
        // Its own description over the default.
        (
            "docs/own.md",
            "---\ntype: guide\ntitle: Own\nsummary: Its own.\n---\n\nIt has one.\n",
        ),
        // A required description left out is frontmatter-missing-field's.
        (
            "docs/note.md",
            "---\ntype: note\ntitle: Note\n---\n\nA note.\n",
        ),
        // A description that isn't a string is frontmatter-type-mismatch's.
        (
            "docs/tip.md",
            "---\ntype: tip\ntitle: Tip\ndescription: 3\n---\n\nA tip.\n",
        ),
    ];
    for (path, text) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
        fs::write(path, text).expect("write");
    }
    let said = parity(&root, false);
    assert_eq!(
        said,
        Said {
            overdue: 1,
            reported: 0,
            measured: 2,
            missing: 0
        }
    );
}

#[test]
fn this_page_agrees_with_check_and_build_for_quill() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().canonicalize().expect("real path");
    copy_dir(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill"),
        &root,
    );
    parity(&root, true);
}
