//! The diagnostic every tool reports.

use serde::Serialize;
use tessera_core::{DiagnosticSlug, Fix, Issue, Location};

use crate::registry::Registry;

/// How serious a diagnostic is. An error means the document isn't
/// conforming and a build fails; a warning doesn't fail a build (SPEC §8.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// A warning.
    Warning,
    /// An error.
    Error,
}

impl Severity {
    /// `"error"` or `"warning"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
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
    /// The registry code, such as `TSR001`.
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
    /// belongs to none of them (Q101).
    pub unpublished: bool,
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
            code: entry.map_or("TSR000", |e| e.code.as_str()),
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
        }
    }

    /// What to add to the message so a reader knows which builds it's about:
    /// `only in build `a``, `only in builds `a`, `b``, or a note that no build
    /// publishes the content. `None` for a diagnostic that doesn't depend on
    /// a build, one that appears in all `total_builds` builds, and the rows
    /// whose message already names the builds.
    pub fn builds_note(&self, total_builds: usize) -> Option<String> {
        if self.unpublished {
            return Some("in content that no build publishes".to_owned());
        }
        let names_builds = matches!(
            self.slug.as_str(),
            "variant-no-arm-survives" | "link-id-removed" | "link-page-dropped"
        );
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
