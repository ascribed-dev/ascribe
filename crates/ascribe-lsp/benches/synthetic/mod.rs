//! The synthetic project both benchmarks run against: pages in thirty
//! directories, each with a title, headings, an include of a fragment, three
//! links to other pages, an image, and (one page in ten) an availability
//! marker. Shared by `keystroke.rs` and `completion.rs`.

#![allow(dead_code, clippy::unwrap_used)]

use std::fmt::Write as _;
use std::path::Path;
use std::time::Duration;

use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types::{PublishDiagnosticsParams, Uri};

pub const FRAGMENTS: usize = 100;
pub const IMAGES: usize = 60;

pub const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[phrases]
product = "Quill"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[builds.site]
variants = "switch"
availability = "badge"
"#;

pub fn page_path(i: usize) -> String {
    format!("s{}/p{i}.md", i % 30)
}

pub fn page_text(i: usize, pages: usize) -> String {
    let mut t = String::new();
    let _ = write!(
        t,
        "---\ntitle: Page {i}\n---\n\n# Page {i}\n\nIntro for {{product}}, typed: \n\n"
    );
    let _ = write!(
        t,
        "## Setup\n\nSteps for page {i}.\n\n## Usage\n\nMore.\n\n"
    );
    let _ = write!(t, "@include: /_f/f{}.md\n\n", i % FRAGMENTS);
    for k in 1..=3 {
        let target = (i * 7 + k * 131) % pages;
        if k == 1 {
            let _ = writeln!(t, "See [the setup](/{}#setup).", page_path(target));
        } else {
            let _ = writeln!(t, "See [page {target}](/{}).", page_path(target));
        }
    }
    let _ = write!(t, "\n![diagram](/img/i{}.png)\n", i % IMAGES);
    if i.is_multiple_of(10) {
        t.push_str("\n## Cloud only\n@available: cloud\n\nCloud text.\n");
    }
    t
}

pub fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Writes a project of `pages` pages, with its fragments and images, under
/// `root`.
pub fn write_project(root: &Path, pages: usize) {
    write(root, "ascribe.toml", MODEL);
    for i in 0..pages {
        write(
            root,
            &format!("docs/{}", page_path(i)),
            &page_text(i, pages),
        );
    }
    for i in 0..FRAGMENTS {
        write(
            root,
            &format!("docs/_f/f{i}.md"),
            &format!("## Shared {i}\n\nShared text.\n"),
        );
    }
    for i in 0..IMAGES {
        write(root, &format!("docs/img/i{i}.png"), "");
    }
}

/// A scripted client over an in-memory connection.
pub struct Client {
    pub conn: Connection,
    pub next: i32,
}

impl Client {
    pub fn request(&mut self, method: &str, params: serde_json::Value) -> Response {
        self.next += 1;
        let id = RequestId::from(self.next);
        self.conn
            .sender
            .send(Request::new(id.clone(), method.to_owned(), params).into())
            .unwrap();
        loop {
            match self.conn.receiver.recv().unwrap() {
                Message::Response(r) if r.id == id => return r,
                Message::Request(r) => {
                    self.conn
                        .sender
                        .send(Response::new_ok(r.id, serde_json::Value::Null).into())
                        .unwrap();
                }
                _ => {}
            }
        }
    }

    /// Reads until the publication for `uri` with `version` (any, when
    /// `None`) arrives.
    pub fn wait_publish(&mut self, uri: &Uri, version: Option<i32>) -> PublishDiagnosticsParams {
        loop {
            match self
                .conn
                .receiver
                .recv_timeout(Duration::from_secs(60))
                .unwrap()
            {
                Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => {
                    let p: PublishDiagnosticsParams = serde_json::from_value(n.params).unwrap();
                    if &p.uri == uri && (version.is_none() || p.version == version) {
                        return p;
                    }
                }
                Message::Request(r) => {
                    self.conn
                        .sender
                        .send(Response::new_ok(r.id, serde_json::Value::Null).into())
                        .unwrap();
                }
                _ => {}
            }
        }
    }
}
