//! An address that answers with a server error is asked about again
//! (`ascribe_check::links::check`), against a local server that answers 500
//! and then 200: the second answer is the one that counts. When asking
//! again fails, the first answer stands, and isn't said to be the second.

#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use ascribe_check::Project;
use ascribe_check::links::{self, Answer, LinkChecker, Request};
use ascribe_check::tool::ToolError;
use ascribe_core::{FileId, RelPath};

/// A server on a port of its own that answers each request with the next of
/// `statuses`, and closes after the last.
fn server(statuses: &'static [&'static str]) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let port = listener.local_addr().expect("an address").port();
    thread::spawn(move || {
        for status in statuses {
            let (mut stream, _) = listener.accept().expect("a request");
            let mut reader = BufReader::new(stream.try_clone().expect("a stream"));
            let mut line = String::new();
            while reader.read_line(&mut line).expect("a line") > 2 {
                line.clear();
            }
            let response =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            stream.write_all(response.as_bytes()).expect("answered");
        }
    });
    port
}

/// Asks with a plain `GET`, as lychee would, and says what the server
/// answered: an address that answers `2xx` is fine, and is left out.
#[derive(Default)]
struct Asker;

impl LinkChecker for Asker {
    fn check(&self, request: &Request) -> Result<BTreeMap<String, Answer>, ToolError> {
        let mut out = BTreeMap::new();
        for url in &request.urls {
            let rest = url.strip_prefix("http://").expect("an http address");
            let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
            let mut stream = TcpStream::connect(authority).expect("connected");
            write!(
                stream,
                "GET /{path} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n"
            )
            .expect("asked");
            let mut answer = String::new();
            stream.read_to_string(&mut answer).expect("an answer");
            let status = answer
                .lines()
                .next()
                .and_then(|l| l.split_once(' '))
                .map(|(_, status)| status.trim().to_owned())
                .expect("a status line");
            if !status.starts_with('2') {
                out.insert(url.clone(), Answer::Broken { status });
            }
        }
        Ok(out)
    }
}

/// Asks like [`Asker`] the first time, and fails to run every time after.
#[derive(Default)]
struct FailsAgain {
    runs: AtomicUsize,
}

impl LinkChecker for FailsAgain {
    fn check(&self, request: &Request) -> Result<BTreeMap<String, Answer>, ToolError> {
        if self.runs.fetch_add(1, Ordering::Relaxed) > 0 {
            return Err(ToolError::Failed {
                command: "lychee".to_owned(),
                reason: "it stopped".to_owned(),
            });
        }
        Asker.check(request)
    }
}

fn project(page: String) -> Project {
    let text = "spec = \"0.1\"\n";
    let model = ascribe_model::load_str(text, FileId::new(0)).expect("a model");
    let sources = Vec::<(ascribe_core::RelPath, String)>::from_iter([(
        RelPath::parse("index.md").expect("a path"),
        page,
    )]);
    Project::new(
        ascribe_check::in_memory(
            std::env::temp_dir(),
            RelPath::root(),
            model,
            text.to_owned(),
            sources,
        )
        .snapshot(),
    )
}

fn check(project: &Project) -> links::Checked {
    check_with(project, &Asker)
}

fn check_with(project: &Project, checker: &dyn LinkChecker) -> links::Checked {
    links::check(
        project,
        checker,
        Duration::from_secs(5),
        Duration::from_millis(10),
        None,
        None,
    )
    .expect("it runs")
}

#[test]
fn a_server_error_that_passes_isn_t_reported() {
    let port = server(&["500 Internal Server Error", "200 OK"]);
    let project = project(format!(
        "# Links\n\nSee the [tools](http://127.0.0.1:{port}/tools).\n"
    ));
    let found = check(&project);
    assert_eq!(found.asked, 1);
    assert!(found.diagnostics.is_empty(), "{:?}", found.diagnostics);
}

#[test]
fn a_server_error_that_persists_is_reported_as_the_server_s() {
    let port = server(&["500 Internal Server Error", "502 Bad Gateway"]);
    let project = project(format!(
        "# Links\n\nSee the [tools](http://127.0.0.1:{port}/tools).\n"
    ));
    let found = check(&project);
    let [d] = found.diagnostics.as_slice() else {
        panic!("one diagnostic: {:?}", found.diagnostics);
    };
    assert_eq!(
        d.message,
        format!(
            "`http://127.0.0.1:{port}/tools` answered 502 Bad Gateway when asked twice: the server failed, which may be temporary"
        )
    );
}

#[test]
fn a_server_error_asked_about_once_isn_t_said_to_be_asked_twice() {
    // Asking again fails to run, so the first answer stands.
    let port = server(&["503 Service Unavailable"]);
    let project = project(format!(
        "# Links\n\nSee the [tools](http://127.0.0.1:{port}/tools).\n"
    ));
    let checker = FailsAgain::default();
    let found = check_with(&project, &checker);
    assert_eq!(
        checker.runs.load(Ordering::Relaxed),
        2,
        "it tried to ask again"
    );
    let [d] = found.diagnostics.as_slice() else {
        panic!("one diagnostic: {:?}", found.diagnostics);
    };
    assert_eq!(
        d.message,
        format!(
            "`http://127.0.0.1:{port}/tools` answered 503 Service Unavailable: the server failed, which may be temporary"
        )
    );
}

#[test]
fn a_page_that_s_gone_is_asked_about_once() {
    // A second request would find nothing listening.
    let port = server(&["404 Not Found"]);
    let project = project(format!(
        "# Links\n\nSee the [tools](http://127.0.0.1:{port}/tools).\n"
    ));
    let found = check(&project);
    let [d] = found.diagnostics.as_slice() else {
        panic!("one diagnostic: {:?}", found.diagnostics);
    };
    assert!(
        d.message
            .ends_with("answered 404 Not Found, so the link is broken for readers"),
        "{}",
        d.message
    );
}
