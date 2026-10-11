//! The targets a broken link most likely meant: what `ascribe link` lists as
//! closest, and what `ascribe check` offers as the link's fixes, so the two
//! agree.

use std::collections::BTreeSet;

use ascribe_core::{Applicability, FileId, Fix, RelPath, Span, TextEdit};
use ascribe_model::edit_distance;

use crate::index::{FileKind, Target};
use crate::links::{encode_destination, link_path};
use crate::project::{Missing, Project, Resolution};

/// How many targets are suggested for one link.
const CLOSEST: usize = 5;

/// A target a link could name instead of the one it does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Closest {
    /// The page or file, as a content path.
    pub path: RelPath,
    /// The heading id, when it's a heading.
    pub id: Option<String>,
    /// The destination to write on the page the link is on.
    pub href: String,
}

/// The targets a link written on `from` most likely meant, best first, when
/// `resolution` says it doesn't work; none when it does, or for a route,
/// whose page is the route check's fix.
///
/// - A missing page: a file whose name differs only in case, then the pages
///   with a heading of the link's `#id`, wherever they are and whatever
///   they're called (a page that was renamed or moved keeps its headings),
///   then pages within a few edits of the path or of its file name.
/// - A heading the page lacks: the page's headings, closest id first.
/// - A fragment: the pages that include it.
/// - A file that isn't a page, named with the wrong case: the file.
pub fn closest_targets(
    project: &Project,
    from: &RelPath,
    target: &Target,
    resolution: &Resolution,
) -> Vec<Closest> {
    let (written, fragment) = match target {
        Target::Local(local) => (local.written.as_str(), local.fragment.as_deref()),
        Target::External => return Vec::new(),
    };
    match resolution {
        Resolution::Source {
            target: path,
            fragment: true,
            ..
        } => project
            .including_pages(path)
            .iter()
            .take(CLOSEST)
            .map(|page| closest(from, page, None))
            .collect(),
        Resolution::Source {
            target: path,
            id: Some(id),
            ..
        } if project.page_heading(path, id).is_none() => closest_headings(project, from, path, id),
        Resolution::SourceMissing { actual } => {
            closest_pages(project, from, written, fragment, actual.as_ref())
        }
        Resolution::AssetMissing(Missing::Case(actual)) => vec![Closest {
            path: actual.clone(),
            id: None,
            href: encode_destination(&link_path(actual, from)),
        }],
        _ => Vec::new(),
    }
}

/// The fixes a broken link is offered: one for each of its
/// [`closest_targets`], which replaces the destination written at
/// `destination` in `file`. None when the destination isn't written in the
/// link itself (a reference form's is in its definition).
pub fn closest_fixes(
    project: &Project,
    from: &RelPath,
    target: &Target,
    resolution: &Resolution,
    file: FileId,
    destination: Option<Span>,
) -> Vec<Fix> {
    let Some(span) = destination else {
        return Vec::new();
    };
    closest_targets(project, from, target, resolution)
        .into_iter()
        .map(|closest| Fix {
            title: format!("Link to `{}`", closest.href),
            file,
            edits: vec![TextEdit::replace(span, closest.href)],
            // A guess at what the link meant, which only the author can
            // confirm.
            applicability: Applicability::Unsafe,
        })
        .collect()
}

/// The destination of a link from `from` to `path`, and the heading `id` on
/// it: `#id` on the page itself. What the editor's completion writes.
pub fn link_href(from: &RelPath, path: &RelPath, id: Option<&str>) -> String {
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

fn closest(from: &RelPath, path: &RelPath, id: Option<&str>) -> Closest {
    Closest {
        href: link_href(from, path, id),
        path: path.clone(),
        id: id.map(str::to_owned),
    }
}

/// The headings of the page at `path` whose ids are closest to `id`: the
/// page exists, so its headings are the likely targets, however far.
fn closest_headings(project: &Project, from: &RelPath, path: &RelPath, id: &str) -> Vec<Closest> {
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
        .take(CLOSEST)
        .map(|(_, id)| closest(from, path, Some(&id)))
        .collect()
}

/// The pages a link to the missing page `written`, with the heading `id`,
/// most likely meant: a file whose name differs only in case first, then
/// pages with a heading whose id is `id`, then pages within a few edits of
/// the path or of its file name. Each keeps the `#id` when it has the
/// heading.
fn closest_pages(
    project: &Project,
    from: &RelPath,
    written: &str,
    id: Option<&str>,
    actual: Option<&RelPath>,
) -> Vec<Closest> {
    let written = written.trim_start_matches("./");
    let name = |path: &str| path.rsplit('/').next().unwrap_or(path).to_owned();
    let limit = (written.chars().count() / 3).max(3);
    let headed = id
        .map(|id| pages_with_heading(project, id))
        .unwrap_or_default();
    let has_heading = |path: &RelPath| headed.contains(path);
    // Every page but the case twin, scored by how far its path is from the
    // one written.
    let mut scored: Vec<(usize, &RelPath)> = project
        .pages()
        .filter(|f| f.kind == FileKind::Page && Some(&f.path) != actual)
        .map(|f| {
            let candidate = link_path(&f.path, from);
            let distance = edit_distance(written, &candidate)
                .min(edit_distance(&name(written), &name(&candidate)));
            (distance, &f.path)
        })
        .collect();
    scored.sort();
    let with_heading = scored
        .iter()
        .filter(|(_, path)| has_heading(path))
        .map(|(_, path)| *path);
    let near = scored
        .iter()
        .filter(|(distance, path)| *distance <= limit && !has_heading(path))
        .map(|(_, path)| *path);
    actual
        .into_iter()
        .chain(with_heading)
        .chain(near)
        .take(CLOSEST)
        .map(|path| closest(from, path, id.filter(|_| has_heading(path))))
        .collect()
}

/// The pages with a heading whose id is `id`, their own or from a fragment
/// they include. Only the files that have such a heading are expanded, so
/// looking costs little in a large project.
fn pages_with_heading(project: &Project, id: &str) -> BTreeSet<RelPath> {
    let mut candidates: BTreeSet<RelPath> = BTreeSet::new();
    for file in project.files().filter(|f| f.heading_by_id(id).is_some()) {
        match file.kind {
            FileKind::Page => {
                candidates.insert(file.path.clone());
            }
            FileKind::Fragment => candidates.extend(project.including_pages(&file.path)),
        }
    }
    // An include of one section of a fragment may leave the heading out.
    candidates.retain(|page| project.page_heading(page, id).is_some());
    candidates
}
