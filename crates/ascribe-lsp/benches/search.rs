//! How long the server takes to answer `ascribe/search` as you type, with
//! each page's reading held: the time between sending the request and
//! receiving the answer, once a first search has read every page.
//!
//! Run with `cargo bench -p ascribe-lsp --bench search`. It builds the
//! projects of `keystroke.rs` (`benches/synthetic`), of 20 to 3,000 pages,
//! starts the server in-process, searches once to read the pages, and then
//! asks `SAMPLES` times, a query from `QUERIES` each time: words on every
//! page, a fragment's phrase on every page, words on one page, and words on
//! none. The target is a median of 50 ms at 3,000 pages; the process exits
//! non-zero if a median misses it. With `ASCRIBE_BENCH_OUT` set, the 3,000
//! pages are recorded as `lsp/search-3000`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::print_stdout,
    clippy::panic
)]

use std::thread;
use std::time::{Duration, Instant};

use ascribe_lsp::{Options, serve};
use lsp_server::Notification;
use serde_json::json;

mod synthetic;
use synthetic::{Client, write_project};

const SAMPLES: usize = 100;
const TARGET: Duration = Duration::from_millis(50);

/// What's typed into the search, in turn.
const QUERIES: [&str; 4] = [
    "steps for page",
    "\"shared text\"",
    "\"steps for page 1234\"",
    "zebrafinch",
];

fn run(pages: usize) -> (Duration, Duration, Duration) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    write_project(&root, pages);
    let model = format!("file://{}", root.join("ascribe.toml").display());

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
    let search = |client: &mut Client, query: &str| {
        let began = Instant::now();
        let response = client.request(
            "ascribe/search",
            json!({ "textDocument": { "uri": model }, "query": query }),
        );
        let took = began.elapsed();
        let result = response.response_result.expect("a search result");
        assert_eq!(result["problem"], json!(null), "{result}");
        took
    };
    // The first search reads every page.
    let cold = search(&mut client, QUERIES[0]);
    let mut times: Vec<Duration> = (0..SAMPLES)
        .map(|k| search(&mut client, QUERIES[k % QUERIES.len()]))
        .collect();
    if pages == 3000 {
        ascribe_synthetic::report::record("lsp/search-3000", &mut times.clone());
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
    (cold, q(0.5), q(0.95))
}

fn main() {
    println!(
        "search latency with the readings held, {SAMPLES} queries (first / median / p95), target {TARGET:?}"
    );
    let mut missed = false;
    for pages in [20, 100, 300, 1000, 3000] {
        let (cold, median, p95) = run(pages);
        println!("{pages:>5} pages: {cold:>10.2?} / {median:>8.2?} / {p95:>8.2?}");
        if pages == 3000 && median > TARGET {
            missed = true;
        }
    }
    if missed {
        println!("missed the target at 3,000 pages");
        std::process::exit(1);
    }
}
