//! How long the server takes to answer a completion request, from the
//! keystroke that makes it: the time between sending `textDocument/didChange`
//! (which types the text of one completion context into a page) and receiving
//! the `textDocument/completion` response.
//!
//! Run with `cargo bench -p tessera-lsp --bench completion`. It builds the
//! projects of `keystroke.rs` (`benches/synthetic`), of 20 to 3,000 pages,
//! starts the server in-process, opens a page, and for each kind of completion
//! types into it and asks, `SAMPLES` times, while the server checks the edit in
//! the background as it would in an editor. The target (SPEC §10's authoring
//! environment, phase 16) is 50 ms at 3,000 pages; the process exits non-zero
//! if any 95th percentile misses it.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::print_stdout,
    clippy::panic
)]

use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};

use lsp_server::Notification;
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Position, Range,
    TextDocumentContentChangeEvent, TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
};
use serde_json::json;
use tessera_lsp::{Options, serve};

mod synthetic;
use synthetic::{Client, page_path, page_text, write_project};

const SAMPLES: usize = 100;
const TARGET: Duration = Duration::from_millis(50);

/// What is typed on line 6 (the paragraph "Intro for {product}, typed: "), for
/// each context. `{k}` becomes the sample number, so every request follows a
/// real change.
const KINDS: [(&str, &str); 8] = [
    ("directive", "@no"),
    ("attribute", "@variant {deployment="),
    ("phrase", "Use {pro"),
    ("include path", "@include: ../_f/f{k}"),
    ("include id", "@include: /_f/f{k}.md#"),
    ("link, nothing typed", "See [x]("),
    ("link, page title", "See [x](Page {k}"),
    ("link, heading", "See [x](Setup"),
];

fn run(pages: usize) -> Vec<(String, Duration, Duration)> {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    write_project(&root, pages);
    let edited = pages / 2;
    let path = root.join(format!("docs/{}", page_path(edited)));
    let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();
    let text = page_text(edited, pages);

    let (client_side, server_side) = lsp_server::Connection::memory();
    let server = thread::spawn(move || serve(server_side, Options::default()).unwrap());
    let mut client = Client {
        conn: client_side,
        next: 0,
    };
    client.request(
        "initialize",
        json!({
            "processId": null,
            "workspaceFolders": [{ "uri": format!("file://{}", root.display()), "name": "w" }],
            "capabilities": {},
        }),
    );
    client
        .conn
        .sender
        .send(Notification::new("initialized".into(), json!({})).into())
        .unwrap();
    client
        .conn
        .sender
        .send(
            Notification::new(
                "textDocument/didOpen".into(),
                DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: uri.clone(),
                        language_id: "ascribe".into(),
                        version: 1,
                        text,
                    },
                },
            )
            .into(),
        )
        .unwrap();
    // Loaded: the first diagnostics for the page are out.
    client.wait_publish(&uri, Some(1));

    let mut version = 1;
    let mut current_len = "Intro for {product}, typed: ".len() as u32;
    let mut results = Vec::new();
    for (name, typed) in KINDS {
        let mut times = Vec::new();
        for k in 0..SAMPLES {
            version += 1;
            let line = typed.replace("{k}", &(k % pages.min(100)).to_string());
            let began = Instant::now();
            client
                .conn
                .sender
                .send(
                    Notification::new(
                        "textDocument/didChange".into(),
                        DidChangeTextDocumentParams {
                            text_document: VersionedTextDocumentIdentifier {
                                uri: uri.clone(),
                                version,
                            },
                            content_changes: vec![TextDocumentContentChangeEvent {
                                range: Some(Range::new(
                                    Position::new(6, 0),
                                    Position::new(6, current_len),
                                )),
                                range_length: None,
                                text: line.clone(),
                            }],
                        },
                    )
                    .into(),
                )
                .unwrap();
            current_len = line.len() as u32;
            let response = client.request(
                "textDocument/completion",
                json!({
                    "textDocument": { "uri": uri.as_str() },
                    "position": { "line": 6, "character": current_len },
                }),
            );
            times.push(began.elapsed());
            let result = response.response_result.expect("a completion result");
            assert!(
                !result["items"].as_array().is_none_or(Vec::is_empty),
                "{name}: no items: {result}"
            );
        }
        times.sort();
        let q = |f: f64| times[((times.len() as f64 - 1.0) * f).round() as usize];
        results.push((name.to_owned(), q(0.5), q(0.95)));
    }
    client.request("shutdown", json!(null));
    client
        .conn
        .sender
        .send(Notification::new("exit".into(), json!(null)).into())
        .unwrap();
    server.join().unwrap();
    results
}

fn main() {
    println!(
        "keystroke-to-completion latency, {SAMPLES} requests per context (median / p95), target {TARGET:?}"
    );
    let mut missed = false;
    for pages in [20, 100, 300, 1000, 3000] {
        let results = run(pages);
        println!("{pages:>5} pages:");
        for (name, median, p95) in results {
            let note = if p95 > TARGET {
                missed = true;
                "  <-- over target"
            } else {
                ""
            };
            println!("        {name:<22} {median:>9.3?} / {p95:>9.3?}{note}");
        }
    }
    if missed {
        std::process::exit(1);
    }
}
