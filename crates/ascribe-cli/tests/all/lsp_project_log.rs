//! `ascribe lsp` names the project it loaded on standard error: the nearest
//! `ascribe.toml` at or above its workspace folder, even when that folder is
//! a project nested inside another.

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

fn send(stdin: &mut impl Write, message: &Value) {
    let body = serde_json::to_vec(message).expect("json");
    write!(stdin, "Content-Length: {}\r\n\r\n", body.len()).expect("write");
    stdin.write_all(&body).expect("write");
    stdin.flush().expect("flush");
}

#[test]
fn the_server_logs_the_nearest_project() {
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
    // The protocol's side isn't read here; drain it so the server never blocks.
    let mut stdout = child.stdout.take().expect("stdout");
    thread::spawn(move || {
        let _ = std::io::copy(&mut stdout, &mut std::io::sink());
    });
    let (tx, lines) = mpsc::channel();
    let stderr = child.stderr.take().expect("stderr");
    thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            let Ok(line) = line else { break };
            if tx.send(line).is_err() {
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
    send(
        &mut stdin,
        &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }),
    );

    let logged = loop {
        let line = lines
            .recv_timeout(TIMEOUT)
            .expect("the server logs the project it uses");
        if let Some((_, path)) = line.split_once("using the project at ") {
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
