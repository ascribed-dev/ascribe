//! A page's title, type, and the headings a link to it can name, with their
//! ids and lines, so a `page.md#id` link is right the first time. The
//! headings are the ones the editor's completion offers after `page.md#`
//! ([`Project::link_headings`]): the page's own and those that arrive
//! through the fragments it includes. The page and each heading list the
//! memo files about them (SPEC §7.7), placed or outdated.

use std::collections::HashSet;

use ascribe_core::RelPath;
use ascribe_model::Build;
use ascribe_resolve::{AstroRouter, FileKind, MemoFile, MemoState, Project, Usable, place};
use serde::Serialize;

use crate::lines::{Lines, project_path};
use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// What `ascribe outline <PAGE>` answers.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Outline {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The page's path from the content root, as links write it.
    pub page: String,
    /// The page's file from the project root, as `ascribe check` reports it.
    pub file: String,
    /// Whether it's a fragment, which pages include rather than link to.
    pub fragment: bool,
    /// Its title (frontmatter `title`).
    pub title: Option<String>,
    /// Its content type; `null` for a fragment, or when no one type applies.
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    /// The build the headings are limited to, with `--build`.
    pub build: Option<String>,
    /// Why that build doesn't publish the page, when it doesn't; there are
    /// no headings then.
    pub not_published: Option<String>,
    /// The memo files about the page as a whole, in path order. A detached
    /// memo isn't listed.
    pub memos: Vec<OutlineMemo>,
    /// The headings a link to the page can name, in document order, each id
    /// once.
    pub headings: Vec<OutlineHeading>,
}

/// A heading a link can name.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct OutlineHeading {
    /// 1 to 6.
    pub level: u8,
    /// Its text, phrases replaced by their values.
    pub text: String,
    /// Its id: what a link writes after `#`.
    pub id: String,
    /// Whether the id is the heading's `@id`, which stays when its text
    /// changes.
    pub explicit_id: bool,
    /// The file it's written in, from the project root.
    pub file: String,
    /// Its line in that file, from 1.
    pub line: u32,
    /// The fragment it comes from, as a path from the content root; `null`
    /// for the page's own.
    pub fragment: Option<String>,
    /// The memo files about its section, or a block in it, in path order.
    pub memos: Vec<OutlineMemo>,
}

/// A memo file about a page or a heading.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct OutlineMemo {
    /// The memo file, from the project root.
    pub file: String,
    /// `placed`, on what its `about` names or the block it quotes, or
    /// `outdated`, when no block holds its quote.
    pub state: OutlineMemoState,
    /// Its `quote`; `null` when it has none.
    pub quote: Option<String>,
    /// The first line of its text that isn't blank.
    pub first_line: String,
    /// Whether it's one of the memos kept on this machine only, rather than
    /// the project's `memos/`.
    pub personal: bool,
}

/// Where a listed memo is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum OutlineMemoState {
    /// On its page or section, or the block its quote is in.
    Placed,
    /// Its quote isn't found: it's on its page or section, and no block.
    Outdated,
}

/// Each memo of `memos` that's placed or outdated, with what it's about.
pub(crate) fn placed_memos<'m>(
    project: &Project,
    memos: &'m [MemoFile],
) -> Vec<(Usable, &'m MemoFile, MemoState)> {
    memos
        .iter()
        .filter_map(|memo| {
            let placement = place(memo, project)?;
            Some((placement.target?, memo, placement.state))
        })
        .collect()
}

/// The memos of `placed` about `target`, as the outline lists them.
fn listed(placed: &[(Usable, &MemoFile, MemoState)], target: &Usable) -> Vec<OutlineMemo> {
    placed
        .iter()
        .filter(|(about, _, _)| about == target)
        .map(|(_, memo, state)| OutlineMemo {
            file: memo.path.to_string(),
            state: match state {
                MemoState::Outdated => OutlineMemoState::Outdated,
                MemoState::Placed | MemoState::Detached => OutlineMemoState::Placed,
            },
            quote: memo.quote.as_ref().map(|q| q.value.clone()),
            first_line: memo.first_line().to_owned(),
            personal: false,
        })
        .collect()
}

/// The outline of a source file, at content path `path`, with the memos of
/// `memos` about it; with `build`, only the headings that build publishes.
///
/// # Errors
///
/// `path` isn't one of the project's source files.
pub fn outline(
    project: &Project,
    path: &RelPath,
    build: Option<&Build>,
    memos: &[MemoFile],
) -> Result<Outline, QueryError> {
    let file = project.file(path).ok_or_else(|| QueryError::NotASource {
        path: path.to_string(),
    })?;
    let fragment = file.kind == FileKind::Fragment;
    let mut answer = Outline {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        page: path.to_string(),
        file: project_path(project, path),
        fragment,
        title: file.title.clone(),
        content_type: file.content_type(project.model()).map(|t| t.name.clone()),
        build: build.map(|b| b.name.clone()),
        not_published: None,
        memos: Vec::new(),
        headings: Vec::new(),
    };
    let placed = placed_memos(project, memos);
    answer.memos = listed(&placed, &Usable::File(path.clone()));
    // With a build, the headings it keeps, by where they're written.
    let kept: Option<HashSet<(RelPath, usize)>> = match build {
        Some(build) if !fragment => {
            if let Some(reason) = project.dropped(path, build) {
                answer.not_published = Some(format!(
                    "build `{}` doesn't publish `{path}`: {}",
                    build.name,
                    reason.explanation()
                ));
                return Ok(answer);
            }
            let router = AstroRouter::from_consumer(&project.model().consumer);
            let mut kept = HashSet::new();
            if let Some(page) = project.resolve_page(path, build, &router) {
                page.visit(&mut |block| {
                    if block.heading.is_some()
                        && let Some(written_in) = project.path_of(block.file)
                    {
                        kept.insert((written_in.clone(), block.span.start()));
                    }
                });
            }
            Some(kept)
        }
        _ => None,
    };
    let mut lines = Lines::new(project);
    for (written_in, heading) in project.link_headings(file) {
        let at = (written_in.clone(), heading.span.start());
        if kept.as_ref().is_some_and(|kept| !kept.contains(&at)) {
            continue;
        }
        let (line, _) = lines.at(&written_in, heading.span.start());
        answer.headings.push(OutlineHeading {
            level: heading.level,
            text: heading.text.clone(),
            id: heading.source_id.clone(),
            explicit_id: heading.explicit_id.is_some(),
            file: project_path(project, &written_in),
            line,
            fragment: (written_in != *path).then(|| written_in.to_string()),
            memos: listed(
                &placed,
                &Usable::Heading {
                    file: written_in.clone(),
                    id: heading.source_id.clone(),
                },
            ),
        });
    }
    Ok(answer)
}
