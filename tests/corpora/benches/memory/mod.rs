//! Peak memory: the most resident memory a process held at once, as the
//! platform reports it for a process that has ended (`getrusage`'s
//! `ru_maxrss`, read through `/usr/bin/time`, since the workspace forbids the
//! `unsafe` a direct call needs). In megabytes (2^20 bytes), recorded as
//! `memory/` metrics. Skipped, with a message, where `/usr/bin/time` isn't
//! there or doesn't report it.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};
use tessera_synthetic::report::record_megabytes;
use tessera_synthetic::{CONTENT_ROOT, Synthetic, TYPING_LINE, TYPING_PREFIX};

const TIME: &str = "/usr/bin/time";

/// Runs of each measure. Peak memory varies far less than time between runs.
const RUNS: usize = 3;

/// How `/usr/bin/time` reports peak memory here.
#[derive(Clone, Copy)]
enum Flavor {
    /// GNU time: `-f %M -o FILE` writes kilobytes to a file.
    Gnu,
    /// BSD time (macOS): `-l` prints "maximum resident set size" in bytes to
    /// standard error.
    Bsd,
}

fn flavor() -> Option<Flavor> {
    if !Path::new(TIME).exists() {
        return None;
    }
    if cfg!(target_os = "macos") {
        Some(Flavor::Bsd)
    } else if cfg!(target_os = "linux") {
        Some(Flavor::Gnu)
    } else {
        None
    }
}

/// Measures peak memory. `None` (with a message printed once by
/// [`Meter::new`]) where it can't.
pub struct Meter {
    flavor: Flavor,
    report: PathBuf,
}

/// A process started under `/usr/bin/time`.
struct Measured {
    child: Child,
    /// Standard error, read as it comes so a chatty server can't fill the
    /// pipe and stall.
    stderr: std::thread::JoinHandle<String>,
    flavor: Flavor,
    report: PathBuf,
}

impl Measured {
    /// Waits for the process and returns its peak memory in megabytes.
    fn finish(mut self) -> f64 {
        let status = self.child.wait().expect("waits");
        let stderr = self.stderr.join().expect("reads standard error");
        // Exit code 1 is "the project has errors"; 2 is a failure to run.
        assert!(
            status.code().is_some_and(|c| c <= 1),
            "ascribe failed: {status}\n{stderr}"
        );
        let bytes = match self.flavor {
            Flavor::Gnu => {
                let text = std::fs::read_to_string(&self.report).expect("reads time's report");
                // The last line: GNU time puts "Command exited with non-zero
                // status 1" before it when it was.
                let kb: f64 = text
                    .lines()
                    .last()
                    .and_then(|l| l.trim().parse().ok())
                    .unwrap_or_else(|| panic!("time's report: {text:?}"));
                kb * 1024.0
            }
            Flavor::Bsd => stderr
                .lines()
                .find(|l| l.contains("maximum resident set size"))
                .and_then(|l| l.split_whitespace().next())
                .and_then(|n| n.parse().ok())
                .unwrap_or_else(|| panic!("time's report: {stderr:?}")),
        };
        bytes / (1024.0 * 1024.0)
    }
}

impl Meter {
    /// A meter writing its reports under `dir`, or `None` (and a message)
    /// where peak memory can't be measured.
    pub fn new(dir: &Path) -> Option<Meter> {
        let Some(flavor) = flavor() else {
            println!("peak memory: skipped ({TIME} isn't available)");
            println!();
            return None;
        };
        Some(Meter {
            flavor,
            report: dir.join("time-report.txt"),
        })
    }

    fn spawn(&self, bin: &Path, dir: &Path, args: &[&str], stdin: Stdio) -> Measured {
        let _ = std::fs::remove_file(&self.report);
        let mut command = Command::new(TIME);
        match self.flavor {
            Flavor::Gnu => {
                command.arg("-f").arg("%M").arg("-o").arg(&self.report);
            }
            Flavor::Bsd => {
                command.arg("-l");
            }
        }
        let mut child = command
            .arg(bin)
            .args(args)
            .current_dir(dir)
            .stdin(stdin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("runs ascribe under time");
        let mut pipe = child.stderr.take().expect("standard error");
        let stderr = std::thread::spawn(move || {
            let mut text = String::new();
            let _ = pipe.read_to_string(&mut text);
            text
        });
        Measured {
            child,
            stderr,
            flavor: self.flavor,
            report: self.report.clone(),
        }
    }

    /// Peak memory of one run of `ascribe args` in `dir`, its output thrown
    /// away.
    fn once(&self, bin: &Path, dir: &Path, args: &[&str]) -> f64 {
        let mut run = self.spawn(bin, dir, args, Stdio::null());
        if let Some(mut out) = run.child.stdout.take() {
            std::io::copy(&mut out, &mut std::io::sink()).expect("reads standard output");
        }
        run.finish()
    }

    /// Runs `ascribe args` in `dir` [`RUNS`] times (calling `before` ahead of
    /// each), prints the median peak, and records it as `metric`.
    pub fn cli(
        &self,
        name: &str,
        metric: &str,
        bin: &Path,
        dir: &Path,
        args: &[&str],
        before: impl Fn(),
    ) {
        let mut peaks: Vec<f64> = (0..RUNS)
            .map(|_| {
                before();
                self.once(bin, dir, args)
            })
            .collect();
        show(name, &mut peaks);
        record_megabytes(metric, &mut peaks);
    }

    /// The language server on the standard synthetic project at `root`: its
    /// peak memory once it has loaded the project and published the first
    /// diagnostics of an open page, and again after 100 edits to that page,
    /// each waited for. Each is a separate run of `ascribe lsp`, ended
    /// with `shutdown` and `exit`. Both are peaks, not the memory the server
    /// holds at the end: the second differs from the first only if editing
    /// went above the load peak.
    pub fn lsp(&self, bin: &Path, root: &Path) {
        for (name, metric, edits) in [
            (
                "  language server: loaded, first diagnostics",
                "memory/lsp-load-3000",
                0,
            ),
            (
                "  language server: after 100 edits",
                "memory/lsp-100-edits-3000",
                100,
            ),
        ] {
            let mut peaks: Vec<f64> = (0..RUNS).map(|_| self.lsp_once(bin, root, edits)).collect();
            show(name, &mut peaks);
            record_megabytes(metric, &mut peaks);
        }
    }

    fn lsp_once(&self, bin: &Path, root: &Path, edits: usize) -> f64 {
        let project = Synthetic::standard();
        let edited = project.pages / 2;
        let path = root.join(CONTENT_ROOT).join(project.page_path(edited));
        let uri = format!("file://{}", path.display());
        let text = project.page_text(edited, "typed: ");

        let mut run = self.spawn(bin, root, &["lsp"], Stdio::piped());
        let mut client = Client {
            input: run.child.stdin.take().expect("standard input"),
            output: BufReader::new(run.child.stdout.take().expect("standard output")),
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
        client.notify("initialized", json!({}));
        client.notify(
            "textDocument/didOpen",
            json!({ "textDocument": {
                "uri": uri, "languageId": "ascribe", "version": 1, "text": text,
            }}),
        );
        client.wait_publish(&uri, 1);
        let line_length = TYPING_PREFIX.len() + "typed: ".len();
        for k in 0..edits {
            let version = 2 + k as i64;
            let at = Value::from(line_length + k);
            let position = json!({ "line": TYPING_LINE, "character": at });
            client.notify(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": uri, "version": version },
                    "contentChanges": [{
                        "range": { "start": position, "end": position },
                        "text": "x",
                    }],
                }),
            );
            client.wait_publish(&uri, version);
        }
        client.request("shutdown", Value::Null);
        client.notify("exit", Value::Null);
        drop(client);
        run.finish()
    }
}

fn show(name: &str, peaks: &mut [f64]) {
    peaks.sort_by(f64::total_cmp);
    println!(
        "{name:<52} peak memory median {:>7.1} MB  min {:>7.1}  max {:>7.1}  ({} runs)",
        peaks[peaks.len() / 2],
        peaks[0],
        peaks[peaks.len() - 1],
        peaks.len()
    );
}

/// A scripted LSP client over a child's standard input and output.
struct Client {
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next: i64,
}

impl Client {
    fn send(&mut self, message: &Value) {
        let body = message.to_string();
        write!(self.input, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("writes");
        self.input.flush().expect("flushes");
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(&json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    fn receive(&mut self) -> Value {
        let mut length = None;
        loop {
            let mut line = String::new();
            let n = self.output.read_line(&mut line).expect("reads a header");
            assert!(n > 0, "the server closed its output");
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(v) = line.strip_prefix("Content-Length:") {
                length = Some(v.trim().parse::<usize>().expect("a length"));
            }
        }
        let mut body = vec![0; length.expect("a Content-Length header")];
        self.output.read_exact(&mut body).expect("reads a message");
        serde_json::from_slice(&body).expect("a JSON message")
    }

    /// Answers a request from the server with `null`; anything else is
    /// returned.
    fn next_message(&mut self) -> Value {
        loop {
            let message = self.receive();
            if message.get("method").is_some() && message.get("id").is_some() {
                let id = message["id"].clone();
                self.send(&json!({ "jsonrpc": "2.0", "id": id, "result": null }));
                continue;
            }
            return message;
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        let id = self.next;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let message = self.next_message();
            if message.get("method").is_none() && message["id"] == json!(id) {
                return message;
            }
        }
    }

    /// Reads until the diagnostics of `uri` at `version` are published.
    fn wait_publish(&mut self, uri: &str, version: i64) {
        loop {
            let message = self.next_message();
            if message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == uri
                && message["params"]["version"] == version
            {
                return;
            }
        }
    }
}
