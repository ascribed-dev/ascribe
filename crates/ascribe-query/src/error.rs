//! Why a question can't be answered.

use ascribe_check::UnknownBuild;
use ascribe_core::Coded;

/// Why a question can't be answered: the command that asked it gives exit
/// code 2. An answer that is "no" (a link target that doesn't exist, a page
/// a build doesn't publish) isn't an error; it's in the answer.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    /// No diagnostic has this code or slug.
    #[error("no diagnostic is `{given}`{}", closest_text(.closest))]
    UnknownDiagnostic {
        /// The code or slug asked about.
        given: String,
        /// The closest codes, with their slugs (`ASC036 link-target-missing`).
        closest: Vec<String>,
    },
    /// A path isn't one of the project's source files.
    #[error("`{path}` isn't a page or fragment of the project")]
    NotASource {
        /// The path, as given.
        path: String,
    },
    /// A path is a fragment, where only a page will do.
    #[error("`{path}` is a fragment, not a page; a build publishes only pages")]
    NotAPage {
        /// The path, as given.
        path: String,
    },
    /// The model has several builds and none was named.
    #[error("the content model has several builds, so name one with --build: {}", .builds.join(", "))]
    BuildRequired {
        /// The model's builds, in its order.
        builds: Vec<String>,
    },
    /// A build name isn't one of the model's.
    #[error(transparent)]
    UnknownBuild(#[from] UnknownBuild),
    /// A target isn't written as `refs` reads one.
    #[error("`{given}` isn't a target: {reason}")]
    BadTarget {
        /// The target, as given.
        given: String,
        /// What's wrong with it.
        reason: String,
    },
    /// The page can't be rendered.
    #[error(transparent)]
    Render(#[from] ascribe_emit::EmitError),
}

fn closest_text(closest: &[String]) -> String {
    match closest {
        [] => "; `ascribe explain --list` lists them all".to_owned(),
        [one] => format!("; did you mean {one}?"),
        _ => format!("; the closest are: {}", closest.join(", ")),
    }
}

impl Coded for QueryError {
    fn code(&self) -> &'static str {
        match self {
            QueryError::UnknownDiagnostic { .. } => "unknown_diagnostic",
            QueryError::NotASource { .. } => "not_a_source",
            QueryError::NotAPage { .. } => "not_a_page",
            QueryError::BuildRequired { .. } => "build_required",
            QueryError::UnknownBuild(e) => e.code(),
            QueryError::BadTarget { .. } => "bad_target",
            QueryError::Render(e) => e.code(),
        }
    }
}
