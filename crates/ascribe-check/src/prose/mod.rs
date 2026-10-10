//! The prose, checked with Vale (`[checks.vale]`).
//!
//! Each page's prose is extracted with a map back to its source
//! ([`Prose`]), written where Vale can read it, and linted in one run of
//! Vale for every file asked about; each alert comes back as a `prose`
//! diagnostic at its place in the source, and one in a phrase's text at the
//! phrase. A Vale that can't be run, fails, or runs out of time is one
//! `prose-not-checked` advice on `[checks.vale]`, never a failed check.
//!
//! Fragments are linted as files of their own, once, and an include is
//! a directive line like any other, so nothing included is linted twice.

mod eject;
mod extract;
mod setup;
mod vale;

use std::path::Path;
use std::time::Duration;

use ascribe_core::{Applicability, FileId, Fix, Issue, Location, TextEdit, diagnostics};
use ascribe_model::{CheckLevel, ContentModel, ValeSettings};
use ascribe_syntax::{ParseOptions, parse};

use crate::{Diagnostic, Next, Project, Severity, apply_levels};

pub use eject::{EJECTED_CONFIG, EJECTED_STYLES, EjectError, Ejected, eject, ejected_files};
pub use extract::{Located, Prose};
pub use setup::{FOLDER, VOCABULARY, preset_files, vocabulary, with_settings};
pub use vale::{Action, Alert, Linter, Program, Request, ValeError};

/// How long one run of Vale on a whole project may take.
pub const PROJECT_TIMEOUT: Duration = Duration::from_secs(600);

/// How long one run of Vale on a few files may take: the editor's, on save.
pub const FILE_TIMEOUT: Duration = Duration::from_secs(30);

/// A source file to lint.
#[derive(Clone, Copy, Debug)]
pub struct Page<'a> {
    /// Its id, for the diagnostics' locations.
    pub id: FileId,
    /// Its path from the project root, `/`-separated: where Vale is told it
    /// is.
    pub path: &'a str,
    /// Its text.
    pub text: &'a str,
}

/// The prose diagnostics of the project's source files, or of `files` alone
/// when given, with `[checks]`'s levels applied: what Vale says about them,
/// or one `prose-not-checked` advice when it can't say. Nothing when the
/// project has no `[checks.vale]`.
pub fn lint(
    project: &Project,
    files: Option<&[FileId]>,
    linter: &dyn Linter,
    timeout: Duration,
) -> Vec<Diagnostic> {
    let paths: Vec<(FileId, String)> = project
        .sources()
        .iter()
        .filter(|s| s.unreadable.is_none())
        .filter(|s| files.is_none_or(|files| files.contains(&s.id)))
        .filter_map(|s| Some((s.id, project.display_path(s.id)?)))
        .collect();
    let pages: Vec<Page<'_>> = paths
        .iter()
        .filter_map(|(id, path)| {
            let source = project.sources().iter().find(|s| s.id == *id)?;
            Some(Page {
                id: *id,
                path,
                text: &source.text,
            })
        })
        .collect();
    lint_pages(project.root(), project.model(), &pages, linter, timeout)
}

/// [`lint`], for pages given one by one: the language server's, which keeps
/// the text it's sent.
pub fn lint_pages(
    root: &Path,
    model: &ContentModel,
    pages: &[Page<'_>],
    linter: &dyn Linter,
    timeout: Duration,
) -> Vec<Diagnostic> {
    let Some(settings) = &model.checks.vale else {
        return Vec::new();
    };
    let found = match run(root, model, settings, pages, linter, timeout) {
        Ok(found) => found,
        Err(e) => vec![not_checked(settings, &e)],
    };
    apply_levels(&model.checks, found)
}

fn run(
    root: &Path,
    model: &ContentModel,
    settings: &ValeSettings,
    pages: &[Page<'_>],
    linter: &dyn Linter,
    timeout: Duration,
) -> Result<Vec<Diagnostic>, ValeError> {
    let configs = setup::prepare(root, model, settings).map_err(|reason| ValeError::Failed {
        command: settings.command.clone(),
        reason,
    })?;
    let options = ParseOptions::new(model.directive_schemas())
        .with_note_types(model.notes.iter().map(|n| n.name.clone()).collect());
    let phrase = |key: &str| {
        model
            .phrases
            .iter()
            .find(|p| p.key == key)
            .map(|p| p.value.clone())
    };
    let prose: Vec<(Page<'_>, Prose)> = pages
        .iter()
        .map(|page| {
            let doc = parse(page.text, &options.clone().with_file(page.id));
            (*page, Prose::of(page.text, &doc, &phrase))
        })
        .filter(|(_, prose)| !prose.text.trim().is_empty())
        .collect();
    if prose.is_empty() {
        return Ok(Vec::new());
    }
    let request = Request {
        command: settings.command.clone(),
        root: root.to_path_buf(),
        configs,
        files: prose
            .iter()
            .map(|(page, prose)| (page.path.to_owned(), prose.text.clone()))
            .collect(),
        timeout,
    };
    let mut alerts = linter.lint(&request)?;
    let mut out = Vec::new();
    for (page, prose) in &prose {
        for alert in alerts.remove(page.path).unwrap_or_default() {
            if let Some(d) = diagnostic(settings, page, prose, &alert) {
                out.push(d);
            }
        }
    }
    out.sort_by_key(|d| {
        (
            d.location.file,
            d.location.span.start(),
            d.location.span.end(),
        )
    });
    Ok(out)
}

/// The diagnostic for one alert, at its place in the source.
fn diagnostic(
    settings: &ValeSettings,
    page: &Page<'_>,
    prose: &Prose,
    alert: &Alert,
) -> Option<Diagnostic> {
    let (start, end) = range_of(&prose.text, alert.line, alert.span)?;
    let located = prose.locate(start, end, page.text)?;
    let at = Location::new(page.id, located.span);
    let mut issue = Issue::new(diagnostics::PROSE, at)
        .with_arg("rule", alert.check.clone())
        .with_arg("message", alert.message.trim().to_owned());
    if !alert.link.is_empty() {
        issue = issue.with_arg("link", alert.link.clone());
    }
    if let Some(key) = &located.phrase {
        issue = issue.with_variant("phrase").with_arg("key", key.clone());
    }
    if located.exact {
        for fix in fixes(page.id, &located, alert, page.text) {
            issue = issue.with_fix(fix);
        }
    }
    let mut d = Diagnostic::from_issue(&issue);
    d.severity = severity(&alert.severity).min(cap(settings.max_level));
    d.rule = Some(alert.check.clone());
    if !d.fixes.is_empty() {
        d.next = Some(Next::Fix);
    }
    Some(d)
}

/// The replacements an alert offers, as fixes to review: each changes what
/// the page says.
fn fixes(file: FileId, located: &Located, alert: &Alert, source: &str) -> Vec<Fix> {
    let replacements: Vec<String> = match alert.action.name.as_str() {
        "replace" | "edit" => alert.suggestions.iter().take(3).cloned().collect(),
        "remove" => vec![String::new()],
        _ => Vec::new(),
    };
    let current = source.get(located.span.range()).unwrap_or_default();
    replacements
        .into_iter()
        .filter(|r| r != current)
        .map(|r| Fix {
            title: if r.is_empty() {
                format!("Remove “{current}”")
            } else {
                format!("Change “{current}” to “{r}”")
            },
            file,
            edits: vec![TextEdit::replace(located.span, r)],
            applicability: Applicability::Unsafe,
        })
        .collect()
}

/// The byte range of the prose that an alert's line and characters (from 1,
/// the last included) cover.
fn range_of(text: &str, line: usize, (first, last): (usize, usize)) -> Option<(usize, usize)> {
    let start_of_line = if line <= 1 {
        0
    } else {
        text.match_indices('\n').nth(line - 2)?.0 + 1
    };
    let rest = &text[start_of_line..];
    let line_text = &rest[..rest.find('\n').unwrap_or(rest.len())];
    let byte = |chars: usize| {
        line_text
            .char_indices()
            .nth(chars)
            .map_or(line_text.len(), |(i, _)| i)
    };
    let start = byte(first.saturating_sub(1));
    let end = byte(last.max(first));
    Some((start_of_line + start, start_of_line + end.max(start)))
}

/// The severity of one of Vale's levels.
fn severity(level: &str) -> Severity {
    match level {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        _ => Severity::Advice,
    }
}

/// The severity `max-level` allows at most.
fn cap(level: CheckLevel) -> Severity {
    match level {
        CheckLevel::Off | CheckLevel::Advice => Severity::Advice,
        CheckLevel::Warning => Severity::Warning,
        CheckLevel::Error => Severity::Error,
    }
}

/// The advice that the prose wasn't checked, on `[checks.vale]`.
fn not_checked(settings: &ValeSettings, error: &ValeError) -> Diagnostic {
    let at = Location::new(FileId::new(0), settings.span);
    let issue = Issue::new(diagnostics::PROSE_NOT_CHECKED, at);
    let issue = match error {
        ValeError::NotRun { command, reason } => issue
            .with_arg("command", command.clone())
            .with_arg("reason", reason.clone()),
        ValeError::Failed { command, reason } => issue
            .with_variant("failed")
            .with_arg("command", command.clone())
            .with_arg("reason", reason.clone()),
        ValeError::TimedOut { command, seconds } => issue
            .with_variant("timeout")
            .with_arg("command", command.clone())
            .with_arg("seconds", seconds.to_string()),
    };
    Diagnostic::from_issue(&issue)
}

#[cfg(test)]
mod tests;
