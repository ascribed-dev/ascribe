//! `file:` URIs and paths.
//!
//! `lsp-types`' `Uri` is a parsed string with no notion of files, so the
//! conversions the server needs are here: percent-decoding a `file:` URI into a
//! path (including the `file:///c%3A/dir` form VS Code sends for Windows
//! drives), and the reverse.

use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use lsp_types::Uri;

/// The path a `file:` URI names, or `None` for any other scheme (an
/// `untitled:` buffer, a `vscode-notebook-cell:` URI).
pub fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    let text = uri.as_str();
    let rest = text.strip_prefix("file://")?;
    // The authority (empty, or `localhost`) ends at the first `/`.
    let slash = rest.find('/')?;
    let (authority, path) = rest.split_at(slash);
    if !(authority.is_empty() || authority.eq_ignore_ascii_case("localhost")) {
        // A UNC path: `file://server/share/file`.
        let decoded = percent_decode(path)?;
        return Some(PathBuf::from(format!("//{authority}{decoded}")));
    }
    let decoded = percent_decode(path)?;
    // `/c:/dir/file` names a drive on Windows.
    let bytes = decoded.as_bytes();
    if cfg!(windows) && bytes.len() >= 3 && bytes[0] == b'/' && bytes[2] == b':' {
        return Some(PathBuf::from(&decoded[1..]));
    }
    Some(PathBuf::from(decoded))
}

/// The `file:` URI of an absolute path.
pub fn path_to_uri(path: &Path) -> Option<Uri> {
    let mut out = String::from("file://");
    let mut any = false;
    let text = path.to_string_lossy().replace('\\', "/");
    if !text.starts_with('/') {
        // A Windows drive path, `C:/dir`.
        out.push('/');
    }
    for (i, segment) in text.split('/').enumerate() {
        if i > 0 {
            out.push('/');
        }
        any |= !segment.is_empty();
        for byte in segment.bytes() {
            if byte.is_ascii_alphanumeric() || b"-._~!$&'()*+,;=@".contains(&byte) {
                out.push(byte as char);
            } else if byte == b':' && i == 0 {
                out.push(':');
            } else {
                out.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    if !any {
        return None;
    }
    Uri::from_str(&out).ok()
}

fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = text.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// `path` with `.` and `..` resolved lexically, without touching the disk.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// The path from `base` to `path`, both absolute and normalized, as
/// `/`-separated segments, with `..` where `path` is outside `base`. `None`
/// when they share no root (different drives).
pub fn relative_to(base: &Path, path: &Path) -> Option<String> {
    let base: Vec<Component> = base.components().collect();
    let path: Vec<Component> = path.components().collect();
    let common = base.iter().zip(&path).take_while(|(a, b)| a == b).count();
    if common == 0 {
        return None;
    }
    let mut segments: Vec<String> = Vec::new();
    for _ in common..base.len() {
        segments.push("..".to_owned());
    }
    for component in &path[common..] {
        segments.push(component.as_os_str().to_string_lossy().into_owned());
    }
    Some(segments.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_path_with_spaces_and_unicode() {
        let path = Path::new("/home/a b/héllo/📄.md");
        let uri = path_to_uri(path).expect("a uri");
        assert_eq!(
            uri.as_str(),
            "file:///home/a%20b/h%C3%A9llo/%F0%9F%93%84.md"
        );
        assert_eq!(uri_to_path(&uri).as_deref(), Some(path));
    }

    #[test]
    fn other_schemes_are_not_files() {
        let uri = Uri::from_str("untitled:Untitled-1").expect("a uri");
        assert_eq!(uri_to_path(&uri), None);
    }

    #[test]
    fn relative_paths_climb_out_of_the_base() {
        let base = Path::new("/p/project");
        assert_eq!(
            relative_to(base, Path::new("/p/project/docs/a.md")).as_deref(),
            Some("docs/a.md")
        );
        assert_eq!(
            relative_to(base, Path::new("/p/docs/a.md")).as_deref(),
            Some("../docs/a.md")
        );
    }

    #[test]
    fn normalize_resolves_dots() {
        assert_eq!(
            normalize(Path::new("/a/b/../c/./d")),
            PathBuf::from("/a/c/d")
        );
    }
}
