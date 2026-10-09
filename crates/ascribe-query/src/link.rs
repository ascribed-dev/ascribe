//! Whether a link target exists as seen from a page, its title, and the link
//! to write. A target is resolved as the source index resolves every link
//! ([`resolve_reference`]), the resolution the editor and `ascribe check`
//! use, so the three agree; the link to write is the one the editor's
//! completion writes ([`link_path`], [`encode_destination`]).

use ascribe_core::RelPath;
use ascribe_model::edit_distance;
use ascribe_resolve::{
    FileKind, FileSystem, Missing, Project, RefKind, Resolution, Target, encode_destination,
    link_path, reference_target, resolve_reference,
};
use serde::Serialize;

use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// How many targets an answer suggests when the one asked about doesn't
/// exist.
const SUGGESTIONS: usize = 5;

/// What `ascribe link <TARGET> --from <PAGE>` answers.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct LinkAnswer {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The page the link is written on, as a path from the content root.
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
/// files that aren't sources are found in.
///
/// # Errors
///
/// `from` isn't one of the project's source files.
pub fn link(
    project: &Project,
    fs: &dyn FileSystem,
    from: &RelPath,
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
        from: from.to_string(),
        target: target.to_owned(),
        exists: false,
        kind: LinkKind::Missing,
        path: None,
        id: None,
        title: None,
        href: None,
        problem: None,
        closest: Vec::new(),
    };
    let written = match &parsed {
        Target::Local(local) => local.written.clone(),
        Target::External => String::new(),
    };
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
            answer.closest = project
                .including_pages(&path)
                .iter()
                .take(SUGGESTIONS)
                .map(|page| suggestion(project, from, page, None))
                .collect();
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
                answer.href = Some(href(from, &path, Some(&id)));
            } else {
                answer.problem = Some(format!("`{path}` has no heading with the id `{id}`"));
                answer.closest = closest_headings(project, from, &path, &id);
            }
        }
        Resolution::Source { target: path, .. } => {
            answer.exists = true;
            answer.kind = LinkKind::Page;
            answer.title = project.title_for(&path, None).map(str::to_owned);
            answer.href = Some(href(from, &path, None));
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
        Resolution::SourceMissing { actual } => {
            answer.problem = Some(format!("`{written}` doesn't exist"));
            answer.closest = closest_pages(project, from, &written, actual.as_ref());
        }
        Resolution::AssetMissing(missing) => {
            let actual = match &missing {
                Missing::Case(actual) => Some(actual.clone()),
                _ => None,
            };
            answer.problem = Some(match missing {
                Missing::Outside => {
                    format!("`{written}` is outside the project, so a build can't copy it")
                }
                _ => format!("`{written}` doesn't exist"),
            });
            if let Some(actual) = actual {
                answer.closest.push(LinkSuggestion {
                    href: encode_destination(&link_path(&actual, from)),
                    path: actual.to_string(),
                    id: None,
                    title: None,
                });
            }
        }
        Resolution::Route {
            page, page_exists, ..
        } => {
            answer.problem = Some(format!(
                "this looks like the published route of `{page}`; link to the file instead"
            ));
            if page_exists && let Ok(page) = RelPath::parse(&page) {
                answer.closest.push(suggestion(project, from, &page, None));
            }
        }
    }
    Ok(answer)
}

/// The destination of a link from `from` to `path`, and the heading `id` on
/// it: `#id` on the page itself.
fn href(from: &RelPath, path: &RelPath, id: Option<&str>) -> String {
    match id {
        Some(id) if path == from => format!("#{}", encode_destination(id)),
        Some(id) => format!(
            "{}#{}",
            encode_destination(&link_path(path, from)),
            encode_destination(id)
        ),
        None => encode_destination(&link_path(path, from)),
    }
}

fn suggestion(
    project: &Project,
    from: &RelPath,
    path: &RelPath,
    id: Option<&str>,
) -> LinkSuggestion {
    LinkSuggestion {
        href: href(from, path, id),
        path: path.to_string(),
        id: id.map(str::to_owned),
        title: project.title_for(path, id).map(str::to_owned),
    }
}

/// The headings of the page at `path` whose ids are closest to `id`: the
/// page exists, so its headings are the likely targets, however far.
fn closest_headings(
    project: &Project,
    from: &RelPath,
    path: &RelPath,
    id: &str,
) -> Vec<LinkSuggestion> {
    let Some(file) = project.file(path) else {
        return Vec::new();
    };
    let mut scored: Vec<(usize, String)> = project
        .link_headings(file)
        .into_iter()
        .map(|(_, h)| (edit_distance(id, &h.source_id), h.source_id.clone()))
        .collect();
    scored.sort();
    scored
        .into_iter()
        .take(SUGGESTIONS)
        .map(|(_, id)| suggestion(project, from, path, Some(&id)))
        .collect()
}

/// The pages whose paths, written from `from`, are closest to `written`: a
/// file whose name differs only in case first, then pages within a few
/// edits of the path or of its file name.
fn closest_pages(
    project: &Project,
    from: &RelPath,
    written: &str,
    actual: Option<&RelPath>,
) -> Vec<LinkSuggestion> {
    let written = written.trim_start_matches("./");
    let name = |path: &str| path.rsplit('/').next().unwrap_or(path).to_owned();
    let limit = (written.chars().count() / 3).max(3);
    let mut scored: Vec<(usize, RelPath)> = project
        .pages()
        .filter(|f| f.kind == FileKind::Page && Some(&f.path) != actual)
        .map(|f| {
            let candidate = link_path(&f.path, from);
            let distance = edit_distance(written, &candidate)
                .min(edit_distance(&name(written), &name(&candidate)));
            (distance, f.path.clone())
        })
        .filter(|(distance, _)| *distance <= limit)
        .collect();
    scored.sort();
    actual
        .cloned()
        .into_iter()
        .chain(scored.into_iter().map(|(_, path)| path))
        .take(SUGGESTIONS)
        .map(|path| suggestion(project, from, &path, None))
        .collect()
}
