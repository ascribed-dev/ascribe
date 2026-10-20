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
//!   [`FileSystem`](ascribe_resolve::FileSystem); and
//!   `availability-left-behind` compares each feature's versions with each
//!   target's current line in `[releases]`; `release-line-ended` compares
//!   each live line's `ends` with the day the check runs as;
//!   `crawlers-disallowed` reads `[crawlers]`; and `description-too-long`
//!   measures a description field's `default`, once, where `ascribe.toml`
//!   writes it, since every page that leaves the field to it shares it.
//! - **Per build**, from the build's resolved pages, so they're page level:
//!   `page-orphan`, a page no other page the build publishes links to or
//!   includes, which only the sidebar reaches, or nothing for one that's
//!   `unlisted`, and `title-duplicate` and `description-duplicate`, two pages
//!   the build publishes with one title, or one description. [`PageChecker`](crate::PageChecker) runs them on each build it
//!   resolves.
//!
//! Each is advice, and a project can set its level or turn it off in
//! `[checks]`; one turned off isn't run.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use ascribe_core::availability::{AvailabilitySpec, Detail, Version};
use ascribe_core::intended::{ENTRY_ARG, EntryKind, entry_arg};
use ascribe_core::{Date, DiagnosticSlug, FileId, Issue, Location, RelPath, Span, diagnostics};
use ascribe_model::{
    AvailabilityMode, CheckLevel, Checks, ContentModel, FieldRole, Listing, Permission, Purpose,
    in_line,
};
use ascribe_resolve::{FileIndex, LinkTarget, ResolvedPage, Usable};

use crate::page::Found;
use crate::project::image_id;
use crate::releases;
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
    diagnostics::AVAILABILITY_LEFT_BEHIND,
    diagnostics::AVAILABLE_VERSION_AHEAD,
    diagnostics::AVAILABLE_LINE_UNDECLARED,
    diagnostics::AVAILABLE_SAME_IN_EVERY_LINE,
    diagnostics::AVAILABLE_IN_NO_LINE,
    diagnostics::RELEASE_LINE_ENDED,
    diagnostics::CRAWLERS_DISALLOWED,
    diagnostics::DESCRIPTION_TOO_LONG,
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

/// Whether a check runs: it's on and, when only errors count, `[checks]`
/// raises it to an error, since it's advice otherwise.
fn runs(checks: &Checks, slug: DiagnosticSlug, errors_only: bool) -> bool {
    !checks.is_off(slug) && (!errors_only || checks.level(slug) == Some(CheckLevel::Error))
}

/// What the checks that run for every build found, by file: the content
/// model's entries under its id, each unused fragment under its own, and the
/// image files' under theirs, each list in source order. `index` is the
/// project's source index; `file` gives the project's id of one of its files.
/// The image checks run only when `images` is set, and with `errors_only`
/// only the checks `[checks]` raises to an error run.
pub(crate) fn for_every_build(
    project: &Project,
    index: &ascribe_resolve::Project,
    file: &dyn Fn(FileId) -> FileId,
    images: bool,
    errors_only: bool,
) -> BTreeMap<FileId, Vec<Diagnostic>> {
    let checks = &project.model().checks;
    let on = |slug: DiagnosticSlug| {
        runs(checks, slug, errors_only) && (images || !NOT_IN_EDITOR.contains(&slug))
    };
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
            // A term is reported at its heading, on its glossary page, and
            // acknowledged with `@intended` at the top of its section.
            for t in index.glossary().terms {
                if unused(Usable::Term(t.id.clone())) {
                    issues.push(
                        Issue::new(
                            diagnostics::GLOSSARY_TERM_UNUSED,
                            Location::new(file(t.file), t.heading),
                        )
                        .with_arg("term", t.term.clone()),
                    );
                }
            }
        }
    }

    if on(diagnostics::AVAILABILITY_LEFT_BEHIND) {
        let model = project.model();
        for f in &model.features {
            if let Some((target, latest, current)) = left_behind(model, &f.available) {
                issues.push(
                    Issue::new(
                        diagnostics::AVAILABILITY_LEFT_BEHIND,
                        Location::new(FileId::new(0), f.span),
                    )
                    .with_arg("key", f.key.clone())
                    .with_arg("version", latest.text.clone())
                    .with_arg("target", target.to_owned())
                    .with_arg("current", current.text.clone())
                    .with_arg(ENTRY_ARG, entry_arg(EntryKind::Feature, &f.key)),
                );
            }
        }
    }

    if [
        diagnostics::AVAILABLE_VERSION_AHEAD,
        diagnostics::AVAILABLE_LINE_UNDECLARED,
        diagnostics::AVAILABLE_SAME_IN_EVERY_LINE,
        diagnostics::AVAILABLE_IN_NO_LINE,
    ]
    .into_iter()
    .any(on)
    {
        placement(project.model(), &mut issues);
    }

    if on(diagnostics::RELEASE_LINE_ENDED)
        && let Some(today) = project.today()
    {
        line_ended(project.model(), today, &mut issues);
    }

    if on(diagnostics::CRAWLERS_DISALLOWED) {
        crawlers_disallowed(project.model(), &mut issues);
    }

    if on(diagnostics::DESCRIPTION_TOO_LONG) {
        long_default_descriptions(project, &mut issues);
    }

    if on(diagnostics::IMAGE_UNUSED) || on(diagnostics::IMAGE_LARGE) {
        self::images(project, index, &on, &mut issues);
    }

    if crate::memos::MEMO_CHECKS.iter().any(|slug| on(*slug)) && crate::memos::any(project) {
        crate::memos::check_memos(project, index, &on, &mut issues);
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

/// What `[releases]` says about the specs in `ascribe.toml`: each version
/// a feature's spec or a build's filter names that its target's list can't
/// place, and each feature's entry that no output shows the version of or
/// that no output has. A filter's version is reported at the whole filter,
/// with no fix.
fn placement(model: &ContentModel, issues: &mut Vec<Issue>) {
    let at = |span: Span| Location::new(FileId::new(0), span);
    for f in &model.features {
        let placed = releases::placement_issues(model, &f.available.entries, &at, true);
        if placed.is_empty() {
            issues.extend(releases::inert_issues(model, &f.available, &at, true));
        }
        issues.extend(placed);
    }
    for build in &model.builds {
        if let AvailabilityMode::Filter {
            target,
            version: Some(version),
        } = &build.availability
            && let Some(list) = model.releases_of(target)
        {
            let placed = list.place(version);
            issues.extend(releases::placement_issue(
                model, target, placed, version, &at, false,
            ));
        }
    }
}

/// `release-line-ended`: a live line whose `ends` date is before `today`,
/// at its key in `[releases.<target>.ends]`.
fn line_ended(model: &ContentModel, today: Date, issues: &mut Vec<Issue>) {
    for releases in &model.releases {
        for end in releases.ends.iter().filter(|e| e.date < today) {
            issues.push(
                Issue::new(
                    diagnostics::RELEASE_LINE_ENDED,
                    Location::new(FileId::new(0), end.line.span),
                )
                .with_arg("target", releases.target.clone())
                .with_arg("line", end.line.text.clone())
                .with_arg("date", end.date.to_string()),
            );
        }
    }
}

/// `description-too-long` on a description field's `default`, at the value
/// in `ascribe.toml`: a page that leaves the field out, or empty, has the
/// default as its description (decision #573), so it's measured as a page's
/// own is, and reported once rather than on each such page.
fn long_default_descriptions(project: &Project, issues: &mut Vec<Issue>) {
    let model = project.model();
    for ty in &model.types {
        let Some(field) = ty.frontmatter.field_with_role(FieldRole::Description) else {
            continue;
        };
        let Some(text) = field
            .default
            .as_ref()
            .and_then(serde_yaml_ng::Value::as_str)
            .and_then(|raw| ascribe_resolve::llms::description_text(model, raw))
        else {
            continue;
        };
        let length = text.chars().count();
        if !crate::roles::description_too_long(model, length) {
            continue;
        }
        let Some(span) =
            ascribe_model::field_default_span(project.model_text(), &ty.name, &field.name)
        else {
            continue;
        };
        issues.push(
            Issue::new(
                diagnostics::DESCRIPTION_TOO_LONG,
                Location::new(FileId::new(0), span),
            )
            .with_arg("length", length.to_string())
            .with_arg(
                "limit",
                ascribe_resolve::llms::DESCRIPTION_LIMIT.to_string(),
            ),
        );
    }
}

/// `crawlers-disallowed`: `search` or `on-request` disallowed in
/// `[crawlers]`, at the choice, when the site's pages are listed unless they
/// say otherwise. With `[listing] default = "unlisted"` nothing is listed,
/// so keeping AI search out costs nothing more.
fn crawlers_disallowed(model: &ContentModel, issues: &mut Vec<Issue>) {
    let listed = model
        .listing
        .default
        .as_ref()
        .is_none_or(|rule| rule.listing != Listing::Unlisted);
    if !listed {
        return;
    }
    for choice in &model.crawlers.purposes {
        if choice.permission != Permission::Disallow || choice.purpose == Purpose::Training {
            continue;
        }
        let mut issue = Issue::new(
            diagnostics::CRAWLERS_DISALLOWED,
            Location::new(FileId::new(0), choice.span),
        )
        .with_arg("purpose", choice.purpose.name());
        if choice.purpose == Purpose::OnRequest {
            issue = issue.with_variant("on-request");
        }
        issues.push(issue);
    }
}

/// `image-unused` and `image-large`, for each image file under the content
/// root: files whose extension is an image's, outside folders whose names
/// start with `.`, nested projects' folders, and the output directory, as
/// the sources are found.
fn images(
    project: &Project,
    index: &ascribe_resolve::Project,
    on: &dyn Fn(DiagnosticSlug) -> bool,
    issues: &mut Vec<Issue>,
) {
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
                listed_image(project, index, &path, &content).then_some((path, content))
            })
            .collect()
    });
    let limit = checks
        .limit(diagnostics::IMAGE_LARGE)
        .unwrap_or(IMAGE_LIMIT);
    // An image raw HTML shows or a frontmatter field names is used too.
    let mentioned: HashSet<RelPath> = if listed.is_empty() || !on(diagnostics::IMAGE_UNUSED) {
        HashSet::new()
    } else {
        index.files().flat_map(|f| f.mentioned_paths()).collect()
    };
    for (i, (path, content)) in listed.iter().enumerate() {
        let at = Location::new(image_id(i), Span::new(0, 0));
        let shown = path.to_string();
        let entry = entry_arg(EntryKind::Image, content.as_str());
        if on(diagnostics::IMAGE_UNUSED)
            && index.asset_users(content).is_empty()
            && !mentioned.contains(content)
        {
            issues.push(
                Issue::new(diagnostics::IMAGE_UNUSED, at)
                    .with_arg("path", shown.clone())
                    .with_arg(ENTRY_ARG, entry.clone()),
            );
        }
        if on(diagnostics::IMAGE_LARGE)
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

/// The latest version a feature's spec names, with its target and that
/// target's current line, when every target in it names one and none is
/// later than its target's current line: the feature is about releases that
/// have shipped. A target with no version, such as a versionless one, or
/// with no lines in `[releases]`, keeps it current.
fn left_behind<'s>(
    model: &'s ContentModel,
    spec: &'s AvailabilitySpec,
) -> Option<(&'s str, &'s Version, &'s Version)> {
    let mut latest: Option<(&str, &Version, &Version)> = None;
    for entry in &spec.entries {
        let version = match &entry.detail {
            Detail::Version(v)
            | Detail::State {
                version: Some(v), ..
            } => v,
            Detail::History(steps) => &steps.last()?.version,
            Detail::None | Detail::State { version: None, .. } => return None,
        };
        let target = entry.target.text.as_str();
        let current = model.releases_of(target)?.current()?;
        if version.compare(current) == Ordering::Greater && !in_line(current, version) {
            return None;
        }
        if latest.is_none_or(|(_, l, _)| version.compare(l) == Ordering::Greater) {
            latest = Some((target, version, current));
        }
    }
    latest
}

/// Whether the image checks look at the file at `content`, a path from the
/// content root: one that's there, whose extension is an image's, outside
/// folders whose names start with `.`, nested projects' folders, and the
/// output directory. `ascribe refs` takes it as a target.
pub fn image_file(project: &Project, index: &ascribe_resolve::Project, content: &RelPath) -> bool {
    let Ok(path) = project.content_root().join(content.as_str()) else {
        return false;
    };
    listed_image(project, index, &path, content)
        && project.file_system().probe(&path) == ascribe_resolve::Probe::File
}

/// Whether the file at `path`, `content` from the content root, is one the
/// image checks list, when it's there.
fn listed_image(
    project: &Project,
    index: &ascribe_resolve::Project,
    path: &RelPath,
    content: &RelPath,
) -> bool {
    let layout = project.layout();
    is_image(path)
        && !content.segments().any(|s| s.starts_with('.'))
        && !index
            .nested_projects()
            .iter()
            .any(|n| content.starts_with(n))
        && (layout.output_dir.is_root() || !path.starts_with(&layout.output_dir))
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

/// `page-orphan`, `title-duplicate`, and `description-duplicate` for one build, from every page it
/// publishes, in path order, and with `errors_only` only those `[checks]`
/// raises to an error. Issues are located with the index's file ids.
pub(crate) fn for_build(
    index: &ascribe_resolve::Project,
    checks: &Checks,
    resolved: &[&ResolvedPage],
    errors_only: bool,
) -> Vec<Found> {
    let mut found = Vec::new();
    if runs(checks, diagnostics::PAGE_ORPHAN, errors_only) {
        orphans(index, resolved, &mut found);
    }
    if runs(checks, diagnostics::TITLE_DUPLICATE, errors_only) {
        duplicate_titles(index, resolved, &mut found);
    }
    if runs(checks, diagnostics::DESCRIPTION_DUPLICATE, errors_only) {
        duplicate_descriptions(index, resolved, &mut found);
    }
    found
}

/// The pages no other page of the build links to or includes, other than
/// index pages and a build's only page, each saying what reaches it: the
/// sidebar, or nothing but its address for one that's `unlisted`.
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
        let issue = Issue::new(diagnostics::PAGE_ORPHAN, Location::new(page.file, span));
        // An unlisted page is in no sidebar either (SPEC §7.8).
        let issue = if page.listing == Listing::Unlisted {
            issue.with_variant("unlisted")
        } else {
            issue
        };
        out.push(Found {
            issue,
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
    let titled = resolved.iter().filter_map(|page| {
        let title = page.title.clone()?;
        Some((*page, title, title_span(index.file(&page.path)?)))
    });
    duplicates(
        diagnostics::TITLE_DUPLICATE,
        "a page with the same title",
        titled.collect(),
        out,
    );
}

/// The pages of the build that share a description, ignoring case and runs
/// of spaces: each is reported at its own description, naming the others.
/// A description is the field the page's type marks `role =
/// "description"`, as its line in `llms.txt` has it, when the page writes
/// it rather than leaving it to the field's default.
fn duplicate_descriptions(
    index: &ascribe_resolve::Project,
    resolved: &[&ResolvedPage],
    out: &mut Vec<Found>,
) {
    let model = index.model();
    let described = resolved.iter().filter_map(|page| {
        let text = ascribe_resolve::llms::description(model, page)?;
        let field = ascribe_resolve::llms::description_field(model, page.frontmatter.as_ref())?;
        // The pages that leave the field to its default share it by design:
        // only a description a page writes is its own to change.
        page.frontmatter
            .as_ref()?
            .get(field.name.as_str())
            .and_then(ascribe_resolve::llms::own_description)?;
        let span = field_span(index.file(&page.path)?, &field.name)?;
        Some((*page, text, span))
    });
    duplicates(
        diagnostics::DESCRIPTION_DUPLICATE,
        "a page with the same description",
        described.collect(),
        out,
    );
}

/// Each group of `pages` whose text is the same, ignoring case and runs of
/// spaces: each page is reported as `slug` at its span, with `title` its
/// text and `others` the other pages' paths, and the others related to it
/// as `related`.
fn duplicates(
    slug: DiagnosticSlug,
    related: &'static str,
    pages: Vec<(&ResolvedPage, String, Span)>,
    out: &mut Vec<Found>,
) {
    let mut by_text: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, (_, text, _)) in pages.iter().enumerate() {
        let key = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if !key.is_empty() {
            by_text.entry(key).or_default().push(i);
        }
    }
    let mut groups: Vec<Vec<usize>> = by_text.into_values().filter(|g| g.len() > 1).collect();
    groups.sort();
    for group in groups {
        let shared: Vec<&(&ResolvedPage, String, Span)> =
            group.iter().filter_map(|&i| pages.get(i)).collect();
        for (page, text, span) in &shared {
            let others: Vec<&&(&ResolvedPage, String, Span)> = shared
                .iter()
                .filter(|(p, _, _)| p.path != page.path)
                .collect();
            let named = others
                .iter()
                .map(|(p, _, _)| format!("`{}`", p.path))
                .collect::<Vec<_>>()
                .join(", ");
            let mut issue = Issue::new(slug, Location::new(page.file, *span))
                .with_arg("title", text.clone())
                .with_arg("others", named);
            for (other, _, other_span) in others {
                issue = issue.with_related(Location::new(other.file, *other_span), related);
            }
            out.push(Found {
                issue,
                cause: (page.file, *span, Arc::from([])),
            });
        }
    }
}

/// Where a frontmatter field's value is, when the page writes it.
fn field_span(file: &FileIndex, name: &str) -> Option<Span> {
    let source: &str = &file.source;
    let fm = file.document.frontmatter.as_ref()?;
    let text = source.get(fm.content.range())?;
    Some(YamlIndex::build(text, fm.content.start()).get(name)?.value)
}

/// Where a page's title is: its frontmatter `title` value, or the file's
/// first line when that can't be found.
pub(crate) fn title_span(file: &FileIndex) -> Span {
    field_span(file, "title").unwrap_or_else(|| first_line(&file.source))
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
