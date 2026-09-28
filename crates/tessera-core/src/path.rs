//! Project paths, and how link and image destinations resolve to them.
//!
//! Every path Tessera exchanges between crates is a [`RelPath`]: relative,
//! `/`-separated on every platform, and normalized. Paths of source files
//! and assets are relative to the **content root** (SPEC §2.2), and may begin
//! with `..` segments for files outside it, such as an image elsewhere in the
//! project.
//!
//! [`classify_destination`] and [`LocalDestination::resolve`] implement the
//! resolution rules of `project-docs/contracts/assets.md` ("Resolving a
//! reference"). Phase 11 applies them to links, images, and includes alike.

use std::fmt;

use serde::Serialize;

/// A normalized relative path with `/` separators.
///
/// Invariants: no empty segments, no `.` segments, and `..` segments only at
/// the start. The empty path is the root itself (the content root, for
/// content paths). Segments are compared exactly, so paths are
/// case-sensitive on every platform.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct RelPath(String);

/// Why a path is invalid.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PathError {
    /// The path starts with `/`. For a destination, a leading `/` means the
    /// content root; [`classify_destination`] strips it before parsing.
    #[error("`{0}` is absolute, not relative")]
    Absolute(String),
    /// The path contains a NUL character, which no file name can.
    #[error("`{0}` contains a NUL character")]
    Nul(String),
}

impl RelPath {
    /// The root: the empty path.
    pub fn root() -> RelPath {
        RelPath(String::new())
    }

    /// Parses and normalizes a relative path: drops empty and `.` segments,
    /// and resolves `..` against the segment before it. `..` segments that
    /// have nothing to cancel stay at the start (`a/../../b` is `../b`).
    ///
    /// Only `/` separates segments. A `\` is an ordinary character, so a
    /// path written with backslashes names a file that exists only where
    /// file names can contain them.
    pub fn parse(path: &str) -> Result<RelPath, PathError> {
        if path.starts_with('/') {
            return Err(PathError::Absolute(path.to_owned()));
        }
        if path.contains('\0') {
            return Err(PathError::Nul(path.to_owned()));
        }
        Ok(RelPath::root().joined(path))
    }

    /// This path (a directory) joined with a relative path, normalized.
    /// A leading `/` in `rel` is ignored.
    pub fn join(&self, rel: &str) -> Result<RelPath, PathError> {
        if rel.contains('\0') {
            return Err(PathError::Nul(rel.to_owned()));
        }
        Ok(self.joined(rel))
    }

    fn joined(&self, rel: &str) -> RelPath {
        let mut segs: Vec<&str> = self.segments().collect();
        for seg in rel.split('/') {
            match seg {
                "" | "." => {}
                ".." => {
                    if segs.last().is_some_and(|s| *s != "..") {
                        segs.pop();
                    } else {
                        segs.push("..");
                    }
                }
                s => segs.push(s),
            }
        }
        RelPath(segs.join("/"))
    }

    /// The path as a string: segments joined by `/`, with no leading or
    /// trailing `/`. Empty for the root.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path's segments. None for the root.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/').filter(|s| !s.is_empty())
    }

    /// Whether this is the root.
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// How many `..` segments the path starts with: 0 when it's inside the root.
    pub fn up_count(&self) -> usize {
        self.segments().take_while(|s| *s == "..").count()
    }

    /// Whether the path is inside the root (doesn't start with `..`).
    pub fn is_inside(&self) -> bool {
        self.up_count() == 0
    }

    /// The last segment, unless the path is the root or ends in `..`.
    pub fn file_name(&self) -> Option<&str> {
        self.segments().last().filter(|s| *s != "..")
    }

    /// The file name's extension, after its last `.`, if it has one and the
    /// name isn't only an extension (`.gitignore` has none).
    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name()?;
        let dot = name.rfind('.')?;
        (dot > 0).then(|| &name[dot + 1..])
    }

    /// The directory containing this path: the path minus its last segment.
    /// `None` for the root and for paths that end in `..`.
    pub fn parent(&self) -> Option<RelPath> {
        self.file_name()?;
        Some(match self.0.rfind('/') {
            Some(i) => RelPath(self.0[..i].to_owned()),
            None => RelPath::root(),
        })
    }

    /// Whether `prefix` is this path or one of its ancestors.
    pub fn starts_with(&self, prefix: &RelPath) -> bool {
        let mut mine = self.segments();
        prefix.segments().all(|p| mine.next() == Some(p))
    }

    /// The text of a relative reference from the directory `from_dir` to this
    /// path, as written in output: it always starts with `./` or `../`, so no
    /// consumer mistakes it for a bare name or a package import.
    ///
    /// `None` when `from_dir` starts with more `..` segments than the two
    /// paths share, since the names of those directories are unknown.
    pub fn relative_from(&self, from_dir: &RelPath) -> Option<String> {
        let mine: Vec<&str> = self.segments().collect();
        let theirs: Vec<&str> = from_dir.segments().collect();
        let common = mine.iter().zip(&theirs).take_while(|(a, b)| a == b).count();
        if theirs[common..].contains(&"..") {
            return None;
        }
        let ups = theirs.len() - common;
        let rest = mine[common..].join("/");
        let mut out = if ups == 0 {
            "./".to_owned()
        } else {
            "../".repeat(ups)
        };
        out.push_str(&rest);
        Some(out)
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A link destination or image source, as written in a source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    /// A URL with a scheme (`https:`, `mailto:`, and also `C:`), or a
    /// protocol-relative `//host/…`. Passed through unchanged (SPEC §5.2);
    /// never an asset.
    External,
    /// A path to a file in the project.
    Local(LocalDestination),
}

/// A local destination, split and decoded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalDestination {
    /// Whether it started with `/`, meaning the content root (SPEC §4.2, §5.2).
    pub root_relative: bool,
    /// The path part, percent-decoded, before any `#`. Empty for a
    /// destination that is only a fragment (`#install`), which names the file
    /// it's written in.
    pub path: String,
    /// The part after the first `#`, percent-decoded: a heading's source id
    /// for a page, and passed through unchanged for an asset.
    pub fragment: Option<String>,
}

/// Classifies a destination as written (after CommonMark has processed its
/// escapes and entities).
///
/// Percent-encoded bytes (`%20`) are decoded, as GitHub and browsers do, so
/// `my%20file.png` names `my file.png`. If the decoded bytes aren't UTF-8,
/// the text is kept as written. A `?` has no special meaning.
pub fn classify_destination(dest: &str) -> Destination {
    if dest.starts_with("//") || has_scheme(dest) {
        return Destination::External;
    }
    let (path, fragment) = match dest.split_once('#') {
        Some((p, f)) => (p, Some(percent_decode(f))),
        None => (dest, None),
    };
    let root_relative = path.starts_with('/');
    Destination::Local(LocalDestination {
        root_relative,
        path: percent_decode(path.trim_start_matches('/')),
        fragment,
    })
}

impl LocalDestination {
    /// Resolves the destination from the file it's written in, given as a
    /// content path. For included content that's the fragment, not the page
    /// that includes it (SPEC §4.2). The result may start with `..` when the
    /// destination leads outside the content root; whether that's allowed is
    /// the asset contract's rule, checked against the file system.
    pub fn resolve(&self, written_in: &RelPath) -> Result<RelPath, PathError> {
        if self.path.is_empty() && !self.root_relative {
            return Ok(written_in.clone());
        }
        let base = if self.root_relative {
            RelPath::root()
        } else {
            written_in.parent().unwrap_or_default()
        };
        base.join(&self.path)
    }
}

/// RFC 3986: `ALPHA *( ALPHA / DIGIT / "+" / "-" / "." ) ":"`.
fn has_scheme(dest: &str) -> bool {
    let Some(colon) = dest.find(':') else {
        return false;
    };
    let scheme = &dest[..colon];
    let mut chars = scheme.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

fn percent_decode(text: &str) -> String {
    if !text.contains('%') {
        return text.to_owned();
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let (Some(h), Some(l)) = (
                bytes.get(i + 1).and_then(|b| (*b as char).to_digit(16)),
                bytes.get(i + 2).and_then(|b| (*b as char).to_digit(16)),
            )
        {
            out.push((h * 16 + l) as u8);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> RelPath {
        RelPath::parse(s).unwrap()
    }

    fn local(dest: &str) -> LocalDestination {
        match classify_destination(dest) {
            Destination::Local(l) => l,
            Destination::External => panic!("{dest} is local"),
        }
    }

    #[test]
    fn normalizes() {
        assert_eq!(p("a/./b//c/").as_str(), "a/b/c");
        assert_eq!(p("a/../b").as_str(), "b");
        assert_eq!(p("a/../../b").as_str(), "../b");
        assert_eq!(p("../../x").up_count(), 2);
        assert_eq!(p("").as_str(), "");
        assert!(p(".").is_root());
        assert_eq!(RelPath::parse("/a"), Err(PathError::Absolute("/a".into())));
        assert!(RelPath::parse("a\0b").is_err());
        // Backslashes are ordinary characters.
        assert_eq!(p(r"a\b").segments().count(), 1);
    }

    #[test]
    fn names_and_parents() {
        let path = p("guides/img/shot.v2.png");
        assert_eq!(path.file_name(), Some("shot.v2.png"));
        assert_eq!(path.extension(), Some("png"));
        assert_eq!(path.parent(), Some(p("guides/img")));
        assert_eq!(p("a").parent(), Some(RelPath::root()));
        assert_eq!(RelPath::root().parent(), None);
        assert_eq!(p("..").parent(), None);
        assert_eq!(p(".gitignore").extension(), None);
        assert!(path.starts_with(&p("guides")));
        assert!(!path.starts_with(&p("guide")));
        assert!(path.starts_with(&RelPath::root()));
    }

    #[test]
    fn relative_references_start_with_a_dot() {
        let img = p("_fragments/diagram.png");
        assert_eq!(
            img.relative_from(&p("guides")).as_deref(),
            Some("../_fragments/diagram.png")
        );
        assert_eq!(
            img.relative_from(&p("_fragments")).as_deref(),
            Some("./diagram.png")
        );
        assert_eq!(
            p("a.png").relative_from(&RelPath::root()).as_deref(),
            Some("./a.png")
        );
        assert_eq!(
            p("a/b/c.png").relative_from(&p("a/x/y")).as_deref(),
            Some("../../b/c.png")
        );
        assert_eq!(p("x.png").relative_from(&p("../outside")), None);
    }

    #[test]
    fn classifies_destinations() {
        for external in [
            "https://example.com/a.png",
            "mailto:docs@example.com",
            "//cdn.example.com/x.js",
            "C:/Users/a.png",
        ] {
            assert_eq!(classify_destination(external), Destination::External);
        }
        let l = local("keys.md#rotate-keys");
        assert_eq!(
            (l.path.as_str(), l.fragment.as_deref()),
            ("keys.md", Some("rotate-keys"))
        );
        assert!(!l.root_relative);
        let l = local("/guides/setup.md");
        assert!(l.root_relative);
        assert_eq!(l.path, "guides/setup.md");
        let l = local("#install");
        assert_eq!(
            (l.path.as_str(), l.fragment.as_deref()),
            ("", Some("install"))
        );
        let l = local("my%20file%C3%A9.png");
        assert_eq!(l.path, "my fileé.png");
        // Not a valid escape, or not UTF-8 once decoded: kept as written.
        assert_eq!(local("100%.png").path, "100%.png");
        assert_eq!(local("a%FF.png").path, "a%FF.png");
        // A `:` after a `/` or a digit start isn't a scheme.
        assert!(matches!(
            classify_destination("./a:b.png"),
            Destination::Local(_)
        ));
        assert!(matches!(
            classify_destination("1a:b"),
            Destination::Local(_)
        ));
    }

    #[test]
    fn resolves_from_the_file_it_is_written_in() {
        let fragment = p("_fragments/prerequisites.md");
        assert_eq!(
            local("diagram.png").resolve(&fragment),
            Ok(p("_fragments/diagram.png"))
        );
        assert_eq!(local("../img/a.png").resolve(&fragment), Ok(p("img/a.png")));
        assert_eq!(local("/img/a.png").resolve(&fragment), Ok(p("img/a.png")));
        assert_eq!(local("#setup").resolve(&fragment), Ok(fragment.clone()));
        assert_eq!(
            local("../../shared/logo.png").resolve(&fragment),
            Ok(p("../shared/logo.png"))
        );
        assert_eq!(local("x.png").resolve(&p("index.md")), Ok(p("x.png")));
    }
}
