//! The diagnostic every tool reports.

use ascribe_core::{DiagnosticSlug, Fix, Issue, Location};
use serde::Serialize;

use crate::registry::{Entry, Registry};

/// How serious a diagnostic is. An error means the document isn't
/// conforming and a build fails; a warning doesn't fail a build (SPEC §8.2).
/// Advice is below a warning: it's shown, and never fails `ascribe check`,
/// even with `--deny-warnings`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Advice.
    Advice,
    /// A warning.
    Warning,
    /// An error.
    Error,
}

impl Severity {
    /// `"error"`, `"warning"`, or `"advice"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Advice => "advice",
        }
    }

    /// The severity named `"error"`, `"warning"`, or `"advice"`.
    pub fn from_name(name: &str) -> Option<Severity> {
        match name {
            "error" => Some(Severity::Error),
            "warning" => Some(Severity::Warning),
            "advice" => Some(Severity::Advice),
            _ => None,
        }
    }
}

/// Another place that helps explain a diagnostic, such as the first use of
/// a duplicated id.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RelatedInfo {
    /// Where.
    pub location: Location,
    /// A short note shown with it.
    pub message: String,
}

/// A problem in a documentation set, ready to show.
///
/// Built from an [`Issue`] by [`Diagnostic::from_issue`]: the registry gives
/// the code, severity, and message, so the command line, the build, and the
/// language server say the same thing. Fixes are data (`TextEdit`s), never
/// closures, so the language server and quick fixes can use them as they
/// are.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    /// The registry code, such as `ASC001`.
    pub code: &'static str,
    /// The registry slug, such as `attribute-unknown-key`.
    pub slug: DiagnosticSlug,
    /// The registry severity.
    pub severity: Severity,
    /// The message: the registry's template for the issue's variant, filled in.
    pub message: String,
    /// The primary location: the file and span the problem is reported at.
    pub location: Location,
    /// Other places that help explain it.
    pub related: Vec<RelatedInfo>,
    /// Edits that would fix it.
    pub fixes: Vec<Fix>,
    /// The builds a page-level diagnostic appears in, in the content model's
    /// order. Empty for a file-level diagnostic, which doesn't depend on a
    /// build, and for one in [`unpublished`](Diagnostic::unpublished) content.
    pub builds: Vec<String>,
    /// Whether the diagnostic is in content that no build publishes, and so
    /// belongs to none of them.
    pub unpublished: bool,
    /// What an agent prompt about it carries beyond the message and the
    /// line, as its registry entry's `evidence` names it: the values the
    /// content model allows, a long page's sections, and so on.
    #[serde(skip)]
    pub evidence: Vec<Evidence>,
}

/// One piece of context for an agent prompt: what Ascribe knows that an
/// agent can't cheaply find.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    /// What the prompt calls it: `Allowed values`.
    pub label: &'static str,
    /// The text, which may be several lines.
    pub text: String,
}

impl Diagnostic {
    /// Turns an issue into a diagnostic, looking up its code, severity, and
    /// message in the registry.
    pub fn from_issue(issue: &Issue) -> Diagnostic {
        Diagnostic::with_registry(Registry::global(), issue)
    }

    fn with_registry(registry: &'static Registry, issue: &Issue) -> Diagnostic {
        let entry = registry.get(issue.slug);
        Diagnostic {
            code: entry.map_or("ASC000", |e| e.code.as_str()),
            slug: issue.slug,
            severity: entry.map_or(Severity::Error, |e| e.severity),
            message: registry.message(issue),
            location: issue.location,
            related: issue
                .related
                .iter()
                .map(|r| RelatedInfo {
                    location: r.location,
                    message: r.label.clone(),
                })
                .collect(),
            fixes: issue.fixes.clone(),
            builds: Vec::new(),
            unpublished: false,
            evidence: evidence(entry, issue),
        }
    }

    /// What to add to the message so a reader knows which builds it's about:
    /// `only in build `a``, `only in builds `a`, `b``, or a note that no build
    /// publishes the content. `None` for a diagnostic that doesn't depend on
    /// a build, one that appears in all `total_builds` builds, and one whose
    /// registry message already names the builds (it has a `{build}`
    /// placeholder).
    pub fn builds_note(&self, total_builds: usize) -> Option<String> {
        if self.unpublished {
            return Some("in content that no build publishes".to_owned());
        }
        // A message with a `{build}` placeholder already names the builds.
        let names_builds = Registry::global()
            .get(self.slug)
            .is_some_and(Entry::names_build);
        if self.builds.is_empty() || names_builds || self.builds.len() >= total_builds {
            return None;
        }
        let list = self
            .builds
            .iter()
            .map(|b| format!("`{b}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let noun = if self.builds.len() == 1 {
            "build"
        } else {
            "builds"
        };
        Some(format!("only in {noun} {list}"))
    }

    /// Whether this is an error.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// The evidence of an issue whose registry entry's prompt carries some: each
/// name its `evidence` lists, from the issue's argument for it, in the
/// registry's order. A name the issue has no argument for is left out.
fn evidence(entry: Option<&Entry>, issue: &Issue) -> Vec<Evidence> {
    let Some(entry) = entry else {
        return Vec::new();
    };
    entry
        .evidence
        .iter()
        .filter_map(|name| {
            let (_, arg, label) = EVIDENCE.iter().find(|(n, _, _)| n == name)?;
            let text = issue.args.iter().find(|a| a.name == *arg)?.value.clone();
            Some(Evidence { label, text })
        })
        .collect()
}

/// The evidence an agent prompt can carry: its name in the registry's
/// `evidence`, the issue's argument that holds it, and the label the prompt
/// gives it.
pub const EVIDENCE: &[(&str, &str, &str)] = &[
    // The values the content model allows, as the message lists them, when
    // the problem is a value that isn't one of them.
    ("allowed-values", "values", "Allowed values"),
    // A long page's sections, each with its size, largest first.
    ("page-sections", "sections", "Its sections, largest first"),
    // A page's title and its first paragraph.
    ("page-opening", "opening", "How the page begins"),
    // The first lines of a code block.
    ("code-lines", "lines", "The block begins"),
];
