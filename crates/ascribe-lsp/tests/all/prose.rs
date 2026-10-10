//! Vale on open and on save (`[checks.vale]`), with a stand-in for Vale: its
//! alerts are published with the file's other diagnostics, an edit drops the
//! ones it may have moved, a result for an older text is dropped, and a slow
//! Vale delays nothing else.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use ascribe_check::prose::{Action, Alert, Linter, Request, ValeError};
use ascribe_lsp::Options;

use crate::support::{self, Client, Fixture, Setup, edit, slug};

const VALE_MODEL: &str =
    "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[checks.vale]\npreset = \"quiet\"\n";

const PAGE: &str = "---\ntitle: A\n---\nteh first\n\nteh second [x](nope.md)\n";

/// A stand-in for Vale that flags each `teh`, as a rule whose replacement
/// is `the`. With a gate, each run waits until the test lets it go.
struct Fake {
    runs: AtomicUsize,
    gate: Option<Mutex<mpsc::Receiver<()>>>,
    started: Option<Mutex<mpsc::Sender<()>>>,
    fail: bool,
}

impl Fake {
    fn new() -> Fake {
        Fake {
            runs: AtomicUsize::new(0),
            gate: None,
            started: None,
            fail: false,
        }
    }

    /// A Vale that tells the test when a run starts, and holds it until the
    /// test sends on the gate.
    fn held() -> (Fake, mpsc::Receiver<()>, mpsc::Sender<()>) {
        let (started_tx, started_rx) = mpsc::channel();
        let (gate_tx, gate_rx) = mpsc::channel();
        let fake = Fake {
            gate: Some(Mutex::new(gate_rx)),
            started: Some(Mutex::new(started_tx)),
            ..Fake::new()
        };
        (fake, started_rx, gate_tx)
    }
}

impl Linter for Fake {
    fn lint(&self, request: &Request) -> Result<BTreeMap<String, Vec<Alert>>, ValeError> {
        self.runs.fetch_add(1, Ordering::SeqCst);
        if let Some(started) = &self.started {
            started.lock().unwrap().send(()).unwrap();
        }
        if let Some(gate) = &self.gate {
            gate.lock()
                .unwrap()
                .recv_timeout(support::TIMEOUT)
                .expect("the test lets the run go");
        }
        if self.fail {
            return Err(ValeError::NotRun {
                command: request.command.clone(),
                reason: "not found".to_owned(),
            });
        }
        Ok(request
            .files
            .iter()
            .map(|(path, text)| {
                let mut alerts = Vec::new();
                for (n, line) in text.lines().enumerate() {
                    for (i, _) in line.match_indices("teh") {
                        let first = line[..i].chars().count() + 1;
                        alerts.push(Alert {
                            check: "Test.Typos".to_owned(),
                            message: "Use 'the' instead of 'teh'.".to_owned(),
                            severity: "warning".to_owned(),
                            line: n + 1,
                            span: (first, first + 2),
                            action: Action {
                                name: "replace".to_owned(),
                            },
                            suggestions: vec!["the".to_owned()],
                            ..Alert::default()
                        });
                    }
                }
                (path.clone(), alerts)
            })
            .collect())
    }
}

fn start(f: &Fixture, fake: Arc<Fake>) -> Client {
    Client::start_with(
        &f.root(),
        Setup {
            options: Options {
                linter: Some(fake),
                ..Options::default()
            },
            ..Setup::default()
        },
    )
}

fn prose_lines(client: &Client, path: &std::path::Path) -> Vec<u32> {
    let mut lines: Vec<u32> = client
        .diagnostics(path)
        .iter()
        .filter(|d| slug(d) == "prose")
        .map(|d| d.range.start.line)
        .collect();
    lines.sort_unstable();
    lines
}

#[test]
fn an_open_page_s_prose_is_published_with_its_other_diagnostics() {
    let f = Fixture::new(VALE_MODEL, &[("docs/a.md", PAGE)]);
    let a = f.path("docs/a.md");
    let fake = Arc::new(Fake::new());
    let mut client = start(&f, fake.clone());
    client.open(&a, 1, PAGE);
    client.settle();

    assert_eq!(prose_lines(&client, &a), [3, 5]);
    assert!(
        client
            .diagnostics(&a)
            .iter()
            .any(|d| slug(d) == "link-target-missing"),
        "{:?}",
        client.codes(&a)
    );
    let alert = client
        .diagnostics(&a)
        .into_iter()
        .find(|d| slug(d) == "prose")
        .unwrap();
    assert_eq!(alert.message, "Test.Typos: Use 'the' instead of 'teh'.");
    assert_eq!((alert.range.start.character, alert.range.end.character), (0, 3));
    let data = alert.data.unwrap();
    assert_eq!(data["next"], "fix");
    assert_eq!(data["rule"], "Test.Typos");
    assert_eq!(data["fixes"][0]["edits"][0]["newText"], "the");
    assert_eq!(fake.runs.load(Ordering::SeqCst), 1);
}

#[test]
fn an_edit_drops_the_alerts_from_where_it_starts_and_a_save_checks_again() {
    let f = Fixture::new(VALE_MODEL, &[("docs/a.md", PAGE)]);
    let a = f.path("docs/a.md");
    let fake = Arc::new(Fake::new());
    let mut client = start(&f, fake.clone());
    client.open(&a, 1, PAGE);
    client.settle();
    assert_eq!(prose_lines(&client, &a), [3, 5]);

    // A blank line inserted after the first alert's line moves the second.
    client.change(&a, 2, vec![edit((4, 0), (4, 0), "\n")]);
    client.settle();
    assert_eq!(prose_lines(&client, &a), [3]);
    assert!(
        client
            .diagnostics(&a)
            .iter()
            .any(|d| slug(d) == "link-target-missing")
    );
    assert_eq!(fake.runs.load(Ordering::SeqCst), 1, "typing doesn't run Vale");

    client.save(&a);
    client.settle();
    assert_eq!(prose_lines(&client, &a), [3, 6]);
    assert_eq!(fake.runs.load(Ordering::SeqCst), 2);
}

#[test]
fn a_slow_vale_delays_nothing_else_and_a_result_for_an_older_text_is_dropped() {
    let f = Fixture::new(VALE_MODEL, &[("docs/a.md", PAGE)]);
    let a = f.path("docs/a.md");
    let (fake, started, gate) = Fake::held();
    let fake = Arc::new(fake);
    let mut client = start(&f, fake.clone());
    client.open(&a, 1, PAGE);
    started
        .recv_timeout(support::TIMEOUT)
        .expect("Vale started");

    // While Vale is held, requests are answered and the other diagnostics
    // are published.
    let response = client.request(
        "textDocument/semanticTokens/full",
        serde_json::json!({ "textDocument": { "uri": support::uri(&a).as_str() } }),
    );
    response.response_result.expect("an answer");
    let fixed = "---\ntitle: A\n---\nteh first\n\nteh second\n";
    client.replace(&a, 2, fixed);
    let deadline = std::time::Instant::now() + support::TIMEOUT;
    while client
        .publications(&a)
        .last()
        .is_none_or(|p| p.version != Some(2))
    {
        assert!(std::time::Instant::now() < deadline, "nothing published");
        client.pump();
    }
    assert!(client.codes(&a).is_empty(), "{:?}", client.codes(&a));

    // The run was for version 1: what it found is dropped.
    gate.send(()).unwrap();
    client.settle();
    assert!(client.codes(&a).is_empty(), "{:?}", client.codes(&a));

    client.save(&a);
    started
        .recv_timeout(support::TIMEOUT)
        .expect("Vale started again");
    gate.send(()).unwrap();
    client.settle();
    assert_eq!(prose_lines(&client, &a), [3, 5]);
}

#[test]
fn a_vale_that_can_t_run_is_one_advice_on_ascribe_toml() {
    let f = Fixture::new(VALE_MODEL, &[("docs/a.md", PAGE)]);
    let a = f.path("docs/a.md");
    let model = f.path("ascribe.toml");
    let fake = Arc::new(Fake {
        fail: true,
        ..Fake::new()
    });
    let mut client = start(&f, fake);
    client.open(&a, 1, PAGE);
    client.settle();
    assert_eq!(client.codes(&model), ["prose-not-checked"]);
    let advice = &client.diagnostics(&model)[0];
    assert_eq!(advice.range.start.line, 5, "on `[checks.vale]`");
    assert!(prose_lines(&client, &a).is_empty());
}

#[test]
fn without_checks_vale_nothing_runs() {
    let f = Fixture::new(support::MODEL, &[("docs/a.md", PAGE)]);
    let a = f.path("docs/a.md");
    let fake = Arc::new(Fake::new());
    let mut client = start(&f, fake.clone());
    client.open(&a, 1, PAGE);
    client.save(&a);
    client.settle();
    assert_eq!(fake.runs.load(Ordering::SeqCst), 0);
    assert!(prose_lines(&client, &a).is_empty());
}

#[test]
fn closing_a_page_clears_its_alerts() {
    let f = Fixture::new(VALE_MODEL, &[("docs/a.md", PAGE)]);
    let a = f.path("docs/a.md");
    let mut client = start(&f, Arc::new(Fake::new()));
    client.open(&a, 1, PAGE);
    client.settle();
    assert_eq!(prose_lines(&client, &a), [3, 5]);
    client.close(&a);
    client.settle();
    assert!(prose_lines(&client, &a).is_empty());
    assert_eq!(client.codes(&a), ["link-target-missing"]);
}
