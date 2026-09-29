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
/// `untitled:` buffer, a `vscode-notebook-cell:` URI). A Windows drive path has
/// its drive letter in upper case, so it compares equal to the same path from
/// anywhere else (see [`normalize`]).
pub fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    decode_file_uri(uri.as_str(), cfg!(windows)).map(PathBuf::from)
}

/// The text of the path a `file:` URI names. `windows` says whether
/// `/c:/dir` names a drive (it does only on Windows; elsewhere it's a
/// directory called `c:`). Kept apart from the platform so every platform
/// tests the Windows forms.
fn decode_file_uri(text: &str, windows: bool) -> Option<String> {
    let rest = text.strip_prefix("file://")?;
    // The authority (empty, or `localhost`) ends at the first `/`.
    let slash = rest.find('/')?;
    let (authority, path) = rest.split_at(slash);
    let decoded = percent_decode(path)?;
    if !(authority.is_empty() || authority.eq_ignore_ascii_case("localhost")) {
        // A UNC path: `file://server/share/file`.
        return Some(format!("//{authority}{decoded}"));
    }
    if windows && let Some(drive) = drive_path(&decoded) {
        return Some(drive);
    }
    Some(decoded)
}

/// `/c:/dir/file` (VS Code writes the colon as `%3A` and the drive letter in
/// lower case) as the drive path `C:/dir/file`, with the drive letter in upper
/// case; `None` when the text isn't a drive path.
fn drive_path(decoded: &str) -> Option<String> {
    let rest = decoded.strip_prefix('/')?;
    let bytes = rest.as_bytes();
    if bytes.len() < 2 || !bytes[0].is_ascii_alphabetic() || bytes[1] != b':' {
        return None;
    }
    if bytes.len() > 2 && bytes[2] != b'/' {
        return None;
    }
    Some(upper_drive(rest))
}

/// Whether a path component is a drive, such as `C:`.
fn is_drive(component: &str) -> bool {
    let bytes = component.as_bytes();
    bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// `text` with a leading drive letter in upper case: `c:\dir` is `C:\dir`.
fn upper_drive(text: &str) -> String {
    let bytes = text.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        let mut out = String::with_capacity(text.len());
        out.push(bytes[0].to_ascii_uppercase() as char);
        out.push_str(&text[1..]);
        return out;
    }
    text.to_owned()
}

/// The `file:` URI of an absolute path.
pub fn path_to_uri(path: &Path) -> Option<Uri> {
    let mut out = String::from("file://");
    let mut any = false;
    let text = upper_drive(&path.to_string_lossy().replace('\\', "/"));
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
            } else if byte == b':' && i == 0 && is_drive(segment) {
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
    for (i, component) in path.components().enumerate() {
        match component {
            // A drive letter is compared as upper case.
            Component::Prefix(_) | Component::Normal(_)
                if i == 0 && is_drive(&component.as_os_str().to_string_lossy()) =>
            {
                out.push(upper_drive(&component.as_os_str().to_string_lossy()));
            }
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
    let common = base
        .iter()
        .zip(&path)
        .take_while(|(a, b)| same_component(a, b))
        .count();
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

/// Whether two components are the same. Drive letters are the same in either
/// case: `c:` and `C:` are one drive.
fn same_component(a: &Component, b: &Component) -> bool {
    let (x, y) = (
        a.as_os_str().to_string_lossy(),
        b.as_os_str().to_string_lossy(),
    );
    if is_drive(&x) && is_drive(&y) {
        return x.eq_ignore_ascii_case(&y);
    }
    a == b
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

    #[test]
    fn vs_code_windows_uris_decode_to_upper_case_drive_paths() {
        assert_eq!(
            decode_file_uri("file:///c%3A/Users/a%20b/x.md", true).as_deref(),
            Some("C:/Users/a b/x.md")
        );
        assert_eq!(
            decode_file_uri("file:///C:/Users/x.md", true).as_deref(),
            Some("C:/Users/x.md")
        );
        // Not a drive path: a path that only looks like one.
        assert_eq!(drive_path("/cd:/x"), None);
        assert_eq!(drive_path("/home/x"), None);
        assert_eq!(drive_path("/c:"), Some("C:".to_owned()));
    }

    #[test]
    fn off_windows_a_drive_shaped_path_is_a_plain_path() {
        assert_eq!(
            decode_file_uri("file:///c%3A/Users/x.md", false).as_deref(),
            Some("/c:/Users/x.md")
        );
    }

    #[test]
    fn unc_paths_keep_their_server() {
        assert_eq!(
            decode_file_uri("file://server/share/a.md", true).as_deref(),
            Some("//server/share/a.md")
        );
    }

    #[test]
    fn a_drive_path_becomes_a_uri_with_an_upper_case_drive() {
        let uri = path_to_uri(Path::new("C:\\Users\\x y.md")).expect("a uri");
        assert_eq!(uri.as_str(), "file:///C:/Users/x%20y.md");
        let lower = path_to_uri(Path::new("c:/Users/x.md")).expect("a uri");
        assert_eq!(lower.as_str(), "file:///C:/Users/x.md");
    }

    #[test]
    fn drive_letter_case_does_not_matter_when_comparing() {
        let base = Path::new("C:/Users/kyle/proj");
        assert_eq!(
            relative_to(base, Path::new("c:/Users/kyle/proj/docs/a.md")).as_deref(),
            Some("docs/a.md")
        );
        assert_eq!(
            relative_to(Path::new("c:/Users/kyle"), Path::new("C:/Users/kyle/x.md")).as_deref(),
            Some("x.md")
        );
        // Different drives share nothing.
        assert_eq!(relative_to(base, Path::new("D:/Users/kyle/x.md")), None);
        assert_eq!(normalize(Path::new("c:/a/../b")), PathBuf::from("C:/b"));
    }
}
