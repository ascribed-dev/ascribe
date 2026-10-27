//! Releasing a line, `ascribe versions release "<target> <line>"`: the
//! marks that write `planned` for the target rewritten to the line's
//! version, and the line moved from `planned` to `live`, in that order, so
//! that only the last step changes what a reader sees (README decision 20;
//! decision #811). When the release archives the line that was current, the
//! marks for the target that then say nothing are removed or rewritten, as
//! `available-same-in-every-line`'s fix does (#847).
//!
//! It writes nothing. [`release`] returns `ascribe.toml`'s text and each
//! source file's after the release, and what the release changes in the
//! outputs, built in memory before and after; the command asks git whether
//! the files can be put back, and writes them.

use std::collections::BTreeMap;
use std::path::Path;

use ascribe_check::Project;
use ascribe_core::availability::{Detail, Version, parse_availability};
use ascribe_core::{
    Coded, Date, FileId, Fix, LineIndex, RelPath, Span, TextEdit, apply_edits, diagnostics,
};
use ascribe_emit::{Contents, Output, WriteOptions, outputs_in_memory};
use ascribe_model::{ContentModel, LineStatus, Releases, SourceBound, in_line};
use ascribe_resolve::FileIndex;
use toml_edit::{Array, InlineTable, Value};

use crate::model_file::{Edit, ModelFile, path};

/// What to release.
#[derive(Clone, Copy, Debug)]
pub struct Releasing<'a> {
    /// The target, a versioned dimension value with lines in `[releases]`.
    pub target: &'a str,
    /// The line released.
    pub line: &'a Version,
    /// `--next`: the line planned after it, added to `planned` when it
    /// isn't there.
    pub next: Option<&'a Version>,
}

/// What a release changes, written nowhere yet.
#[derive(Clone, Debug)]
pub struct Released {
    /// The target.
    pub target: String,
    /// The line released, as given.
    pub line: String,
    /// Whether the line was in `planned`. A line that wasn't is a release
    /// nobody planned, such as a patch.
    pub was_planned: bool,
    /// `ascribe.toml`'s text after the release.
    pub model_text: String,
    /// Each source file the release rewrites marks in, in path order.
    pub files: Vec<ReleasedFile>,
    /// `--next`, the line planned after the released one.
    pub next: Option<String>,
    /// Whether `--next` was added to `planned`.
    pub next_added: bool,
    /// The line that was current, moved to `archived` with no address,
    /// when no build is versioned by the target.
    pub archived: Option<String>,
    /// Each source file with marks for the target that say nothing once
    /// `archived` is, removed or rewritten, in path order. Its text is the
    /// file's after the release, and replaces its text in `files`.
    pub cleared: Vec<ClearedFile>,
    /// Each feature whose spec says nothing for the target once `archived`
    /// is, rewritten.
    pub cleared_features: Vec<MovedFeature>,
    /// The builds' outputs the release changes, by build and output.
    pub outputs: Vec<ChangedOutput>,
}

/// A source file with marks that say nothing once the line that was
/// current is archived.
#[derive(Clone, Debug)]
pub struct ClearedFile {
    /// Its content path.
    pub path: RelPath,
    /// Its text after the release.
    pub text: String,
    /// Each mark removed or rewritten, in order.
    pub marks: Vec<ClearedMark>,
}

/// One mark that says nothing, removed or rewritten.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClearedMark {
    /// Its line, from 1, before the marks are cleared.
    pub line: u32,
    /// Its entry for the target, as written.
    pub from: String,
    /// The entry as rewritten; `None` when the mark is removed.
    pub to: Option<String>,
}

/// A source file whose marks the release rewrites.
#[derive(Clone, Debug)]
pub struct ReleasedFile {
    /// Its content path.
    pub path: RelPath,
    /// Its text after the release.
    pub text: String,
    /// Each mark rewritten, in order.
    pub marks: Vec<RewrittenMark>,
}

/// One mark rewritten.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewrittenMark {
    /// Its line, from 1.
    pub line: u32,
    /// The spec as written.
    pub from: String,
    /// The spec as rewritten.
    pub to: String,
}

/// A feature whose spec a command rewrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovedFeature {
    /// Its key.
    pub key: String,
    /// Its spec before.
    pub from: String,
    /// Its spec after.
    pub to: String,
    /// The other targets its spec names, which the release left as they
    /// were: someone should look at them.
    pub kept: Vec<String>,
}

/// One output of one build that the release changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedOutput {
    /// The build: `site`, or a line's, `site@3.4`.
    pub build: String,
    /// The output: `site`, `plain`, or `json`; `_ascribe` for the file the
    /// lines share.
    pub output: String,
    /// The files it gains, by their path in the output.
    pub added: Vec<String>,
    /// The files it loses.
    pub removed: Vec<String>,
    /// The files whose contents change.
    pub changed: Vec<String>,
}

/// Why a line can't be released. Nothing is written.
#[derive(Debug, thiserror::Error)]
pub enum ReleaseError {
    /// The target has no lines in `[releases]`.
    #[error(
        "`{target}` has no lines in [releases]; declare them as [releases.{target}] in ascribe.toml"
    )]
    NoLines {
        /// The target.
        target: String,
    },
    /// The line isn't one the target's lines can hold.
    #[error("can't release `{target} {line}`: {why}")]
    BadLine {
        /// The target.
        target: String,
        /// The line.
        line: String,
        /// Why.
        why: String,
    },
    /// The line is live already.
    #[error("`{target} {line}` is live already")]
    AlreadyLive {
        /// The target.
        target: String,
        /// The line.
        line: String,
    },
    /// The line has left the window.
    #[error("`{target} {line}` is archived")]
    Archived {
        /// The target.
        target: String,
        /// The line.
        line: String,
    },
    /// A planned line comes before this one.
    #[error("`{target} {line}` is planned after `{oldest}`; release `{target} {oldest}` first")]
    NotOldestPlanned {
        /// The target.
        target: String,
        /// The line.
        line: String,
        /// The oldest planned line.
        oldest: String,
    },
    /// The line isn't planned, and isn't between the current line and the
    /// oldest planned one.
    #[error("`{target} {line}` isn't planned, and {why}")]
    Unplanned {
        /// The target.
        target: String,
        /// The line.
        line: String,
        /// Where it falls instead.
        why: String,
    },
    /// `--next` isn't a line that can be planned after this one.
    #[error("--next {next} can't be the next line: {why}")]
    BadNext {
        /// The line given.
        next: String,
        /// Why.
        why: String,
    },
    /// A mark writes `planned` in a way the command can't rewrite.
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
    /// The content model after the release wouldn't load.
    #[error("ascribe.toml wouldn't load after the release: {0}")]
    ModelInvalid(String),
    /// A build's outputs can't be produced.
    #[error("can't build the outputs to compare: {0}")]
    Build(String),
    /// A step that should change no output changes some.
    #[error("{step} would change what a reader sees, in {}; nothing was written", files.join(", "))]
    OutputsChanged {
        /// The step.
        step: &'static str,
        /// The files, `<build>/<output>/<path>`.
        files: Vec<String>,
    },
}

impl Coded for ReleaseError {
    fn code(&self) -> &'static str {
        match self {
            ReleaseError::NoLines { .. } => "release_no_lines",
            ReleaseError::BadLine { .. } => "release_bad_line",
            ReleaseError::AlreadyLive { .. } => "release_line_live",
            ReleaseError::Archived { .. } => "release_line_archived",
            ReleaseError::NotOldestPlanned { .. } => "release_line_not_next",
            ReleaseError::Unplanned { .. } => "release_line_unplanned",
            ReleaseError::BadNext { .. } => "release_next_bad",
            ReleaseError::MarkUnrewritable { .. } => "release_mark_unrewritable",
            ReleaseError::ModelEdit(_) => "model_edit_refused",
            ReleaseError::ModelInvalid(_) => "release_model_invalid",
            ReleaseError::Build(_) => "release_build_failed",
            ReleaseError::OutputsChanged { .. } => "release_outputs_changed",
        }
    }
}

/// What the release does, decided from the list before anything is
/// rewritten.
struct Plan<'m> {
    releases: &'m Releases,
    was_planned: bool,
    /// Whether `planned` names the line, so its marks are rewritten.
    rewrite_planned: bool,
    /// `--next`.
    next: Option<Version>,
    /// Whether `next` is added to `planned`.
    add_next: bool,
    /// The current line, archived by the release.
    archive: Option<&'m Version>,
}

/// Releases `releasing.line` of `releasing.target` in `project`, in
/// memory: the files it would write, and what it changes in the outputs.
///
/// # Errors
///
/// The line can't be released (the [`ReleaseError`]s about the list, the
/// line, and `--next`); a mark or `ascribe.toml` can't be rewritten in
/// place; or rewriting the marks, which should change no output, would
/// change one.
pub fn release(project: &Project, releasing: &Releasing<'_>) -> Result<Released, ReleaseError> {
    let model = project.model();
    let plan = plan(model, releasing)?;
    let target = releasing.target;
    let line = releasing.line;
    let next = plan.next.as_ref();

    // Steps 2 and 3: the marks, and `--next`.
    let index = project.index();
    let mut files = Vec::new();
    if plan.rewrite_planned {
        for file in index.files() {
            if let Some(released) = rewrite_marks(file, target, line)? {
                files.push(released);
            }
        }
    }
    let mut text = project.model_text().to_owned();
    {
        let model_file = ModelFile::parse(&text).map_err(ReleaseError::ModelEdit)?;
        let mut edits = Vec::new();
        if plan.add_next
            && let Some(next) = next
        {
            let mut planned: Vec<&Version> = plan.releases.planned.iter().collect();
            planned.push(next);
            edits.push(set_lines(&model_file, target, "planned", planned)?);
        }
        text = finish(&model_file, edits)?;
    }
    let rewritten = text.clone();

    // Step 4: the line, from `planned` to `live`.
    {
        let model_file = ModelFile::parse(&text).map_err(ReleaseError::ModelEdit)?;
        let mut edits = Vec::new();
        if plan.was_planned {
            let mut planned: Vec<&Version> = plan
                .releases
                .planned
                .iter()
                .filter(|l| l.components != line.components)
                .collect();
            if plan.add_next
                && let Some(next) = next
            {
                planned.push(next);
            }
            edits.push(set_lines(&model_file, target, "planned", planned)?);
        }
        let mut live: Vec<&Version> = plan.releases.live.iter().collect();
        live.push(line);
        edits.push(set_lines(&model_file, target, "live", live)?);
        text = finish(&model_file, edits)?;
    }
    let moved = text.clone();

    // Step 5: the line that was current, to `archived`.
    if let Some(current) = plan.archive {
        let model_file = ModelFile::parse(&text).map_err(ReleaseError::ModelEdit)?;
        let mut live: Vec<&Version> = plan
            .releases
            .live
            .iter()
            .filter(|l| l.components != current.components)
            .collect();
        live.push(line);
        let mut edits = vec![set_lines(&model_file, target, "live", live)?];
        edits.extend(archive(&model_file, plan.releases, current, None, None)?);
        text = finish(&model_file, edits)?;
    }

    // Step 6: every build, before and after each step.
    let root = project.root();
    let before = outputs(project, project.model().clone(), project.model_text(), &[])?;
    let edited: Vec<(RelPath, String)> = files
        .iter()
        .map(|f| (f.path.clone(), f.text.clone()))
        .collect();
    let after_rewrite = outputs(project, load(&rewritten, root)?, &rewritten, &edited)?;
    let unexpected = compare(&before, &after_rewrite);
    if !unexpected.is_empty() {
        return Err(ReleaseError::OutputsChanged {
            step: "rewriting the marks",
            files: unexpected,
        });
    }
    let after_move = outputs(project, load(&moved, root)?, &moved, &edited)?;
    let mut cleared = Vec::new();
    let mut cleared_features = Vec::new();
    let after = if let Some(current) = plan.archive {
        let archived = outputs(project, load(&text, root)?, &text, &edited)?;
        let unexpected = compare(&after_move, &archived);
        if !unexpected.is_empty() {
            return Err(ReleaseError::OutputsChanged {
                step: "archiving the line that was current",
                files: unexpected,
            });
        }
        // Step 7: the marks that say nothing once the line is archived.
        let model = load(&text, root)?;
        let clearing = clear(project, &model, &text, &edited, target, current)?;
        if clearing.files.is_empty() && clearing.features.is_empty() {
            archived
        } else {
            text = clearing.model_text;
            cleared = clearing.files;
            cleared_features = clearing.features;
            let mut edited = edited;
            for file in &cleared {
                edited.retain(|(p, _)| *p != file.path);
                edited.push((file.path.clone(), file.text.clone()));
            }
            outputs(project, load(&text, root)?, &text, &edited)?
        }
    } else {
        after_move
    };

    Ok(Released {
        target: target.to_owned(),
        line: line.text.clone(),
        was_planned: plan.was_planned,
        model_text: text,
        files,
        next: next.map(|n| n.text.clone()),
        next_added: plan.add_next,
        archived: plan.archive.map(|l| l.text.clone()),
        cleared,
        cleared_features,
        outputs: changed_outputs(&before, &after),
    })
}

/// What clearing the marks that say nothing changes.
struct Clearing {
    model_text: String,
    files: Vec<ClearedFile>,
    features: Vec<MovedFeature>,
}

/// The marks and features for `target` that name a version in `archived`
/// and say nothing with `model`, whose text is `model_text`, and the source
/// files in `edited` holding their texts after: each one
/// `available-same-in-every-line` reports, removed or rewritten by its fix. A fix whose edits overlap one already taken in the
/// same file is left for `ascribe check` to report.
fn clear(
    project: &Project,
    model: &ContentModel,
    model_text: &str,
    edited: &[(RelPath, String)],
    target: &str,
    archived: &Version,
) -> Result<Clearing, ReleaseError> {
    let after = with_sources(project, model.clone(), model_text, edited);
    let found = ascribe_check::check_files(&after);
    let mut by: BTreeMap<FileId, Vec<(Span, &Fix)>> = BTreeMap::new();
    for d in &found {
        if d.slug != diagnostics::AVAILABLE_SAME_IN_EVERY_LINE {
            continue;
        }
        let Some(fix) = d.fixes.first() else {
            continue;
        };
        let file = d.location.file;
        let text = if file == MODEL_FILE_ID {
            Some(model_text)
        } else {
            after
                .sources()
                .iter()
                .find(|s| s.id == file)
                .map(|s| s.text.as_str())
        };
        // The entry the diagnostic is reported at, which names a version
        // in the archived line. One that said nothing before is left as it
        // was: the release didn't make it so.
        let entry = text
            .and_then(|t| t.get(d.location.span.range()))
            .and_then(|entry| parse_availability(entry, 0).ok());
        let archived_names = entry.is_some_and(|spec| {
            spec.entries.iter().any(|e| {
                e.target.text == target
                    && entry_versions(&e.detail)
                        .iter()
                        .any(|v| in_line(archived, v))
            })
        });
        if archived_names && fix.file == file {
            by.entry(file).or_default().push((d.location.span, fix));
        }
    }
    let mut out = Clearing {
        model_text: model_text.to_owned(),
        files: Vec::new(),
        features: Vec::new(),
    };
    for (file, fixes) in by {
        let (path, text) = if file == MODEL_FILE_ID {
            (None, model_text)
        } else {
            match after.sources().iter().find(|s| s.id == file) {
                Some(s) => (Some(&s.path), s.text.as_str()),
                None => continue,
            }
        };
        let lines = LineIndex::new(text);
        let mut edits: Vec<TextEdit> = Vec::new();
        let mut marks = Vec::new();
        for (entry, fix) in fixes {
            let overlaps = fix.edits.iter().any(|e| {
                edits
                    .iter()
                    .any(|t| e.span.start() < t.span.end() && t.span.start() < e.span.end())
            });
            if overlaps {
                continue;
            }
            edits.extend(fix.edits.iter().cloned());
            let removed = fix.edits.iter().any(|e| e.new_text.is_empty());
            marks.push(ClearedMark {
                line: lines.line_col(entry.start()).map_or(0, |lc| lc.line + 1),
                from: text.get(entry.range()).unwrap_or_default().to_owned(),
                to: if removed {
                    None
                } else {
                    fix.edits.first().map(|e| e.new_text.clone())
                },
            });
        }
        if marks.is_empty() {
            continue;
        }
        let unrewritable = || ReleaseError::MarkUnrewritable {
            path: path.map_or_else(|| "ascribe.toml".to_owned(), ToString::to_string),
            line: marks.first().map_or(0, |m| m.line),
            written: marks.first().map(|m| m.from.clone()).unwrap_or_default(),
        };
        let rewritten = apply_edits(text, &edits).map_err(|_| unrewritable())?;
        match path {
            Some(path) => out.files.push(ClearedFile {
                path: path.clone(),
                text: rewritten,
                marks,
            }),
            None => {
                let now = load(&rewritten, project.root())?;
                for feature in &model.features {
                    let Some(to) = now.features.iter().find(|f| f.key == feature.key) else {
                        continue;
                    };
                    if to.available_text != feature.available_text {
                        out.features.push(MovedFeature {
                            key: feature.key.clone(),
                            from: feature.available_text.trim().to_owned(),
                            to: to.available_text.trim().to_owned(),
                            kept: Vec::new(),
                        });
                    }
                }
                out.model_text = rewritten;
            }
        }
    }
    out.files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// The id a diagnostic about `ascribe.toml` is reported in.
const MODEL_FILE_ID: FileId = FileId::new(0);

/// Step 1: whether the line can be released, and what the release does.
fn plan<'m>(model: &'m ContentModel, r: &Releasing<'_>) -> Result<Plan<'m>, ReleaseError> {
    let target = r.target;
    let line = r.line;
    let releases = model
        .releases_of(target)
        .filter(|rel| !rel.lines().is_empty())
        .ok_or_else(|| ReleaseError::NoLines {
            target: target.to_owned(),
        })?;
    let bad = |why: String| ReleaseError::BadLine {
        target: target.to_owned(),
        line: line.text.clone(),
        why,
    };
    let parts = releases
        .lines()
        .first()
        .map_or(0, |(l, _)| l.components.len());
    if line.components.len() != parts {
        return Err(bad(format!(
            "a line of `{target}` has {parts} part{}, as the list writes them",
            if parts == 1 { "" } else { "s" }
        )));
    }
    let was_planned = match releases.status_of(line) {
        Some(LineStatus::Current | LineStatus::Live) => {
            return Err(ReleaseError::AlreadyLive {
                target: target.to_owned(),
                line: line.text.clone(),
            });
        }
        Some(LineStatus::Archived) => {
            return Err(ReleaseError::Archived {
                target: target.to_owned(),
                line: line.text.clone(),
            });
        }
        Some(LineStatus::Planned) => {
            if let Some(oldest) = releases.planned.last()
                && oldest.components != line.components
            {
                return Err(ReleaseError::NotOldestPlanned {
                    target: target.to_owned(),
                    line: line.text.clone(),
                    oldest: oldest.text.clone(),
                });
            }
            true
        }
        None => {
            let unplanned = |why: String| ReleaseError::Unplanned {
                target: target.to_owned(),
                line: line.text.clone(),
                why,
            };
            if let Some(current) = releases.current()
                && line.compare(current).is_le()
            {
                return Err(unplanned(format!(
                    "it isn't newer than the current line, `{}`",
                    current.text
                )));
            }
            if releases.current().is_none()
                && let Some(newest) = releases.archived.first()
                && line.compare(&newest.line).is_le()
            {
                return Err(unplanned(format!(
                    "it isn't newer than the archived line `{}`",
                    newest.line.text
                )));
            }
            if let Some(oldest) = releases.planned.last()
                && line.compare(oldest).is_ge()
            {
                return Err(unplanned(format!(
                    "it isn't older than the oldest planned line, `{}`; add it to `planned`, or release `{target} {}`",
                    oldest.text, oldest.text
                )));
            }
            false
        }
    };
    let rewrite_planned = model
        .planned_line(target)
        .is_some_and(|p| p.components == line.components);
    let (next, add_next) = match r.next {
        Some(next) => {
            let bad_next = |why: String| ReleaseError::BadNext {
                next: next.text.clone(),
                why,
            };
            if next.components.len() != parts {
                return Err(bad_next(format!(
                    "a line of `{target}` has {parts} part{}, as the list writes them",
                    if parts == 1 { "" } else { "s" }
                )));
            }
            if next.compare(line).is_le() {
                return Err(bad_next(format!(
                    "it isn't newer than `{}`, the line released",
                    line.text
                )));
            }
            match releases.status_of(next) {
                None => (Some(next.clone()), true),
                Some(LineStatus::Planned) => (Some(next.clone()), false),
                Some(status) => {
                    return Err(bad_next(format!("it's {}", status.as_str())));
                }
            }
        }
        None => (None, false),
    };
    let versioned = model
        .builds
        .iter()
        .any(|b| b.versions.as_deref() == Some(target));
    Ok(Plan {
        releases,
        was_planned,
        rewrite_planned,
        next,
        add_next,
        archive: if versioned { None } else { releases.current() },
    })
}

pub(crate) fn entry_versions(detail: &Detail) -> Vec<&Version> {
    match detail {
        Detail::None | Detail::State { version: None, .. } => Vec::new(),
        Detail::Version(v)
        | Detail::State {
            version: Some(v), ..
        } => vec![v],
        Detail::History(steps) => steps.iter().map(|s| &s.version).collect(),
    }
}

/// The edit that sets one of a target's lists, newest first, written as
/// the list writes its lines.
pub(crate) fn set_lines(
    file: &ModelFile<'_>,
    target: &str,
    key: &str,
    mut lines: Vec<&Version>,
) -> Result<Edit, ReleaseError> {
    lines.sort_by(|a, b| b.compare(a));
    let array: Array = lines.iter().map(|l| l.text.as_str()).collect();
    let value = Value::Array(array);
    let at = path(&["releases", target, key]);
    if file.get(&at).is_some() {
        file.replace_value(&at, value)
    } else {
        file.add_entry(&["releases", target], key, value)
    }
    .map_err(ReleaseError::ModelEdit)
}

/// The edits that add `line` to the target's `archived`, at `url` when
/// one's given, and the day its support ended: the one `ends` has, which
/// then leaves `ends`, or else `today` when one's given.
pub(crate) fn archive(
    file: &ModelFile<'_>,
    releases: &Releases,
    line: &Version,
    url: Option<&str>,
    today: Option<Date>,
) -> Result<Vec<Edit>, ReleaseError> {
    let current = line;
    let target = releases.target.as_str();
    let mut entry = InlineTable::new();
    let mut edits = Vec::new();
    if let Some(url) = url {
        entry.insert("url", Value::from(url));
    }
    let end = releases
        .ends
        .iter()
        .find(|e| e.line.components == current.components);
    if let Some(end) = end {
        edits.push(remove_end(file, target, &end.line.text)?);
    }
    if let Some(ended) = end.map(|e| e.date).or(today) {
        let date: toml_edit::Datetime = ended
            .to_string()
            .parse()
            .map_err(|_| ReleaseError::ModelEdit(format!("`{ended}` isn't a date")))?;
        entry.insert("ended", Value::from(date));
    }
    entry.fmt();
    let value = Value::InlineTable(entry);
    let table = ["releases", target, "archived"];
    let edit = if file.get(&path(&table)).is_some() {
        file.add_entry(&table, &current.text, value)
    } else {
        file.add_table(&table, &[(&current.text, value)], &["releases", target])
    };
    edits.push(edit.map_err(ReleaseError::ModelEdit)?);
    Ok(edits)
}

/// The edit that takes `line` out of the target's `ends`: its entry in an
/// `[releases.<target>.ends]` table, or in an inline `ends = { … }`, which
/// goes whole with its last entry.
fn remove_end(file: &ModelFile<'_>, target: &str, line: &str) -> Result<Edit, ReleaseError> {
    let at = path(&["releases", target, "ends"]);
    if let Some(toml_edit::Item::Value(Value::InlineTable(ends))) = file.get(&at) {
        let mut rest = ends.clone();
        rest.remove(line);
        return if rest.is_empty() {
            file.remove_entry(&["releases", target], "ends")
        } else {
            rest.fmt();
            file.replace_value(&at, Value::InlineTable(rest))
        }
        .map_err(ReleaseError::ModelEdit);
    }
    file.remove_entry(&["releases", target, "ends"], line)
        .map_err(ReleaseError::ModelEdit)
}

/// The text the edits give, checked.
pub(crate) fn finish(file: &ModelFile<'_>, edits: Vec<Edit>) -> Result<String, ReleaseError> {
    file.finish(edits)
        .map(|finished| finished.text)
        .map_err(ReleaseError::ModelEdit)
}

/// The content model in `text`.
pub(crate) fn load(text: &str, root: &Path) -> Result<ContentModel, ReleaseError> {
    ascribe_check::type_files::load_model(text, root, SourceBound::Anywhere).map_err(|issues| {
        let messages: Vec<String> = issues
            .iter()
            .map(ascribe_check::Diagnostic::from_issue)
            .filter(|d| d.severity == ascribe_check::Severity::Error)
            .map(|d| d.message)
            .collect();
        ReleaseError::ModelInvalid(messages.join("; "))
    })
}

/// Every build's site and plain outputs, in memory, for `project` with `model` and the
/// source files in `edited` holding their texts after.
pub(crate) fn outputs(
    project: &Project,
    model: ContentModel,
    model_text: &str,
    edited: &[(RelPath, String)],
) -> Result<BTreeMap<String, Contents>, ReleaseError> {
    let after = with_sources(project, model, model_text, edited);
    // What a reader sees. The JSON output records where each block is
    // written in its source and each mark as written, so every rewritten
    // mark changes it.
    let options = WriteOptions {
        outputs: vec![Output::Site, Output::Plain],
        anchors: false,
    };
    outputs_in_memory(after.index(), project.root(), &options)
        .map_err(|e| ReleaseError::Build(e.to_string()))
}

/// `project` with `model`, and the source files in `edited` holding their
/// texts after.
fn with_sources(
    project: &Project,
    model: ContentModel,
    model_text: &str,
    edited: &[(RelPath, String)],
) -> Project {
    let sources = project
        .sources()
        .iter()
        .map(|s| {
            let mut s = s.clone();
            if let Some((_, text)) = edited.iter().find(|(p, _)| *p == s.path) {
                s.text = text.clone();
            }
            s
        })
        .collect();
    Project::from_parts(
        project.root().to_path_buf(),
        project.content_root().clone(),
        model,
        model_text.to_owned(),
        sources,
    )
}

/// The files that differ between two sets of outputs.
pub(crate) fn compare(
    before: &BTreeMap<String, Contents>,
    after: &BTreeMap<String, Contents>,
) -> Vec<String> {
    let mut out: Vec<String> = before
        .iter()
        .filter(|(path, contents)| after.get(*path) != Some(*contents))
        .map(|(path, _)| path.clone())
        .collect();
    out.extend(after.keys().filter(|p| !before.contains_key(*p)).cloned());
    out.sort();
    out
}

/// What differs between two sets of outputs, by build and output.
pub(crate) fn changed_outputs(
    before: &BTreeMap<String, Contents>,
    after: &BTreeMap<String, Contents>,
) -> Vec<ChangedOutput> {
    #[derive(Clone, Copy)]
    enum How {
        Added,
        Removed,
        Changed,
    }
    let mut by: BTreeMap<(String, String), ChangedOutput> = BTreeMap::new();
    let mut note = |key: &str, how: How| {
        let Some((build, output, file)) = split_output(key) else {
            return;
        };
        let c = by
            .entry((build.clone(), output.clone()))
            .or_insert_with(|| ChangedOutput {
                build,
                output,
                added: Vec::new(),
                removed: Vec::new(),
                changed: Vec::new(),
            });
        match how {
            How::Added => c.added.push(file),
            How::Removed => c.removed.push(file),
            How::Changed => c.changed.push(file),
        }
    };
    for (key, contents) in before {
        match after.get(key) {
            None => note(key, How::Removed),
            Some(other) if other != contents => note(key, How::Changed),
            Some(_) => {}
        }
    }
    for key in after.keys().filter(|k| !before.contains_key(*k)) {
        note(key, How::Added);
    }
    by.into_values().collect()
}

/// `<build>/<output>/<path>` as its three parts; `_ascribe/<path>` is the
/// shared file's.
fn split_output(key: &str) -> Option<(String, String, String)> {
    let (build, rest) = key.split_once('/')?;
    if build == "_ascribe" {
        return Some((String::new(), build.to_owned(), rest.to_owned()));
    }
    let (output, file) = rest.split_once('/')?;
    Some((build.to_owned(), output.to_owned(), file.to_owned()))
}

/// The file with each `planned` its marks write for `target` rewritten to
/// `line`, the line `planned` names; `None` when it has none.
fn rewrite_marks(
    file: &FileIndex,
    target: &str,
    line: &Version,
) -> Result<Option<ReleasedFile>, ReleaseError> {
    let source: &str = &file.source;
    let mut found: Vec<Span> = file
        .availability
        .iter()
        .filter_map(|marker| match (&marker.spec, &marker.primary) {
            (Some(Ok(_)), Some((_, span))) => Some(*span),
            _ => None,
        })
        .collect();
    found.extend(frontmatter_available(file).map(|(_, span)| span));
    found.sort_by_key(|span| (span.start(), span.end()));
    let lines = LineIndex::new(source);
    let mut edits = Vec::new();
    let mut marks = Vec::new();
    for span in found {
        let Some(written) = source.get(span.range()) else {
            continue;
        };
        let Ok(spec) = parse_availability(written, span.start()) else {
            continue;
        };
        let words: Vec<Span> = spec
            .entries
            .iter()
            .filter(|e| e.target.text == target)
            .flat_map(|e| entry_versions(&e.detail))
            .filter(|v| v.is_planned())
            .map(|v| v.span)
            .collect();
        if words.is_empty() {
            continue;
        }
        let at = lines.line_col(span.start()).map_or(0, |lc| lc.line + 1);
        // A row's value or a frontmatter value whose escapes or line breaks
        // move the spec off its source can't be rewritten where it's
        // written.
        if written.contains(['\n', '\\']) {
            return Err(ReleaseError::MarkUnrewritable {
                path: file.path.to_string(),
                line: at,
                written: written.trim().to_owned(),
            });
        }
        let mine: Vec<TextEdit> = words
            .iter()
            .map(|w| TextEdit::replace(*w, line.text.clone()))
            .collect();
        let local: Vec<TextEdit> = words
            .iter()
            .map(|w| {
                TextEdit::replace(
                    Span::new(w.start() - span.start(), w.end() - span.start()),
                    line.text.clone(),
                )
            })
            .collect();
        let to = apply_edits(written, &local).map_err(|_| ReleaseError::MarkUnrewritable {
            path: file.path.to_string(),
            line: at,
            written: written.trim().to_owned(),
        })?;
        edits.extend(mine);
        marks.push(RewrittenMark {
            line: at,
            from: written.trim().to_owned(),
            to: to.trim().to_owned(),
        });
    }
    if marks.is_empty() {
        return Ok(None);
    }
    let text = apply_edits(source, &edits).map_err(|_| ReleaseError::MarkUnrewritable {
        path: file.path.to_string(),
        line: marks.first().map_or(0, |m| m.line),
        written: String::new(),
    })?;
    Ok(Some(ReleasedFile {
        path: file.path.clone(),
        text,
        marks,
    }))
}

/// The frontmatter's `available` value, unquoted, and where it's written,
/// when it's on its key's line.
pub(crate) fn frontmatter_available(file: &FileIndex) -> Option<(String, Span)> {
    let fm = file.document.frontmatter.as_ref()?;
    let source: &str = &file.source;
    let base = fm.content.start();
    let text = source.get(fm.content.range())?;
    ascribe_syntax::frontmatter_lines(text)
        .into_iter()
        .find(|l| l.indent == 0 && l.item.is_none() && !l.block && l.key == Some("available"))
        .and_then(|l| crate::nav::frontmatter_available(file, base + l.start))
}

/// Whether a spec can be written as a plain YAML scalar.
pub(crate) fn yaml_plain(spec: &str) -> bool {
    !spec.contains(": ")
        && !spec.contains(" #")
        && !spec.starts_with(|c: char| "[]{}&*!|>'\"%@`#-?:,".contains(c))
}
