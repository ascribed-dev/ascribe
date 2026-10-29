//! The preview's server: the files of one folder, over HTTP, at
//! `127.0.0.1` alone.
//!
//! This is the one place the binary opens a socket, and it only listens
//! on the loopback address, so nothing beyond this computer reaches it and
//! it reaches nothing (decision #981). It answers `GET` and `HEAD` with a
//! file of the folder, `index.html` for a folder, and nothing else: no
//! file outside the folder, and no listing.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use super::{PORT, PreviewError};

/// The longest a request's head may be.
const MAX_HEAD: usize = 16 * 1024;

/// How long a connection may take to send its request.
const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// A folder served at a port of `127.0.0.1`.
pub struct Server {
    listener: TcpListener,
    folder: PathBuf,
}

impl Server {
    /// Listens at `port` of `127.0.0.1`; without one, at [`PORT`], or at a
    /// port the system picks when that one's taken.
    ///
    /// # Errors
    ///
    /// The port can't be listened on.
    pub fn bind(folder: PathBuf, port: Option<u16>) -> Result<Server, PreviewError> {
        let at = |port| SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let listener = match port {
            Some(port) => TcpListener::bind(at(port)),
            None => TcpListener::bind(at(PORT)).or_else(|_| TcpListener::bind(at(0))),
        }
        .map_err(|e| PreviewError::Listen {
            address: at(port.unwrap_or(PORT)).to_string(),
            message: e.to_string(),
        })?;
        Ok(Server { listener, folder })
    }

    /// The address it's served at, as a browser opens it:
    /// `http://127.0.0.1:4880/`.
    pub fn address(&self) -> String {
        match self.listener.local_addr() {
            Ok(at) => format!("http://{at}/"),
            Err(_) => format!("http://{}:{PORT}/", Ipv4Addr::LOCALHOST),
        }
    }

    /// Answers requests on a thread of its own, each connection on one of
    /// its own, for as long as the process runs.
    pub fn spawn(self) {
        thread::spawn(move || {
            let port = self.listener.local_addr().map_or(PORT, |at| at.port());
            for stream in self.listener.incoming().flatten() {
                let folder = self.folder.clone();
                thread::spawn(move || answer(stream, port, &folder));
            }
        });
    }
}

/// Reads one request from `stream` and answers it from `folder`. A
/// connection that breaks off is let go.
fn answer(stream: TcpStream, port: u16, folder: &Path) {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let Ok(read) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(read).take(MAX_HEAD as u64);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    // The rest of the head: of its fields, only `Host` matters.
    let mut host = None;
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) if line == "\r\n" || line == "\n" => break,
            Ok(_) => {
                if let Some((name, value)) = line.split_once(':')
                    && name.trim().eq_ignore_ascii_case("host")
                {
                    host = Some(value.trim().to_owned());
                }
            }
        }
    }
    let response = respond(&request_line, host.as_deref(), port, folder);
    let mut stream = stream;
    let _ = stream.write_all(&response);
    let _ = stream.flush();
}

/// The whole response to `request_line`, `GET /guide/ HTTP/1.1`, with the
/// request's `Host`, from `folder`, served at `port`.
///
/// A request whose `Host` isn't this server's own address, `127.0.0.1` or
/// `localhost` with its port, is refused: a page elsewhere that points a
/// name of its own at 127.0.0.1 (DNS rebinding) can't read the preview.
fn respond(request_line: &str, host: Option<&str>, port: u16, folder: &Path) -> Vec<u8> {
    if !host.is_some_and(|host| is_own(host, port)) {
        return status(
            403,
            "Forbidden",
            "The preview answers only at its own address, 127.0.0.1 or localhost with its port.",
        );
    }
    let mut words = request_line.split_whitespace();
    let (Some(method), Some(target)) = (words.next(), words.next()) else {
        return status(
            400,
            "Bad Request",
            "The request isn't one this server reads.",
        );
    };
    let head_only = match method {
        "GET" => false,
        "HEAD" => true,
        _ => return status(405, "Method Not Allowed", "The preview is only read."),
    };
    let path = target.split(['?', '#']).next().unwrap_or_default();
    let Some(segments) = segments(path) else {
        return status(404, "Not Found", "No file of the preview is there.");
    };
    let mut file = folder.to_owned();
    let mut rel = std::path::PathBuf::new();
    for segment in &segments {
        file.push(segment);
        rel.push(segment);
    }
    // Outside FileSystem: the preview's own output, which `ascribe preview`
    // just wrote, not a project file. Nothing on the way to the file is a
    // link: the folder is written beside a manifest a repository can commit,
    // and a committed link in it would serve any file on the machine.
    if ascribe_resolve::link_on_the_way(folder, &rel).is_some() {
        return status(404, "Not Found", "No file of the preview is there.");
    }
    let is_folder = std::fs::metadata(&file).is_ok_and(|m| m.is_dir());
    if is_folder {
        if !path.ends_with('/') {
            // From the path's own segments, so `//x` leads to `/x/`, never
            // to another host.
            let raw: Vec<&str> = path
                .split('/')
                .filter(|s| !s.is_empty() && *s != ".")
                .collect();
            return redirect(&format!("/{}/", raw.join("/")));
        }
        file.push("index.html");
    }
    // Outside FileSystem: as above.
    let Ok(body) = std::fs::read(&file) else {
        return status(404, "Not Found", "No file of the preview is there.");
    };
    let mut out = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        content_type(&file),
        body.len()
    )
    .into_bytes();
    if !head_only {
        out.extend(body);
    }
    out
}

/// Whether `host`, a request's `Host`, is this server's own address:
/// `127.0.0.1` or `localhost`, with `port`.
fn is_own(host: &str, port: u16) -> bool {
    let host = host.to_ascii_lowercase();
    [Ipv4Addr::LOCALHOST.to_string(), "localhost".to_owned()]
        .iter()
        .any(|name| host == format!("{name}:{port}"))
}

/// The segments of a request's path, each decoded; `None` for one that
/// leads out of the folder or can't be a file's name.
fn segments(path: &str) -> Option<Vec<String>> {
    let path = path.strip_prefix('/')?;
    let mut out = Vec::new();
    for raw in path.split('/') {
        let segment = decode(raw)?;
        match segment.as_str() {
            "" | "." => {}
            ".." => return None,
            s if s.contains(['/', '\\', '\0']) || s.contains(':') => return None,
            _ => out.push(segment),
        }
    }
    Some(out)
}

/// `%`-decodes a segment of a path; `None` when it isn't UTF-8 or a `%` has
/// no two hex digits after it.
fn decode(segment: &str) -> Option<String> {
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%' {
            let hex = segment.get(at + 1..at + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            at += 3;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The media type of a file, by its extension.
fn content_type(file: &Path) -> &'static str {
    let extension = file
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("txt" | "md") => "text/plain; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("ico") => "image/x-icon",
        Some("pdf") => "application/pdf",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        _ => "application/octet-stream",
    }
}

/// A response with no file: its status and a line of text saying why.
fn status(code: u16, reason: &str, text: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}\n",
        text.len() + 1
    )
    .into_bytes()
}

/// A response that sends the browser to `to`, a folder's address with its
/// `/`, so the relative links in its `index.html` lead where they should.
fn redirect(to: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 301 Moved Permanently\r\nLocation: {to}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("guide")).unwrap();
        std::fs::write(dir.path().join("index.html"), "top").unwrap();
        std::fs::write(dir.path().join("guide/index.html"), "guide").unwrap();
        std::fs::write(dir.path().join("guide/a b.html"), "spaced").unwrap();
        dir
    }

    fn text(response: &[u8]) -> String {
        String::from_utf8_lossy(response).into_owned()
    }

    /// The response to `request_line`, asked at the server's own address.
    fn ask(request_line: &str, dir: &Path) -> String {
        text(&respond(request_line, Some("127.0.0.1:4880"), 4880, dir))
    }

    #[test]
    fn only_a_request_for_its_own_address_is_answered() {
        let dir = folder();
        for host in ["127.0.0.1:4880", "localhost:4880", "LocalHost:4880"] {
            let response = text(&respond("GET / HTTP/1.1", Some(host), 4880, dir.path()));
            assert!(response.starts_with("HTTP/1.1 200 "), "{host}: {response}");
        }
        for host in [
            Some("evil.example:4880"),
            Some("127.0.0.1:4881"),
            Some("127.0.0.1"),
            Some("localhost.evil.example:4880"),
            None,
        ] {
            let response = text(&respond("GET / HTTP/1.1", host, 4880, dir.path()));
            assert!(
                response.starts_with("HTTP/1.1 403 "),
                "{host:?}: {response}"
            );
        }
    }

    #[test]
    fn a_file_is_answered_with_its_type() {
        let dir = folder();
        let response = ask("GET /guide/a%20b.html HTTP/1.1\r\n", dir.path());
        assert!(response.starts_with("HTTP/1.1 200 OK\r\n"), "{response}");
        assert!(response.contains("Content-Type: text/html; charset=utf-8\r\n"));
        assert!(response.ends_with("\r\n\r\nspaced"));
    }

    #[test]
    fn a_folder_is_its_index_and_its_address_ends_with_a_slash() {
        let dir = folder();
        assert!(ask("GET / HTTP/1.1", dir.path()).ends_with("top"));
        assert!(ask("GET /guide/?q=1 HTTP/1.1", dir.path()).ends_with("guide"));
        assert!(ask("GET /guide HTTP/1.1", dir.path()).contains("Location: /guide/\r\n"));
        assert!(ask("GET //guide HTTP/1.1", dir.path()).contains("Location: /guide/\r\n"));
    }

    #[test]
    fn head_has_no_body() {
        let dir = folder();
        let response = ask("HEAD / HTTP/1.1", dir.path());
        assert!(response.contains("Content-Length: 3\r\n"));
        assert!(response.ends_with("\r\n\r\n"));
    }

    #[test]
    fn nothing_outside_the_folder_is_answered() {
        let dir = folder();
        for target in [
            "/../secret",
            "/guide/%2e%2e/%2e%2e/x",
            "/a%2fb",
            "/%zz",
            "relative",
        ] {
            let response = ask(&format!("GET {target} HTTP/1.1"), dir.path());
            assert!(
                response.starts_with("HTTP/1.1 404 "),
                "{target}: {response}"
            );
        }
        let response = ask("POST / HTTP/1.1", dir.path());
        assert!(response.starts_with("HTTP/1.1 405 "));
    }

    /// A link in the folder, to a file or a folder outside it, isn't followed.
    #[cfg(unix)]
    #[test]
    fn nothing_through_a_link_is_answered() {
        let dir = folder();
        let outside = tempfile::tempdir().expect("a folder outside");
        std::fs::write(outside.path().join("secret.txt"), "secret").expect("a file outside");
        std::os::unix::fs::symlink(
            outside.path().join("secret.txt"),
            dir.path().join("file-link"),
        )
        .expect("a link to a file");
        std::os::unix::fs::symlink(outside.path(), dir.path().join("dir-link"))
            .expect("a link to a folder");
        for target in ["/file-link", "/dir-link/secret.txt", "/dir-link/"] {
            let response = ask(&format!("GET {target} HTTP/1.1"), dir.path());
            assert!(
                response.starts_with("HTTP/1.1 404 "),
                "{target}: {response}"
            );
            assert!(!response.contains("secret"), "{target}: {response}");
        }
    }
}
