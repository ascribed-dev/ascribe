//! The git guard every command that writes shares: it refuses to change a
//! file git can't put back as it was, unless `--force`, and `--dry-run`
//! changes nothing (decision #965).

use std::path::{Component, Path, PathBuf};

use ascribe_core::{Coded, RelPath};
use ascribe_diff::{DiffError, Repository};
use clap::Args as ClapArgs;

use crate::answer;

/// `--dry-run` and `--force`, as every command that writes takes them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ClapArgs)]
pub struct Options {
    /// Say what it would change, and change nothing.
    #[arg(long)]
    pub dry_run: bool,

    /// Change files even when git couldn't put them back as they are: ones
    /// with changes not committed, ones git doesn't track, or ones in no
    /// repository.
    #[arg(long)]
    pub force: bool,
}

/// Why the guard refused.
#[derive(Debug, thiserror::Error)]
pub enum GuardError {
    /// git can't say whether the files can be put back.
    #[error(
        "this would change files that only git could put back, and {why}: {}; run it in a repository, or pass --force",
        .files.join(", ")
    )]
    NoGit {
        /// Why git can't.
        why: String,
        /// The files, from the current folder.
        files: Vec<String>,
    },
    /// Files the command would change aren't tracked, or have changes.
    #[error(
        "this would change files git can't put back as they are: {}; commit or stash them, or pass --force",
        .files.join(", ")
    )]
    NotRestorable {
        /// The files, from the current folder.
        files: Vec<String>,
    },
}

impl Coded for GuardError {
    fn code(&self) -> &'static str {
        match self {
            GuardError::NoGit { .. } => "guard_no_git",
            GuardError::NotRestorable { .. } => "guard_unrestorable",
        }
    }
}

impl Options {
    /// Whether the command may change `files`: always with `--force` or
    /// `--dry-run`, which changes nothing, and otherwise when git could put
    /// back each one that's there. A file that isn't there yet is only
    /// added, so it's never refused.
    ///
    /// # Errors
    ///
    /// [`GuardError`] when one of them can't be put back, or git can't say.
    pub fn allow<P: AsRef<Path>>(&self, files: &[P]) -> Result<(), GuardError> {
        if self.force || self.dry_run {
            return Ok(());
        }
        allow(files)
    }
}

/// Whether git could put back each of `files` that's there.
fn allow<P: AsRef<Path>>(files: &[P]) -> Result<(), GuardError> {
    // Outside FileSystem: whether a file the command writes is there, and
    // where, to ask git about it; nothing is read. Each is named as the
    // command gave it, and found by its canonical path.
    let there: Vec<(String, PathBuf)> = files
        .iter()
        .map(AsRef::as_ref)
        .filter(|f| f.exists())
        .filter_map(|f| Some((answer::path_from_here(f), std::fs::canonicalize(f).ok()?)))
        .collect();
    let Some((_, first)) = there.first() else {
        return Ok(());
    };
    let no_git = |why: String| GuardError::NoGit {
        why,
        files: there.iter().map(|(shown, _)| shown.clone()).collect(),
    };
    let dir = first.parent().unwrap_or(first);
    let found = Repository::discover(dir).map_err(|e| {
        no_git(match e {
            DiffError::GitNotFound => "git can't be run".to_owned(),
            DiffError::NotARepository { .. } => "they aren't in a git repository".to_owned(),
            other => other.to_string(),
        })
    })?;
    let root = std::fs::canonicalize(&found.root).map_err(|e| no_git(e.to_string()))?;
    let at_root = Repository::discover(&root).map_err(|e| no_git(e.to_string()))?;
    let mut paths: Vec<(String, RelPath)> = Vec::new();
    let mut outside = Vec::new();
    for (shown, file) in &there {
        match file.strip_prefix(&root).ok().and_then(rel_path) {
            Some(path) => paths.push((shown.clone(), path)),
            None => outside.push(shown.clone()),
        }
    }
    if !outside.is_empty() {
        return Err(GuardError::NoGit {
            why: "they aren't in the repository the others are in".to_owned(),
            files: outside,
        });
    }
    let asked: Vec<RelPath> = paths.iter().map(|(_, path)| path.clone()).collect();
    let unrestorable = at_root
        .not_restorable(&asked)
        .map_err(|e| no_git(e.to_string()))?;
    if unrestorable.is_empty() {
        return Ok(());
    }
    Err(GuardError::NotRestorable {
        files: paths
            .into_iter()
            .filter(|(_, path)| unrestorable.contains(path))
            .map(|(shown, _)| shown)
            .collect(),
    })
}

/// `path`, below the repository's root, as git names it: `/`-separated.
fn rel_path(path: &Path) -> Option<RelPath> {
    let parts: Option<Vec<&str>> = path
        .components()
        .map(|c| match c {
            Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect();
    RelPath::parse(&parts?.join("/")).ok()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::fs;
    use std::process::Command;

    use super::*;

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(dir)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap()
            .status;
        assert!(status.success(), "git {args:?}");
    }

    fn guard() -> Options {
        Options::default()
    }

    #[test]
    fn a_file_committed_and_unchanged_is_allowed() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q"]);
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::write(dir.path().join("docs/a.md"), "a\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-qm", "a"]);
        guard().allow(&[dir.path().join("docs/a.md")]).unwrap();
    }

    #[test]
    fn a_changed_or_untracked_file_is_refused_and_named() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q"]);
        fs::write(dir.path().join("a.md"), "a\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-qm", "a"]);
        fs::write(dir.path().join("a.md"), "changed\n").unwrap();
        fs::write(dir.path().join("b.md"), "b\n").unwrap();
        let files = [dir.path().join("a.md"), dir.path().join("b.md")];
        let e = guard().allow(&files).unwrap_err();
        assert_eq!(e.code(), "guard_unrestorable");
        let message = e.to_string();
        assert!(
            message.contains("a.md") && message.contains("b.md"),
            "{message}"
        );
        let force = Options {
            force: true,
            ..guard()
        };
        force.allow(&files).unwrap();
        let dry_run = Options {
            dry_run: true,
            ..guard()
        };
        dry_run.allow(&files).unwrap();
    }

    #[test]
    fn a_file_not_there_yet_is_allowed_anywhere() {
        let dir = tempfile::tempdir().unwrap();
        guard().allow(&[dir.path().join("new.md")]).unwrap();
    }

    #[test]
    fn a_file_in_no_repository_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.md"), "a\n").unwrap();
        let e = guard().allow(&[dir.path().join("a.md")]).unwrap_err();
        assert_eq!(e.code(), "guard_no_git", "{e}");
    }
}
