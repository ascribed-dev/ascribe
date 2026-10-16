//! A project's release lines (`ascribe versions list`), and what differs
//! between two of them (`ascribe versions changes`), read from
//! `[releases]` and the marks, with nothing built.

use std::collections::{BTreeMap, HashSet, VecDeque};

use ascribe_core::RelPath;
use ascribe_core::availability::{AvailabilitySpec, Version, parse_availability};
use ascribe_model::{ContentModel, Releases, named_line};
use ascribe_resolve::{FileIndex, InLine, Project, in_release_line};
use serde::Serialize;

use crate::lines::{Lines, project_path};
use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// Every target's release lines.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct VersionsList {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// Each target with a list in `[releases]`, in the order `ascribe.toml`
    /// declares them.
    pub targets: Vec<TargetLines>,
}

/// One target's lines.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct TargetLines {
    /// The target, a versioned dimension value.
    pub target: String,
    /// The builds versioned by it, which give each live line an output.
    pub builds: Vec<String>,
    /// Its lines, newest first.
    pub lines: Vec<ListedLine>,
}

/// One line of a target.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ListedLine {
    /// The line, as the list writes it.
    pub line: String,
    /// `current` (the newest live line), `live`, `planned`, or `archived`.
    pub status: &'static str,
    /// For a live line, its output in each build versioned by the target.
    pub outputs: Vec<LineOutput>,
    /// For a live line, the last day it's supported, from `ends`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends: Option<String>,
    /// For an archived line, the day its support ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended: Option<String>,
    /// For an archived line, where it can still be read; absent when
    /// nowhere.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// A live line's output in one build.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct LineOutput {
    /// The output's build: the build's own name for the current line,
    /// `<build>@<line>` for an older one.
    pub build: String,
    /// The path its pages are under, ending in `/`.
    pub base: String,
    /// How many pages it publishes.
    pub pages: usize,
}

/// Every target's lines, from the project's `[releases]` and builds. It
/// builds nothing: a line's pages are counted from what its build drops.
pub fn list(project: &Project) -> VersionsList {
    let model = project.model();
    let targets = model
        .releases
        .iter()
        .map(|releases| target_lines(project, releases))
        .collect();
    VersionsList {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        targets,
    }
}

fn target_lines(project: &Project, releases: &Releases) -> TargetLines {
    let model = project.model();
    let target = releases.target.as_str();
    let versioned: Vec<&ascribe_model::Build> = model
        .builds
        .iter()
        .filter(|b| b.versions.as_deref() == Some(target))
        .collect();
    let mut builds: Vec<String> = Vec::new();
    for b in &versioned {
        let name = b.declared_name().to_owned();
        if !builds.contains(&name) {
            builds.push(name);
        }
    }
    let lines = releases
        .lines()
        .into_iter()
        .map(|(line, status)| {
            let outputs = versioned
                .iter()
                .filter(|b| {
                    b.line
                        .as_ref()
                        .is_some_and(|l| l.line.components == line.components)
                })
                .map(|b| LineOutput {
                    build: b.name.clone(),
                    base: ascribe_emit::line_base(model, b),
                    pages: project
                        .pages()
                        .filter(|p| project.dropped(&p.path, b).is_none())
                        .count(),
                })
                .collect();
            let archived = releases
                .archived
                .iter()
                .find(|a| a.line.components == line.components);
            ListedLine {
                line: line.text.clone(),
                status: status.as_str(),
                outputs,
                ends: releases
                    .ends
                    .iter()
                    .find(|e| e.line.components == line.components)
                    .map(|e| e.date.to_string()),
                ended: archived
                    .and_then(|a| a.ended.as_ref())
                    .map(ToString::to_string),
                url: archived.and_then(|a| a.url.clone()),
            }
        })
        .collect();
    TargetLines {
        target: target.to_owned(),
        builds,
        lines,
    }
}

/// What differs between two lines of one target.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct VersionChanges {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The target.
    pub target: String,
    /// The line compared from, as the list writes it.
    pub from: String,
    /// The line compared to.
    pub to: String,
    /// What `to` has and `from` doesn't, in page order.
    pub added: Vec<LineChange>,
    /// What `from` has and `to` doesn't.
    pub removed: Vec<LineChange>,
    /// What both have, in a different state.
    pub changed: Vec<LineChange>,
}

/// Content one line has and another doesn't, or has in another state.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct LineChange {
    /// The file its mark is in, from the project root, as `ascribe check`
    /// reports it.
    pub file: String,
    /// The mark's line, from 1.
    pub line: u32,
    /// What the mark is on: `page` (the frontmatter's `available`),
    /// `section` (a heading's), `block` (the block after it), or `row` (a
    /// table row's).
    pub kind: &'static str,
    /// The page's title, or the file's path when it has none.
    pub page: String,
    /// The heading it's under, or the section's own heading; absent at the
    /// top of the page, and for a page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    /// Its first line, as written.
    pub text: String,
    /// The state in `from`, for a change of state: a lifecycle state, such
    /// as `preview`, or `ga`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_state: Option<String>,
    /// The state in `to`, for a change of state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_state: Option<String>,
    /// For a page added, the page `from` has in its place and `to`
    /// doesn't, when `replaces` links the two, either way: the pair is one
    /// change, so that page isn't listed as removed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_place_of: Option<PlaceOf>,
}

/// The page another takes the place of.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct PlaceOf {
    /// Its file, from the project root.
    pub file: String,
    /// Its mark's line, from 1.
    pub line: u32,
    /// Its title, or the file's path when it has none.
    pub page: String,
}

/// The target and line `text` names, `self-managed 3.4`, when the target
/// has lines and declares it.
///
/// # Errors
///
/// It isn't a line's name, the target has no lines, or the line isn't one.
pub fn declared_line(model: &ContentModel, text: &str) -> Result<(String, Version), QueryError> {
    let (target, line) = named_line(text).ok_or_else(|| QueryError::BadLine {
        given: text.to_owned(),
    })?;
    check_declared(model, &target, line)
}

fn check_declared(
    model: &ContentModel,
    target: &str,
    line: Version,
) -> Result<(String, Version), QueryError> {
    let releases = model
        .releases_of(target)
        .ok_or_else(|| QueryError::NoLines {
            target: target.to_owned(),
            targets: model.releases.iter().map(|r| r.target.clone()).collect(),
        })?;
    match releases.status_of(&line) {
        Some(_) => Ok((target.to_owned(), line)),
        None => Err(QueryError::UndeclaredLine {
            target: target.to_owned(),
            line: line.text.clone(),
            lines: releases
                .lines()
                .iter()
                .map(|(l, _)| l.text.clone())
                .collect(),
        }),
    }
}

/// What differs between `from`, `"<target> <line>"`, and `to`, another
/// line of the same target: each page, section, block, and row whose mark
/// puts it in one line and not the other, or in both in a different
/// state. It reads the marks alone and builds nothing. A mark on a page
/// whose own mark puts it in one line only isn't listed apart from the
/// page, and a page added in place of one removed, by `replaces`, is one
/// change.
///
/// # Errors
///
/// A line isn't named as a mark names one, or isn't declared.
pub fn changes(project: &Project, from: &str, to: &str) -> Result<VersionChanges, QueryError> {
    let model = project.model();
    let (target, from) = declared_line(model, from)?;
    let to = ascribe_core::availability::parse_version(to.trim(), 0).map_err(|_| {
        QueryError::BadLine {
            given: format!("{target} {to}"),
        }
    })?;
    let (_, to) = check_declared(model, &target, to)?;
    let mut out = VersionChanges {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        target: target.clone(),
        from: from.text.clone(),
        to: to.text.clone(),
        added: Vec::new(),
        removed: Vec::new(),
        changed: Vec::new(),
    };
    let mut lines = Lines::new(project);
    // The pages added and removed whole, by their entry.
    let mut added_pages: Vec<(usize, RelPath)> = Vec::new();
    let mut removed_pages: Vec<(usize, RelPath)> = Vec::new();
    // The pages whose own mark keeps them out of both lines, which a chain
    // of `replaces` runs through.
    let mut in_neither: HashSet<RelPath> = HashSet::new();
    let mut files: Vec<&FileIndex> = project.files().collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    for file in files {
        let page = file.title.clone().unwrap_or_else(|| file.path.to_string());
        let mut marks: Vec<(usize, &'static str, AvailabilitySpec)> = Vec::new();
        let mut page_level = None;
        if let Some(text) = file
            .frontmatter
            .as_ref()
            .and_then(|fm| fm.get("available"))
            .and_then(|v| v.as_str())
            && let Ok(spec) = parse_availability(text, 0)
        {
            let spec = resolved(model, spec);
            let at = file.document.frontmatter.as_ref().map_or(0, |f| {
                // The `available:` line, when it starts one.
                let source: &str = &file.source;
                source
                    .get(f.content.range())
                    .and_then(|text| {
                        text.find("\navailable:")
                            .map(|i| i + 1)
                            .or_else(|| text.starts_with("available:").then_some(0))
                    })
                    .map_or(f.span.start(), |i| f.content.start() + i)
            });
            page_level = Some(compare(model, &spec, &target, &from, &to));
            let available =
                |line: &Version| in_release_line(model, &spec, &target, line).map(|l| l.available);
            if available(&from) == Some(false) && available(&to) == Some(false) {
                in_neither.insert(file.path.clone());
            }
            marks.push((at, "page", spec));
        }
        for marker in &file.availability {
            let (Some(Ok(spec)), Some((_, span))) = (&marker.spec, &marker.primary) else {
                continue;
            };
            let kind = match marker.binding {
                None => "row",
                Some(_) if marker.binds_section() => "section",
                Some(_) => "block",
            };
            marks.push((span.start(), kind, resolved(model, spec.clone())));
        }
        // Content on a page that's in one line only goes with the page.
        let page_moves = matches!(page_level, Some(Some(Compared::Added | Compared::Removed)));
        for (at, kind, spec) in marks {
            let Some(compared) = compare(model, &spec, &target, &from, &to) else {
                continue;
            };
            if page_moves && kind != "page" {
                continue;
            }
            let (line, _) = lines.at(&file.path, at);
            let change = |from_state: Option<String>, to_state: Option<String>| LineChange {
                file: project_path(project, &file.path),
                line,
                kind,
                page: page.clone(),
                heading: heading_of(file, at, kind),
                text: first_line(file, at, kind),
                from_state,
                to_state,
                in_place_of: None,
            };
            if kind == "page" {
                match compared {
                    Compared::Added => added_pages.push((out.added.len(), file.path.clone())),
                    Compared::Removed => {
                        removed_pages.push((out.removed.len(), file.path.clone()));
                    }
                    Compared::Changed(..) => {}
                }
            }
            match compared {
                Compared::Added => out.added.push(change(None, None)),
                Compared::Removed => out.removed.push(change(None, None)),
                Compared::Changed(a, b) => out.changed.push(change(a, b)),
            }
        }
    }
    pair_replaced(project, &mut out, &added_pages, &removed_pages, &in_neither);
    Ok(out)
}

/// Makes each page added in place of one removed one change: the added
/// page's entry names the removed one, whose own entry goes. The two are
/// linked by `replaces`, either way, directly or through pages neither
/// line has: a page in a line between them stops the chain, since it took
/// the place itself. An added page takes the removed page nearest along
/// the chain, the newer side first, that no page before it took.
fn pair_replaced(
    project: &Project,
    out: &mut VersionChanges,
    added: &[(usize, RelPath)],
    removed: &[(usize, RelPath)],
    in_neither: &HashSet<RelPath>,
) {
    if added.is_empty() || removed.is_empty() {
        return;
    }
    let links = project.replacements();
    let mut taken: Vec<usize> = Vec::new();
    for (entry, page) in added {
        let Some(gone) = nearest_removed(&links, page, removed, &taken, in_neither) else {
            continue;
        };
        taken.push(gone);
        let old = &out.removed[gone];
        let place = PlaceOf {
            file: old.file.clone(),
            line: old.line,
            page: old.page.clone(),
        };
        if let Some(change) = out.added.get_mut(*entry) {
            change.in_place_of = Some(place);
        }
    }
    taken.sort_unstable();
    for gone in taken.into_iter().rev() {
        out.removed.remove(gone);
    }
}

/// The entry of the removed page `replaces` links `page` with, nearest
/// first, that isn't `taken`, walking on only through pages `in_neither`
/// line.
fn nearest_removed(
    links: &BTreeMap<RelPath, RelPath>,
    page: &RelPath,
    removed: &[(usize, RelPath)],
    taken: &[usize],
    in_neither: &HashSet<RelPath>,
) -> Option<usize> {
    let mut seen: HashSet<&RelPath> = HashSet::from([page]);
    let mut queue: VecDeque<&RelPath> = VecDeque::from([page]);
    while let Some(at) = queue.pop_front() {
        let newer = links.iter().filter(|(_, r)| *r == at).map(|(by, _)| by);
        let older = links.get(at);
        for next in newer.chain(older) {
            if !seen.insert(next) {
                continue;
            }
            if let Some((entry, _)) = removed
                .iter()
                .find(|(entry, p)| p == next && !taken.contains(entry))
            {
                return Some(*entry);
            }
            if in_neither.contains(next) {
                queue.push_back(next);
            }
        }
    }
    None
}

/// How one mark differs between two lines.
#[derive(Debug)]
enum Compared {
    Added,
    Removed,
    Changed(Option<String>, Option<String>),
}

fn compare(
    model: &ContentModel,
    spec: &AvailabilitySpec,
    target: &str,
    from: &Version,
    to: &Version,
) -> Option<Compared> {
    let a: InLine = in_release_line(model, spec, target, from)?;
    let b: InLine = in_release_line(model, spec, target, to)?;
    match (a.available, b.available) {
        (false, true) => Some(Compared::Added),
        (true, false) => Some(Compared::Removed),
        (true, true) if a.state != b.state => Some(Compared::Changed(a.state, b.state)),
        _ => None,
    }
}

/// A spec with a feature key replaced by the feature's spec.
fn resolved(model: &ContentModel, spec: AvailabilitySpec) -> AvailabilitySpec {
    match spec.bare_name().and_then(|n| model.feature(&n.text)) {
        Some(feature) => feature.available.clone(),
        None => spec,
    }
}

/// The heading a mark at `at` is under: the last before it. A page's mark
/// is under none.
fn heading_of(file: &FileIndex, at: usize, kind: &str) -> Option<String> {
    if kind == "page" {
        return None;
    }
    file.headings
        .iter()
        .rev()
        .find(|h| h.span.start() < at)
        .map(|h| h.text.clone())
}

/// What a mark at `at` is on, by its first line: a page's title, a
/// section's heading, the first line after a block's mark, and a row's
/// own line.
fn first_line(file: &FileIndex, at: usize, kind: &str) -> String {
    let source: &str = &file.source;
    match kind {
        "page" => file.title.clone().unwrap_or_default(),
        "section" => heading_of(file, at, kind).unwrap_or_default(),
        "row" => {
            let start = source[..at].rfind('\n').map_or(0, |i| i + 1);
            let end = source[at..].find('\n').map_or(source.len(), |i| at + i);
            source[start..end].trim().to_owned()
        }
        _ => {
            let after = source[at..].find('\n').map_or(source.len(), |i| at + i + 1);
            source[after..]
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or_default()
                .to_owned()
        }
    }
}
