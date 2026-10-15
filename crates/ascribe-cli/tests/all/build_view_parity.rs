//! The build lens's parity with `ascribe build`: what `ascribe/buildView`
//! says a build leaves out of a page is what `ascribe build --build <name>`
//! leaves out, for every page and build of the example projects.
//!
//! The server runs as a real process over stdio. The plain output says which
//! pages a build publishes; it carries no source positions, so the blocks
//! are compared through the JSON output, which is the same resolved page
//! with each block's source span. A build that switches and badges (`site`)
//! leaves nothing out, so its JSON lists every block of each page: one
//! inside a range the lens dims is missing from the build's JSON, and one
//! outside every range is there.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ascribe_core::{LineIndex, WideEncoding, WideLineCol};
use ascribe_model::{AvailabilityMode, VariantMode};
use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(30);

fn examples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
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
        // UTF-16 columns, the protocol's default.
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

    /// `ascribe/buildView`, once the project has loaded: before then the
    /// answer has no build.
    fn build_view(&mut self, page: &Path, build: &str) -> Value {
        let params = json!({ "textDocument": { "uri": file_uri(page) }, "build": build });
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let result = self.request("ascribe/buildView", params.clone());
            if result["build"] == build || Instant::now() > deadline {
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

/// The spans of the blocks a page's JSON output has from the page's own
/// text, with each block's type, nested blocks included.
fn own_blocks(json: &Value, page: &str) -> BTreeSet<(usize, usize, String)> {
    fn walk(value: &Value, page: &str, out: &mut BTreeSet<(usize, usize, String)>) {
        match value {
            Value::Object(map) => {
                if let Some(source) = map.get("source")
                    && source["file"] == page
                    && source.get("via").is_none()
                    && let Some(kind) = map.get("type").and_then(Value::as_str)
                {
                    let span = &source["span"];
                    out.insert((
                        span[0].as_u64().unwrap() as usize,
                        span[1].as_u64().unwrap() as usize,
                        kind.to_owned(),
                    ));
                }
                map.values().for_each(|v| walk(v, page, out));
            }
            Value::Array(items) => items.iter().for_each(|v| walk(v, page, out)),
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    walk(&json["blocks"], page, &mut out);
    out
}

/// The byte ranges of what the lens dims.
fn dimmed(text: &str, result: &Value) -> Vec<(usize, usize)> {
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
            .expect("a position in the page")
    };
    result["excluded"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|e| (at(&e["range"]["start"]), at(&e["range"]["end"])))
        .collect()
}

fn read_json(path: &Path) -> Option<Value> {
    let mut text = String::new();
    fs::File::open(path).ok()?.read_to_string(&mut text).ok()?;
    Some(serde_json::from_str(&text).expect("JSON output"))
}

/// Compares the lens with `ascribe build` for every page and build of the
/// project at `source`. Returns how many ranges the lens dimmed, so a caller
/// can check the project exercises it.
fn parity(source: &Path) -> usize {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().canonicalize().expect("real path");
    copy_dir(source, &root);
    let model = ascribe_model::load(root.join("ascribe.toml")).expect("the model loads");
    let status = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(&root)
        .args(["build", "--emit", "plain,json"])
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe build");
    assert!(status.status.success(), "{status:?}");
    let out = root.join(&model.project.output_dir);
    let content = root.join(&model.project.content_root);
    // The build that leaves nothing out lists every page and every block.
    let whole = model
        .builds
        .iter()
        .find(|b| {
            matches!(b.variants, VariantMode::Switch)
                && matches!(b.availability, AvailabilityMode::Badge)
        })
        .expect("a build that switches and badges");
    let json_dir = |build: &str| out.join(build).join("json");
    let mut pages = Vec::new();
    let mut stack = vec![json_dir(&whole.name)];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "json") {
                let rel = path.strip_prefix(json_dir(&whole.name)).unwrap();
                let rel = rel
                    .with_extension("md")
                    .to_string_lossy()
                    .replace('\\', "/");
                pages.push(rel);
            }
        }
    }
    pages.sort();
    assert!(!pages.is_empty());

    let mut server = Server::start(&root);
    let mut ranges = 0;
    for page in &pages {
        let file = content.join(page);
        let text = fs::read_to_string(&file).expect("the page");
        let every = own_blocks(
            &read_json(&json_dir(&whole.name).join(page).with_extension("json")).unwrap(),
            page,
        );
        for build in &model.builds {
            let result = server.build_view(&file, &build.name);
            assert_eq!(result["build"], build.name.as_str(), "{page}");
            let published = out.join(&build.name).join("plain").join(page).is_file();
            assert_eq!(
                result["pageIncluded"], published,
                "{page} in {}: the lens and the plain output disagree on whether it's published",
                build.name
            );
            let Some(json) = read_json(&json_dir(&build.name).join(page).with_extension("json"))
            else {
                assert!(!published);
                assert_eq!(result["excluded"], json!([]));
                continue;
            };
            let kept = own_blocks(&json, page);
            let dimmed = dimmed(&text, &result);
            ranges += dimmed.len();
            for (start, end) in &dimmed {
                assert!(
                    every.iter().any(|(s, e, _)| start <= s && e <= end),
                    "{page} in {}: {start}..{end} is dimmed but holds no block",
                    build.name
                );
            }
            for block in &every {
                let (s, e, kind) = block;
                let inside = dimmed.iter().any(|(start, end)| start <= s && e <= end);
                if inside {
                    assert!(
                        !kept.contains(block),
                        "{page} in {}: the {kind} at {s}..{e} is dimmed, but the build keeps it",
                        build.name
                    );
                } else if kind != "group" {
                    // A group reduced to one arm is that arm's blocks.
                    assert!(
                        kept.contains(block),
                        "{page} in {}: the build leaves out the {kind} at {s}..{e}, but it isn't dimmed",
                        build.name
                    );
                }
            }
        }
    }
    server.shutdown();
    ranges
}

#[test]
fn the_lens_dims_what_each_quill_build_leaves_out() {
    assert!(parity(&examples().join("quill")) > 0);
}

#[test]
fn the_lens_dims_what_each_lantern_build_leaves_out() {
    assert!(parity(&examples().join("monorepo/docs")) > 0);
}
