//! A page as a reader of one build sees it: the page resolved for the build,
//! then written as one of the build's outputs writes it ([`emit_page`] with
//! the [`PlainEmitter`] or the [`SiteEmitter`]), without writing anything.
//! And a permalink's help, as the build's `_ascribe/help.json` has it
//! ([`permalink`]).

use std::path::Path;

use ascribe_core::RelPath;
use ascribe_emit::{
    EmitContext, Help, HelpEntry, PlainEmitter, SiteEmitter, emit_page, help, help_header,
    split_frontmatter,
};
use ascribe_model::{Build, edit_distance};
use ascribe_resolve::{AstroRouter, FileKind, Project};
use serde::Serialize;

use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// Which output a page is written as, by the name `ascribe build --emit`
/// gives it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Emit {
    /// The plain output: fully resolved CommonMark with no HTML.
    #[default]
    Plain,
    /// The site output: Markdown with Ascribe's elements, as a site's
    /// integration receives it.
    Site,
}

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
    /// Which output `text` is written as: `plain`, the build's plain
    /// output, or `site`, its site output.
    pub emit: Emit,
    /// Why the build doesn't publish the page, when it doesn't; `text` is
    /// empty then.
    pub not_published: Option<String>,
    /// The page's route in the build's site.
    pub route: Option<String>,
    /// The page as the output `emit` names writes it: with its frontmatter
    /// first when asked for.
    pub text: String,
}

/// The page at content path `path`, resolved for `build` and written as the
/// output `emit` names writes it; with `frontmatter`, its frontmatter first,
/// between `---` lines: the resolved frontmatter for the plain form, the site
/// output's own for the site form. The site form has no source anchors, and
/// its assets are named where a build would put them, not copied.
/// `project_root` is the directory of `ascribe.toml`.
///
/// # Errors
///
/// `path` isn't a page of the project, or the page can't be written.
pub fn resolve(
    project: &Project,
    project_root: &Path,
    path: &RelPath,
    build: &Build,
    emit: Emit,
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
        emit,
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
    match emit {
        Emit::Plain => {
            // One page alone: a link keeps its page's HTML address, since
            // the anchors its fragment has in another page's Markdown are
            // the build's to work out (decision #1023).
            let plain = PlainEmitter::new(project.model()).with_html_links();
            let emitted = emit_page(&plain, &cx, &page)?;
            if frontmatter && let Some(value) = page.frontmatter.as_ref().filter(|v| !v.is_null()) {
                // A value parsed from YAML always writes back as YAML; were it
                // not to, the page is still worth showing without its
                // frontmatter.
                let yaml = serde_yaml_ng::to_string(value).unwrap_or_default();
                answer.text.push_str("---\n");
                answer.text.push_str(&yaml);
                if !yaml.ends_with('\n') {
                    answer.text.push('\n');
                }
                answer.text.push_str("---\n\n");
            }
            answer.text.push_str(&emitted.text);
        }
        Emit::Site => {
            // The site output's file, as `ascribe build --emit site` writes
            // it: its frontmatter block, then the body.
            let emitted = emit_page(&SiteEmitter::new(project.model()), &cx, &page)?;
            answer.text = if frontmatter {
                emitted.text
            } else {
                split_frontmatter(&emitted.text).1.to_owned()
            };
        }
    }
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
    Found(Box<ResolvedPermalink>),
    /// The permalink is retired.
    Retired {
        /// Why, as `ascribe.addresses` writes it.
        reason: String,
    },
    /// The permalink was moved to a page.
    Moved {
        /// Where its address leads now, as `ascribe.addresses` writes it.
        to: String,
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
            PermalinkHelp::Found(Box::new(ResolvedPermalink {
                schema_version: SCHEMA_VERSION,
                ascribe_version: ASCRIBE_VERSION,
                version,
                build: build.to_owned(),
                line: line.map(str::to_owned),
                name: name.to_owned(),
                entry,
            }))
        }
        Help::Retired { reason } => PermalinkHelp::Retired { reason },
        Help::Moved { to } => PermalinkHelp::Moved { to },
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    /// The site form of every page of Quill, for each of its builds and
    /// lines, is the file `emit_page` writes for it with the build's
    /// `SiteEmitter`: whole with its frontmatter, and its body without.
    #[test]
    fn the_site_form_is_the_site_outputs_file() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
        let loaded = ascribe_check::load(&root.join("ascribe.toml"))
            .map(|w| ascribe_check::Project::new(w.snapshot()))
            .expect("Quill loads");
        let index = loaded.index();
        let mut compared = 0;
        for build in &loaded.model().builds {
            let project = index.for_build(build);
            let project = project.as_ref();
            let router = AstroRouter::from_consumer(&project.model().consumer);
            let cx = EmitContext::new(project, &root, build);
            let emitter = SiteEmitter::new(project.model());
            let paths: Vec<RelPath> = index.pages().map(|file| file.path.clone()).collect();
            for path in paths {
                let whole = resolve(index, &root, &path, build, Emit::Site, true).unwrap();
                let body = resolve(index, &root, &path, build, Emit::Site, false).unwrap();
                assert_eq!(whole.emit, Emit::Site);
                if whole.not_published.is_some() {
                    assert!(whole.text.is_empty() && body.text.is_empty());
                    continue;
                }
                let page = project.resolve_page(&path, build, &router).unwrap();
                let file = emit_page(&emitter, &cx, &page).unwrap().text;
                assert_eq!(whole.text, file, "{path} in {}", build.name);
                assert_eq!(
                    body.text,
                    split_frontmatter(&file).1,
                    "{path} in {}",
                    build.name
                );
                assert!(!body.text.starts_with("---\n"), "{path} in {}", build.name);
                compared += 1;
            }
        }
        assert!(compared > 10, "only {compared} pages compared");
    }

    /// With the outputs for agents on, the plain form keeps a link's HTML
    /// address, as it did before the agents' Markdown linked to Markdown: a
    /// link to a heading with an `@id` keeps the page id.
    #[test]
    fn the_plain_form_links_to_pages_by_their_html_address() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/astro-site");
        let loaded = ascribe_check::load(&root.join("ascribe.toml"))
            .map(|w| ascribe_check::Project::new(w.snapshot()))
            .expect("the example loads");
        let index = loaded.index();
        let build = &loaded.model().builds[0];
        let path = RelPath::parse("Reference/Options.md").unwrap();
        let text = resolve(index, &root, &path, build, Emit::Plain, false)
            .unwrap()
            .text;
        assert!(
            text.contains("(https://docs.example.com/docs/guides/my-setup#weave-config)"),
            "{text}"
        );
        assert!(!text.contains("my-setup.md"), "{text}");
    }
}
