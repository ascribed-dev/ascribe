//! `ascribe preview`, by running the binary in a project `ascribe init`
//! started: the build rendered into `.ascribe/preview/` and served at
//! `127.0.0.1`, a page with errors rendered as far as it goes, the problems
//! printed as `ascribe check` prints them, and the build rendered again
//! when a page changes. What a page renders to is the archive's renderer,
//! which `crates/ascribe-emit/tests/` holds.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// How long the preview may take to start, or to render again.
const WAIT: Duration = Duration::from_secs(60);

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir.path())
        .arg("init")
        .stdout(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
    page(
        dir.path(),
        "index.md",
        "Welcome",
        "Hello. See [setup](setup.md).",
    );
    dir
}

fn page(root: &Path, path: &str, title: &str, body: &str) {
    fs::write(
        root.join("docs").join(path),
        format!("---\ntype: page\ntitle: {title}\ndescription: A page.\n---\n\n{body}\n"),
    )
    .unwrap();
}

/// A running preview, stopped when it's dropped, and the lines it prints.
struct Preview {
    child: Child,
    lines: mpsc::Receiver<String>,
    printed: String,
}

impl Preview {
    fn start(root: &Path, args: &[&str]) -> Preview {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ascribe"))
            .current_dir(root)
            .arg("preview")
            .args(["--port", "0"])
            .args(args)
            .env("NO_COLOR", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (send, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if send.send(line).is_err() {
                    break;
                }
            }
        });
        Preview {
            child,
            lines,
            printed: String::new(),
        }
    }

    /// Reads what it prints until a line starts with `start`, and returns
    /// that line.
    fn until(&mut self, start: &str) -> String {
        let deadline = Instant::now() + WAIT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.lines.recv_timeout(left) {
                Ok(line) => {
                    self.printed.push_str(&line);
                    self.printed.push('\n');
                    if line.starts_with(start) {
                        return line;
                    }
                }
                Err(_) => panic!("no line starting {start:?}; printed:\n{}", self.printed),
            }
        }
    }

    /// The address it serves at, from the line that says so.
    fn address(&mut self) -> String {
        let line = self.until("previewing build ");
        let at = line.find("http://").unwrap();
        line[at + "http://".len()..]
            .trim_end_matches('/')
            .to_owned()
    }
}

impl Drop for Preview {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The response to `GET path` at `address`: its status line and body.
fn get(address: &str, path: &str) -> (String, String) {
    let mut stream = TcpStream::connect(address).unwrap();
    write!(stream, "GET {path} HTTP/1.1\r\nHost: {address}\r\n\r\n").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let (head, body) = response.split_once("\r\n\r\n").unwrap();
    (head.lines().next().unwrap().to_owned(), body.to_owned())
}

#[test]
fn a_fresh_project_is_served_with_its_problems_printed_and_rendered_again_on_a_change() {
    let dir = project();
    let mut preview = Preview::start(dir.path(), &[]);
    let address = preview.address();
    // The problem, as `ascribe check` prints it, then what was rendered.
    assert!(
        preview
            .printed
            .contains("[ASC036] Error: `setup.md` doesn't exist"),
        "{}",
        preview.printed
    );
    assert!(
        preview.printed.contains(
            "checked 1 file for every build, reported on 1: 1 error, 0 warnings, 0 advice\n\
             rendered build `site`: 1 page, 0 assets, in .ascribe/preview\n"
        ),
        "{}",
        preview.printed
    );
    // The page with the error is rendered, its broken link as its text.
    let (status, body) = get(&address, "/");
    assert_eq!(status, "HTTP/1.1 200 OK");
    assert!(
        body.contains("<title>Welcome · site preview</title>"),
        "{body}"
    );
    assert!(body.contains("<p>Hello. See setup.</p>"), "{body}");
    assert!(
        body.contains("A preview of build <code>site</code>"),
        "{body}"
    );
    assert!(dir.path().join(".ascribe/preview/index.html").is_file());
    let (status, _) = get(&address, "/style.css");
    assert_eq!(status, "HTTP/1.1 200 OK");

    // A new page fixes the link, and the build is rendered again.
    page(dir.path(), "setup.md", "Setup", "Set it up.");
    preview.until("checked 2 files for every build, reported on 2: 0 errors");
    preview.until("rendered build `site`: 2 pages");
    let (status, body) = get(&address, "/setup.html");
    assert_eq!(status, "HTTP/1.1 200 OK");
    assert!(body.contains("<p>Set it up.</p>"), "{body}");
    let (_, body) = get(&address, "/index.html");
    assert!(
        body.contains("<a href=\"./setup.html\">setup</a>"),
        "{body}"
    );
}

#[test]
fn a_build_that_isnt_the_models_stops_it_before_it_serves() {
    let dir = project();
    let output = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir.path())
        .args(["preview", "--port", "0", "--build", "nope"])
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("nope"), "{stderr}");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("previewing"));
}
