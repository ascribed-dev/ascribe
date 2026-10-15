//! The adapter for the prose checks: Vale's alerts, through `[checks.vale]`.
//!
//! It handles the `prose` tag. Its diagnostics are the case's file-level
//! diagnostics with `ascribe_check::prose::lint`'s, the call `ascribe check
//! --vale` makes. Vale itself isn't run: a stand-in for it flags each word
//! written twice in a row outside code spans, as `Ascribe.Repeated` with the
//! preset's ignored scopes does, so a case shows what
//! Ascribe gives Vale and where an alert lands, on every platform. A
//! `[checks.vale] command` of `not-installed` is a Vale that can't be run.
//!
//! The case is checked in a copy, since the check writes the preset and the
//! vocabulary under the project's `.ascribe/`.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use ascribe_check::prose::{self, Action, Alert, Linter, Request, ValeError};
use ascribe_conformance::{AdapterError, AdapterResult, Case, ConformanceAdapter, Diagnostic};

/// Handles the cases that expect Vale's alerts.
pub struct ProseAdapter;

impl ConformanceAdapter for ProseAdapter {
    fn name(&self) -> &str {
        "prose"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "prose"
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        let copy = tempfile::tempdir().map_err(|e| AdapterError(e.to_string()))?;
        copy_dir(&case.dir, copy.path()).map_err(|e| AdapterError(e.to_string()))?;
        let mut copied = case.clone();
        copied.dir = copy.path().to_path_buf();
        if !case.model_is_shared {
            copied.model = copy.path().join("ascribe.toml");
        }
        let project = super::check::project(&copied)?;
        let mut found = ascribe_check::check_files(&project);
        found.extend(prose::lint(
            &project,
            None,
            &Repeated,
            Duration::from_secs(5),
        ));
        let leveled = ascribe_check::apply_levels(&project.model().checks, found);
        super::to_conformance(&project, leveled).map(Some)
    }
}

/// A stand-in for Vale that flags each word written twice in a row.
struct Repeated;

impl Linter for Repeated {
    fn lint(&self, request: &Request) -> Result<BTreeMap<String, Vec<Alert>>, ValeError> {
        if request.command == "not-installed" {
            return Err(ValeError::NotRun {
                command: request.command.clone(),
                reason: "it isn't installed, or isn't on the PATH".to_owned(),
            });
        }
        Ok(request
            .files
            .iter()
            .map(|(path, text)| (path.clone(), repeated(text)))
            .collect())
    }
}

/// The alerts for each word written twice in a row, by line and character
/// from 1, as Vale gives them.
fn repeated(text: &str) -> Vec<Alert> {
    let mut alerts = Vec::new();
    for (n, line) in text.lines().enumerate() {
        // A code span's text reads as nothing: Vale ignores the `code` scope.
        let mut in_code = false;
        let line: String = line
            .chars()
            .map(|c| {
                if c == '`' {
                    in_code = !in_code;
                }
                if in_code || c == '`' { ' ' } else { c }
            })
            .collect();
        let line = line.as_str();
        let words: Vec<(usize, &str)> = line
            .char_indices()
            .filter(|(i, c)| {
                c.is_alphanumeric()
                    && line[..*i]
                        .chars()
                        .next_back()
                        .is_none_or(|p| !p.is_alphanumeric())
            })
            .map(|(i, _)| {
                let end = line[i..]
                    .find(|c: char| !c.is_alphanumeric())
                    .map_or(line.len(), |e| i + e);
                (i, &line[i..end])
            })
            .collect();
        for pair in words.windows(2) {
            let [(_, first), (at, second)] = pair else {
                continue;
            };
            let (start, _) = pair[0];
            let between = &line[start + first.len()..*at];
            if first == second && between.chars().all(char::is_whitespace) {
                let from = line[..start].chars().count() + 1;
                let to = line[..at + second.len()].chars().count();
                alerts.push(Alert {
                    check: "Ascribe.Repeated".to_owned(),
                    message: format!("'{second}' is repeated."),
                    severity: "suggestion".to_owned(),
                    line: n + 1,
                    span: (from, to),
                    action: Action {
                        name: "remove".to_owned(),
                        ..Action::default()
                    },
                    ..Alert::default()
                });
            }
        }
    }
    alerts
}

/// Copies a folder and everything in it.
fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
