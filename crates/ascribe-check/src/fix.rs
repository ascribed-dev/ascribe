//! Applying the safe fixes: `ascribe check --fix`.
//!
//! A fix is [`Applicability::Safe`] only when applying it can't change what
//! the page says and leaves nothing to decide (decision 51), so these are
//! applied without asking, as `ascribe fmt` rewrites what it can. Unsafe
//! fixes are never applied here.
//!
//! [`safe_fixes`] works out each file's new text from what the check read;
//! [`write`] writes it. A fix whose edits overlap one already taken in the
//! same file is left for a later run, which reports its problem again.

use std::path::{Path, PathBuf};

use ascribe_core::{Applicability, FileId, TextEdit, apply_edits};

use crate::{Diagnostic, LOCK_FILE_ID, NAV_FILE_ID, PERMALINKS_FILE_ID, Project};

/// One fix applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    /// The code of the diagnostic it fixes, such as `ASC036`.
    pub code: &'static str,
    /// The diagnostic's name.
    pub slug: String,
    /// What the fix does.
    pub title: String,
}

/// A file the safe fixes change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedFile {
    /// The file.
    pub file: FileId,
    /// Where it is on disk.
    pub path: PathBuf,
    /// The path a report shows for it, as [`Project::display_path`] gives.
    pub display_path: String,
    /// The fixes applied to it, in the order of their diagnostics.
    pub applied: Vec<Applied>,
    /// Its text with the fixes applied.
    pub text: String,
}

/// Why the fixed files couldn't be written.
#[derive(Debug, thiserror::Error)]
#[error("can't write {}: {source}", path.display())]
pub struct FixError {
    /// The file.
    pub path: PathBuf,
    /// What went wrong.
    pub source: std::io::Error,
}

impl ascribe_core::Coded for FixError {
    fn code(&self) -> &'static str {
        "fix_unwritable"
    }
}

/// The files the safe fixes of `diagnostics` change, in file order, with
/// their new text: the text `project` read, with each safe fix's edits
/// applied. `model` is the content model's file, which the project knows
/// only as the folder it's in.
///
/// The same fix offered by two diagnostics is applied once. A fix whose
/// edits overlap one already taken, or don't fit the file, is left out.
/// Only the content model, the lock, the permalinks list, the navigation
/// file, and the source files that were read are changed; nothing else a fix
/// could name is.
pub fn safe_fixes<'a>(
    project: &Project,
    model: &Path,
    diagnostics: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<FixedFile> {
    let mut taken: Vec<Taken> = Vec::new();
    for d in diagnostics {
        for fix in &d.fixes {
            if fix.applicability != Applicability::Safe || fix.edits.is_empty() {
                continue;
            }
            let Some(text) = writable_text(project, fix.file) else {
                continue;
            };
            let i = match taken.iter().position(|t| t.file == fix.file) {
                Some(i) => i,
                None => {
                    taken.push(Taken {
                        file: fix.file,
                        edits: Settled::default(),
                        applied: Vec::new(),
                    });
                    taken.len() - 1
                }
            };
            let file = &mut taken[i];
            if !file.edits.take(text, &fix.edits) {
                continue;
            }
            file.applied.push(Applied {
                code: d.code,
                slug: d.slug.to_string(),
                title: fix.title.clone(),
            });
        }
    }
    let mut files: Vec<FixedFile> = taken
        .into_iter()
        .filter(|t| !t.edits.edits().is_empty())
        .filter_map(
            |Taken {
                 file,
                 edits,
                 applied,
             }| {
                let text = apply_edits(writable_text(project, file)?, edits.edits()).ok()?;
                Some(FixedFile {
                    file,
                    path: path_on_disk(project, model, file)?,
                    display_path: project.display_path(file)?,
                    applied,
                    text,
                })
            },
        )
        .collect();
    files.sort_by_key(|f| f.file);
    files
}

/// The fixes taken for one file so far: their edits, and what they fix.
struct Taken {
    file: FileId,
    edits: Settled,
    applied: Vec<Applied>,
}

/// The edits of the fixes taken for one file, settled as `--fix` settles
/// them: the editor's action that applies a page's safe fixes takes them
/// the same way.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settled {
    edits: Vec<TextEdit>,
}

impl Settled {
    /// Takes a fix's edits of `text`, unless they overlap one already taken,
    /// are the same fix offered again, or don't fit the text: whether it
    /// took them.
    pub fn take(&mut self, text: &str, fix: &[TextEdit]) -> bool {
        if overlaps(&self.edits, fix) {
            return false;
        }
        let mut with = self.edits.clone();
        with.extend(fix.iter().cloned());
        if apply_edits(text, &with).is_err() {
            return false;
        }
        self.edits = with;
        true
    }

    /// The edits taken, in the order they were.
    pub fn edits(&self) -> &[TextEdit] {
        &self.edits
    }
}

/// Writes each fixed file's new text.
///
/// # Errors
///
/// A file couldn't be written. The files before it in the list were.
pub fn write(files: &[FixedFile]) -> Result<(), FixError> {
    for file in files {
        std::fs::write(&file.path, &file.text).map_err(|source| FixError {
            path: file.path.clone(),
            source,
        })?;
    }
    Ok(())
}

/// Whether an edit of `fix` touches the text an edit of `taken` does, or
/// starts where one does: two fixes at one place are one too many, unless
/// they're the same fix, offered by two diagnostics, which is taken once.
fn overlaps(taken: &[TextEdit], fix: &[TextEdit]) -> bool {
    taken.iter().any(|a| {
        fix.iter().any(|b| {
            a.span.start() == b.span.start()
                || (a.span.start() < b.span.end() && b.span.start() < a.span.end())
        })
    })
}

/// The text of a file the fixes may change: the content model, the lock,
/// the permalinks list, the navigation file, or a source file that was read.
fn writable_text(project: &Project, file: FileId) -> Option<&str> {
    if file == FileId::new(0)
        || file == LOCK_FILE_ID
        || file == PERMALINKS_FILE_ID
        || file == NAV_FILE_ID
    {
        return project.file(file).map(|entry| entry.text);
    }
    project
        .sources()
        .iter()
        .find(|s| s.id == file && s.unreadable.is_none())
        .map(|s| s.text.as_str())
}

/// Where a file the fixes may change is on disk.
fn path_on_disk(project: &Project, model: &Path, file: FileId) -> Option<PathBuf> {
    if file == FileId::new(0) {
        return Some(model.to_owned());
    }
    let entry = project.file(file)?;
    Some(project.root().join(entry.display_path))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ascribe_core::{Fix, Issue, Location, diagnostics};

    fn project(page: &str) -> Project {
        Project::from_parts(
            PathBuf::from("/p"),
            ascribe_core::RelPath::parse("docs").unwrap(),
            ascribe_model::load_str("spec = \"0.1\"\n", FileId::new(0)).unwrap(),
            "spec = \"0.1\"\n".to_owned(),
            Project::from_sources([(
                ascribe_core::RelPath::parse("a.md").unwrap(),
                page.to_owned(),
            )]),
        )
    }

    fn with_fix(edits: Vec<TextEdit>, applicability: Applicability) -> Diagnostic {
        let file = FileId::new(1);
        Diagnostic::from_issue(
            &Issue::new(diagnostics::LINK_ROUTE, Location::new(file, 0..1))
                .with_arg("page", "b.md")
                .with_arg("suggestion", "/b.md")
                .with_fix(Fix {
                    title: "Fix it".to_owned(),
                    file,
                    edits,
                    applicability,
                }),
        )
    }

    #[test]
    fn applies_safe_fixes_once_and_leaves_unsafe_and_overlapping_ones() {
        let project = project("abcdef");
        let safe = with_fix(vec![TextEdit::replace(0..2, "AB")], Applicability::Safe);
        let again = safe.clone();
        let overlapping = with_fix(vec![TextEdit::replace(1..3, "x")], Applicability::Safe);
        let unsafe_ = with_fix(vec![TextEdit::replace(4..6, "EF")], Applicability::Unsafe);
        let later = with_fix(vec![TextEdit::delete(3..4)], Applicability::Safe);
        let outside = with_fix(vec![TextEdit::insert(99, "!")], Applicability::Safe);
        let fixed = safe_fixes(
            &project,
            Path::new("/p/ascribe.toml"),
            [&safe, &again, &overlapping, &unsafe_, &later, &outside],
        );
        assert_eq!(fixed.len(), 1);
        assert_eq!(fixed[0].text, "ABcef");
        assert_eq!(fixed[0].applied.len(), 2);
        assert_eq!(fixed[0].path, Path::new("/p").join("docs/a.md"));
    }

    #[test]
    fn settled_keeps_the_first_of_two_fixes_that_overlap() {
        let mut settled = Settled::default();
        assert!(settled.take("abcdef", &[TextEdit::replace(0..2, "AB")]));
        assert!(!settled.take("abcdef", &[TextEdit::replace(1..3, "x")]));
        assert!(!settled.take("abcdef", &[TextEdit::replace(0..2, "AB")]));
        assert!(!settled.take("abcdef", &[TextEdit::insert(99, "!")]));
        assert!(settled.take("abcdef", &[TextEdit::delete(3..4)]));
        assert_eq!(
            settled.edits(),
            [TextEdit::replace(0..2, "AB"), TextEdit::delete(3..4)]
        );
    }

    #[test]
    fn nothing_safe_changes_nothing() {
        let project = project("abc");
        let unsafe_ = with_fix(vec![TextEdit::replace(0..1, "x")], Applicability::Unsafe);
        assert!(safe_fixes(&project, Path::new("/p/ascribe.toml"), [&unsafe_]).is_empty());
    }
}
