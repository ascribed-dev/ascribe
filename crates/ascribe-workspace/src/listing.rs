//! A listing of a project's files, and the changes between two: how a
//! [`Workspace`](crate::Workspace) kept without a file watcher learns what
//! changed on disk.
//!
//! A listing is each file's path, size, and modification time, with no file
//! read. It holds what loading reads: `ascribe.toml`, the side files, every
//! file under the content root, every file in `memos/` and
//! `.ascribe/memos/`, and whether each local source's folder is there. In
//! the content root, folders whose names start with `.`, `node_modules`, and
//! the output directory are left out, and a symbolic link to a folder isn't
//! followed. Nothing else in the project's folder is listed, so a build's
//! output beside `ascribe.toml` costs nothing. The code files snippets read
//! can be anywhere, and are read after loading, so they're stamped as
//! they're first seen read ([`Kept`](crate::Kept)).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ascribe_core::RelPath;
use ascribe_core::path::{normalize, relative_path};
use ascribe_model::{ContentModel, MODEL_FILE};
use ascribe_resolve::side_file::Entry;
use ascribe_resolve::{Change, Layout};

use crate::workspace::{Changed, WorkspaceSnapshot};

/// A file as a listing sees it: its path, size, and modification time.
pub type Stamp = (PathBuf, u64, Option<SystemTime>);

/// A project's files, in path order, as the module says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing(Vec<Stamp>);

impl Listing {
    /// The files of the project at `root` (absolute) with this model and
    /// these side files.
    pub fn of(root: &Path, layout: &Layout, model: &ContentModel, side_files: &[Entry]) -> Listing {
        let mut files = content_listing(root, layout, side_files);
        for folder in [
            ascribe_resolve::MEMOS_FOLDER,
            ascribe_resolve::PERSONAL_MEMOS_FOLDER,
        ] {
            walk(&root.join(folder), &|_| false, &mut files);
        }
        for source in &model.sources {
            files.push(presence(&normalize(&root.join(&source.path))));
        }
        Listing::sorted(files)
    }

    /// `ascribe.toml`, the side files (there or not), and every file under
    /// the content root: the files a page's check reads.
    pub fn content(root: &Path, layout: &Layout, side_files: &[Entry]) -> Listing {
        Listing::sorted(content_listing(root, layout, side_files))
    }

    fn sorted(mut files: Vec<Stamp>) -> Listing {
        files.sort();
        files.dedup();
        Listing(files)
    }

    /// Each file's stamp, in path order.
    pub fn stamps(&self) -> &[Stamp] {
        &self.0
    }

    /// This listing, then `more`.
    #[must_use]
    pub fn with(mut self, more: impl IntoIterator<Item = Stamp>) -> Listing {
        self.0.extend(more);
        Listing::sorted(self.0)
    }

    /// The files whose stamps differ between this listing and `now`: those
    /// in `now` that are new or changed, and those gone from it, each in
    /// path order.
    pub fn differences(&self, now: &Listing) -> Differences {
        let then: BTreeMap<&Path, &Stamp> = self.0.iter().map(|s| (s.0.as_path(), s)).collect();
        let present: BTreeMap<&Path, &Stamp> = now.0.iter().map(|s| (s.0.as_path(), s)).collect();
        Differences {
            written: now
                .0
                .iter()
                .filter(|s| then.get(s.0.as_path()) != Some(s))
                .map(|s| s.0.clone())
                .collect(),
            removed: self
                .0
                .iter()
                .filter(|s| !present.contains_key(s.0.as_path()))
                .map(|s| s.0.clone())
                .collect(),
            missing: self
                .0
                .iter()
                .chain(&now.0)
                .filter(|s| s.2.is_none())
                .map(|s| s.0.clone())
                .collect(),
        }
    }
}

/// What differs between two listings, by absolute path.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Differences {
    /// The files that appeared or changed.
    pub written: Vec<PathBuf>,
    /// The files that went.
    pub removed: Vec<PathBuf>,
    /// The paths either listing has without a time: a file not there, or a
    /// folder that couldn't be read.
    missing: Vec<PathBuf>,
}

impl Differences {
    /// Whether nothing differs.
    pub fn is_empty(&self) -> bool {
        self.written.is_empty() && self.removed.is_empty()
    }

    /// Whether `file`, from the project root, is among them.
    pub fn touches(&self, root: &Path, file: &str) -> bool {
        let file = root.join(file);
        self.written.contains(&file) || self.removed.contains(&file)
    }

    /// The changes these differences are to `snapshot`'s project, whose
    /// root is `root` (absolute), each file read once, through the
    /// snapshot's file system: a source that appeared or changed is read
    /// and edited (or can't be read), one gone is deleted, a side file is
    /// read again, and any other file is a file that isn't a source
    /// appearing or going. `ascribe.toml` isn't among them: it's loaded
    /// apart, by what loaded it, and given as [`Changed::Model`].
    ///
    /// `None` when they can't be told as changes, and the project is loaded
    /// again: a local source's folder appeared or went, or a folder under
    /// the content root did, or couldn't be read.
    pub fn changes(&self, root: &Path, snapshot: &WorkspaceSnapshot) -> Option<Vec<Changed>> {
        let layout = snapshot.layout();
        let content = normalize(&root.join(layout.content_root.as_str()));
        let folders: Vec<PathBuf> = snapshot
            .model()
            .sources
            .iter()
            .map(|s| normalize(&root.join(&s.path)))
            .collect();
        let fs = snapshot.file_system();
        let mut changes = Vec::new();
        let paths = self
            .written
            .iter()
            .map(|p| (p, true))
            .chain(self.removed.iter().map(|p| (p, false)));
        for (path, present) in paths {
            if folders.contains(path) || path.starts_with(&content) && self.missing.contains(path) {
                return None;
            }
            let Some(project_path) = relative_path(root, path) else {
                continue;
            };
            if project_path.as_str() == MODEL_FILE {
                continue;
            }
            if let Some(entry) = snapshot
                .side_file_list()
                .iter()
                .find(|e| e.path == project_path.as_str())
            {
                changes.push(Changed::SideFile(entry.path.to_owned()));
                continue;
            }
            let source = relative_path(&content, path)
                .filter(|p| p.is_inside() && snapshot.index().is_source(p));
            let change = match (source, present) {
                (Some(path), true) => match fs.read(&path) {
                    Ok(text) => Change::Edited { path, text },
                    Err(e) => Change::Unreadable {
                        path,
                        reason: e.to_string(),
                    },
                },
                (Some(path), false) => Change::Deleted { path },
                (None, true) => Change::AssetCreated { path: project_path },
                (None, false) => Change::AssetDeleted { path: project_path },
            };
            changes.push(Changed::Index(change));
        }
        Some(changes)
    }
}

/// The code files a kept project's snippets read, each stamped as it was
/// when it was first seen read: they can be outside the project's folder,
/// and they're read when a check asks for them, after loading, so no
/// listing has them.
#[derive(Clone, Debug, Default)]
pub struct CodeStamps(Vec<(RelPath, Stamp)>);

impl CodeStamps {
    /// The code files `snapshot`'s snippets have read, or name, whose
    /// stamps differ from when each was first seen, by path from the
    /// project root `root` (absolute), each with whether it's there now;
    /// those seen for the first time are stamped. Called right after
    /// loading, it stamps every file a snippet names as it's read.
    pub fn changed(&mut self, root: &Path, snapshot: &WorkspaceSnapshot) -> Vec<(RelPath, bool)> {
        let mut read: Vec<RelPath> = snapshot
            .code_files()
            .all()
            .iter()
            .map(|f| f.path.clone())
            .collect();
        let index = snapshot.index();
        for file in index.files() {
            read.extend(
                file.snippets
                    .iter()
                    .filter_map(|s| index.snippet_at(&file.path, s.span))
                    .map(|s| s.path.clone()),
            );
        }
        read.sort();
        read.dedup();
        let mut changed = Vec::new();
        for path in read {
            let now = stamp(&normalize(&root.join(path.as_str())));
            match self.0.iter_mut().find(|(p, _)| *p == path) {
                Some((_, then)) if *then != now => {
                    changed.push((path, now.2.is_some()));
                    *then = now;
                }
                Some(_) => {}
                None => self.0.push((path, now)),
            }
        }
        changed
    }

    /// Forgets every stamp, for a project loaded again.
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// A code file a snippet read, by path from the project root, as a change
/// to it is told: there now, or gone.
pub(crate) fn code_change(path: RelPath, present: bool) -> Changed {
    Changed::Index(if present {
        Change::AssetCreated { path }
    } else {
        Change::AssetDeleted { path }
    })
}

/// `ascribe.toml` and the side files (there or not), and every file under
/// the content root, unsorted.
fn content_listing(root: &Path, layout: &Layout, side_files: &[Entry]) -> Vec<Stamp> {
    let output = normalize(&root.join(layout.output_dir.as_str()));
    let content = normalize(&root.join(layout.content_root.as_str()));
    let skip = |dir: &Path| dir != content && dir == output;
    let mut files = vec![stamp(&root.join(MODEL_FILE))];
    files.extend(side_files.iter().map(|entry| stamp(&root.join(entry.path))));
    walk(&content, &skip, &mut files);
    files
}

fn walk(dir: &Path, skip: &dyn Fn(&Path) -> bool, files: &mut Vec<Stamp>) {
    // Outside FileSystem: the listing reads no file, only names, sizes, and
    // times, to see what changed since the project was loaded; what changed
    // is read through FileSystem.
    let Ok(entries) = std::fs::read_dir(dir) else {
        files.push((dir.to_owned(), 0, None));
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let hidden = name.to_string_lossy().starts_with('.') || name == "node_modules";
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            if !hidden && !skip(&path) {
                walk(&path, skip, files);
            }
        } else if !hidden {
            files.push(stamp(&path));
        }
    }
}

/// A file's stamp, following a symbolic link; a file that can't be read has
/// no size or time.
pub fn stamp(path: &Path) -> Stamp {
    // Outside FileSystem: as in `walk`.
    match std::fs::metadata(path) {
        Ok(meta) => (path.to_owned(), meta.len(), meta.modified().ok()),
        Err(_) => (path.to_owned(), 0, None),
    }
}

/// Whether a folder is there: its path, and 2 for a folder, 1 for anything
/// else, 0 for nothing. Its time isn't kept: what's in it is checked as
/// it's read.
fn presence(path: &Path) -> Stamp {
    // Outside FileSystem: as in `walk`.
    let kind = match std::fs::metadata(path) {
        Ok(meta) if meta.is_dir() => 2,
        Ok(_) => 1,
        Err(_) => 0,
    };
    (path.to_owned(), kind, None)
}
