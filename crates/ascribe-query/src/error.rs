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
    /// A content type isn't one of the model's.
    #[error("no content type is `{given}`{}", type_text(.closest.as_deref(), .types))]
    UnknownType {
        /// The type, as given.
        given: String,
        /// The closest of the model's types, when one is close.
        closest: Option<String>,
        /// The model's types, in its order.
        types: Vec<String>,
    },
    /// A path to search in names no page or fragment of the project.
    #[error("{path} {}{}", if *.missing { "doesn't exist" } else { "has no page or fragment of the project" }, quote_text(.query.as_deref()))]
    NoSource {
        /// The path, as given.
        path: String,
        /// Whether nothing is there at all.
        missing: bool,
        /// When every path is a plain word, the query and them together,
        /// quoted as one.
        query: Option<String>,
    },
    /// A path to search in can't be in the project.
    #[error(transparent)]
    Scope(#[from] ascribe_check::ScopeError),
    /// A query isn't written as `search` reads one.
    #[error(transparent)]
    BadQuery(#[from] ascribe_resolve::reading::BadQuery),
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
    /// A permalinks list can't be read.
    #[error("{file} can't be read: {detail}")]
    PermalinksListInvalid {
        /// The list, as shown.
        file: String,
        /// Its first problem.
        detail: String,
    },
    /// A name can't be retired.
    #[error("can't retire `{name}`: {why}")]
    NotRetirable {
        /// The name, or the page or folder, as given.
        name: String,
        /// Why not.
        why: String,
    },
    /// A line isn't named as a mark names one.
    #[error(
        "`{given}` isn't a line: name one as a mark does, with its target, such as `self-managed 3.4`"
    )]
    BadLine {
        /// The line, as given.
        given: String,
    },
    /// A target has no lines in `[releases]`.
    #[error("`{target}` has no lines in [releases]{}", targets_text(.targets))]
    NoLines {
        /// The target, as given.
        target: String,
        /// The targets that have lines.
        targets: Vec<String>,
    },
    /// A target's list doesn't declare a line.
    #[error("[releases.{target}] doesn't declare `{line}`; its lines are {}", .lines.join(", "))]
    UndeclaredLine {
        /// The target.
        target: String,
        /// The line, as given.
        line: String,
        /// The lines it declares, newest first.
        lines: Vec<String>,
    },
}

fn targets_text(targets: &[String]) -> String {
    if targets.is_empty() {
        "; the project lists none".to_owned()
    } else {
        format!("; the targets with lines are {}", targets.join(", "))
    }
}

fn type_text(closest: Option<&str>, types: &[String]) -> String {
    match closest {
        Some(closest) => format!("; did you mean {closest}?"),
        None => format!("; the types are {}", types.join(", ")),
    }
}

fn quote_text(query: Option<&str>) -> String {
    query.map_or_else(String::new, |query| {
        format!("; to search for several words, quote them together: ascribe search {query}")
    })
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
            QueryError::UnknownType { .. } => "unknown_type",
            QueryError::NoSource { missing: true, .. } => "path_missing",
            QueryError::NoSource { missing: false, .. } => "no_source_at_path",
            QueryError::Scope(e) => e.code(),
            QueryError::BadQuery(e) => e.code(),
            QueryError::BadTarget { .. } => "bad_target",
            QueryError::Render(e) => e.code(),
            QueryError::PermalinksListInvalid { .. } => "permalinks_list_invalid",
            QueryError::NotRetirable { .. } => "permalink_not_retirable",
            QueryError::BadLine { .. } => "bad_line",
            QueryError::NoLines { .. } => "no_lines",
            QueryError::UndeclaredLine { .. } => "undeclared_line",
        }
    }
}
