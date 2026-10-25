//! Whether a link target exists as seen from a page, its title, and the link
//! to write. A target is resolved as the source index resolves every link
//! ([`resolve_reference`]), the resolution the editor and `ascribe check`
//! use, so the three agree; the link to write is the one the editor's
//! completion writes ([`link_path`], [`encode_destination`]).

use ascribe_core::RelPath;
use ascribe_resolve::{
    Closest, DefaultRouter, FileSystem, Missing, Project, RefKind, Resolution, Target,
    closest_targets, encode_destination, link_href, link_path, reference_target, resolve_reference,
};
use serde::Serialize;

use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// What `ascribe link <TARGET> --from <PAGE>` answers.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct LinkAnswer {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The page the link is written on, as the command was given it: a path
    /// from the folder it was run in, as `ascribe check` names a file.
    pub from: String,
    /// The target, as given.
    pub target: String,
    /// Whether a link from that page to the target works. An external URL
    /// counts as existing; it isn't checked.
    pub exists: bool,
    /// What the target is.
    pub kind: LinkKind,
    /// The file it names, as a path from the content root; `null` for an
    /// external URL, or a file that doesn't exist.
    pub path: Option<String>,
    /// The heading id after `#`, if any.
    pub id: Option<String>,
    /// The page's title, or the heading's text: what a link with no text
    /// shows.
    pub title: Option<String>,
    /// The destination to write on that page, when the target exists.
    pub href: Option<String>,
    /// Why it doesn't work, when it doesn't.
    pub problem: Option<String>,
    /// The closest targets that exist, best first, when it doesn't.
    pub closest: Vec<LinkSuggestion>,
    /// Where a link to a page goes in each line of each versioned build,
    /// newest first; absent when the target isn't a page or no build is
    /// versioned.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<LinkLine>,
}

/// Where a link goes in one line of a versioned build (SPEC §5.2).
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct LinkLine {
    /// The line's build: the declared build for the current line, and
    /// `<build>@<line>` for each other.
    pub build: String,
    /// The line, as `[releases.<target>]` writes it.
    pub line: String,
    /// The page it lands on, as a path from the content root: the target,
    /// or the page that takes its place in the line. `null` when no live
    /// line publishes the target, which is `link-page-dropped`.
    pub page: Option<String>,
    /// For a link out of the line, the line it leads to.
    pub leads_to: Option<String>,
}

/// What a link target is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    /// A page.
    Page,
    /// A heading on a page.
    Heading,
    /// A fragment, which pages include and links can't name.
    Fragment,
    /// A file that isn't a page, which a build copies: an image or a
    /// download.
    File,
    /// A URL with a scheme.
    External,
    /// Nothing: no file has that path.
    Missing,
}

/// A target a link could name instead.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct LinkSuggestion {
    /// The destination to write.
    pub href: String,
    /// The page, as a path from the content root.
    pub path: String,
    /// The heading id, if it's a heading.
    pub id: Option<String>,
    /// The page's title, or the heading's text.
    pub title: Option<String>,
}

/// Whether `target`, written on the source file at content path `from`,
/// names something that exists; `fs` is the project's file system, which
/// files that aren't sources are found in. `named` is the page as the
/// caller names it back, in [`LinkAnswer::from`].
///
/// # Errors
///
/// `from` isn't one of the project's source files.
pub fn link(
    project: &Project,
    fs: &dyn FileSystem,
    from: &RelPath,
    named: &str,
    target: &str,
) -> Result<LinkAnswer, QueryError> {
    if project.file(from).is_none() {
        return Err(QueryError::NotASource {
            path: from.to_string(),
        });
    }
    let model = project.model();
    let parsed = reference_target(RefKind::Link, target, &[], from, model);
    let resolution = resolve_reference(
        RefKind::Link,
        &parsed,
        from,
        model,
        project,
        fs,
        project.layout(),
    );
    let mut answer = LinkAnswer {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        from: named.to_owned(),
        target: target.to_owned(),
        exists: false,
        kind: LinkKind::Missing,
        path: None,
        id: None,
        title: None,
        href: None,
        problem: None,
        closest: Vec::new(),
        lines: Vec::new(),
    };
    let written = match &parsed {
        Target::Local(local) => local.written.clone(),
        Target::External => String::new(),
    };
    let closest = closest_targets(project, from, &parsed, &resolution);
    match resolution {
        Resolution::External => {
            answer.exists = true;
            answer.kind = LinkKind::External;
            answer.href = Some(target.to_owned());
        }
        Resolution::Source {
            target: path,
            fragment: true,
            ..
        } => {
            answer.kind = LinkKind::Fragment;
            answer.problem = Some(format!(
                "`{path}` is a fragment, which isn't published on its own; include it with `@include`, or link to a page that includes it"
            ));
            answer.path = Some(path.to_string());
        }
        Resolution::Source {
            target: path,
            id: Some(id),
            ..
        } => {
            answer.path = Some(path.to_string());
            answer.id = Some(id.clone());
            answer.kind = LinkKind::Heading;
            if project.page_heading(&path, &id).is_some() {
                answer.exists = true;
                answer.title = project.title_for(&path, Some(&id)).map(str::to_owned);
                answer.href = Some(link_href(from, &path, Some(&id)));
            } else {
                answer.problem = Some(format!("`{path}` has no heading with the id `{id}`"));
            }
        }
        Resolution::Source { target: path, .. } => {
            answer.exists = true;
            answer.kind = LinkKind::Page;
            answer.title = project.title_for(&path, None).map(str::to_owned);
            answer.href = Some(link_href(from, &path, None));
            answer.path = Some(path.to_string());
        }
        Resolution::Asset { path, fragment } => {
            answer.exists = true;
            answer.kind = LinkKind::File;
            let mut written = encode_destination(&link_path(&path, from));
            if let Some(fragment) = fragment {
                written.push('#');
                written.push_str(&fragment);
            }
            answer.href = Some(written);
            answer.path = Some(path.to_string());
        }
        // An operation's own answer waits for its decision; until then it's
        // the spec, a file, as a link to it with a fragment always was, and
        // the operation is checked.
        Resolution::Operation { api, id } => {
            answer.kind = LinkKind::File;
            answer.path = match &parsed {
                Target::Local(local) => local.path.as_ref().map(ToString::to_string),
                Target::External => None,
            };
            answer.id = Some(id.clone());
            let spec = project.apis().spec(&api);
            if spec.is_some_and(|s| s.operation(&id).is_some()) {
                answer.exists = true;
                answer.href = Some(format!("{written}#{}", encode_destination(&id)));
            } else if spec.is_some() {
                answer.problem = Some(format!("`{written}` has no operation `{id}`"));
            } else {
                answer.problem = Some(format!("the spec of API `{api}` can't be read"));
            }
        }
        Resolution::SpecOutside { api } => {
            answer.kind = LinkKind::File;
            answer.problem = Some(format!(
                "`{written}` is the spec of API `{api}`, outside the project's folder; link to one of its operations with `#` and its `operationId`"
            ));
        }
        Resolution::SourceMissing { .. } => {
            answer.problem = Some(format!("`{written}` doesn't exist"));
        }
        Resolution::AssetMissing(missing) => {
            answer.problem = Some(match missing {
                Missing::Outside => {
                    format!("`{written}` is outside the project, so a build can't copy it")
                }
                _ => format!("`{written}` doesn't exist"),
            });
        }
        Resolution::Route {
            page, page_exists, ..
        } => {
            answer.problem = Some(format!(
                "this looks like the published route of `{page}`; link to the file instead"
            ));
            if page_exists && let Ok(page) = RelPath::parse(&page) {
                let closest = Closest {
                    href: link_href(from, &page, None),
                    path: page,
                    id: None,
                };
                answer.closest.push(suggested(project, closest));
            }
        }
    }
    answer
        .closest
        .extend(closest.into_iter().map(|c| suggested(project, c)));
    if answer.exists
        && matches!(answer.kind, LinkKind::Page | LinkKind::Heading)
        && let Some(path) = answer.path.as_deref().and_then(|p| RelPath::parse(p).ok())
    {
        answer.lines = lines(project, &path);
    }
    Ok(answer)
}

/// Where a link to `target` goes in each line of each versioned build.
fn lines(project: &Project, target: &RelPath) -> Vec<LinkLine> {
    let router = DefaultRouter::from_consumer(&project.model().consumer);
    project
        .model()
        .builds
        .iter()
        .filter_map(|build| {
            let line = build.line.as_ref()?;
            let landing = project.resolver(build, &router).link_landing(target);
            Some(LinkLine {
                build: build.name.clone(),
                line: line.line.text.clone(),
                page: landing.as_ref().map(|(page, _)| page.to_string()),
                leads_to: landing
                    .and_then(|(_, to)| to)
                    .and_then(|to| to.line.as_ref())
                    .map(|l| l.line.text.clone()),
            })
        })
        .collect()
}

/// A closest target, with the title a link with no text shows: none for a
/// file that isn't a source.
fn suggested(project: &Project, closest: Closest) -> LinkSuggestion {
    let title = project
        .title_for(&closest.path, closest.id.as_deref())
        .map(str::to_owned);
    LinkSuggestion {
        href: closest.href,
        path: closest.path.to_string(),
        id: closest.id,
        title,
    }
}
