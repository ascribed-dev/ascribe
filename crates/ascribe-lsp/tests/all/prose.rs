//! Vale on open and on save (`[checks.vale]`), with a stand-in for Vale: its
//! alerts are published with the file's other diagnostics, an edit drops the
//! ones it may have moved, a result for an older text is dropped, and a slow
//! Vale delays nothing else.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use ascribe_lsp::Options;
use ascribe_report::prose::{Action, Alert, Linter, Request, ValeError};

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
                                ..Action::default()
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
    assert_eq!(
        (alert.range.start.character, alert.range.end.character),
        (0, 3)
    );
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
    assert_eq!(
        fake.runs.load(Ordering::SeqCst),
        1,
        "typing doesn't run Vale"
    );

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
    let model_text = format!("{VALE_MODEL}in-check = true\n");
    let f = Fixture::new(&model_text, &[("docs/a.md", PAGE)]);
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
fn a_vale_that_can_t_run_is_said_only_with_in_check() {
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
    assert!(
        client.codes(&model).is_empty(),
        "{:?}",
        client.codes(&model)
    );
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

/// A model whose `[checks.vale] command` is `command`.
/// With `in-check`, so a Vale that doesn't run is said.
fn model_running(command: &str) -> String {
    format!("{VALE_MODEL}command = \"{command}\"\nin-check = true\n")
}

/// Opens two pages and saves one, with a client set up to answer `ask`,
/// and returns how many times Vale ran, how many times the client was
/// asked, and the advice on `ascribe.toml`.
fn open_and_save(command: &str, ask: Option<Option<&'static str>>) -> (usize, usize, Vec<String>) {
    let f = Fixture::new(
        &model_running(command),
        &[("docs/a.md", PAGE), ("docs/b.md", PAGE)],
    );
    let (a, b) = (f.path("docs/a.md"), f.path("docs/b.md"));
    let fake = Arc::new(Fake::new());
    let mut client = Client::start_with(
        &f.root(),
        Setup {
            options: Options {
                linter: Some(fake.clone()),
                ..Options::default()
            },
            ask,
            ..Setup::default()
        },
    );
    client.open(&a, 1, PAGE);
    client.open(&b, 1, PAGE);
    client.settle();
    client.save(&a);
    client.settle();
    let asked = client
        .requests_from_server
        .iter()
        .filter(|r| r.method == "window/showMessageRequest")
        .count();
    let advice = client
        .diagnostics(&f.path("ascribe.toml"))
        .into_iter()
        .map(|d| d.message)
        .collect();
    (fake.runs.load(Ordering::SeqCst), asked, advice)
}

#[test]
fn a_program_in_the_project_runs_once_the_person_says_so_asked_once_a_session() {
    let (runs, asked, advice) = open_and_save("tools/vale", Some(Some("Run it")));
    assert_eq!(asked, 1);
    assert!(runs >= 2, "{runs}");
    assert!(advice.is_empty(), "{advice:?}");
}

#[test]
fn a_program_in_the_project_doesnt_run_when_the_person_says_no() {
    for answer in [Some("Don't run it"), None] {
        let (runs, asked, advice) = open_and_save("tools/vale", Some(answer));
        assert_eq!((runs, asked), (0, 1), "{answer:?}");
        assert_eq!(advice.len(), 1, "{advice:?}");
        assert!(
            advice[0].contains(
                "the prose wasn't checked: running `tools/vale`, a program in the project, \
                 wasn't allowed. `ascribe check --vale` checks it"
            ),
            "{advice:?}"
        );
    }
}

#[test]
fn a_program_in_the_project_doesnt_run_when_the_client_cant_ask() {
    let (runs, asked, advice) = open_and_save("tools/vale", None);
    assert_eq!((runs, asked), (0, 0));
    assert!(
        advice.iter().any(|a| a.contains(
            "the editor can't ask whether to run `tools/vale`, a program in the project"
        )),
        "{advice:?}"
    );
}

#[test]
fn a_command_found_on_the_path_runs_without_asking() {
    for ask in [None, Some(Some("Run it"))] {
        let (runs, asked, advice) = open_and_save("vale", ask);
        assert_eq!(asked, 0, "{ask:?}");
        assert!(runs >= 2, "{ask:?}: {runs}");
        assert!(advice.is_empty(), "{advice:?}");
    }
}
