//! A page as a reader of one build sees it: the page resolved for the build,
//! then written as the build's plain output writes it ([`emit_page`] with the
//! [`PlainEmitter`]), without writing anything. And a permalink's help, as
//! the build's `_ascribe/help.json` has it ([`permalink`]).

use std::path::Path;

use ascribe_core::RelPath;
use ascribe_emit::{EmitContext, Help, HelpEntry, PlainEmitter, emit_page, help, help_header};
use ascribe_model::{Build, edit_distance};
use ascribe_resolve::{AstroRouter, FileKind, Project};
use serde::Serialize;

use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// What `ascribe resolve <PAGE> --format json` answers.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Resolved {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The page, as a path from the content root.
    pub page: String,
    /// The build.
    pub build: String,
    /// Why the build doesn't publish the page, when it doesn't; `text` is
    /// empty then.
    pub not_published: Option<String>,
    /// The page's route in the build's site.
    pub route: Option<String>,
    /// The page as plain Markdown, as the build's `plain` output writes it:
    /// with its frontmatter first when asked for.
    pub text: String,
}

/// The page at content path `path`, resolved for `build` and written as its
/// plain output writes it; with `frontmatter`, its resolved frontmatter first, between `---`
/// lines. `project_root` is the directory of `ascribe.toml`.
///
/// # Errors
///
/// `path` isn't a page of the project, or the page can't be written.
pub fn resolve(
    project: &Project,
    project_root: &Path,
    path: &RelPath,
    build: &Build,
    frontmatter: bool,
) -> Result<Resolved, QueryError> {
    let file = project.file(path).ok_or_else(|| QueryError::NotASource {
        path: path.to_string(),
    })?;
    if file.kind == FileKind::Fragment {
        return Err(QueryError::NotAPage {
            path: path.to_string(),
        });
    }
    let mut answer = Resolved {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        page: path.to_string(),
        build: build.name.clone(),
        not_published: None,
        route: None,
        text: String::new(),
    };
    if let Some(reason) = project.dropped(path, build) {
        answer.not_published = Some(format!(
            "build `{}` doesn't publish `{path}`: {}",
            build.name,
            reason.explanation()
        ));
        return Ok(answer);
    }
    // An older line's addresses are under its folder, as its output's are.
    let project = project.for_build(build);
    let project = project.as_ref();
    let router = AstroRouter::from_consumer(&project.model().consumer);
    let Some(page) = project.resolve_page(path, build, &router) else {
        return Err(QueryError::NotAPage {
            path: path.to_string(),
        });
    };
    let cx = EmitContext::new(project, project_root, build);
    let emitter = PlainEmitter::new(project.model());
    let emitted = emit_page(&emitter, &cx, &page)?;
    if frontmatter && let Some(value) = page.frontmatter.as_ref().filter(|v| !v.is_null()) {
        // A value parsed from YAML always writes back as YAML; were it not
        // to, the page is still worth showing without its frontmatter.
        let yaml = serde_yaml_ng::to_string(value).unwrap_or_default();
        answer.text.push_str("---\n");
        answer.text.push_str(&yaml);
        if !yaml.ends_with('\n') {
            answer.text.push('\n');
        }
        answer.text.push_str("---\n\n");
    }
    answer.text.push_str(&emitted.text);
    answer.route = Some(page.route.clone());
    Ok(answer)
}

/// What `ascribe resolve go:<name> --format json` answers: the name's entry
/// as the build's `_ascribe/help.json` writes it, with the file's `version`,
/// `build`, and `line`.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ResolvedPermalink {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The version of `help.json`'s format, which this shares. A later
    /// version adds keys and changes none of these.
    pub version: u32,
    /// The build, without its line.
    pub build: String,
    /// The line, for a line of a versioned build that has more than one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
    /// The permalink's name, as asked for.
    pub name: String,
    /// Its entry.
    #[serde(flatten)]
    pub entry: HelpEntry,
}

/// What a build has under a permalink's name.
#[derive(Debug)]
pub enum PermalinkHelp {
    /// Its entry.
    Found(ResolvedPermalink),
    /// The permalink is retired.
    Retired {
        /// Why, as `ascribe.toml` writes it.
        reason: String,
        /// Where its address leads now, as `ascribe.toml` writes it.
        to: Option<String>,
    },
    /// The build publishes no permalink by the name.
    Missing {
        /// The names it publishes closest to the one asked for, at most
        /// three.
        closest: Vec<String>,
    },
}

/// The help `build` gives for the permalink `name`: its entry in the build's
/// `_ascribe/help.json`, from the function that writes the file, so the two
/// can't disagree. `project_root` is the directory of `ascribe.toml`.
pub fn permalink(
    project: &Project,
    project_root: &Path,
    name: &str,
    build: &Build,
) -> PermalinkHelp {
    // An older line's addresses are under its folder, as its output's are.
    let project = project.for_build(build);
    let project = project.as_ref();
    let router = AstroRouter::from_consumer(&project.model().consumer);
    let resolved = project.resolve_build(build, &router);
    let cx = EmitContext::new(project, project_root, build);
    match help(&cx, &resolved, name) {
        Help::Entry(entry) => {
            let (version, build, line) = help_header(&cx);
            PermalinkHelp::Found(ResolvedPermalink {
                schema_version: SCHEMA_VERSION,
                ascribe_version: ASCRIBE_VERSION,
                version,
                build: build.to_owned(),
                line: line.map(str::to_owned),
                name: name.to_owned(),
                entry,
            })
        }
        Help::Retired { reason, to } => PermalinkHelp::Retired { reason, to },
        Help::Missing { names } => PermalinkHelp::Missing {
            closest: closest(&names, name),
        },
    }
}

/// The names closest to `given`, at most three: within a third of its
/// length in edits, or containing it.
fn closest(names: &[String], given: &str) -> Vec<String> {
    let mut scored: Vec<(usize, &String)> = names
        .iter()
        .map(|name| (edit_distance(given, name), name))
        .filter(|(distance, name)| {
            *distance <= (given.len() / 3).max(2) || (!given.is_empty() && name.contains(given))
        })
        .collect();
    scored.sort();
    scored
        .into_iter()
        .take(3)
        .map(|(_, name)| name.clone())
        .collect()
}
