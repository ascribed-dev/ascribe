//! Problems found by any crate, reported by registry slug.

use std::fmt;

use serde::Serialize;

use crate::{FileId, Location, TextEdit};

/// The slug of a diagnostic in the registry, `tests/conformance/diagnostics.toml`.
///
/// The only values are the constants in [`diagnostics`](crate::diagnostics):
/// the field is private, so no other crate can make up a slug.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct DiagnosticSlug(pub(crate) &'static str);

impl DiagnosticSlug {
    /// The slug's text, such as `"attribute-unknown-key"`.
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// The slug with this text, if the registry has one.
    pub fn from_name(name: &str) -> Option<DiagnosticSlug> {
        crate::diagnostics::ALL
            .iter()
            .copied()
            .find(|s| s.0 == name)
    }
}

impl fmt::Display for DiagnosticSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// A problem found by any Ascribe crate: the parser, the content-model
/// loader, the checks, or resolution.
///
/// An issue says *what* is wrong (its [`slug`](Issue::slug) and message
/// arguments) and *where*. It carries no code, severity, or text: the checks
/// look those up in the registry by slug and turn the issue into a
/// user-facing diagnostic, so every tool words and ranks a problem the same
/// way.
///
/// ```
/// use ascribe_core::{diagnostics, FileId, Issue, Location};
///
/// let issue = Issue::new(diagnostics::ATTRIBUTE_UNKNOWN_KEY, Location::new(FileId::new(0), 6..11))
///     .with_arg("name", "note")
///     .with_arg("key", "kind")
///     .with_arg("suggestion", "type")
///     .with_variant("suggestion");
/// assert_eq!(issue.arg("key"), Some("kind"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Issue {
    /// Which diagnostic this is.
    pub slug: DiagnosticSlug,
    /// Where it's reported: the smallest span that shows the problem, such as
    /// the attribute key rather than the whole directive line. A page-level
    /// problem inside included content is reported at the include site
    /// (SPEC §8.1); the location inside the fragment goes in `related`.
    pub location: Location,
    /// Which of the entry's alternative messages to use (a key of its
    /// `messages` table), or `None` for its main `message`.
    pub variant: Option<&'static str>,
    /// Values for the message template's placeholders, by name. Every
    /// placeholder in the chosen template must have one; values are plain
    /// text, inserted as they are.
    pub args: Vec<Arg>,
    /// Other places that help explain the problem, such as the first use of
    /// a duplicated id, or the fragment line behind an include-site report.
    pub related: Vec<Related>,
    /// Edits that would fix the problem, offered as quick fixes.
    pub fixes: Vec<Fix>,
}

/// A named value for a message placeholder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Arg {
    /// The placeholder's name, as in the template's `{name}`.
    pub name: &'static str,
    /// The text to insert.
    pub value: String,
}

/// A secondary location for an [`Issue`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Related {
    /// Where.
    pub location: Location,
    /// A short plain-text note shown with it, such as "first used here".
    pub label: String,
}

/// A fix for an [`Issue`]: edits to one file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fix {
    /// What the fix does, as shown to the author, such as "Add a trailing colon".
    pub title: String,
    /// The file the edits apply to.
    pub file: FileId,
    /// Simultaneous edits to that file's current text (see [`TextEdit`]).
    pub edits: Vec<TextEdit>,
    /// Whether the edits can be applied as they are, without a person
    /// deciding. Every fix says, where it's made.
    pub applicability: Applicability,
}

/// Whether a [`Fix`] can be applied as it is. Nothing applies fixes on its
/// own; the label lets a reader, such as an agent, tell a fix that only
/// rewrites what's already meant from one that guesses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Applicability {
    /// Applying it can't change what the page says, and leaves nothing to
    /// decide.
    Safe,
    /// Anything else: it changes what the page says, or it's one guess among
    /// others, such as the nearest spelling. When unsure, a fix is unsafe.
    Unsafe,
}

impl Applicability {
    /// `"safe"` or `"unsafe"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Applicability::Safe => "safe",
            Applicability::Unsafe => "unsafe",
        }
    }
}

impl Issue {
    /// An issue with no arguments, related locations, or fixes, using the
    /// entry's main message.
    pub fn new(slug: DiagnosticSlug, location: Location) -> Issue {
        Issue {
            slug,
            location,
            variant: None,
            args: Vec::new(),
            related: Vec::new(),
            fixes: Vec::new(),
        }
    }

    /// Adds a placeholder value.
    pub fn with_arg(mut self, name: &'static str, value: impl Into<String>) -> Issue {
        self.args.push(Arg {
            name,
            value: value.into(),
        });
        self
    }

    /// Chooses one of the entry's alternative messages.
    pub fn with_variant(mut self, variant: &'static str) -> Issue {
        self.variant = Some(variant);
        self
    }

    /// Adds a related location.
    pub fn with_related(mut self, location: Location, label: impl Into<String>) -> Issue {
        self.related.push(Related {
            location,
            label: label.into(),
        });
        self
    }

    /// Adds a fix.
    pub fn with_fix(mut self, fix: Fix) -> Issue {
        self.fixes.push(fix);
        self
    }

    /// The value given for a placeholder, if any. The last one wins if a
    /// name was given twice.
    pub fn arg(&self, name: &str) -> Option<&str> {
        self.args
            .iter()
            .rev()
            .find(|a| a.name == name)
            .map(|a| a.value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics;

    #[test]
    fn slugs_are_looked_up_by_name() {
        assert_eq!(
            DiagnosticSlug::from_name("container-unclosed"),
            Some(diagnostics::CONTAINER_UNCLOSED)
        );
        assert_eq!(DiagnosticSlug::from_name("no-such-diagnostic"), None);
        assert_eq!(
            diagnostics::CONTAINER_UNCLOSED.to_string(),
            "container-unclosed"
        );
    }

    #[test]
    fn builder_collects_everything() {
        let file = FileId::new(3);
        let issue = Issue::new(diagnostics::ID_DUPLICATE, Location::new(file, 10..20))
            .with_arg("id", "setup")
            .with_arg("id", "install")
            .with_related(Location::new(file, 0..5), "first used here")
            .with_fix(Fix {
                title: "Rename the id".into(),
                file,
                edits: vec![TextEdit::replace(14..20, "install-2")],
                applicability: Applicability::Unsafe,
            });
        assert_eq!(issue.arg("id"), Some("install"));
        assert_eq!(issue.arg("path"), None);
        assert_eq!(issue.related.len(), 1);
        assert_eq!(issue.fixes[0].edits.len(), 1);
        assert_eq!(issue.variant, None);
    }
}
