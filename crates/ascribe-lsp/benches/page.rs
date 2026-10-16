//! What a cursor move costs the editor's This page view, which asks
//! `ascribe/page` once, against what it cost Used by, which asked
//! `ascribe/context` and then `textDocument/references` at the heading it
//! found (and not again while the cursor stayed under the same heading).
//!
//! Run with `cargo bench -p ascribe-lsp --bench page`. It starts the server
//! in-process on Ascribe's own docs (`docs/`), opens a long page and a short
//! one, and moves the cursor to the same 50 lines of each, spread through the
//! page, asking both ways at each move once the server is idle. For each way
//! it reports two medians (and p95s) per move: the server's time, from when
//! the server starts handling a request to its answer, summed over the move's
//! requests; and the round trip, from sending the first request to receiving
//! the last answer. Each request the editor sends also crosses the process
//! boundary to `ascribe lsp`, which an in-process connection doesn't, so a
//! move with one request saves one more crossing than these numbers show. The
//! process exits non-zero if `ascribe/page` is slower than Used by's requests
//! on either measure.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::print_stdout,
    clippy::panic
)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use ascribe_lsp::{Options, serve};
use lsp_server::{Message, Notification, Request, RequestId};
use lsp_types::{DidOpenTextDocumentParams, TextDocumentItem, Uri};
use serde_json::{Value, json};

const MOVES: usize = 50;

/// The pages, from the content root: a long one and a short one.
const PAGES: [&str; 2] = ["writing/directives.md", "agents/measuring.md"];

/// When the server began handling each request, by method, oldest first.
type Began = Arc<Mutex<HashMap<String, Vec<Instant>>>>;

struct Client {
    conn: lsp_server::Connection,
    next: i32,
    began: Began,
    idle: Arc<AtomicBool>,
}

/// One request's times.
struct Timed {
    result: Value,
    sent: Instant,
    /// From when the server began handling it to its answer.
    server: Duration,
    received: Instant,
}

impl Client {
    fn request(&mut self, method: &str, params: Value) -> Timed {
        self.next += 1;
        let id = RequestId::from(self.next);
        let sent = Instant::now();
        self.conn
            .sender
            .send(Request::new(id.clone(), method.to_owned(), params).into())
            .unwrap();
        loop {
            match self.conn.receiver.recv().unwrap() {
                Message::Response(r) if r.id == id => {
                    let received = Instant::now();
                    let began = self
                        .began
                        .lock()
                        .unwrap()
                        .get_mut(method)
                        .and_then(Vec::pop)
                        // `initialize` is handled before the hook is in place.
                        .unwrap_or(sent);
                    return Timed {
                        result: r.response_result.expect("an answer"),
                        sent,
                        server: received - began,
                        received,
                    };
                }
                Message::Request(r) => {
                    self.conn
                        .sender
                        .send(lsp_server::Response::new_ok(r.id, Value::Null).into())
                        .unwrap();
                }
                _ => {}
            }
        }
    }

    fn notify(&self, method: &str, params: Value) {
        self.conn
            .sender
            .send(Notification::new(method.to_owned(), params).into())
            .unwrap();
    }

    /// Waits until the server has nothing queued and isn't computing,
    /// answering what it asks and dropping its notifications meanwhile.
    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            while let Ok(message) = self.conn.receiver.try_recv() {
                if let Message::Request(r) = message {
                    self.conn
                        .sender
                        .send(lsp_server::Response::new_ok(r.id, Value::Null).into())
                        .unwrap();
                }
            }
            if self.idle.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(20));
                if self.idle.load(Ordering::SeqCst) && self.conn.receiver.is_empty() {
                    return;
                }
            }
            assert!(Instant::now() < deadline, "the server never settled");
            thread::sleep(Duration::from_millis(5));
        }
    }
}

/// A move's two measures.
#[derive(Default)]
struct Measures {
    server: Vec<Duration>,
    round_trip: Vec<Duration>,
}

impl Measures {
    fn push(&mut self, requests: &[Timed]) {
        let (Some(first), Some(last)) = (requests.first(), requests.last()) else {
            return;
        };
        self.server.push(requests.iter().map(|t| t.server).sum());
        self.round_trip.push(last.received - first.sent);
    }
}

fn quantiles(times: &mut [Duration]) -> (Duration, Duration) {
    times.sort();
    let q = |f: f64| times[((times.len() as f64 - 1.0) * f).round() as usize];
    (q(0.5), q(0.95))
}

fn uri(path: &Path) -> Uri {
    Uri::from_str(&format!("file://{}", path.display())).unwrap()
}

/// The heading `ascribe/context` says the position is on, as Used by found
/// it: its range's start.
fn heading_start(context: &Value) -> Option<Value> {
    context["at"]
        .as_array()?
        .iter()
        .find(|node| node["kind"] == "heading")
        .map(|node| node["range"]["start"].clone())
}

fn run(client: &mut Client, docs: &Path, page: &str) -> (Measures, Measures) {
    let path = docs.join("content").join(page);
    let text = std::fs::read_to_string(&path).unwrap();
    let uri = uri(&path);
    client.notify(
        "textDocument/didOpen",
        serde_json::to_value(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "markdown".into(),
                version: 1,
                text: text.clone(),
            },
        })
        .unwrap(),
    );
    client.settle();
    let lines = text.lines().count();
    let mut used_by = Measures::default();
    let mut this_page = Measures::default();
    let mut question: Option<String> = None;
    for k in 0..MOVES {
        let position = json!({ "line": k * lines / MOVES, "character": 0 });

        // Used by: where the cursor is, then the references of the heading it's
        // on (or of the page), unless that's what it asked last.
        client.settle();
        let context = client.request(
            "ascribe/context",
            json!({
                "textDocument": { "uri": uri.as_str() },
                "range": { "start": position, "end": position },
            }),
        );
        // At the very start of the file, the server answers for the page.
        let start = json!({ "line": 0, "character": 0 });
        let at = match heading_start(&context.result) {
            Some(heading) if heading == start => json!({ "line": 0, "character": 1 }),
            Some(heading) => heading,
            None => start,
        };
        let asked = at.to_string();
        let mut requests = vec![context];
        if question.as_deref() != Some(asked.as_str()) {
            question = Some(asked);
            requests.push(client.request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": uri.as_str() },
                    "position": at,
                    "context": { "includeDeclaration": false },
                }),
            ));
        }
        used_by.push(&requests);

        // This page: one request.
        client.settle();
        let page = client.request(
            "ascribe/page",
            json!({ "textDocument": { "uri": uri.as_str() }, "position": position }),
        );
        assert!(
            page.result["about"].is_object(),
            "{page:?}",
            page = page.result
        );
        this_page.push(&[page]);
    }
    (used_by, this_page)
}

fn main() {
    let docs: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs")
        .canonicalize()
        .unwrap();
    let began: Began = Arc::default();
    let idle = Arc::new(AtomicBool::new(false));
    let (client_side, server_side) = lsp_server::Connection::memory();
    let hook = {
        let began = began.clone();
        Arc::new(move |method: &str| {
            began
                .lock()
                .unwrap()
                .entry(method.to_owned())
                .or_default()
                .push(Instant::now());
        })
    };
    let options = Options {
        before_request: Some(hook),
        idle: Some(idle.clone()),
        ..Options::default()
    };
    let server = thread::spawn(move || serve(server_side, options).unwrap());
    let mut client = Client {
        conn: client_side,
        next: 0,
        began,
        idle,
    };
    client.request(
        "initialize",
        json!({
            "processId": null,
            "workspaceFolders": [{ "uri": uri(&docs).as_str(), "name": "docs" }],
            "capabilities": {},
        }),
    );
    client.notify("initialized", json!({}));
    client.settle();

    println!("A cursor move on Ascribe's docs, {MOVES} moves per page: median (p95) per move");
    println!(
        "{:<24} {:>8}  {:>22}  {:>22}",
        "page", "lines", "server", "round trip"
    );
    let mut slower = false;
    for page in PAGES {
        let lines = std::fs::read_to_string(docs.join("content").join(page))
            .unwrap()
            .lines()
            .count();
        let (mut used_by, mut this_page) = run(&mut client, &docs, page);
        let used_by_server = quantiles(&mut used_by.server);
        let used_by_trip = quantiles(&mut used_by.round_trip);
        let page_server = quantiles(&mut this_page.server);
        let page_trip = quantiles(&mut this_page.round_trip);
        let row = |name: &str, server: (Duration, Duration), trip: (Duration, Duration)| {
            println!(
                "  {name:<22} {:>8}  {:>10.3?} ({:>9.3?})  {:>10.3?} ({:>9.3?})",
                "", server.0, server.1, trip.0, trip.1
            );
        };
        println!("{page:<24} {lines:>8}");
        row("Used by (2 requests)", used_by_server, used_by_trip);
        row("ascribe/page", page_server, page_trip);
        if page_server.0 > used_by_server.0 || page_trip.0 > used_by_trip.0 {
            slower = true;
        }
    }

    client.request("shutdown", json!(null));
    client.notify("exit", json!(null));
    server.join().unwrap();
    if slower {
        println!("ascribe/page is slower than Used by's requests");
        std::process::exit(1);
    }
}
