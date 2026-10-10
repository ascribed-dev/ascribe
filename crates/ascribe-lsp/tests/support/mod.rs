//! A scripted LSP client over an in-memory connection.

#![allow(dead_code, clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ascribe_lsp::{Exit, Options, serve};
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    Diagnostic, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, FileChangeType, FileEvent, Position,
    PositionEncodingKind, PublishDiagnosticsParams, Range, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
};
use serde_json::json;

pub const TIMEOUT: Duration = Duration::from_secs(20);

pub struct Client {
    conn: Connection,
    server: Option<JoinHandle<Exit>>,
    pub idle: Arc<AtomicBool>,
    next_id: i32,
    inbox: VecDeque<Message>,
    /// Every `publishDiagnostics`, in order.
    pub log: Vec<PublishDiagnosticsParams>,
    /// The last one for each URI.
    pub latest: BTreeMap<String, PublishDiagnosticsParams>,
    pub initialize_result: serde_json::Value,
    pub requests_from_server: Vec<Request>,
}

pub struct Setup {
    pub encodings: Option<Vec<PositionEncodingKind>>,
    pub watch: bool,
    pub options: Options,
}

impl Default for Setup {
    fn default() -> Setup {
        Setup {
            encodings: None,
            watch: true,
            options: Options::default(),
        }
    }
}

/// The real path of `path`, with symlinks resolved. On Windows, `canonicalize`
/// writes a drive path as `\\?\C:\dir`; clients send `C:\dir`, so that's the
/// form returned.
pub fn real_path(path: &Path) -> PathBuf {
    let real = path.canonicalize().expect("a real path");
    let text = real.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(drive) if drive.as_bytes().get(1) == Some(&b':') => PathBuf::from(drive),
        _ => real,
    }
}

pub fn uri(path: &Path) -> Uri {
    let path = path.to_string_lossy().replace('\\', "/");
    let mut text = String::from("file://");
    // A drive path, `C:/dir`, follows a third slash.
    if !path.starts_with('/') {
        text.push('/');
    }
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~!$&'()*+,;=:@".contains(&byte) {
            text.push(byte as char);
        } else {
            text.push_str(&format!("%{byte:02X}"));
        }
    }
    Uri::from_str(&text).expect("a uri")
}

impl Client {
    pub fn start(root: &Path) -> Client {
        Client::start_with(root, Setup::default())
    }

    pub fn start_with(root: &Path, setup: Setup) -> Client {
        let (client_side, server_side) = Connection::memory();
        let idle = Arc::new(AtomicBool::new(false));
        let mut options = setup.options;
        options.idle = Some(idle.clone());
        let server = thread::spawn(move || serve(server_side, options).expect("the server runs"));
        let mut general = json!({});
        if let Some(encodings) = &setup.encodings {
            general["positionEncodings"] = json!(encodings);
        }
        let params = json!({
            "processId": null,
            "rootUri": null,
            "workspaceFolders": [{ "uri": uri(root).as_str(), "name": "w" }],
            "capabilities": {
                "general": general,
                "workspace": { "didChangeWatchedFiles": { "dynamicRegistration": setup.watch } },
                "window": { "showDocument": { "support": true } },
            },
        });
        let mut client = Client {
            conn: client_side,
            server: Some(server),
            idle,
            next_id: 0,
            inbox: VecDeque::new(),
            log: Vec::new(),
            latest: BTreeMap::new(),
            initialize_result: json!(null),
            requests_from_server: Vec::new(),
        };
        let response = client.request("initialize", params);
        client.initialize_result = response.response_result.expect("initialize succeeds");
        client.notify("initialized", json!({}));
        client
    }

    fn take(&mut self, message: Message) -> Option<Response> {
        match message {
            Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => {
                let params: PublishDiagnosticsParams =
                    serde_json::from_value(n.params).expect("publish params");
                self.latest
                    .insert(params.uri.as_str().to_owned(), params.clone());
                self.log.push(params);
                None
            }
            Message::Notification(_) => None,
            Message::Request(request) => {
                // registerCapability and friends: accept.
                let id = request.id.clone();
                self.requests_from_server.push(request);
                self.conn
                    .sender
                    .send(Response::new_ok(id, serde_json::Value::Null).into())
                    .expect("send");
                None
            }
            Message::Response(response) => Some(response),
        }
    }

    pub fn notify(&self, method: &str, params: serde_json::Value) {
        self.conn
            .sender
            .send(Notification::new(method.to_owned(), params).into())
            .expect("send");
    }

    pub fn request(&mut self, method: &str, params: serde_json::Value) -> Response {
        self.next_id += 1;
        let id = RequestId::from(1000 + self.next_id);
        self.conn
            .sender
            .send(Request::new(id.clone(), method.to_owned(), params).into())
            .expect("send");
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let message = self
                .conn
                .receiver
                .recv_timeout(left)
                .unwrap_or_else(|_| panic!("no response to {method}"));
            if let Some(response) = self.take(message)
                && response.id == id
            {
                return response;
            }
        }
    }

    /// Reads whatever the server has sent, without waiting.
    pub fn pump(&mut self) {
        while let Ok(message) = self.conn.receiver.try_recv() {
            self.take(message);
        }
    }

    /// Waits until every message sent so far has been handled and every
    /// diagnostic it caused is published.
    pub fn settle(&mut self) {
        // A request is answered after every notification before it is handled.
        let _ = self.request(
            "textDocument/semanticTokens/full",
            json!({ "textDocument": { "uri": "untitled:settle" } }),
        );
        let deadline = Instant::now() + TIMEOUT;
        while !self.idle.load(Ordering::SeqCst) {
            assert!(Instant::now() < deadline, "the server never went idle");
            if let Ok(message) = self.conn.receiver.recv_timeout(Duration::from_millis(5)) {
                self.take(message);
            }
        }
        // Publications sent just before it went idle.
        thread::sleep(Duration::from_millis(5));
        self.pump();
    }

    pub fn open(&mut self, path: &Path, version: i32, text: &str) {
        self.notify(
            "textDocument/didOpen",
            serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri(path),
                    language_id: "ascribe".into(),
                    version,
                    text: text.into(),
                },
            })
            .expect("params"),
        );
    }

    pub fn change(
        &mut self,
        path: &Path,
        version: i32,
        edits: Vec<TextDocumentContentChangeEvent>,
    ) {
        self.notify(
            "textDocument/didChange",
            serde_json::to_value(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri(path),
                    version,
                },
                content_changes: edits,
            })
            .expect("params"),
        );
    }

    pub fn replace(&mut self, path: &Path, version: i32, text: &str) {
        self.change(
            path,
            version,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.into(),
            }],
        );
    }

    pub fn save(&mut self, path: &Path) {
        self.notify(
            "textDocument/didSave",
            serde_json::json!({ "textDocument": { "uri": uri(path).as_str() } }),
        );
    }

    pub fn close(&mut self, path: &Path) {
        self.notify(
            "textDocument/didClose",
            serde_json::to_value(DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri: uri(path) },
            })
            .expect("params"),
        );
    }

    pub fn watched(&mut self, events: &[(&Path, FileChangeType)]) {
        self.notify(
            "workspace/didChangeWatchedFiles",
            serde_json::to_value(DidChangeWatchedFilesParams {
                changes: events
                    .iter()
                    .map(|(path, typ)| FileEvent {
                        uri: uri(path),
                        typ: *typ,
                    })
                    .collect(),
            })
            .expect("params"),
        );
    }

    /// The diagnostics last published for a file (empty when none were).
    pub fn diagnostics(&self, path: &Path) -> Vec<Diagnostic> {
        self.latest
            .get(uri(path).as_str())
            .map(|p| p.diagnostics.clone())
            .unwrap_or_default()
    }

    /// The codes of the diagnostics last published for a file.
    pub fn codes(&self, path: &Path) -> Vec<String> {
        let mut codes: Vec<String> = self.diagnostics(path).iter().map(slug).collect();
        codes.sort();
        codes
    }

    /// Every publication for a file, in order.
    pub fn publications(&self, path: &Path) -> Vec<&PublishDiagnosticsParams> {
        let want = uri(path);
        self.log.iter().filter(|p| p.uri == want).collect()
    }

    /// `exit` with no `shutdown` first.
    pub fn exit_without_shutdown(mut self) -> Exit {
        self.notify("exit", json!(null));
        self.server
            .take()
            .expect("running")
            .join()
            .expect("the server thread ends")
    }

    pub fn shutdown(mut self) -> Exit {
        let response = self.request("shutdown", json!(null));
        assert!(response.response_result.is_ok());
        self.notify("exit", json!(null));
        self.server
            .take()
            .expect("running")
            .join()
            .expect("the server thread ends")
    }
}

/// The registry slug carried in a diagnostic's data.
pub fn slug(d: &Diagnostic) -> String {
    d.data
        .as_ref()
        .and_then(|data| data["slug"].as_str())
        .expect("a slug")
        .to_owned()
}

pub fn edit(a: (u32, u32), b: (u32, u32), text: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(a.0, a.1), Position::new(b.0, b.1))),
        range_length: None,
        text: text.into(),
    }
}

/// A project in a temporary directory.
pub struct Fixture {
    pub dir: tempfile::TempDir,
}

/// A content model with the content checks across the project off: most
/// tests are about other things, and `tests/all/across.rs` covers them.
pub const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[checks]\npage-orphan = \"off\"\nfragment-unused = \"off\"\nphrase-unused = \"off\"\nfeature-unused = \"off\"\nglossary-term-unused = \"off\"\ntitle-duplicate = \"off\"\n";

impl Fixture {
    pub fn new(model: &str, files: &[(&str, &str)]) -> Fixture {
        let dir = tempfile::tempdir().expect("a temp dir");
        let f = Fixture { dir };
        f.write("ascribe.toml", model);
        for (path, text) in files {
            f.write(path, text);
        }
        f
    }

    /// The project root, with symlinks resolved (the server normalizes paths
    /// lexically, so tests use the real path everywhere).
    pub fn root(&self) -> PathBuf {
        real_path(self.dir.path())
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        // A relative path is written with `/`; the paths the server reports
        // use the platform's separator.
        self.root()
            .join(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
    }

    pub fn write(&self, rel: &str, text: &str) {
        let path = self.dir.path().join(rel);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        std::fs::write(path, text).expect("write");
    }

    pub fn remove(&self, rel: &str) {
        std::fs::remove_file(self.dir.path().join(rel)).expect("remove");
    }
}
