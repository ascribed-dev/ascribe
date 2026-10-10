//! The projects the MCP server keeps loaded between calls.
//!
//! A project is loaded once and kept, with its source index once a tool asks
//! for it. Before each call the server lists the project's files again
//! (paths, sizes, and modification times) and compares the listing with the
//! one it loaded from; any difference, a new or a deleted file included,
//! loads the project again. The project's files are what loading reads:
//! `ascribe.toml`, `ascribe.lock`, and the content root, and whether each
//! local source's folder is there. The code files snippets read are checked
//! as they're read. Nothing else in the project's folder is listed, so a
//! build's output beside `ascribe.toml` costs nothing. Listing is cheap
//! beside loading: no file is read.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use ascribe_check::{LoadError, MODEL_FILE, Project};
use ascribe_core::path::normalize;
use ascribe_model::LOCK_FILE;

use crate::answer::{Loaded, Projects};

/// A file as the listing sees it: its path, size, and modification time.
pub(crate) type Stamp = (PathBuf, u64, Option<SystemTime>);

/// A project kept loaded.
struct Entry {
    /// The project's files when it was loaded.
    listing: Vec<Stamp>,
    /// The code files snippets have read, each as it was when it was first
    /// seen read: they can be outside the project's folder, and they're read
    /// when a check asks for them, after loading.
    code: RefCell<Vec<Stamp>>,
    loaded: Rc<Loaded>,
}

/// The projects loaded so far, by the path of their `ascribe.toml`.
#[derive(Default)]
pub struct Cache {
    entries: RefCell<HashMap<PathBuf, Entry>>,
}

impl Projects for Cache {
    fn load(&self, config: &Path) -> Result<Rc<Loaded>, LoadError> {
        let key = absolute(config);
        let root = key.parent().map(Path::to_owned).unwrap_or_default();
        let mut entries = self.entries.borrow_mut();
        if let Some(entry) = entries.get(&key) {
            let listing = list(&root, &entry.loaded.project);
            if listing == entry.listing && entry.code_unchanged() {
                return Ok(Rc::clone(&entry.loaded));
            }
        }
        entries.remove(&key);
        let project = Project::load(config)?.with_today(crate::clock::today());
        let listing = list(&root, &project);
        let loaded = Rc::new(Loaded::new(project));
        entries.insert(
            key,
            Entry {
                listing,
                code: RefCell::new(Vec::new()),
                loaded: Rc::clone(&loaded),
            },
        );
        Ok(loaded)
    }
}

impl Entry {
    /// Whether every code file read so far is as it was when it was first
    /// seen read.
    fn code_unchanged(&self) -> bool {
        let project = &self.loaded.project;
        let mut seen = self.code.borrow_mut();
        for file in project.code_files().all() {
            let path = normalize(&project.root().join(file.path.as_str()));
            let now = stamp(&path);
            match seen.iter().find(|s| s.0 == path) {
                Some(then) if *then != now => return false,
                Some(_) => {}
                None => seen.push(now),
            }
        }
        true
    }
}

/// The project's files, in path order: `ascribe.toml`, `ascribe.lock`
/// (there or not), every file under the content root, and each local
/// source's folder (there or not). In the content root, folders whose names
/// start with `.`, `node_modules`, and the output directory are left out,
/// and a symbolic link to a folder isn't followed.
fn list(root: &Path, project: &Project) -> Vec<Stamp> {
    let mut files = content_listing(root, project.content_root(), &project.layout().output_dir);
    for source in &project.model().sources {
        files.push(presence(&normalize(&root.join(&source.path))));
    }
    files.sort();
    files.dedup();
    files
}

/// `ascribe.toml`, `ascribe.lock` (there or not), and every file under the
/// content root, as [`list`] lists them, unsorted.
pub(crate) fn content_listing(
    root: &Path,
    content_root: &ascribe_core::RelPath,
    output_dir: &ascribe_core::RelPath,
) -> Vec<Stamp> {
    let output = normalize(&root.join(output_dir.as_str()));
    let content = normalize(&root.join(content_root.as_str()));
    let skip = |dir: &Path| dir != content && dir == output;
    let mut files = vec![stamp(&root.join(MODEL_FILE)), stamp(&root.join(LOCK_FILE))];
    walk(&content, &skip, &mut files);
    files
}

fn walk(dir: &Path, skip: &dyn Fn(&Path) -> bool, files: &mut Vec<Stamp>) {
    // Outside FileSystem: the listing reads no file, only names, sizes, and
    // times, to see whether the project changed since it was loaded; loading
    // it again reads through FileSystem.
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
fn stamp(path: &Path) -> Stamp {
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

/// `path` made absolute from the current directory, and normalized.
fn absolute(path: &Path) -> PathBuf {
    normalize(&std::path::absolute(path).unwrap_or_else(|_| path.to_owned()))
}
