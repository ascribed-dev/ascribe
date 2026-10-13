//! Sources in other repositories (SPEC §7.4): `ascribe.lock` against the
//! content model, and each source's copies against the lock. Only files are
//! read: no history, and no network.

use std::collections::BTreeSet;

use ascribe_core::{FileId, Issue, Location, RelPath, Span, diagnostics};
use ascribe_model::examples::{DEFAULT_OBSERVATION, EXAMPLES_FILE, observation_path};
use ascribe_model::{Lock, LockedSource, file_hash};

use crate::project::LOCK_FILE_ID;
use crate::{Diagnostic, Project};

/// The diagnostics of the lock and the copies. Run after every source file's
/// file-level checks, which read the code files snippets use: a copy that
/// none of them read is unused.
pub(crate) fn check_sources(project: &Project) -> Vec<Diagnostic> {
    let model = project.model();
    let default = Lock::default();
    let lock = match project.lock() {
        None => &default,
        Some(Ok(lock)) => lock,
        // A lock that can't be read says nothing about the copies.
        Some(Err(issues)) => return issues.iter().map(Diagnostic::from_issue).collect(),
    };
    let mut issues: Vec<Issue> = Vec::new();
    let at_lock = |span: Span| Location::new(LOCK_FILE_ID, span);

    // Each pin names a source in another repository, in the same one.
    let mut pinned: Vec<&LockedSource> = Vec::new();
    for locked in &lock.sources {
        let issue = Issue::new(diagnostics::LOCK_SOURCE_UNKNOWN, at_lock(locked.span))
            .with_arg("source", locked.name.clone());
        match model.source(&locked.name) {
            None => issues.push(issue),
            Some(source) => match &source.git {
                None => issues.push(issue.with_variant("path")),
                Some(remote) if remote.url != locked.git => issues.push(
                    issue
                        .with_variant("moved")
                        .with_arg("locked", locked.git.clone())
                        .with_arg("git", remote.url.clone()),
                ),
                Some(_) => pinned.push(locked),
            },
        }
    }

    let fs = project.file_system();
    let read: BTreeSet<RelPath> = project
        .code_files()
        .all()
        .iter()
        .map(|f| f.path.clone())
        .collect();
    for source in &model.sources {
        let Some(remote) = &source.git else {
            continue;
        };
        let Ok(folder) = RelPath::parse(&source.path) else {
            continue;
        };
        let locked = lock.source(&source.name);
        // A pin to another repository is reported above; its copies are
        // whatever that repository had, so they aren't checked against it.
        if locked.is_some_and(|l| l.git != remote.url) {
            continue;
        }
        let locked = locked.filter(|l| pinned.contains(l));
        let copies: Vec<RelPath> = fs.files_in(&folder);
        let relative = |path: &RelPath| -> String {
            path.segments()
                .skip(folder.segments().count())
                .collect::<Vec<_>>()
                .join("/")
        };
        let on_disk: BTreeSet<String> = copies.iter().map(relative).collect();
        let shown = |path: &str| format!("{}/{path}", source.path);

        if let Some(locked) = locked {
            for (path, file) in &locked.files {
                let issue = Issue::new(diagnostics::SOURCE_COPY_CHANGED, at_lock(file.span))
                    .with_arg("path", shown(path))
                    .with_arg("source", source.name.clone());
                if !on_disk.contains(path) {
                    issues.push(issue.with_variant("missing"));
                    continue;
                }
                let Ok(copy) = folder.join(path) else {
                    continue;
                };
                match fs.read_file(&copy) {
                    Ok(bytes) if file_hash(&bytes) == file.hash => {
                        if !read.contains(&copy) {
                            issues.push(
                                Issue::new(diagnostics::SOURCE_COPY_UNUSED, at_lock(file.span))
                                    .with_arg("path", shown(path)),
                            );
                        }
                    }
                    _ => issues.push(issue),
                }
            }
        }
        // A copy the lock doesn't list is reported at the source's pin, or at
        // its declaration when it has none.
        let at = match locked {
            Some(locked) => at_lock(locked.span),
            None => Location::new(FileId::new(0), source.span),
        };
        for path in &on_disk {
            if locked.is_some_and(|l| l.files.contains_key(path)) {
                continue;
            }
            issues.push(
                Issue::new(diagnostics::SOURCE_COPY_UNLOCKED, at)
                    .with_arg("path", shown(path))
                    .with_arg("source", source.name.clone()),
            );
        }
    }
    issues.iter().map(Diagnostic::from_issue).collect()
}

/// The problems of each source folder's `ascribe.examples.toml` and its
/// observation, for a source in the same repository: reported at the source
/// in ascribe.toml, since a config that can't be read means none of the
/// source's examples can be checked. An observation is read only beside a
/// config.
pub(crate) fn check_examples(project: &Project) -> Vec<Diagnostic> {
    let model = project.model();
    if model.checks.is_off(diagnostics::EXAMPLES_CONFIG_INVALID) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for source in model.sources.iter().filter(|s| s.git.is_none()) {
        let Ok(folder) = RelPath::parse(&source.path) else {
            continue;
        };
        let examples = project
            .code_files()
            .examples(project.file_system(), &folder);
        let observation = observation_path(DEFAULT_OBSERVATION);
        let (file, problems) = match (&examples.config, &examples.observation) {
            (Some(Err(problems)), _) => (EXAMPLES_FILE, problems),
            (Some(Ok(_)), Some(Err(problems))) => (observation.as_str(), problems),
            _ => continue,
        };
        let shown = folder
            .join(file)
            .map_or_else(|_| file.to_owned(), |p| p.to_string());
        for problem in problems {
            let issue = Issue::new(
                diagnostics::EXAMPLES_CONFIG_INVALID,
                Location::new(FileId::new(0), source.span),
            )
            .with_arg("source", source.name.clone())
            .with_arg("file", shown.clone())
            .with_arg("detail", problem.detail.clone());
            out.push(Diagnostic::from_issue(&issue));
        }
    }
    out
}
