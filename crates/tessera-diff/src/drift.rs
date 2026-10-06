//! Which pages' examples changed between a base revision and the working
//! tree: the snippets whose code differs, and, for each page that shows one,
//! whether the page itself changed too; and the snippets that resolved at the
//! base and don't now.
//!
//! A page covers the code it shows and nothing else: every region it takes
//! a snippet from, and every whole file it takes as one, its fragments'
//! snippets included. The comparison reads as little as it can: one `git
//! diff` listing of the change, and, at the base, only the files that
//! listing names and a snippet uses. The base's pages are read only when
//! some example changed, to tell whether the words around it did.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::Serialize;
use similar::{Algorithm, DiffTag, capture_diff_slices};
use tessera_core::{FileId, RelPath, Span};
use tessera_resolve::snippet::tags::{self, comment_style};
use tessera_resolve::snippet::{CodeFiles, SnippetError, resolve_snippet};
use tessera_resolve::{Address, AstroRouter, FileKind, FileSystem, Project, Snippet};

use crate::compare::changed_apart_from_snippets;
use crate::git::{Base, FileChange, Repository};
use crate::gitfs::Revision;
use crate::{BaseInfo, DiffError, RepositoryInfo, Side};

/// The version of the drift report's JSON. It changes only when a field is
/// removed or changes meaning; fields can be added without a new version.
pub const DRIFT_SCHEMA_VERSION: u32 = 1;

/// The report, as `ascribe drift --format json` writes it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct DriftReport {
    /// [`DRIFT_SCHEMA_VERSION`].
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// What was compared with.
    pub base: BaseInfo,
    /// Where the project is.
    pub repository: RepositoryInfo,
    /// Every page with an example that changed or broke, in path order.
    pub pages: Vec<DriftPage>,
}

/// A page with an example that changed or broke.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct DriftPage {
    /// The page's content path.
    pub path: String,
    /// Its route, in the first of the builds it's in.
    pub route: String,
    /// The builds that show it with a changed example, in the order asked
    /// for.
    pub builds: Vec<String>,
    /// Whether the page changed apart from its examples: its own file, or
    /// its resolved content through anything else it uses (a fragment, the
    /// content model), in any of those builds. When it didn't, the example
    /// changed and the words around it didn't.
    pub page_changed: bool,
    /// The examples that changed, by address. Empty when only `broken`
    /// isn't.
    pub examples: Vec<ChangedExample>,
    /// The examples that resolved at the base and don't now, by address.
    pub broken: Vec<BrokenExample>,
}

/// A snippet that resolved at the base and doesn't in the working tree: its
/// region, file, or source is gone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct BrokenExample {
    /// Its address, as the page writes it.
    pub address: String,
    /// The source the address names.
    pub source: String,
    /// The slug of the diagnostic `ascribe check` reports for it
    /// (`snippet-region-missing`).
    pub problem: &'static str,
    /// Why it doesn't resolve, in a few words.
    pub reason: String,
}

/// A snippet whose code changed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct ChangedExample {
    /// Its address, as the page writes it.
    pub address: String,
    /// The source the address names.
    pub source: String,
    /// The code file now, from the repository's root.
    pub file: String,
    /// The code file at the base, when it was renamed since.
    pub was_file: Option<String>,
    /// Lines only in the code now.
    pub added: usize,
    /// Lines only in the code at the base.
    pub removed: usize,
}

impl DriftReport {
    /// The pages with an example that no longer resolves.
    pub fn broken_pages(&self) -> impl Iterator<Item = &DriftPage> {
        self.pages.iter().filter(|p| !p.broken.is_empty())
    }

    /// The pages whose examples changed while the page didn't.
    pub fn unchanged_pages(&self) -> impl Iterator<Item = &DriftPage> {
        self.pages
            .iter()
            .filter(|p| !p.page_changed && !p.examples.is_empty())
    }

    /// The pages whose examples changed along with the page.
    pub fn changed_pages(&self) -> impl Iterator<Item = &DriftPage> {
        self.pages
            .iter()
            .filter(|p| p.page_changed && !p.examples.is_empty())
    }

    /// Whether a page needs reading: an example of it broke, or changed
    /// while the page didn't.
    pub fn needs_reading(&self) -> bool {
        self.broken_pages().next().is_some() || self.unchanged_pages().next().is_some()
    }
}

/// A `@snippet` of the working tree that doesn't resolve.
struct Unresolved {
    /// The source file it's written in, and where.
    path: RelPath,
    file: FileId,
    span: Span,
    written: String,
    broken: BrokenExample,
}

/// One snippet of the working tree, by its written address.
struct Used {
    snippet: Arc<Snippet>,
    address: Address,
    /// The source files that have it: pages and fragments.
    files: BTreeSet<RelPath>,
}

/// The pages of `builds` (by name) whose examples changed, or stopped
/// resolving, between `base` and `now`, in `repo`. `now` is the working
/// tree, and `fs` its files, for saying why an example doesn't resolve.
///
/// # Errors
///
/// [`DiffError::Git`] when git fails, and the errors of [`Revision::read`]
/// when the base's pages have to be read and can't be.
pub fn drift(
    repo: &Repository,
    base: &Base,
    now: Side<'_>,
    fs: &dyn FileSystem,
    builds: &[&str],
) -> Result<DriftReport, DiffError> {
    drift_with(repo, base, now, fs, builds, None, || {
        read_base(repo, base.compared())
    })
}

/// The project at a revision, read and indexed; `None` when it has no
/// `ascribe.toml` there.
pub(crate) type BaseProject = Option<(Revision, Project)>;

/// The project at `commit`, read and indexed.
pub(crate) fn read_base(repo: &Repository, commit: &str) -> Result<BaseProject, DiffError> {
    Ok(Revision::read(repo, commit)?.map(|revision| {
        let project = revision.project();
        (revision, project)
    }))
}

/// [`drift`], with the `git diff` listing of the change when it's already
/// known, and the base's project from `read_base` when it's needed.
pub(crate) fn drift_with(
    repo: &Repository,
    base: &Base,
    now: Side<'_>,
    fs: &dyn FileSystem,
    builds: &[&str],
    changes: Option<BTreeMap<RelPath, FileChange>>,
    read_base: impl FnOnce() -> Result<BaseProject, DiffError>,
) -> Result<DriftReport, DiffError> {
    let mut report = DriftReport {
        schema_version: DRIFT_SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        base: BaseInfo::of(base),
        repository: RepositoryInfo::of(repo),
        pages: Vec::new(),
    };
    let used = snippets(now);
    let unresolved = unresolved(now, fs);
    if used.is_empty() && unresolved.is_empty() {
        return Ok(report);
    }
    let examples = if used.is_empty() {
        BTreeMap::new()
    } else {
        let changes = match changes {
            Some(changes) => changes,
            None => repo.changed_files(base.compared())?,
        };
        changed_examples(repo, base.compared(), &changes, &used)?
    };
    if examples.is_empty() && unresolved.is_empty() {
        return Ok(report);
    }

    // The base: whether an example that doesn't resolve now did then, and
    // whether each page changed apart from its examples.
    let before = read_base()?;
    let before_project = before.as_ref().map(|(_, project)| project);
    let before_side = before.as_ref().map(|(revision, project)| Side {
        project,
        model_text: &revision.model_text,
    });
    // An example that never resolved is the page's own new mistake, which
    // `ascribe check` reports; one that did has broken.
    let broken: Vec<Unresolved> = unresolved
        .into_iter()
        .filter(|u| {
            before_project.is_some_and(|project| {
                project.file(&u.path).is_some_and(|file| {
                    file.snippets.iter().any(|s| {
                        s.written == u.written && project.snippet_at(&u.path, s.span).is_some()
                    })
                })
            })
        })
        .collect();
    if examples.is_empty() && broken.is_empty() {
        return Ok(report);
    }

    // The pages that show a changed or broken example: its files, and the
    // pages that include them. Which build shows which is for the resolver
    // to say.
    let mut holders: BTreeSet<&RelPath> = BTreeSet::new();
    for address in examples.keys() {
        holders.extend(&used[address].files);
    }
    holders.extend(broken.iter().map(|u| &u.path));
    let mut candidates: BTreeSet<RelPath> = BTreeSet::new();
    for file in holders {
        if now
            .project
            .file(file)
            .is_some_and(|f| f.kind == FileKind::Page)
        {
            candidates.insert(file.clone());
        }
        candidates.extend(now.project.including_pages(file));
    }
    let router = AstroRouter::from_consumer(&now.project.model().consumer);
    let mut found: BTreeMap<RelPath, DriftPage> = BTreeMap::new();
    for name in builds {
        let Some(build) = now.project.model().build(name) else {
            continue;
        };
        let resolver = now.project.resolver(build, &router);
        for path in &candidates {
            let Some(page) = resolver.page(path) else {
                continue;
            };
            let mut shown: BTreeSet<&str> = BTreeSet::new();
            let mut gone: BTreeSet<usize> = BTreeSet::new();
            page.visit(&mut |block| {
                if let Some(snippet) = &block.snippet
                    && examples.contains_key(&snippet.address)
                {
                    shown.insert(snippet.address.as_str());
                }
                if let Some(i) = broken
                    .iter()
                    .position(|u| u.file == block.file && u.span == block.span)
                {
                    gone.insert(i);
                }
            });
            if shown.is_empty() && gone.is_empty() {
                continue;
            }
            let entry = found.entry(path.clone()).or_insert_with(|| DriftPage {
                path: path.to_string(),
                route: page.route.clone(),
                builds: Vec::new(),
                page_changed: false,
                examples: Vec::new(),
                broken: Vec::new(),
            });
            entry.builds.push((*name).to_owned());
            for address in shown {
                if !entry.examples.iter().any(|e| e.address == address) {
                    entry.examples.push(examples[address].clone());
                }
            }
            for i in gone {
                let example = &broken[i].broken;
                if !entry.broken.contains(example) {
                    entry.broken.push(example.clone());
                }
            }
        }
    }
    if found.is_empty() {
        return Ok(report);
    }

    for name in builds {
        let paths: Vec<&RelPath> = found
            .iter()
            .filter(|(_, page)| page.builds.iter().any(|b| b == name))
            .map(|(path, _)| path)
            .collect();
        if paths.is_empty() {
            continue;
        }
        let changed = changed_apart_from_snippets(before_side, now, name, &paths);
        for path in changed {
            if let Some(page) = found.get_mut(&path) {
                page.page_changed = true;
            }
        }
    }
    report.pages = found
        .into_values()
        .map(|mut page| {
            page.examples.sort_by(|a, b| a.address.cmp(&b.address));
            page.broken.sort_by(|a, b| a.address.cmp(&b.address));
            page
        })
        .collect();
    Ok(report)
}

/// Every `@snippet` of the working tree with a well-formed address that
/// doesn't resolve, with why.
fn unresolved(now: Side<'_>, fs: &dyn FileSystem) -> Vec<Unresolved> {
    let code = CodeFiles::new();
    let mut out = Vec::new();
    for file in now.project.files() {
        for snippet_use in &file.snippets {
            let Some(Ok(address)) = &snippet_use.address else {
                continue;
            };
            if now
                .project
                .snippet_at(&file.path, snippet_use.span)
                .is_some()
            {
                continue;
            }
            let Err(error) = resolve_snippet(snippet_use, address, now.project.model(), fs, &code)
            else {
                continue;
            };
            let (problem, reason) = why(&error, address);
            out.push(Unresolved {
                path: file.path.clone(),
                file: file.file,
                span: snippet_use.span,
                written: snippet_use.written.clone(),
                broken: BrokenExample {
                    address: snippet_use.written.clone(),
                    source: address.source.clone(),
                    problem,
                    reason,
                },
            });
        }
    }
    out
}

/// The diagnostic's slug for a snippet that doesn't resolve, and the reason
/// in a few words.
fn why(error: &SnippetError, address: &Address) -> (&'static str, String) {
    match error {
        SnippetError::UnknownSource => (
            "snippet-source-unknown",
            format!("ascribe.toml has no source `{}`", address.source),
        ),
        SnippetError::Missing { .. } => ("snippet-file-missing", "the file isn't there".to_owned()),
        SnippetError::NotIncluded => (
            "snippet-file-missing",
            "the source doesn't include the file".to_owned(),
        ),
        SnippetError::Link => (
            "snippet-file-missing",
            "the file is a link out of the source".to_owned(),
        ),
        SnippetError::NotText(reason) => (
            "snippet-file-not-text",
            format!("the file isn't text: {reason}"),
        ),
        SnippetError::Tags(_) => ("snippet-tags", "the file's tags don't balance".to_owned()),
        SnippetError::RegionMissing(_) => (
            "snippet-region-missing",
            format!(
                "the file has no region `{}`",
                address.region.as_deref().unwrap_or_default()
            ),
        ),
    }
}

/// Every snippet the working tree's files take, by written address.
fn snippets(now: Side<'_>) -> BTreeMap<String, Used> {
    let mut used: BTreeMap<String, Used> = BTreeMap::new();
    for file in now.project.files() {
        for snippet_use in &file.snippets {
            let Some(Ok(address)) = &snippet_use.address else {
                continue;
            };
            let Some(snippet) = now.project.snippet_at(&file.path, snippet_use.span) else {
                continue;
            };
            used.entry(snippet.address.clone())
                .or_insert_with(|| Used {
                    snippet: snippet.clone(),
                    address: address.clone(),
                    files: BTreeSet::new(),
                })
                .files
                .insert(file.path.clone());
        }
    }
    used
}

/// The snippets whose code differs between `commit` and the working tree,
/// by written address. Only the files `git diff` lists (`changes`) are read
/// at the commit, each once.
fn changed_examples(
    repo: &Repository,
    commit: &str,
    changes: &BTreeMap<RelPath, FileChange>,
    used: &BTreeMap<String, Used>,
) -> Result<BTreeMap<String, ChangedExample>, DiffError> {
    let project_dir = repo.project_dir();
    // Each snippet's file now, and where it was at the commit, for those
    // whose file changed and isn't new.
    let mut moved: Vec<(&String, RelPath, RelPath)> = Vec::new();
    for (address, u) in used {
        let Ok(now) = project_dir.join(u.snippet.path.as_str()) else {
            continue;
        };
        let was = match changes.get(&now) {
            None | Some(FileChange::Added) => continue,
            Some(FileChange::Modified) => now.clone(),
            Some(FileChange::Renamed(was)) => was.clone(),
        };
        moved.push((address, now, was));
    }
    if moved.is_empty() {
        return Ok(BTreeMap::new());
    }
    let wanted: BTreeSet<RelPath> = moved.iter().map(|(_, _, was)| was.clone()).collect();
    let names: Vec<String> = wanted.iter().map(|p| format!("{commit}:{p}")).collect();
    let objects: Vec<&str> = names.iter().map(String::as_str).collect();
    let texts: BTreeMap<RelPath, Option<String>> = wanted
        .into_iter()
        .zip(repo.read_blobs(&objects)?)
        .map(|(path, bytes)| (path, String::from_utf8(bytes).ok()))
        .collect();

    let mut out = BTreeMap::new();
    for (address, now, was) in moved {
        let u = &used[address];
        let Some(Some(text)) = texts.get(&was) else {
            continue;
        };
        // A region the file didn't have is a new example, not a changed one.
        let Some(code) = extract(&was, text, u.address.region.as_deref()) else {
            continue;
        };
        let Some((added, removed)) = line_changes(&code, &u.snippet.code) else {
            continue;
        };
        out.insert(
            address.clone(),
            ChangedExample {
                address: address.clone(),
                source: u.address.source.clone(),
                file: now.to_string(),
                was_file: (was != now).then(|| was.to_string()),
                added,
                removed,
            },
        );
    }
    Ok(out)
}

/// The code a snippet takes from `text`, the file at `path`: the region's,
/// or the whole file's, as the snippet itself is taken. `None` when the file
/// has no such region.
fn extract(path: &RelPath, text: &str, region: Option<&str>) -> Option<String> {
    let found = path
        .extension()
        .and_then(comment_style)
        .map(|style| tags::scan(text, style))
        .unwrap_or_default();
    let region = match region {
        Some(name) => Some(found.region(name)?),
        None => None,
    };
    Some(tags::extract(text, &found, region).code)
}

/// How many lines were added and removed between two versions of a
/// snippet's code, already dedented, compared with trailing whitespace
/// trimmed; `None` when they're the same.
fn line_changes(was: &str, now: &str) -> Option<(usize, usize)> {
    let lines =
        |code: &str| -> Vec<String> { code.lines().map(|l| l.trim_end().to_owned()).collect() };
    let (was, now) = (lines(was), lines(now));
    if was == now {
        return None;
    }
    let mut added = 0;
    let mut removed = 0;
    for op in capture_diff_slices(Algorithm::Myers, &was, &now) {
        let (tag, old, new) = op.as_tag_tuple();
        match tag {
            DiffTag::Equal => {}
            DiffTag::Delete => removed += old.len(),
            DiffTag::Insert => added += new.len(),
            DiffTag::Replace => {
                removed += old.len();
                added += new.len();
            }
        }
    }
    Some((added, removed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_changes_ignore_trailing_whitespace() {
        assert_eq!(line_changes("a\nb\n", "a  \nb\r\n"), None);
        assert_eq!(line_changes("a\nb\n", "a\nc\nd\n"), Some((2, 1)));
        assert_eq!(line_changes("a\n", ""), Some((0, 1)));
    }

    #[test]
    fn extracting_a_region_and_a_whole_file() {
        let path = RelPath::parse("app.py").unwrap_or_default();
        let text = "import os\n    # :snippet-start: main\n    run()\n    # :snippet-end:\n";
        assert_eq!(
            extract(&path, text, Some("main")).as_deref(),
            Some("run()\n")
        );
        assert_eq!(extract(&path, text, Some("other")), None);
        assert_eq!(
            extract(&path, text, None).as_deref(),
            Some("import os\n    run()\n")
        );
    }
}
