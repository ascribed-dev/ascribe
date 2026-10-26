//! The targets a broken link most likely meant: what `ascribe link` lists as
//! closest, and what `ascribe check` offers as the link's fixes, so the two
//! agree.

use std::collections::BTreeSet;

use ascribe_core::{Applicability, Arg, FileId, Fix, Issue, RelPath, Span, TextEdit, diagnostics};
use ascribe_model::{ApiReference, edit_distance, edit_distance_within};

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
/// - A heading the page lacks: the page's headings, those the id was cut
///   down from first, then those cut down to it, then the rest, each
///   closest id first (see [`closest_headings`]).
/// - A fragment: the pages that include it.
/// - A file that isn't a page, named with the wrong case: the file.
/// - An operation the spec lacks: the one it was renamed to, then the spec's
///   operations, closest first, by id, or by method and path for one with
///   no id.
pub fn closest_targets(
    project: &Project,
    from: &RelPath,
    target: &Target,
    resolution: &Resolution,
) -> Vec<Closest> {
    ranked(project, from, target, resolution).targets
}

/// [`closest_targets`], and whether the first clearly wins: it's the only
/// one of the best kind there is, such as the only heading whose id the
/// written one was cut down from.
struct Ranked {
    targets: Vec<Closest>,
    clear: bool,
}

impl Ranked {
    fn none() -> Ranked {
        Ranked {
            targets: Vec::new(),
            clear: false,
        }
    }

    /// Targets where the first wins when it's the only one.
    fn alone(targets: Vec<Closest>) -> Ranked {
        Ranked {
            clear: targets.len() == 1,
            targets,
        }
    }
}

fn ranked(project: &Project, from: &RelPath, target: &Target, resolution: &Resolution) -> Ranked {
    let (written, fragment) = match target {
        Target::Local(local) => (local.written.as_str(), local.fragment.as_deref()),
        Target::External => return Ranked::none(),
    };
    match resolution {
        Resolution::Source {
            target: path,
            fragment: true,
            ..
        } => Ranked::alone(
            project
                .including_pages(path)
                .iter()
                .take(CLOSEST)
                .map(|page| closest(from, page, None))
                .collect(),
        ),
        Resolution::Source {
            target: path,
            id: Some(id),
            ..
        } if project.page_heading(path, id).is_none() => closest_headings(project, from, path, id),
        Resolution::SourceMissing { actual } => {
            closest_pages(project, from, written, fragment, actual.as_ref())
        }
        Resolution::Operation { api, id } => {
            let (Some(spec), Target::Local(local)) = (project.apis().spec(api), target) else {
                return Ranked::none();
            };
            if spec.named(id).is_some() {
                return Ranked::none();
            }
            let by_path = project
                .model()
                .api(api)
                .is_some_and(|a| !matches!(a.reference, ApiReference::Template(_)));
            let path = local.path.clone().unwrap_or_default();
            // The operation check names its own closest.
            Ranked {
                targets: crate::openapi::closest_operations(project.apis(), api, id, by_path)
                    .into_iter()
                    .map(|closest| Closest {
                        path: path.clone(),
                        href: format!("{written}#{}", encode_destination(&closest)),
                        id: Some(closest),
                    })
                    .collect(),
                clear: false,
            }
        }
        Resolution::AssetMissing(Missing::Case(actual)) => Ranked::alone(vec![Closest {
            path: actual.clone(),
            id: None,
            href: encode_destination(&link_path(actual, from)),
        }]),
        _ => Ranked::none(),
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
        .map(|closest| link_fix(closest, file, span))
        .collect()
}

/// Offers a broken link's [`closest_fixes`] on `issue`, and, when one target
/// clearly wins, names it in the message of a `link-id-missing` or a
/// `link-target-missing` that has no other variant: "did you mean
/// `keys.md#rotate-keys`?". The concise output and the edit hook show only
/// the message, so the guess is there without asking for the fixes.
pub fn offer_closest(
    issue: &mut Issue,
    project: &Project,
    from: &RelPath,
    target: &Target,
    resolution: &Resolution,
    file: FileId,
    destination: Option<Span>,
) {
    let ranked = ranked(project, from, target, resolution);
    let named = [
        diagnostics::LINK_ID_MISSING,
        diagnostics::LINK_TARGET_MISSING,
    ];
    if ranked.clear
        && issue.variant.is_none()
        && named.contains(&issue.slug)
        && let Some(first) = ranked.targets.first()
    {
        issue.variant = Some("closest");
        issue.args.push(Arg {
            name: "closest",
            value: first.href.clone(),
        });
    }
    if let Some(span) = destination {
        issue.fixes = ranked
            .targets
            .into_iter()
            .map(|closest| link_fix(closest, file, span))
            .collect();
    }
}

/// The fix that replaces the destination at `span` with `closest`'s.
fn link_fix(closest: Closest, file: FileId, span: Span) -> Fix {
    Fix {
        title: format!("Link to `{}`", closest.href),
        file,
        edits: vec![TextEdit::replace(span, closest.href)],
        // A guess at what the link meant, which only the author can
        // confirm.
        applicability: Applicability::Unsafe,
    }
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
///
/// An id is often a guess from a heading's first words, so the headings
/// come in three kinds, each closest id first:
///
/// 1. those `id` was cut down from: each of its words starts one of theirs,
///    in order (`ask-instead` for `ask-instead-of-guessing`);
/// 2. those cut down to `id` (`sign-webhook-events` for `sign-webhooks`);
/// 3. the rest.
///
/// The first clearly wins when it's the only one of the first kind there
/// is, or, with none of the first two, the only one within a few edits and
/// [`near_enough`] to be the one meant.
fn closest_headings(project: &Project, from: &RelPath, path: &RelPath, id: &str) -> Ranked {
    let Some(file) = project.file(path) else {
        return Ranked::none();
    };
    let mut scored: Vec<(u8, usize, String)> = project
        .link_headings(file)
        .into_iter()
        .map(|(_, h)| {
            let candidate = &h.source_id;
            let kind = if shares_words(id, candidate) {
                0
            } else if shares_words(candidate, id) {
                1
            } else {
                2
            };
            (kind, edit_distance(id, candidate), candidate.clone())
        })
        .collect();
    scored.sort();
    let clear = match scored.as_slice() {
        [] => false,
        [(kind @ (0 | 1), ..), rest @ ..] => rest.first().is_none_or(|(k, ..)| k != kind),
        [(_, distance, _), rest @ ..] => {
            let limit = (id.chars().count() / 3).max(3);
            near_enough(*distance, id) && rest.first().is_none_or(|(_, d, _)| *d > limit)
        }
    };
    Ranked {
        targets: scored
            .into_iter()
            .take(CLOSEST)
            .map(|(_, _, id)| closest(from, path, Some(&id)))
            .collect(),
        clear,
    }
}

/// Whether a target `distance` edits from `written` is near enough to name
/// as the one meant: a third of its length at most, so `keys.md` is for
/// `key.md`, and `index.md` isn't for `new.md`.
fn near_enough(distance: usize, written: &str) -> bool {
    distance * 3 <= written.chars().count()
}

/// Whether each of `short`'s words starts one of `long`'s, in order, or one
/// of `long`'s starts it: `ask-instead` and `ask-instead-of-guessing`,
/// `sign-webhook` and `sign-webhooks-now`.
fn shares_words(short: &str, long: &str) -> bool {
    let words = |id: &str| -> Vec<String> {
        id.split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_owned)
            .collect()
    };
    let short = words(short);
    let mut long = words(long).into_iter();
    !short.is_empty()
        && short.iter().all(|word| {
            long.any(|other| other.starts_with(word.as_str()) || word.starts_with(other.as_str()))
        })
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
) -> Ranked {
    let written = written.trim_start_matches("./");
    let name = |path: &str| path.rsplit('/').next().unwrap_or(path).to_owned();
    let limit = (written.chars().count() / 3).max(3);
    let headed = id
        .map(|id| pages_with_heading(project, id))
        .unwrap_or_default();
    let has_heading = |path: &RelPath| headed.contains(path);
    let written_name = name(written);
    // Every page but the case twin, scored by how far its path is from the
    // one written. A page without the heading is only offered within the
    // limit, so its distance isn't worked out further.
    let mut scored: Vec<(usize, &RelPath)> = project
        .pages()
        .filter(|f| f.kind == FileKind::Page && Some(&f.path) != actual)
        .filter_map(|f| {
            let candidate = link_path(&f.path, from);
            let bound = if has_heading(&f.path) {
                usize::MAX
            } else {
                limit
            };
            let distance = edit_distance_within(written, &candidate, bound)
                .into_iter()
                .chain(edit_distance_within(
                    &written_name,
                    &name(&candidate),
                    bound,
                ))
                .min()?;
            Some((distance, &f.path))
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
    // The first clearly wins when it's the case twin, the only page with the
    // heading, or, with none, the only page near, and near enough.
    let clear = actual.is_some()
        || match with_heading.clone().count() {
            0 => match scored.as_slice() {
                [(distance, _)] => near_enough(*distance, &written_name),
                _ => false,
            },
            count => count == 1,
        };
    let targets = actual
        .into_iter()
        .chain(with_heading)
        .chain(near)
        .take(CLOSEST)
        .map(|path| closest(from, path, id.filter(|_| has_heading(path))))
        .collect();
    Ranked { targets, clear }
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
