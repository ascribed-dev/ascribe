//! How long the server takes to answer `ascribe/inventory`, which the editor's
//! Pages and Content model views ask for on every save: the time between
//! sending the request, after an edit, and receiving the answer.
//!
//! Run with `cargo bench -p ascribe-lsp --bench inventory`. It builds the
//! projects of `keystroke.rs` (`benches/synthetic`), of 20 to 3,000 pages,
//! with a glossary term added so that the count scans every page's prose,
//! starts the server in-process, opens a page, and `SAMPLES` times changes it
//! and asks, while the server checks the edit in the background as it would in
//! an editor. The target is a median of 50 ms at 3,000 pages; the process
//! exits non-zero if a median misses it.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::print_stdout,
    clippy::panic
)]

use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};

use ascribe_lsp::{Options, serve};
use lsp_server::Notification;
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Position, Range,
    TextDocumentContentChangeEvent, TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
};
use serde_json::json;

mod synthetic;
use synthetic::{Client, MODEL, page_path, page_text, write, write_project};

const SAMPLES: usize = 100;
const TARGET: Duration = Duration::from_millis(50);

/// A glossary term that is in every page's prose ("Steps for page 7.").
const GLOSSARY: &str = "\n[glossary.terms.page]\nterm = \"page\"\ndefinition = \"A page.\"\n";

fn run(pages: usize) -> (Duration, Duration) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    write_project(&root, pages);
    write(&root, "ascribe.toml", &format!("{MODEL}{GLOSSARY}"));
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
    client.wait_publish(&uri, Some(1));

    let mut current_len = "Intro for {product}, typed: ".len() as u32;
    let mut times = Vec::new();
    for k in 0..SAMPLES {
        let version = k as i32 + 2;
        // A link to another page, so every answer counts something new.
        let line = format!("See [p](/{}).", page_path(k % pages));
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
        let began = Instant::now();
        let response = client.request(
            "ascribe/inventory",
            json!({ "textDocument": { "uri": uri.as_str() } }),
        );
        times.push(began.elapsed());
        let result = response.response_result.expect("an inventory");
        assert_eq!(
            result["pages"].as_array().map(Vec::len),
            Some(pages),
            "every page is listed"
        );
    }
    client.request("shutdown", json!(null));
    client
        .conn
        .sender
        .send(Notification::new("exit".into(), json!(null)).into())
        .unwrap();
    server.join().unwrap();
    times.sort();
    let q = |f: f64| times[((times.len() as f64 - 1.0) * f).round() as usize];
    (q(0.5), q(0.95))
}

fn main() {
    println!(
        "ascribe/inventory after an edit, {SAMPLES} requests (median / p95), target {TARGET:?}"
    );
    let mut missed = false;
    for pages in [20, 100, 300, 1000, 3000] {
        let (median, p95) = run(pages);
        let note = if median > TARGET {
            missed = true;
            "  <-- over target"
        } else {
            ""
        };
        println!("{pages:>5} pages: {median:>9.3?} / {p95:>9.3?}{note}");
    }
    if missed {
        std::process::exit(1);
    }
}
