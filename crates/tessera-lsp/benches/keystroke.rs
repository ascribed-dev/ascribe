//! How long the server takes from a keystroke to the diagnostics of the page
//! being edited: the time between sending `textDocument/didChange` and
//! receiving the `publishDiagnostics` for that version.
//!
//! Run with `cargo bench -p tessera-lsp --bench keystroke`. It builds projects
//! of several sizes in a temporary directory (every page has a title, headings,
//! an include of a fragment, links to other pages, an image, and one page in ten
//! has an availability marker), starts the server in-process over a memory
//! connection, opens a page, types into it, and prints the load time and the
//! min, median, and 95th percentile of the keystroke latency. A plain `main`
//! rather than a benchmark framework: nothing here needs more statistics than
//! that.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::print_stdout,
    clippy::panic
)]

use std::str::FromStr;
use std::thread;
use std::time::Instant;

use lsp_server::{Connection, Notification};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Position, Range,
    TextDocumentContentChangeEvent, TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
};
use serde_json::json;
use tessera_lsp::{Options, serve};

mod synthetic;
use synthetic::{Client, FRAGMENTS, page_path, page_text, write_project};

const KEYSTROKES: usize = 100;

fn run(pages: usize) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    write_project(&root, pages);
    let edited = 1.min(pages - 1) * (pages / 2);
    let path = root.join(format!("docs/{}", page_path(edited)));
    let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();
    let text = page_text(edited, pages);

    let (client_side, server_side) = Connection::memory();
    let server = thread::spawn(move || serve(server_side, Options::default()).unwrap());
    let mut client = Client {
        conn: client_side,
        next: 0,
    };
    let start = Instant::now();
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
    // The first publication for the page we'll edit, once it's open.
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
                        text: text.clone(),
                    },
                },
            )
            .into(),
        )
        .unwrap();
    client.wait_publish(&uri, Some(1));
    let load = start.elapsed();

    // Type into the paragraph on line 6 ("Intro for {product}, typed: ").
    let line_length = "Intro for {product}, typed: ".len() as u32;
    let mut times = Vec::new();
    for k in 0..KEYSTROKES {
        let version = 2 + k as i32;
        let at = line_length + k as u32;
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
                            range: Some(Range::new(Position::new(6, at), Position::new(6, at))),
                            range_length: None,
                            text: "x".into(),
                        }],
                    },
                )
                .into(),
            )
            .unwrap();
        client.wait_publish(&uri, Some(version));
        times.push(began.elapsed());
    }
    times.sort();
    let q = |f: f64| times[((times.len() as f64 - 1.0) * f).round() as usize];

    // Type into a fragment that pages include (each of the first fragments is
    // included by `pages / 100` pages).
    let fragment = root.join("docs/_f/f0.md");
    let fragment_uri = Uri::from_str(&format!("file://{}", fragment.display())).unwrap();
    client
        .conn
        .sender
        .send(
            Notification::new(
                "textDocument/didOpen".into(),
                DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: fragment_uri.clone(),
                        language_id: "ascribe".into(),
                        version: 1,
                        text: "## Shared 0\n\nShared text.\n".into(),
                    },
                },
            )
            .into(),
        )
        .unwrap();
    let mut fragment_times = Vec::new();
    for k in 0..KEYSTROKES {
        let version = 2 + k as i32;
        let at = "Shared text.".len() as u32 + k as u32;
        let began = Instant::now();
        client
            .conn
            .sender
            .send(
                Notification::new(
                    "textDocument/didChange".into(),
                    DidChangeTextDocumentParams {
                        text_document: VersionedTextDocumentIdentifier {
                            uri: fragment_uri.clone(),
                            version,
                        },
                        content_changes: vec![TextDocumentContentChangeEvent {
                            range: Some(Range::new(Position::new(2, at), Position::new(2, at))),
                            range_length: None,
                            text: "x".into(),
                        }],
                    },
                )
                .into(),
            )
            .unwrap();
        client.wait_publish(&fragment_uri, Some(version));
        fragment_times.push(began.elapsed());
    }
    fragment_times.sort();
    let f = |x: f64| fragment_times[((fragment_times.len() as f64 - 1.0) * x).round() as usize];
    println!(
        "{pages:>5} pages: load+first diagnostics {load:>10.3?}   page keystroke: median {:>9.3?} p95 {:>9.3?}   fragment keystroke ({} includers): median {:>9.3?} p95 {:>9.3?}",
        q(0.5),
        q(0.95),
        pages / FRAGMENTS,
        f(0.5),
        f(0.95)
    );
    client.request("shutdown", json!(null));
    client
        .conn
        .sender
        .send(Notification::new("exit".into(), json!(null)).into())
        .unwrap();
    server.join().unwrap();
}

fn main() {
    println!("keystroke-to-diagnostics latency, {KEYSTROKES} keystrokes per size");
    for pages in [20, 100, 300, 1000, 3000] {
        run(pages);
    }
}
