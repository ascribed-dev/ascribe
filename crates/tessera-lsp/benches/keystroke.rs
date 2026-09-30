//! How long the server takes from a keystroke to the diagnostics of the page
//! being edited: the time between sending `textDocument/didChange` and
//! receiving the `publishDiagnostics` for that version.
//!
//! Run with `cargo bench -p tessera-lsp --bench keystroke`. It builds projects
//! of several sizes in a temporary directory, with the generator every
//! performance test shares (`tessera-synthetic`: every page has a title,
//! headings, an include of a fragment, links to other pages, an image, and one
//! page in ten has an availability marker), starts the server in-process over a memory
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
use std::time::{Duration, Instant};

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Position, PublishDiagnosticsParams,
    Range, TextDocumentContentChangeEvent, TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
};
use serde_json::json;
use tessera_lsp::{Options, serve};
use tessera_synthetic::{
    CONTENT_ROOT, FRAGMENT_BODY, FRAGMENTS, Synthetic, TYPING_LINE, TYPING_PREFIX,
};
mod synthetic;
use synthetic::Client;

const KEYSTROKES: usize = 100;

fn run(pages: usize) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let project = Synthetic::new(pages);
    project.write_to(&root).unwrap();
    let edited = pages / 2;
    let path = root.join(format!("{CONTENT_ROOT}/{}", project.page_path(edited)));
    let uri = Uri::from_str(&format!("file://{}", path.display())).unwrap();
    let text = project.page_text(edited, "typed: ");

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

    // Type at the end of the intro paragraph ("Intro for {product}, typed: ").
    let line_length = (TYPING_PREFIX.len() + "typed: ".len()) as u32;
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
                            range: Some(Range::new(
                                Position::new(TYPING_LINE, at),
                                Position::new(TYPING_LINE, at),
                            )),
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
    if pages == 3000 {
        tessera_synthetic::report::record("lsp/keystroke-page-3000", &mut times.clone());
    }
    times.sort();
    let q = |f: f64| times[((times.len() as f64 - 1.0) * f).round() as usize];

    // The same keystrokes with the preview open: each change is
    // followed by an `ascribe/preview` request for the page, as the editor sends
    // it after its debounce. `preview` is the time from the change to the
    // answer; `with preview` is the time from the change to its diagnostics,
    // which must stay what it was without one.
    let mut preview_times = Vec::new();
    let mut with_preview_times = Vec::new();
    for k in 0..KEYSTROKES {
        let version = 2 + KEYSTROKES as i32 + k as i32;
        let at = line_length + (KEYSTROKES + k) as u32;
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
        client.next += 1;
        let id = RequestId::from(client.next);
        client
            .conn
            .sender
            .send(
                Request::new(
                    id.clone(),
                    "ascribe/preview".into(),
                    json!({ "textDocument": { "uri": uri.as_str() } }),
                )
                .into(),
            )
            .unwrap();
        let (mut answered, mut published) = (None, None);
        while answered.is_none() || published.is_none() {
            match client
                .conn
                .receiver
                .recv_timeout(Duration::from_secs(60))
                .unwrap()
            {
                Message::Response(r) if r.id == id => {
                    let result = r.response_result.expect("the preview answers");
                    assert!(!result["page"].is_null(), "{result}");
                    answered = Some(began.elapsed());
                }
                Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => {
                    let p: PublishDiagnosticsParams = serde_json::from_value(n.params).unwrap();
                    if p.uri == uri && p.version == Some(version) {
                        published = Some(began.elapsed());
                    }
                }
                Message::Request(r) => {
                    client
                        .conn
                        .sender
                        .send(Response::new_ok(r.id, serde_json::Value::Null).into())
                        .unwrap();
                }
                _ => {}
            }
        }
        preview_times.push(answered.unwrap());
        with_preview_times.push(published.unwrap());
    }
    preview_times.sort();
    with_preview_times.sort();
    let pv = |f: f64| preview_times[((preview_times.len() as f64 - 1.0) * f).round() as usize];
    let wp =
        |f: f64| with_preview_times[((with_preview_times.len() as f64 - 1.0) * f).round() as usize];

    // Type into a fragment that pages include (each of the first fragments is
    // included by `pages / 100` pages).
    let fragment = root.join(format!("{CONTENT_ROOT}/{}", project.fragment_path(0)));
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
                        text: project.fragment_text(0, FRAGMENT_BODY),
                    },
                },
            )
            .into(),
        )
        .unwrap();
    let mut fragment_times = Vec::new();
    for k in 0..KEYSTROKES {
        let version = 2 + k as i32;
        let at = FRAGMENT_BODY.len() as u32 + k as u32;
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
    if pages == 3000 {
        tessera_synthetic::report::record(
            "lsp/keystroke-fragment-3000",
            &mut fragment_times.clone(),
        );
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
    println!(
        "        with the preview open: change to answer median {:>9.3?} p95 {:>9.3?}   change to diagnostics median {:>9.3?} p95 {:>9.3?}",
        pv(0.5),
        pv(0.95),
        wp(0.5),
        wp(0.95)
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
