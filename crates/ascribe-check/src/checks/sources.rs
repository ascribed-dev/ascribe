//! Sources in other repositories (SPEC §7.4): `ascribe.lock` against the
//! content model, and each source's copies against the lock. Only files are
//! read: no history, and no network.

use std::collections::BTreeSet;

use ascribe_core::{FileId, Issue, Location, RelPath, Span, diagnostics};
use ascribe_model::examples::{
    DEFAULT_OBSERVATION, EXAMPLES_FILE, Observation, Problem, is_examples_file, observation_path,
};
use ascribe_model::{Lock, LockedSource, file_hash};
use ascribe_resolve::{FolderExamples, Probe};

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
                        // The config and the observations are read by the
                        // examples' checks, not by a snippet.
                        if !read.contains(&copy) && !is_examples_file(path) {
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
/// observations: reported at the source in ascribe.toml, since a config
/// that can't be read means none of the source's examples can be checked.
/// Observations are read only beside a config that can be. For a source in
/// another repository, they're the copies in its folder. Then, for a source
/// in the same repository, each file an observation lists that isn't in the
/// folder is reported at its line in the observation.
pub(crate) fn check_examples(project: &Project) -> Vec<Diagnostic> {
    let model = project.model();
    let invalid = !model.checks.is_off(diagnostics::EXAMPLES_CONFIG_INVALID);
    let missing = !model.checks.is_off(diagnostics::OBSERVATION_FILE_MISSING);
    if !invalid && !missing {
        return Vec::new();
    }
    let (fs, code) = (project.file_system(), project.code_files());
    let mut out = Vec::new();
    for source in &model.sources {
        let Ok(folder) = RelPath::parse(&source.path) else {
            continue;
        };
        let examples = code.examples(fs, &folder);
        let mut unreadable: Vec<(&str, &[Problem])> = Vec::new();
        match &examples.config {
            Some(Err(problems)) => unreadable.push((EXAMPLES_FILE, problems)),
            Some(Ok(_)) => unreadable.extend(
                examples
                    .observations
                    .iter()
                    .filter_map(|o| Some((o.path.as_str(), o.read.as_ref().err()?.as_slice()))),
            ),
            None => continue,
        }
        if invalid {
            for (file, problems) in unreadable {
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
        }
        // A copy has only the files snippets use, so a listed file that
        // isn't there says nothing about the other repository.
        if missing && source.git.is_none() && matches!(examples.config, Some(Ok(_))) {
            out.extend(missing_files(project, &folder, &examples));
        }
    }
    out
}

/// Each file an observation lists that isn't in the folder, at its line in
/// the observation, which is read as a code file so a report can show it.
fn missing_files(
    project: &Project,
    folder: &RelPath,
    examples: &FolderExamples,
) -> Vec<Diagnostic> {
    let (fs, code) = (project.file_system(), project.code_files());
    let mut out = Vec::new();
    for (name, observation) in examples.readable() {
        let gone: Vec<&String> = observation
            .files
            .keys()
            .filter(|path| {
                folder
                    .join(path)
                    .map_or(true, |p| fs.probe(&p) != Probe::File)
            })
            .collect();
        if gone.is_empty() {
            continue;
        }
        let path = observation_path(name);
        let Ok(at) = folder.join(&path) else {
            continue;
        };
        let Ok(file) = code.read(fs, &at) else {
            continue;
        };
        let spans = Observation::file_spans(&file.text);
        let command = if name == DEFAULT_OBSERVATION {
            "ascribe observe".to_owned()
        } else {
            format!("ascribe observe --name {name}")
        };
        for listed in gone {
            let span = spans.get(listed).copied().unwrap_or(Span::empty(0));
            let issue = Issue::new(
                diagnostics::OBSERVATION_FILE_MISSING,
                Location::new(file.id, span),
            )
            .with_arg("observation", at.to_string())
            .with_arg("path", listed.clone())
            .with_arg("folder", folder.to_string())
            .with_arg("command", command.clone());
            out.push(Diagnostic::from_issue(&issue));
        }
    }
    out
}
