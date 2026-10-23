//! `ascribe lsp` names the project it loaded, with its version, in a
//! `window/logMessage` at Info: the nearest `ascribe.toml` at or above its
//! workspace folder, even when that folder is a project nested inside another.
//! The version, there and in `serverInfo`, is the one `ascribe --version`
//! prints, `+dev` and commit included.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(30);

/// The `file:` URI of an absolute path: `file:///C:/dir` for a drive path.
fn file_uri(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let text = text.strip_prefix("//?/").unwrap_or(&text);
    if text.starts_with('/') {
        format!("file://{text}")
    } else {
        format!("file:///{text}")
    }
}

/// One message from the server's standard output.
fn read_frame(reader: &mut impl BufRead) -> Option<Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        length = Some(
            line.strip_prefix("Content-Length: ")?
                .parse::<usize>()
                .ok()?,
        );
    }
    let mut body = vec![0; length?];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

fn send(stdin: &mut impl Write, message: &Value) {
    let body = serde_json::to_vec(message).expect("json");
    write!(stdin, "Content-Length: {}\r\n\r\n", body.len()).expect("write");
    stdin.write_all(&body).expect("write");
    stdin.flush().expect("flush");
}

/// What `ascribe --version` prints after the name, such as
/// `0.4.1+dev (7ae1340440fc)`.
fn full_version() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .arg("--version")
        .output()
        .expect("run ascribe --version");
    let text = String::from_utf8(output.stdout).expect("utf-8");
    text.trim()
        .strip_prefix("ascribe ")
        .expect("the name, then the version")
        .to_owned()
}

#[test]
fn the_server_logs_the_nearest_project() {
    let version = full_version();
    let dir = tempfile::tempdir().expect("temp dir");
    let outer = dir.path().join("outer");
    let inner = outer.join("pages").join("inner");
    for project in [&outer, &inner] {
        fs::create_dir_all(project.join("docs")).expect("mkdir");
        fs::write(
            project.join("ascribe.toml"),
            "[project]\ncontent-root = \"docs\"\n",
        )
        .expect("write model");
    }

    let mut child = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .arg("lsp")
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start ascribe lsp");
    let mut stdin = child.stdin.take().expect("stdin");
    // Read every message, so the server never blocks, and pass on its log.
    let stdout = child.stdout.take().expect("stdout");
    let (tx, logged) = mpsc::channel();
    let (init_tx, initialized) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        while let Some(message) = read_frame(&mut reader) {
            if message["id"] == 1 {
                let _ = init_tx.send(message["result"].clone());
            } else if message["method"] == "window/logMessage"
                && tx.send(message["params"].clone()).is_err()
            {
                break;
            }
        }
    });

    send(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "processId": null,
                "workspaceFolders": [{ "uri": file_uri(&inner), "name": "inner" }],
                "capabilities": {},
            },
        }),
    );
    let result = initialized
        .recv_timeout(TIMEOUT)
        .expect("the server answers initialize");
    assert_eq!(result["serverInfo"]["name"], "ascribe", "{result}");
    assert_eq!(
        result["serverInfo"]["version"],
        version.as_str(),
        "{result}"
    );
    send(
        &mut stdin,
        &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }),
    );

    let logged = loop {
        let params = logged
            .recv_timeout(TIMEOUT)
            .expect("the server logs the project it uses");
        let message = params["message"].as_str().unwrap_or_default();
        if let Some((_, path)) = message.split_once("using the project at ") {
            // Info, since nothing is wrong, and with the server's version.
            assert_eq!(params["type"], 3, "{params}");
            assert!(
                message.starts_with(&format!("ascribe-lsp {version}: ")),
                "{message}"
            );
            break path.to_owned();
        }
    };
    assert!(
        Path::new(&logged).ends_with(
            Path::new("outer")
                .join("pages")
                .join("inner")
                .join("ascribe.toml")
        ),
        "the nested project, not the one around it: {logged}"
    );

    send(
        &mut stdin,
        &json!({ "jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": null }),
    );
    send(&mut stdin, &json!({ "jsonrpc": "2.0", "method": "exit" }));
    drop(stdin);
    let status = child.wait().expect("the server exits");
    assert!(status.success(), "the server exits with 0: {status:?}");
}

#[test]
fn the_server_refuses_a_config_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .args(["lsp", "--config", "ascribe.toml"])
        .stdin(Stdio::null())
        .output()
        .expect("run ascribe lsp");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("doesn't take --config"), "stderr: {stderr}");
}
