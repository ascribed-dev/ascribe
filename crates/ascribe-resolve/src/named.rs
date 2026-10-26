//! What a memo file's `about` names (SPEC §7.7, ABNF rule `about-target`),
//! and `ascribe refs` takes: a page or a fragment by its content path, a
//! heading as `page.md#id`, or a content model entry as `<prefix>:<name>`,
//! such as `phrase:product`. This is the one reading of that grammar, and
//! [`Named::written`] the one writing of it.

use ascribe_core::RelPath;
use ascribe_core::path::percent_decode;

use crate::project::Project;
use crate::uses::Usable;

/// The kinds of content model entry a target can name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// A phrase, `phrase:<key>`.
    Phrase,
    /// A feature, `feature:<key>`.
    Feature,
    /// A glossary term, `term:<id>`.
    Term,
    /// A dimension, `dimension:<name>`.
    Dimension,
    /// A note type, `note:<type>`.
    Note,
    /// A project widget, `widget:<name>`.
    Widget,
}

/// The prefixes that name a content model entry, and the kind each names.
pub const PREFIXES: &[(&str, EntryKind)] = &[
    ("phrase", EntryKind::Phrase),
    ("feature", EntryKind::Feature),
    ("term", EntryKind::Term),
    ("dimension", EntryKind::Dimension),
    ("note", EntryKind::Note),
    ("widget", EntryKind::Widget),
];

/// What a target names, as written: it may name nothing in the project.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Named {
    /// A source file, by content path, or a heading on it.
    Path {
        /// The file.
        path: RelPath,
        /// The heading's id, after `#`.
        id: Option<String>,
    },
    /// A content model entry.
    Entry {
        /// Which kind.
        kind: EntryKind,
        /// Its key, id, or name.
        name: String,
    },
}

/// Why a target can't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadTarget {
    /// The target, as given.
    pub given: String,
    /// What's wrong with it.
    pub reason: String,
}

impl Named {
    /// What a memo file's `about` names, by the ABNF rule `about-target`: a
    /// content model entry, when `given` starts with one of [`PREFIXES`] and
    /// a colon, or else a content path, percent-decoded, with an optional
    /// `#id`.
    ///
    /// # Errors
    ///
    /// `given` doesn't match the rule.
    pub fn parse(given: &str) -> Result<Named, BadTarget> {
        Named::entry(given).unwrap_or_else(|| Named::about_path(given))
    }

    /// A content model entry, when `given` starts with one of [`PREFIXES`]
    /// and a colon: `phrase:product`, or `phrase:{product}`, as a phrase is
    /// written. The prefix and the name are compared as written, so case
    /// counts.
    ///
    /// # Errors
    ///
    /// Nothing follows the colon, or the name has whitespace or a brace.
    pub fn entry(given: &str) -> Option<Result<Named, BadTarget>> {
        let (prefix, name) = given.split_once(':')?;
        let (_, kind) = PREFIXES.iter().find(|(p, _)| *p == prefix)?;
        let name = name
            .strip_prefix('{')
            .and_then(|n| n.strip_suffix('}'))
            .unwrap_or(name);
        let bad = |reason: String| {
            Some(Err(BadTarget {
                given: given.to_owned(),
                reason,
            }))
        };
        if name.is_empty() {
            return bad(format!("give a name after `{prefix}:`"));
        }
        if name
            .chars()
            .any(|c| c == ' ' || c.is_ascii_control() || c == '{' || c == '}')
        {
            return bad(format!(
                "the name after `{prefix}:` has a space or a brace, which no name has"
            ));
        }
        Some(Ok(Named::Entry {
            kind: *kind,
            name: name.to_owned(),
        }))
    }

    /// A file and an optional heading, from a content path, as `about`
    /// writes one: segments parted by `/`, after an optional `./`, each
    /// percent-decoded as a link destination is and then none empty, `.`,
    /// `..`, or holding a `/`,
    /// then an optional `#` and an id. The first `#` ends the path, so a
    /// `#` in a file's name is written `%23`.
    fn about_path(given: &str) -> Result<Named, BadTarget> {
        let bad = |reason: &str| BadTarget {
            given: given.to_owned(),
            reason: reason.to_owned(),
        };
        let (path, id) = match given.split_once('#') {
            Some((path, id)) => (path, Some(id)),
            None => (given, None),
        };
        if let Some(id) = id {
            if id.is_empty() {
                return Err(bad("give a heading's id after `#`, or leave the `#` out"));
            }
            if id.chars().any(|c| c == ' ' || c.is_ascii_control()) {
                return Err(bad("a heading's id has no spaces or tabs"));
            }
        }
        let path = path.strip_prefix("./").unwrap_or(path);
        if path.is_empty() {
            return Err(bad(
                "give a page's or a fragment's content path, or an entry such as `phrase:product`",
            ));
        }
        // Each segment is decoded and then checked, so `%2E%2E` is `..` and
        // `%2F` can't make two segments of one.
        let mut segments = Vec::new();
        for segment in path.split('/') {
            let decoded = percent_decode(segment);
            if decoded.is_empty() || decoded == "." || decoded == ".." || decoded.contains('/') {
                return Err(bad(
                    "a content path has no empty, `.`, or `..` segments, decoded or not, \
                     doesn't start with `/`, and writes no `/` as `%2F`",
                ));
            }
            if decoded.chars().any(|c| c.is_ascii_control()) {
                return Err(bad("a content path has no tabs or line breaks"));
            }
            segments.push(decoded);
        }
        let path = RelPath::parse(&segments.join("/")).map_err(|_| {
            bad("it isn't a path in the project, or an entry such as `phrase:product`")
        })?;
        Ok(Named::Path {
            path,
            id: id.map(str::to_owned),
        })
    }

    /// A file and an optional heading, from a content path as the command
    /// line gives it, not decoded, with an optional `#id`.
    ///
    /// # Errors
    ///
    /// `given` isn't a content path.
    pub fn path(given: &str) -> Result<Named, BadTarget> {
        let (path, id) = match given.split_once('#') {
            Some((path, id)) => (path, Some(id).filter(|id| !id.is_empty())),
            None => (given, None),
        };
        let path = RelPath::parse(path.trim_start_matches("./")).map_err(|_| BadTarget {
            given: given.to_owned(),
            reason: "it isn't a path in the project, or an entry such as `phrase:product`"
                .to_owned(),
        })?;
        Ok(Named::Path {
            path,
            id: id.map(str::to_owned),
        })
    }

    /// How a memo file's `about` writes this, which [`Named::parse`] reads
    /// back: a path's `%` and `#` percent-encoded, and `./` before a path
    /// that would read as an entry.
    pub fn written(&self) -> String {
        match self {
            Named::Entry { kind, name } => {
                let prefix = PREFIXES
                    .iter()
                    .find(|(_, k)| k == kind)
                    .map_or("", |(p, _)| p);
                format!("{prefix}:{name}")
            }
            Named::Path { path, id } => {
                let mut out = String::new();
                for c in path.as_str().chars() {
                    if c == '%' || c == '#' {
                        out.push_str(&format!("%{:02X}", u32::from(c)));
                    } else {
                        out.push(c);
                    }
                }
                if Named::entry(&out).is_some() {
                    out.insert_str(0, "./");
                }
                if let Some(id) = id {
                    out.push('#');
                    out.push_str(id);
                }
                out
            }
        }
    }

    /// The thing in `project` this names, when it exists. A page's heading
    /// can be written in a fragment it includes: that heading, where it's
    /// written.
    pub fn find(&self, project: &Project) -> Option<Usable> {
        let model = project.model();
        match self {
            Named::Path { path, id: None } => project
                .file(path)
                .is_some()
                .then(|| Usable::File(path.clone())),
            Named::Path { path, id: Some(id) } => {
                project
                    .page_heading(path, id)
                    .map(|(file, h)| Usable::Heading {
                        file,
                        id: h.source_id.clone(),
                    })
            }
            Named::Entry { kind, name } => {
                let name = name.clone();
                match kind {
                    EntryKind::Phrase => model.has_phrase(&name).then_some(Usable::Phrase(name)),
                    EntryKind::Feature => model.feature(&name).map(|_| Usable::Feature(name)),
                    EntryKind::Term => project
                        .glossary()
                        .term(&name)
                        .is_some()
                        .then_some(Usable::Term(name)),
                    EntryKind::Dimension => model.dimension(&name).map(|_| Usable::Dimension(name)),
                    EntryKind::Note => model.note_type(&name).map(|_| Usable::Note(name)),
                    EntryKind::Widget => model.widget(&name).map(|_| Usable::Widget(name)),
                }
            }
        }
    }
}
