//! The content checks that need every file at once: what nothing uses, what
//! nothing links to, and image files.
//!
//! Two kinds, by when they can be answered:
//!
//! - **For every build**, from the source index, so they're file level:
//!   `fragment-unused`, `phrase-unused`, `feature-unused`, and
//!   `glossary-term-unused` count uses with the search Find All References
//!   and the editor's inventory make (`ascribe_resolve::Project::use_counts`),
//!   so a count is the same everywhere; `image-unused` and `image-large` read
//!   the image files under the content root, through the project's
//!   [`FileSystem`](ascribe_resolve::FileSystem).
//! - **Per build**, from the build's resolved pages, so they're page level:
//!   `page-orphan`, a page no other page the build publishes links to or
//!   includes, and `title-duplicate`, two pages the build publishes with one
//!   title. [`PageChecker`](crate::PageChecker) runs them on each build it
//!   resolves.
//!
//! Each is advice, and a project can set its level or turn it off in
//! `[checks]`; one turned off isn't run.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use ascribe_core::intended::{ENTRY_ARG, EntryKind, entry_arg};
use ascribe_core::{DiagnosticSlug, FileId, Issue, Location, RelPath, Span, diagnostics};
use ascribe_model::{Checks, GlossaryMatch};
use ascribe_resolve::{FileIndex, LinkTarget, ResolvedPage, Usable};

use crate::page::Found;
use crate::project::image_id;
use crate::yaml::YamlIndex;
use crate::{Diagnostic, Project};

/// The extensions of the files the image checks look at, in lowercase.
pub const IMAGE_EXTENSIONS: &[&str] = &[
    "apng", "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "tif", "tiff", "webp",
];

/// The size over which `image-large` reports an image when the project sets
/// no `limit`: 500 KB.
pub const IMAGE_LIMIT: u64 = 500_000;

/// The checks of this module that run for every build.
const FILE_LEVEL: &[DiagnosticSlug] = &[
    diagnostics::FRAGMENT_UNUSED,
    diagnostics::PHRASE_UNUSED,
    diagnostics::FEATURE_UNUSED,
    diagnostics::GLOSSARY_TERM_UNUSED,
    diagnostics::IMAGE_UNUSED,
    diagnostics::IMAGE_LARGE,
];

/// The checks of this module the editor doesn't report: an image isn't a
/// file it shows problems in. `ascribe check --editor-build` leaves them out
/// too.
pub const NOT_IN_EDITOR: &[DiagnosticSlug] = &[diagnostics::IMAGE_UNUSED, diagnostics::IMAGE_LARGE];

/// Whether any of the checks that run for every build is on, so the source
/// index is worth making for them.
pub(crate) fn any_file_level(checks: &Checks) -> bool {
    FILE_LEVEL.iter().any(|slug| !checks.is_off(*slug))
}

/// What the checks that run for every build found, by file: the content
/// model's entries under its id, each unused fragment under its own, and the
/// image files' under theirs, each list in source order. `index` is the
/// project's source index; `file` gives the project's id of one of its files.
/// The image checks run only when `images` is set.
pub(crate) fn for_every_build(
    project: &Project,
    index: &ascribe_resolve::Project,
    file: &dyn Fn(FileId) -> FileId,
    images: bool,
) -> BTreeMap<FileId, Vec<Diagnostic>> {
    let checks = &project.model().checks;
    let on =
        |slug: DiagnosticSlug| !checks.is_off(slug) && (images || !NOT_IN_EDITOR.contains(&slug));
    let mut issues: Vec<Issue> = Vec::new();

    if on(diagnostics::FRAGMENT_UNUSED) {
        for fragment in index.fragments() {
            if index.includers(&fragment.path).is_empty() {
                let at = Location::new(file(fragment.file), first_line(&fragment.source));
                issues.push(Issue::new(diagnostics::FRAGMENT_UNUSED, at));
            }
        }
    }

    let entries = [
        diagnostics::PHRASE_UNUSED,
        diagnostics::FEATURE_UNUSED,
        diagnostics::GLOSSARY_TERM_UNUSED,
    ];
    if entries.into_iter().any(on) {
        let counts = index.use_counts();
        let unused = |used: Usable| !counts.contains_key(&used);
        let model = project.model();
        let at = |span: Span| Location::new(FileId::new(0), span);
        if on(diagnostics::PHRASE_UNUSED) {
            for p in &model.phrases {
                if unused(Usable::Phrase(p.key.clone())) {
                    issues.push(
                        Issue::new(diagnostics::PHRASE_UNUSED, at(p.span))
                            .with_arg("key", p.key.clone())
                            .with_arg(ENTRY_ARG, entry_arg(EntryKind::Phrase, &p.key)),
                    );
                }
            }
        }
        if on(diagnostics::FEATURE_UNUSED) {
            for f in &model.features {
                if unused(Usable::Feature(f.key.clone())) {
                    issues.push(
                        Issue::new(diagnostics::FEATURE_UNUSED, at(f.span))
                            .with_arg("key", f.key.clone())
                            .with_arg(ENTRY_ARG, entry_arg(EntryKind::Feature, &f.key)),
                    );
                }
            }
        }
        if on(diagnostics::GLOSSARY_TERM_UNUSED) {
            // A marked term's uses are links to its page, which the search
            // counts as the page's.
            for t in &model.glossary.terms {
                if t.match_mode != GlossaryMatch::Marked && unused(Usable::Term(t.id.clone())) {
                    issues.push(
                        Issue::new(diagnostics::GLOSSARY_TERM_UNUSED, at(t.span))
                            .with_arg("term", t.term.clone())
                            .with_arg(ENTRY_ARG, entry_arg(EntryKind::Term, &t.id)),
                    );
                }
            }
        }
    }

    if on(diagnostics::IMAGE_UNUSED) || on(diagnostics::IMAGE_LARGE) {
        self::images(project, index, &mut issues);
    }

    let mut out: BTreeMap<FileId, Vec<Diagnostic>> = BTreeMap::new();
    for issue in &issues {
        out.entry(issue.location.file)
            .or_default()
            .push(Diagnostic::from_issue(issue));
    }
    for list in out.values_mut() {
        list.sort_by_key(|d| d.location.span);
    }
    out
}

/// `image-unused` and `image-large`, for each image file under the content
/// root: files whose extension is an image's, outside folders whose names
/// start with `.`, nested projects' folders, and the output directory, as
/// the sources are found.
fn images(project: &Project, index: &ascribe_resolve::Project, issues: &mut Vec<Issue>) {
    let checks = &project.model().checks;
    let layout = project.layout();
    let listed = project.images_or(|| {
        let root = &layout.content_root;
        project
            .file_system()
            .files_in(root)
            .into_iter()
            .filter_map(|path| {
                let content = path.relative_to(root)?;
                let kept = is_image(&path)
                    && !content.segments().any(|s| s.starts_with('.'))
                    && !index
                        .nested_projects()
                        .iter()
                        .any(|n| content.starts_with(n))
                    && (layout.output_dir.is_root() || !path.starts_with(&layout.output_dir));
                kept.then_some((path, content))
            })
            .collect()
    });
    let limit = checks
        .limit(diagnostics::IMAGE_LARGE)
        .unwrap_or(IMAGE_LIMIT);
    // An image raw HTML shows or a frontmatter field names is used too.
    let mentioned: HashSet<RelPath> =
        if listed.is_empty() || checks.is_off(diagnostics::IMAGE_UNUSED) {
            HashSet::new()
        } else {
            index.files().flat_map(|f| f.mentioned_paths()).collect()
        };
    for (i, (path, content)) in listed.iter().enumerate() {
        let at = Location::new(image_id(i), Span::new(0, 0));
        let shown = path.to_string();
        let entry = entry_arg(EntryKind::Image, content.as_str());
        if !checks.is_off(diagnostics::IMAGE_UNUSED)
            && index.asset_users(content).is_empty()
            && !mentioned.contains(content)
        {
            issues.push(
                Issue::new(diagnostics::IMAGE_UNUSED, at)
                    .with_arg("path", shown.clone())
                    .with_arg(ENTRY_ARG, entry.clone()),
            );
        }
        if !checks.is_off(diagnostics::IMAGE_LARGE)
            && let Some(size) = project.file_system().size(path)
            && size > limit
        {
            issues.push(
                Issue::new(diagnostics::IMAGE_LARGE, at)
                    .with_arg("path", shown)
                    .with_arg("size", size_text(size))
                    .with_arg("limit", size_text(limit))
                    .with_arg(ENTRY_ARG, entry),
            );
        }
    }
}

/// Whether a path's extension is an image's.
fn is_image(path: &RelPath) -> bool {
    path.extension()
        .is_some_and(|e| IMAGE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

/// A size as people read it: `820 B`, `164 KB`, `1.2 MB`, with a kilobyte of
/// 1,000 bytes, as `[checks]` reads one.
pub fn size_text(bytes: u64) -> String {
    // Sizes are shown to one decimal place at most, so the conversion loses
    // nothing anyone reads.
    #[allow(clippy::cast_precision_loss)]
    let value = bytes as f64;
    if bytes < 1_000 {
        format!("{bytes} B")
    } else if bytes < 1_000_000 {
        format!("{} KB", trimmed(value / 1_000.0))
    } else {
        format!("{} MB", trimmed(value / 1_000_000.0))
    }
}

/// A number to one decimal place, without a trailing `.0`.
fn trimmed(value: f64) -> String {
    let text = format!("{value:.1}");
    text.strip_suffix(".0").unwrap_or(&text).to_owned()
}

/// `page-orphan` and `title-duplicate` for one build, from every page it
/// publishes, in path order. Issues are located with the index's file ids.
pub(crate) fn for_build(
    index: &ascribe_resolve::Project,
    checks: &Checks,
    resolved: &[&ResolvedPage],
) -> Vec<Found> {
    let mut found = Vec::new();
    if !checks.is_off(diagnostics::PAGE_ORPHAN) {
        orphans(index, resolved, &mut found);
    }
    if !checks.is_off(diagnostics::TITLE_DUPLICATE) {
        duplicate_titles(index, resolved, &mut found);
    }
    found
}

/// The pages no other page of the build links to or includes, other than
/// index pages and a build's only page.
fn orphans(index: &ascribe_resolve::Project, resolved: &[&ResolvedPage], out: &mut Vec<Found>) {
    // A build's only page has no other page to be linked from.
    if resolved.len() < 2 {
        return;
    }
    let mut reached: BTreeSet<&RelPath> = BTreeSet::new();
    for page in resolved {
        page.visit(&mut |block| {
            if block.file != page.file
                && let Some(path) = index.path_of(block.file)
            {
                reached.insert(path);
            }
            for link in &block.links {
                if let LinkTarget::Page { page: target, .. } = &link.target
                    && target != &page.path
                {
                    reached.insert(target);
                }
            }
        });
    }
    for page in resolved {
        if reached.contains(&page.path) || page.path.file_name() == Some("index.md") {
            continue;
        }
        let Some(file) = index.file(&page.path) else {
            continue;
        };
        let span = title_span(file);
        out.push(Found {
            issue: Issue::new(diagnostics::PAGE_ORPHAN, Location::new(page.file, span)),
            cause: (page.file, span, Arc::from([])),
        });
    }
}

/// The pages of the build that share a title, ignoring case and runs of
/// spaces: each is reported at its own title, naming the others.
fn duplicate_titles(
    index: &ascribe_resolve::Project,
    resolved: &[&ResolvedPage],
    out: &mut Vec<Found>,
) {
    let mut by_title: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, page) in resolved.iter().enumerate() {
        let Some(title) = page.title.as_deref() else {
            continue;
        };
        let key = title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if !key.is_empty() {
            by_title.entry(key).or_default().push(i);
        }
    }
    let mut groups: Vec<Vec<usize>> = by_title.into_values().filter(|g| g.len() > 1).collect();
    groups.sort();
    for group in groups {
        let pages: Vec<(&ResolvedPage, Span)> = group
            .iter()
            .filter_map(|&i| {
                let page = *resolved.get(i)?;
                Some((page, title_span(index.file(&page.path)?)))
            })
            .collect();
        for (page, span) in &pages {
            let others: Vec<&(&ResolvedPage, Span)> =
                pages.iter().filter(|(p, _)| p.path != page.path).collect();
            let named = others
                .iter()
                .map(|(p, _)| format!("`{}`", p.path))
                .collect::<Vec<_>>()
                .join(", ");
            let mut issue = Issue::new(
                diagnostics::TITLE_DUPLICATE,
                Location::new(page.file, *span),
            )
            .with_arg("title", page.title.clone().unwrap_or_default())
            .with_arg("others", named);
            for (other, other_span) in others {
                issue = issue.with_related(
                    Location::new(other.file, *other_span),
                    "a page with the same title",
                );
            }
            out.push(Found {
                issue,
                cause: (page.file, *span, Arc::from([])),
            });
        }
    }
}

/// Where a page's title is: its frontmatter `title` value, or the file's
/// first line when that can't be found.
fn title_span(file: &FileIndex) -> Span {
    let source: &str = &file.source;
    if let Some(fm) = &file.document.frontmatter
        && let Some(text) = source.get(fm.content.range())
        && let Some(node) = YamlIndex::build(text, fm.content.start()).get("title")
    {
        return node.value;
    }
    first_line(source)
}

/// A file's first line, without its line break.
fn first_line(text: &str) -> Span {
    let end = text.find(['\n', '\r']).unwrap_or(text.len());
    Span::new(0, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_as_people_write_them() {
        assert_eq!(size_text(820), "820 B");
        assert_eq!(size_text(164_060), "164.1 KB");
        assert_eq!(size_text(500_000), "500 KB");
        assert_eq!(size_text(1_260_000), "1.3 MB");
    }

    #[test]
    fn the_registry_states_the_default_limit() {
        let entry = crate::Registry::global()
            .get(diagnostics::IMAGE_LARGE)
            .and_then(|e| e.fix.clone())
            .unwrap_or_default();
        assert!(
            entry.contains(&format!("The limit is {}", size_text(IMAGE_LIMIT))),
            "image-large's fix names the default limit, {}",
            size_text(IMAGE_LIMIT)
        );
    }
}
