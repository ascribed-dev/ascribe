//! Retiring a line, `ascribe versions retire "<target> <line>"`: the line
//! moved from `live` to `archived` with its address, each mark for the
//! target rewritten to the simplest spec that means the same in every line
//! still in the window, the content no line still has listed, and the
//! retired line's addresses each given where it should go (README decisions
//! 16, 17, and 21), and, with `ascribe.addresses`, those to a page in the
//! current line recorded there (decision #946).
//!
//! It writes nothing. [`retire_line`] returns `ascribe.toml`'s text and each
//! source file's after the retirement; the command asks git whether the
//! files can be put back, and writes them. The rewrite is proved by
//! building every output in memory with the line archived, before and
//! after the marks change: a live line's output that changes is an error,
//! and another build's is reported, since a build that isn't a line shows a
//! version from before the window. A change to the target's own part of
//! another build's availability lines is the retirement doing its job, so
//! it's listed apart and needs no `--force` (decision #1050).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ascribe_check::Project;
use ascribe_core::availability::{
    AvailabilitySpec, Detail, Version, parse_availability, spec_text, without_versions_through,
};
use ascribe_core::{Coded, FileId, LineIndex, RelPath, Span, TextEdit, apply_edits};
use ascribe_emit::Contents;
use ascribe_model::{ContentModel, LineStatus, Releases};
use ascribe_resolve::{AstroRouter, Availability, FileIndex, in_release_line, replaces_span};
use ascribe_syntax::{Block, BlockKind, Bound};
use toml_edit::Value;

use crate::model_file::{ModelFile, path};
use crate::release::{
    ChangedOutput, MovedFeature, ReleaseError, ReleasedFile, RewrittenMark, archive,
    changed_outputs, finish, frontmatter_available, load, outputs, set_lines, yaml_plain,
};

/// What to retire.
#[derive(Clone, Copy, Debug)]
pub struct RetiringLine<'a> {
    /// The target, a versioned dimension value with lines in `[releases]`.
    pub target: &'a str,
    /// The line retired: the oldest live one.
    pub line: &'a Version,
    /// `--url`: where the line's archive is read.
    pub url: Option<&'a str>,
    /// `--force`: rewrite marks even when a build that isn't a line of the
    /// target changes.
    pub force: bool,
    /// `--remove-unused`: delete the content no line still has, and the
    /// marks that say nothing.
    pub remove_unused: bool,
}

/// What a retirement changes, written nowhere yet.
#[derive(Clone, Debug)]
pub struct RetiredLine {
    /// The target.
    pub target: String,
    /// The line retired, as the list writes it.
    pub line: String,
    /// Where the line's archive is read.
    pub url: String,
    /// The day the line's support ended: its `ends`, or today.
    pub ended: Option<String>,
    /// `ascribe.toml`'s text after the retirement.
    pub model_text: String,
    /// Each source file whose marks are rewritten, in path order.
    pub files: Vec<ReleasedFile>,
    /// Each feature whose spec is rewritten.
    pub features: Vec<MovedFeature>,
    /// The content no live or planned line has, and the marks that say
    /// nothing once rewritten.
    pub unused: Vec<Unused>,
    /// Each source file `--remove-unused` changes: its text after, or
    /// `None` when it's deleted. Empty without it.
    pub removals: Vec<(RelPath, Option<String>)>,
    /// Where each address the line published should go.
    pub redirects: Vec<Redirect>,
    /// `ascribe.addresses`' text after the retirement, when the project has
    /// a file that reads: each redirect to a page in the current line
    /// recorded in `[moved]`, keyed by the old address, so the build writes
    /// it. `None` without the file, or when it changes nothing.
    pub addresses_text: Option<String>,
    /// How many of `redirects` `ascribe.addresses` records: those to a page
    /// in the current line, when the project has a file that reads; else 0.
    pub recorded: usize,
    /// The outputs of builds that aren't lines of the target that the
    /// rewrite changes, by build and output, apart from `expected`.
    pub builds_changed: Vec<ChangedOutput>,
    /// The pages of builds that aren't lines of the target whose only
    /// change is the target's part of their availability lines: its state
    /// and versions. They need no `--force`.
    pub expected: Vec<ChangedOutput>,
}

/// Content no live or planned line has, or a mark that says nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unused {
    /// The file's content path.
    pub path: RelPath,
    /// The mark's line, from 1.
    pub line: u32,
    /// What it is: `page`, `section`, `block`, `row`, or `mark`, for a mark
    /// that says nothing and can go while its content stays.
    pub kind: &'static str,
    /// Its first line: a page's title, a section's heading, a block's or a
    /// row's first line, or the mark.
    pub text: String,
    /// What `--remove-unused` deletes from the file; the whole file for a
    /// page.
    span: Span,
}

/// An address the retired line published, and where it should go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Redirect {
    /// The address, from the site's root: `/3.4/install/`.
    pub from: String,
    /// Where it should go: the same page in the current line, the page that
    /// replaces it there, or the page in the archive, a full address.
    pub to: String,
    /// The page in the current line it goes to, by its content path;
    /// `None` when it goes to the archive.
    pub page: Option<RelPath>,
    /// Why it goes there.
    pub why: RedirectWhy,
}

/// Why an address goes where it does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedirectWhy {
    /// The current line has the page.
    Same,
    /// The current line has the page that replaces it.
    Replaced,
    /// The current line has neither, so it goes to the archive.
    Archive,
}

/// Why a line can't be retired. Nothing is written.
#[derive(Debug, thiserror::Error)]
pub enum RetireError {
    /// The target has no lines in `[releases]`.
    #[error(
        "`{target}` has no lines in [releases]; declare them as [releases.{target}] in ascribe.toml"
    )]
    NoLines {
        /// The target.
        target: String,
    },
    /// The line isn't live.
    #[error("`{target} {line}` isn't live: {why}")]
    NotLive {
        /// The target.
        target: String,
        /// The line.
        line: String,
        /// What it is instead.
        why: String,
    },
    /// The line is the current one.
    #[error(
        "`{target} {line}` is the current line, and stays; retire the oldest live line, `{target} {oldest}`"
    )]
    Current {
        /// The target.
        target: String,
        /// The line.
        line: String,
        /// The oldest live line.
        oldest: String,
    },
    /// An older live line comes first.
    #[error("`{target} {line}` isn't the oldest live line; retire `{target} {oldest}` first")]
    NotOldest {
        /// The target.
        target: String,
        /// The line.
        line: String,
        /// The oldest live line.
        oldest: String,
    },
    /// No build is versioned by the target.
    #[error(
        "no build is versioned by `{target}`, so no line of it was published alone and no mark of it does nothing; `ascribe versions release` keeps its list, archiving the line before each release"
    )]
    NotVersioned {
        /// The target.
        target: String,
    },
    /// The line has no address.
    #[error(
        "`{target} {line}` has no address; run `ascribe versions export \"{target} {line}\" --to <folder>` first, host the folder, and give its address with --url: once the marks are rewritten, the source no longer holds what the line needs"
    )]
    NoAddress {
        /// The target.
        target: String,
        /// The line.
        line: String,
    },
    /// A mark is written in a way the command can't rewrite.
    #[error(
        "{path}:{line}: the mark `{written}` can't be rewritten in place; write it on one line, then run the command again"
    )]
    MarkUnrewritable {
        /// The file's content path.
        path: String,
        /// The mark's line, from 1.
        line: u32,
        /// The mark as written.
        written: String,
    },
    /// `ascribe.toml` can't be changed in place.
    #[error("{0}")]
    ModelEdit(String),
    /// The content model after the retirement wouldn't load.
    #[error("ascribe.toml wouldn't load after the retirement: {0}")]
    ModelInvalid(String),
    /// A build's outputs can't be produced.
    #[error("can't build the outputs to compare: {0}")]
    Build(String),
    /// Rewriting the marks would change a live line's output.
    #[error("rewriting the marks would change what a reader sees in a live line, in {}; nothing was written", files.join(", "))]
    OutputsChanged {
        /// The files, `<build>/<output>/<path>`.
        files: Vec<String>,
    },
    /// Rewriting the marks would change a build that isn't a line.
    #[error(
        "rewriting the marks changes builds that aren't lines of `{target}`, which show a version from before the window: {}; pass --force to rewrite them anyway",
        builds.join(", ")
    )]
    OtherBuildsChange {
        /// The target.
        target: String,
        /// Each build and output, with the pages that change.
        builds: Vec<String>,
    },
}

impl Coded for RetireError {
    fn code(&self) -> &'static str {
        match self {
            RetireError::NoLines { .. } => "retire_no_lines",
            RetireError::NotLive { .. } => "retire_line_not_live",
            RetireError::Current { .. } => "retire_line_current",
            RetireError::NotOldest { .. } => "retire_line_not_oldest",
            RetireError::NotVersioned { .. } => "retire_not_versioned",
            RetireError::NoAddress { .. } => "retire_no_address",
            RetireError::MarkUnrewritable { .. } => "retire_mark_unrewritable",
            RetireError::ModelEdit(_) => "model_edit_refused",
            RetireError::ModelInvalid(_) => "retire_model_invalid",
            RetireError::Build(_) => "retire_build_failed",
            RetireError::OutputsChanged { .. } => "retire_outputs_changed",
            RetireError::OtherBuildsChange { .. } => "retire_builds_changed",
        }
    }
}

impl From<ReleaseError> for RetireError {
    fn from(e: ReleaseError) -> Self {
        match e {
            ReleaseError::ModelInvalid(m) => RetireError::ModelInvalid(m),
            ReleaseError::Build(m) => RetireError::Build(m),
            ReleaseError::ModelEdit(m) => RetireError::ModelEdit(m),
            other => RetireError::ModelEdit(other.to_string()),
        }
    }
}

/// Retires `retiring.line` of `retiring.target` in `project`, in memory:
/// the files it would write, what's left in no line, and where the line's
/// addresses should go.
///
/// # Errors
///
/// The line can't be retired (the [`RetireError`]s about the list, the
/// line, its address, and the builds); a mark or `ascribe.toml` can't be
/// rewritten in place; or the rewrite would change a live line's output,
/// or, without `force`, another build's.
pub fn retire_line(
    project: &Project,
    retiring: &RetiringLine<'_>,
) -> Result<RetiredLine, RetireError> {
    let model = project.model();
    let target = retiring.target;
    let line = retiring.line;
    let releases = refuse(model, retiring)?;
    let url = retiring.url.ok_or_else(|| RetireError::NoAddress {
        target: target.to_owned(),
        line: line.text.clone(),
    })?;

    // The list: the line from `live` to `archived`.
    let today = project.today();
    let moved = {
        let file = ModelFile::parse(project.model_text()).map_err(RetireError::ModelEdit)?;
        let live: Vec<&Version> = releases
            .live
            .iter()
            .filter(|l| l.components != line.components)
            .collect();
        let mut edits = vec![set_lines(&file, target, "live", live)?];
        edits.extend(archive(&file, releases, line, Some(url), today)?);
        finish(&file, edits)?
    };
    let ended = releases
        .ends
        .iter()
        .find(|e| e.line.components == line.components)
        .map(|e| e.date)
        .or(today)
        .map(|d| d.to_string());
    let root = project.root();
    let after_model = load(&moved, root)?;
    let window = Window::of(&after_model, target);

    // The marks, and the features.
    let index = project.index();
    let mut files = Vec::new();
    for file in index.files() {
        if let Some(rewritten) = rewrite_marks(file, &after_model, &window)? {
            files.push(rewritten);
        }
    }
    let (text, features) = rewrite_features(&moved, &after_model, &window)?;

    // The proof: with the line archived, every output before and after the
    // marks change.
    let edited: Vec<(RelPath, String)> = files
        .iter()
        .map(|f| (f.path.clone(), f.text.clone()))
        .collect();
    let before = outputs(project, after_model.clone(), &moved, &[])?;
    let rewritten_model = load(&text, root)?;
    let after = outputs(project, rewritten_model.clone(), &text, &edited)?;
    let lines: Vec<&str> = rewritten_model
        .builds
        .iter()
        .filter(|b| b.versions.as_deref() == Some(target))
        .map(|b| b.name.as_str())
        .collect();
    let (in_lines, elsewhere): (Vec<ChangedOutput>, Vec<ChangedOutput>) =
        changed_outputs(&before, &after)
            .into_iter()
            .partition(|o| o.build.is_empty() || lines.contains(&o.build.as_str()));
    if !in_lines.is_empty() {
        return Err(RetireError::OutputsChanged {
            files: in_lines
                .iter()
                .flat_map(|o| {
                    o.changed
                        .iter()
                        .chain(&o.added)
                        .chain(&o.removed)
                        .map(move |f| format!("{}/{}/{f}", o.build, o.output))
                })
                .collect(),
        });
    }
    let (expected, elsewhere) =
        expected_changes(elsewhere, &before, &after, &rewritten_model, target);
    if !elsewhere.is_empty() && !retiring.force {
        return Err(RetireError::OtherBuildsChange {
            target: target.to_owned(),
            builds: elsewhere
                .iter()
                .map(|o| {
                    let mut pages: Vec<&str> = o
                        .changed
                        .iter()
                        .chain(&o.added)
                        .chain(&o.removed)
                        .map(String::as_str)
                        .collect();
                    pages.sort_unstable();
                    format!("{}'s {} output ({})", o.build, o.output, pages.join(", "))
                })
                .collect(),
        });
    }

    // What's left in no line, and the marks that say nothing, in the
    // project as the retirement leaves it.
    let retired = crate::release::with_sources(project, rewritten_model, &text, &edited);
    let unused = unused_content(&retired, target);
    let removals = if retiring.remove_unused {
        removals(&retired, &unused)
    } else {
        Vec::new()
    };

    let redirects = redirects(project, releases, line, url);
    let (recorded, addresses_text) = recorded(project, &redirects);
    Ok(RetiredLine {
        target: target.to_owned(),
        line: line.text.clone(),
        url: url.to_owned(),
        ended,
        model_text: text,
        files,
        features,
        unused,
        removals,
        redirects,
        addresses_text,
        recorded,
        builds_changed: elsewhere,
        expected,
    })
}

/// `changed` split in two: the pages whose only change is `target`'s part
/// of their availability lines, and the rest. A page added or removed, or
/// in an output other than the site's or the plain one, is the rest.
fn expected_changes(
    changed: Vec<ChangedOutput>,
    before: &BTreeMap<String, Contents>,
    after: &BTreeMap<String, Contents>,
    model: &ContentModel,
    target: &str,
) -> (Vec<ChangedOutput>, Vec<ChangedOutput>) {
    let label = model.value_label(target).unwrap_or(target);
    let mut expected = Vec::new();
    let mut rest = Vec::new();
    for o in changed {
        let key = |file: &str| format!("{}/{}/{file}", o.build, o.output);
        let text = |outputs: &BTreeMap<String, Contents>, file: &str| match outputs.get(&key(file))
        {
            Some(Contents::Text(t)) => Some(without_target(&o.output, t, target, label)),
            _ => None,
        };
        let (only, other): (Vec<String>, Vec<String>) = o
            .changed
            .iter()
            .cloned()
            .partition(|file| text(before, file).is_some_and(|b| Some(b) == text(after, file)));
        if !only.is_empty() {
            expected.push(ChangedOutput {
                changed: only,
                added: Vec::new(),
                removed: Vec::new(),
                ..o.clone()
            });
        }
        if !other.is_empty() || !o.added.is_empty() || !o.removed.is_empty() {
            rest.push(ChangedOutput {
                changed: other,
                ..o
            });
        }
    }
    (expected, rest)
}

/// A page of `output` with `target`'s part of each availability line left
/// out, its label kept: in the site output, the state, versions, and text
/// of its `<ascribe-availability-target>` and of its entry in the
/// frontmatter's `available`; in the plain output, the parentheses after
/// its label on an `Available:` line. A page of another output as it is.
fn without_target(output: &str, text: &str, target: &str, label: &str) -> Option<String> {
    match output {
        "site" => {
            let (yaml, body) = ascribe_emit::split_frontmatter(text);
            let yaml = if yaml.is_empty() {
                String::new()
            } else {
                frontmatter_without(yaml, target)?
            };
            Some(format!("{yaml}\n{}", elements_without(body, target)))
        }
        "plain" => Some(
            text.lines()
                .map(|line| availability_without(line, label))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        _ => None,
    }
}

/// The frontmatter's YAML with `target`'s entries in `available` cut to
/// their target and dimension. `None` when it doesn't parse.
fn frontmatter_without(yaml: &str, target: &str) -> Option<String> {
    let mut value: serde_yaml_ng::Value = serde_yaml_ng::from_str(yaml).ok()?;
    if let Some(serde_yaml_ng::Value::Sequence(entries)) = value.get_mut("available") {
        for entry in entries {
            let serde_yaml_ng::Value::Mapping(map) = entry else {
                continue;
            };
            if map.get("target").and_then(serde_yaml_ng::Value::as_str) == Some(target) {
                map.retain(|k, _| matches!(k.as_str(), Some("target" | "dimension")));
            }
        }
    }
    serde_yaml_ng::to_string(&value).ok()
}

/// The site output's body with each `<ascribe-availability-target>` for
/// `target` written with its target alone, and no text.
fn elements_without(body: &str, target: &str) -> String {
    let element = ascribe_core::names::ELEMENT_AVAILABILITY_TARGET;
    let open = format!("<{element} target=\"{target}\"");
    let close = format!("</{element}>");
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(at) = rest.find(&open) {
        let after = &rest[at + open.len()..];
        let ends = (after.starts_with([' ', '>']))
            .then(|| after.find(&close))
            .flatten();
        let Some(end) = ends else {
            out.push_str(&rest[..at + open.len()]);
            rest = after;
            continue;
        };
        out.push_str(&rest[..at]);
        out.push_str(&open);
        out.push('>');
        rest = &after[end + close.len()..];
    }
    out.push_str(rest);
    out
}

/// A line of the plain output with the parentheses after `label` left out
/// where it begins an entry of an `Available:` line: `Available: Quill
/// Cloud (GA); self-managed (preview, 3.4+)` is `Available: Quill Cloud
/// (GA); self-managed`.
fn availability_without(line: &str, label: &str) -> String {
    let Some(start) = line.find("Available: ") else {
        return line.to_owned();
    };
    let (head, tail) = line.split_at(start + "Available: ".len());
    // Each entry follows `; `, the first as well once one is put before it.
    let tail = format!("; {tail}");
    let entry = format!("; {label} (");
    let mut out = String::new();
    let mut rest = tail.as_str();
    while let Some(at) = rest.find(&entry) {
        let inside = at + entry.len();
        let Some(close) = rest[inside..].find(')') else {
            break;
        };
        out.push_str(&rest[..inside - 2]);
        rest = &rest[inside + close + 1..];
    }
    out.push_str(rest);
    format!("{head}{}", &out[2..])
}

/// What a line that's left the window leaves in no line: the content no
/// live or planned line of `target` has, and the marks that say nothing,
/// with, when `remove` is set, each file's text once they're deleted.
pub fn unused_in(
    project: &Project,
    target: &str,
    remove: bool,
) -> (Vec<Unused>, Vec<(RelPath, Option<String>)>) {
    let unused = unused_content(project, target);
    let removed = if remove {
        removals(project, &unused)
    } else {
        Vec::new()
    };
    (unused, removed)
}

/// `ascribe.toml`'s text with each feature's spec rewritten for the window,
/// and the features rewritten.
fn rewrite_features(
    moved: &str,
    model: &ContentModel,
    window: &Window,
) -> Result<(String, Vec<MovedFeature>), RetireError> {
    let file = ModelFile::parse(moved).map_err(RetireError::ModelEdit)?;
    let mut edits = Vec::new();
    let mut features = Vec::new();
    for feature in &model.features {
        let Some(simpler) = window.simpler(model, &feature.available) else {
            continue;
        };
        let to = spec_text(&simpler);
        edits.push(
            file.replace_value(
                &path(&["features", &feature.key, "available"]),
                Value::from(to.as_str()),
            )
            .map_err(RetireError::ModelEdit)?,
        );
        features.push(MovedFeature {
            key: feature.key.clone(),
            from: feature.available_text.trim().to_owned(),
            to,
            kept: Vec::new(),
        });
    }
    let text = if edits.is_empty() {
        moved.to_owned()
    } else {
        finish(&file, edits)?
    };
    Ok((text, features))
}

/// Whether the line can be retired: the target's list, when it can.
fn refuse<'m>(
    model: &'m ContentModel,
    retiring: &RetiringLine<'_>,
) -> Result<&'m Releases, RetireError> {
    let target = retiring.target;
    let line = retiring.line;
    let releases = model
        .releases_of(target)
        .filter(|r| !r.lines().is_empty())
        .ok_or_else(|| RetireError::NoLines {
            target: target.to_owned(),
        })?;
    if !model
        .builds
        .iter()
        .any(|b| b.versions.as_deref() == Some(target))
    {
        return Err(RetireError::NotVersioned {
            target: target.to_owned(),
        });
    }
    let not_live = |why: &str| RetireError::NotLive {
        target: target.to_owned(),
        line: line.text.clone(),
        why: why.to_owned(),
    };
    let oldest = releases
        .oldest_live()
        .map_or_else(String::new, |l| l.text.clone());
    match releases.status_of(line) {
        None => return Err(not_live("it isn't declared")),
        Some(LineStatus::Archived) => return Err(not_live("it's archived already")),
        Some(LineStatus::Planned) => return Err(not_live("it's planned")),
        Some(LineStatus::Current) => {
            return Err(RetireError::Current {
                target: target.to_owned(),
                line: line.text.clone(),
                oldest,
            });
        }
        Some(LineStatus::Live) => {}
    }
    if releases
        .oldest_live()
        .is_some_and(|o| o.components != line.components)
    {
        return Err(RetireError::NotOldest {
            target: target.to_owned(),
            line: line.text.clone(),
            oldest,
        });
    }
    Ok(releases)
}

/// The lines left in the window: the live and planned ones.
struct Window<'t> {
    target: &'t str,
    oldest: Option<Version>,
    lines: Vec<Version>,
}

impl<'t> Window<'t> {
    fn of(model: &ContentModel, target: &'t str) -> Window<'t> {
        let releases = model.releases_of(target);
        Window {
            target,
            oldest: releases.and_then(|r| r.oldest_live()).cloned(),
            lines: releases
                .map(|r| r.live.iter().chain(&r.planned).cloned().collect())
                .unwrap_or_default(),
        }
    }

    /// Whether some line in the window has content with `spec`. A spec
    /// that doesn't name the target says the same of every line.
    fn holds(&self, model: &ContentModel, spec: &AvailabilitySpec) -> bool {
        let mut said = false;
        for line in &self.lines {
            match in_release_line(model, spec, self.target, line) {
                None => return true,
                Some(in_line) if in_line.available => return true,
                Some(_) => said = true,
            }
        }
        !said
    }

    /// The simplest spec that means the same in every line in the window,
    /// when that isn't `spec`; `None` too for content in no line, which is
    /// listed instead.
    fn simpler(&self, model: &ContentModel, spec: &AvailabilitySpec) -> Option<AvailabilitySpec> {
        let oldest = self.oldest.as_ref()?;
        if !self.holds(model, spec) {
            return None;
        }
        without_versions_through(spec, self.target, oldest)
    }
}

/// Where a mark is written.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    /// `@available:`'s primary.
    Directive,
    /// A table row's `available` attribute.
    Row,
    /// The frontmatter's `available`.
    Frontmatter,
}

/// A mark in a file, as written.
struct Mark<'f> {
    place: Place,
    /// The directive line or the row's attribute block; the value, for the
    /// frontmatter.
    at: Span,
    /// The spec's text, and where it's written.
    text: &'f str,
    span: Span,
}

/// Every mark in `file` whose spec is written there, not a feature key's.
fn marks<'f>(file: &'f FileIndex, model: &ContentModel) -> Vec<(Mark<'f>, AvailabilitySpec)> {
    let source: &str = &file.source;
    let mut out = Vec::new();
    for marker in &file.availability {
        let (Some(Ok(spec)), Some((_, span))) = (&marker.spec, &marker.primary) else {
            continue;
        };
        if spec
            .bare_name()
            .is_some_and(|n| model.feature(&n.text).is_some())
        {
            continue;
        }
        let Some(text) = source.get(span.range()) else {
            continue;
        };
        let place = if marker.binding.is_some() {
            Place::Directive
        } else {
            Place::Row
        };
        let mark = Mark {
            place,
            at: marker.span,
            text,
            span: *span,
        };
        out.push((mark, spec.clone()));
    }
    if let Some((_, span)) = frontmatter_available(file)
        && let Some(text) = source.get(span.range())
        && let Ok(spec) = parse_availability(text, span.start())
        && !spec
            .bare_name()
            .is_some_and(|n| model.feature(&n.text).is_some())
    {
        let mark = Mark {
            place: Place::Frontmatter,
            at: span,
            text,
            span,
        };
        out.push((mark, spec));
    }
    out.sort_by_key(|(m, _)| (m.span.start(), m.span.end()));
    out
}

/// The file with each mark for the window's target rewritten to its
/// simplest spec; `None` when none changes.
fn rewrite_marks(
    file: &FileIndex,
    model: &ContentModel,
    window: &Window<'_>,
) -> Result<Option<ReleasedFile>, RetireError> {
    let source: &str = &file.source;
    let lines = LineIndex::new(source);
    let mut edits = Vec::new();
    let mut rewritten = Vec::new();
    for (mark, spec) in marks(file, model) {
        let Some(simpler) = window.simpler(model, &spec) else {
            continue;
        };
        let line = lines
            .line_col(mark.span.start())
            .map_or(0, |lc| lc.line + 1);
        let to = spec_text(&simpler);
        let quoted = mark.span.start() > 0
            && matches!(
                source.as_bytes().get(mark.span.start() - 1),
                Some(b'"' | b'\'')
            );
        let replacement = match mark.place {
            _ if quoted => {
                if to.contains(['"', '\'', '\\']) {
                    return Err(RetireError::MarkUnrewritable {
                        path: file.path.to_string(),
                        line,
                        written: mark.text.to_owned(),
                    });
                }
                to.clone()
            }
            Place::Directive => to.clone(),
            Place::Row if to.contains([' ', ',']) => format!("\"{to}\""),
            Place::Row => to.clone(),
            Place::Frontmatter if yaml_plain(&to) => to.clone(),
            Place::Frontmatter => format!("\"{to}\""),
        };
        if mark.text.contains('\n') {
            return Err(RetireError::MarkUnrewritable {
                path: file.path.to_string(),
                line,
                written: mark.text.to_owned(),
            });
        }
        edits.push(TextEdit::replace(mark.span, replacement));
        rewritten.push(RewrittenMark {
            line,
            from: mark.text.trim().to_owned(),
            to,
        });
    }
    if edits.is_empty() {
        return Ok(None);
    }
    let text = apply_edits(source, &edits).map_err(|_| RetireError::MarkUnrewritable {
        path: file.path.to_string(),
        line: rewritten.first().map_or(0, |m| m.line),
        written: String::new(),
    })?;
    Ok(Some(ReleasedFile {
        path: file.path.clone(),
        text,
        marks: rewritten,
    }))
}

/// The content no live or planned line of `target` has, and the marks that
/// say nothing, in path order, then by line. Content inside content listed
/// already isn't listed again.
fn unused_content(project: &Project, target: &str) -> Vec<Unused> {
    let model = project.model();
    let window = Window::of(model, target);
    let index = project.index();
    let says_nothing = marks_that_say_nothing(index, model);
    let mut out: Vec<Unused> = Vec::new();
    for file in index.files() {
        let source: &str = &file.source;
        let lines = LineIndex::new(source);
        let line_of = |at: usize| lines.line_col(at).map_or(0, |lc| lc.line + 1);
        let mut found: Vec<Unused> = Vec::new();
        for (mark, spec) in marks(file, model) {
            let line = line_of(mark.at.start());
            let resolved = spec.clone();
            if !window.holds(model, &resolved) {
                if let Some((kind, span, text)) = content_of(file, &mark) {
                    found.push(Unused {
                        path: file.path.clone(),
                        line,
                        kind,
                        text,
                        span,
                    });
                }
            } else if says_nothing.contains(&(file.file, mark.at))
                && let Some(span) = mark_only(file, &mark)
            {
                found.push(Unused {
                    path: file.path.clone(),
                    line,
                    kind: "mark",
                    text: source
                        .get(span.range())
                        .unwrap_or_default()
                        .trim()
                        .to_owned(),
                    span,
                });
            }
        }
        // A whole page goes with everything in it; otherwise what's inside
        // something listed goes with it.
        if let Some(page) = found.iter().find(|u| u.kind == "page") {
            out.push(page.clone());
            continue;
        }
        let mut kept: Vec<Unused> = Vec::new();
        for u in found {
            if !kept.iter().any(|k| k.span.contains_span(u.span)) {
                kept.retain(|k| !u.span.contains_span(k.span));
                kept.push(u);
            }
        }
        kept.sort_by_key(|u| u.line);
        out.extend(kept);
    }
    out
}

/// The marks, by file and where each is written, that say nothing: every
/// entry generally available with no version, naming exactly the targets
/// the scope it sits in allows, with no version in that scope for them, so
/// that nothing inside them depends on them either. A fragment's mark says
/// nothing only when it does in every page that includes it.
fn marks_that_say_nothing(
    index: &ascribe_resolve::Project,
    model: &ContentModel,
) -> BTreeSet<(FileId, Span)> {
    let values: Vec<&str> = model
        .dimensions
        .iter()
        .flat_map(|d| d.values.iter().map(|v| v.value.as_str()))
        .collect();
    let mut verdicts: BTreeMap<(FileId, Span), bool> = BTreeMap::new();
    for page in index.pages() {
        for a in index.scopes(&page.path) {
            let at = (a.written_at.file, a.written_at.span);
            let verdict = says_nothing(model, &values, &a);
            verdicts
                .entry(at)
                .and_modify(|v| *v &= verdict)
                .or_insert(verdict);
        }
    }
    verdicts
        .into_iter()
        .filter_map(|(at, v)| v.then_some(at))
        .collect()
}

fn says_nothing(model: &ContentModel, values: &[&str], a: &Arc<Availability>) -> bool {
    let ga = |d: &Detail| match d {
        Detail::None => true,
        Detail::State {
            state,
            version: None,
        } => state.text == "ga",
        _ => false,
    };
    if !a.spec.entries.iter().all(|e| ga(&e.detail)) {
        return false;
    }
    let listed = |spec: &AvailabilitySpec, value: &str| entry_for(model, spec, value).is_some();
    let mine: BTreeSet<&str> = values
        .iter()
        .copied()
        .filter(|v| listed(&a.spec, v))
        .collect();
    let versioned = |d: &Detail| !matches!(d, Detail::None | Detail::State { version: None, .. });
    let mut scope: BTreeSet<&str> = values.iter().copied().collect();
    let mut at = a.enclosing.as_deref();
    while let Some(e) = at {
        scope.retain(|v| listed(&e.spec, v));
        // A scope that gives one of the mark's targets a version keeps the
        // mark: a scope inside it may start earlier only because of it.
        if mine
            .iter()
            .any(|v| entry_for(model, &e.spec, v).is_some_and(|x| versioned(&x.detail)))
        {
            return false;
        }
        at = e.enclosing.as_deref();
    }
    mine == scope
}

/// The entry of a spec that decides `value`: a direct one, else one
/// through the value's dimension name.
fn entry_for<'s>(
    model: &ContentModel,
    spec: &'s AvailabilitySpec,
    value: &str,
) -> Option<&'s ascribe_core::availability::Entry> {
    let dimension = model.dimension_of_value(value).map(|d| d.name.as_str());
    spec.entries
        .iter()
        .find(|e| e.target.text == value)
        .or_else(|| {
            spec.entries
                .iter()
                .find(|e| Some(e.target.text.as_str()) == dimension)
        })
}

/// What a mark holds, as `--remove-unused` deletes it: its kind, the text
/// it spans, and its first line.
fn content_of(file: &FileIndex, mark: &Mark<'_>) -> Option<(&'static str, Span, String)> {
    let source: &str = &file.source;
    match mark.place {
        Place::Frontmatter => Some((
            "page",
            Span::new(0, source.len()),
            file.title.clone().unwrap_or_else(|| file.path.to_string()),
        )),
        Place::Row => {
            let row = rows(&file.document.blocks)
                .into_iter()
                .find(|r| r.span.contains_span(mark.at))?;
            Some((
                "row",
                whole_lines(source, row.span),
                first_line(source, row.span),
            ))
        }
        Place::Directive => {
            let (siblings, i) = siblings_of(&file.document.blocks, mark.at)?;
            let BlockKind::Directive(line) = &siblings[i].kind else {
                return None;
            };
            match line.binding {
                Some(Bound::Heading) => {
                    let start = (0..i)
                        .rev()
                        .find(|&k| matches!(siblings[k].kind, BlockKind::Heading(_)));
                    let (from, level, heading) = match start {
                        Some(k) => {
                            let BlockKind::Heading(h) = &siblings[k].kind else {
                                return None;
                            };
                            let text = source.get(h.content.range()).unwrap_or_default();
                            (k, h.level, text.trim().to_owned())
                        }
                        None => (i, 7, first_line(source, siblings[i].span)),
                    };
                    let end = siblings[from + 1..]
                        .iter()
                        .find(|b| matches!(&b.kind, BlockKind::Heading(h) if h.level <= level))
                        .map_or_else(
                            || siblings.last().map_or(source.len(), |b| b.span.end()),
                            |b| line_start(source, b.span.start()),
                        );
                    let span = Span::new(line_start(source, siblings[from].span.start()), end);
                    Some(("section", whole_lines(source, span), heading))
                }
                _ => {
                    let j = (i + 1..siblings.len()).find(|&j| !binds_following(&siblings[j]))?;
                    let span = Span::new(siblings[i].span.start(), siblings[j].span.end());
                    Some((
                        "block",
                        whole_lines(source, span),
                        first_line(source, siblings[j].span),
                    ))
                }
            }
        }
    }
}

/// What a mark that says nothing takes with it: its line, or for a row its
/// attribute block and the space before it.
fn mark_only(file: &FileIndex, mark: &Mark<'_>) -> Option<Span> {
    let source: &str = &file.source;
    match mark.place {
        Place::Directive => Some(whole_lines(source, mark.at)),
        Place::Frontmatter => Some(whole_lines(source, mark.at)),
        Place::Row => {
            let start = source[..mark.at.start()]
                .trim_end_matches([' ', '\t'])
                .len();
            Some(Span::new(start, mark.at.end()))
        }
    }
}

fn binds_following(block: &Block) -> bool {
    block.binding() == Some(Bound::FollowingBlock)
}

/// The list of sibling blocks holding the directive written at `at`, and
/// its place in it.
fn siblings_of(blocks: &[Block], at: Span) -> Option<(&[Block], usize)> {
    for (i, block) in blocks.iter().enumerate() {
        if matches!(&block.kind, BlockKind::Directive(l) if l.span == at) {
            return Some((blocks, i));
        }
        if !block.span.contains_span(at) {
            continue;
        }
        for children in child_lists(block) {
            if let Some(found) = siblings_of(children, at) {
                return Some(found);
            }
        }
    }
    None
}

fn child_lists(block: &Block) -> Vec<&[Block]> {
    match &block.kind {
        BlockKind::BlockQuote(q) => vec![&q.children],
        BlockKind::List(l) => l.items.iter().map(|i| i.children.as_slice()).collect(),
        BlockKind::Container(c) => vec![&c.children],
        BlockKind::Group(g) => g.arms.iter().map(|a| a.children.as_slice()).collect(),
        _ => Vec::new(),
    }
}

/// Every table row in `blocks`, at any depth.
fn rows(blocks: &[Block]) -> Vec<&ascribe_syntax::TableRow> {
    let mut out = Vec::new();
    for block in blocks {
        if let BlockKind::Table(t) = &block.kind {
            out.extend(t.rows.iter().filter(|r| !r.header));
        }
        for children in child_lists(block) {
            out.extend(rows(children));
        }
    }
    out
}

/// The start of the line holding `at`.
fn line_start(source: &str, at: usize) -> usize {
    source[..at].rfind('\n').map_or(0, |i| i + 1)
}

/// `span` widened to whole lines, its last line's end included, and one
/// blank line after it when there's one before it too, so that removing it
/// leaves one blank line between what's around it.
fn whole_lines(source: &str, span: Span) -> Span {
    let start = line_start(source, span.start());
    let mut end = if span.end() > start && source[..span.end()].ends_with('\n') {
        span.end()
    } else {
        source[span.end()..]
            .find('\n')
            .map_or(source.len(), |i| span.end() + i + 1)
    };
    let blank_before = start == 0 || source[..start].ends_with("\n\n");
    if blank_before && source[end..].starts_with('\n') {
        end += 1;
    }
    Span::new(start, end)
}

/// The first line of `span`'s text, trimmed.
fn first_line(source: &str, span: Span) -> String {
    source
        .get(span.range())
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned()
}

/// Each file's text once `unused` is deleted, or `None` for a page that
/// goes; and each page that replaced a page that goes, without its
/// `replaces` line.
fn removals(project: &Project, unused: &[Unused]) -> Vec<(RelPath, Option<String>)> {
    let index = project.index();
    let mut by_file: BTreeMap<RelPath, Vec<Span>> = BTreeMap::new();
    let mut gone: Vec<RelPath> = Vec::new();
    for u in unused {
        if u.kind == "page" {
            gone.push(u.path.clone());
        } else {
            by_file.entry(u.path.clone()).or_default().push(u.span);
        }
    }
    // The `replaces` line goes with the page it names.
    for (by, replaced) in index.replacements() {
        if gone.contains(&replaced)
            && !gone.contains(&by)
            && let Some(file) = index.file(&by)
            && let Some(span) = replaces_span(file)
        {
            by_file
                .entry(by)
                .or_default()
                .push(whole_lines(&file.source, span));
        }
    }
    let mut out: Vec<(RelPath, Option<String>)> = gone.into_iter().map(|p| (p, None)).collect();
    for (path, spans) in by_file {
        let Some(file) = index.file(&path) else {
            continue;
        };
        let edits: Vec<TextEdit> = spans
            .into_iter()
            .map(|s| TextEdit::replace(s, String::new()))
            .collect();
        if let Ok(text) = apply_edits(&file.source, &edits) {
            out.push((path, Some(text)));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// `ascribe.addresses`' text once each of `redirects` that goes to a page
/// in the current line is recorded in `[moved]`, keyed by the address the
/// line published, which is from the host's root (SPEC §7.10): the build
/// then writes each. An address that goes to the archive isn't recorded,
/// since a `[moved]` value is a page of the project. With how many are
/// recorded; the text is `None` when the project has no file, it doesn't
/// read, or nothing changes.
fn recorded(project: &Project, redirects: &[Redirect]) -> (usize, Option<String>) {
    let Some(Ok(file)) = project.addresses() else {
        return (0, None);
    };
    let mut after = file.clone();
    let mut count = 0;
    for r in redirects {
        if let Some(page) = &r.page {
            after.record_move(&r.from, page.as_str());
            count += 1;
        }
    }
    let text = after.to_toml();
    (
        count,
        (project.addresses_text() != Some(text.as_str())).then_some(text),
    )
}

/// Where each page the retired line published should go: the same page in
/// the current line, the page that replaces it there, or the page in the
/// archive.
fn redirects(project: &Project, releases: &Releases, line: &Version, url: &str) -> Vec<Redirect> {
    let index = project.index();
    let model = index.model();
    let target = releases.target.as_str();
    let replacements = index.replacements();
    let url = url.trim_end_matches('/');
    let mut out = Vec::new();
    let mut declared: Vec<&str> = Vec::new();
    for build in &model.builds {
        if build.versions.as_deref() == Some(target) && !declared.contains(&build.declared_name()) {
            declared.push(build.declared_name());
        }
    }
    for name in declared {
        let (Some(old), Some(current)) = (
            model.line_build(name, line),
            releases.current().and_then(|c| model.line_build(name, c)),
        ) else {
            continue;
        };
        let router =
            |b: &ascribe_model::Build| AstroRouter::from_consumer(&model.for_build(b).consumer);
        let (old_router, current_router) = (router(&old), router(&current));
        for page in index.pages() {
            if index.dropped(&page.path, &old).is_some() {
                continue;
            }
            let from = ascribe_core::Router::route(&old_router, &page.path);
            let in_current = |p: &RelPath| index.dropped(p, &current).is_none();
            let (to, page, why) = if in_current(&page.path) {
                (
                    ascribe_core::Router::route(&current_router, &page.path),
                    Some(page.path.clone()),
                    RedirectWhy::Same,
                )
            } else if let Some((by, _)) = replacements
                .iter()
                .find(|(by, replaced)| **replaced == page.path && in_current(by))
            {
                (
                    ascribe_core::Router::route(&current_router, by),
                    Some(by.clone()),
                    RedirectWhy::Replaced,
                )
            } else {
                (
                    format!("{url}/{}", ascribe_emit::archive_page_file(&page.path)),
                    None,
                    RedirectWhy::Archive,
                )
            };
            out.push(Redirect {
                from,
                to,
                page,
                why,
            });
        }
    }
    out.sort_by(|a, b| a.from.cmp(&b.from));
    out
}

#[cfg(test)]
mod tests {
    use super::without_target;

    /// Whether two pages of `output` differ only in `self-managed`'s part
    /// of their availability lines.
    fn only_its_part(output: &str, before: &str, after: &str) -> bool {
        let without = |text| without_target(output, text, "self-managed", "Self-managed");
        without(before).is_some() && without(before) == without(after)
    }

    #[test]
    fn a_plain_line_may_change_the_targets_part_alone() {
        let before = "# Install\n\nAvailable: Quill Cloud (GA); Self-managed (preview, 3.4+)\n";
        let after = "# Install\n\nAvailable: Quill Cloud (GA); Self-managed (preview)\n";
        assert!(only_its_part("plain", before, after));
        let first = "| `a` (Available: Self-managed (GA, 3.4+); Quill Cloud (GA)) | A. |";
        let then = "| `a` (Available: Self-managed (GA); Quill Cloud (GA)) | A. |";
        assert!(only_its_part("plain", first, then));
    }

    #[test]
    fn another_targets_part_of_the_same_line_is_a_change() {
        let before = "Available: Quill Cloud (GA); Self-managed (preview, 3.4+)\n";
        let after = "Available: Quill Cloud (preview); Self-managed (preview)\n";
        assert!(!only_its_part("plain", before, after));
        // Nor may the target leave the line.
        assert!(!only_its_part(
            "plain",
            before,
            "Available: Quill Cloud (GA)\n"
        ));
        // Nor may the page change elsewhere.
        assert!(!only_its_part(
            "plain",
            "Self-managed (3.4+) only.\n",
            "Self-managed only.\n"
        ));
    }

    #[test]
    fn a_site_page_may_change_the_targets_element_and_frontmatter_entry_alone() {
        let page = |versions: &str, text: &str, cloud: &str| {
            format!(
                "---\ntitle: Install\navailable:\n- target: cloud\n  dimension: deployment\n  states:\n  - {cloud}\n  text: Quill Cloud\n- target: self-managed\n  dimension: deployment\n  states:\n  - preview\n{versions}  text: {text}\nsource: install.md\n---\n\n<ascribe-availability-target target=\"cloud\" dimension=\"deployment\" states=\"{cloud}\">Quill Cloud</ascribe-availability-target>; <ascribe-availability-target target=\"self-managed\" dimension=\"deployment\" states=\"preview\"{}>{text}</ascribe-availability-target>\n",
                if versions.is_empty() {
                    ""
                } else {
                    " versions=\"3.4\""
                }
            )
        };
        let before = page(
            "  versions:\n  - '3.4'\n",
            "Self-managed (preview, 3.4+)",
            "ga",
        );
        let after = page("", "Self-managed (preview)", "ga");
        assert!(only_its_part("site", &before, &after));
        let other = page("", "Self-managed (preview)", "preview");
        assert!(!only_its_part("site", &before, &other));
    }

    #[test]
    fn another_output_is_never_only_the_targets_part() {
        assert!(!only_its_part("json", "{}", "{}"));
    }
}
