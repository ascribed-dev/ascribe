//! The projects the MCP server keeps loaded between calls.
//!
//! A project is loaded once and kept, as an `ascribe_workspace::Kept`.
//! Before each call the server lists the project's files again (paths,
//! sizes, and modification times; `ascribe_workspace::Listing` says which)
//! and applies what differs to the project's `Workspace`, reading only the
//! files that do; a difference it can't take in place, as a project nested
//! in the content root appearing, loads the project again. The code files
//! snippets read are checked as they're read. What a call kept with the
//! project, its checks and readings (`answer::Loaded`), belongs to the
//! snapshot it was made from, and a new snapshot starts without them.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ascribe_check::{LOADER, Project};
use ascribe_core::path::normalize;
use ascribe_model::SourceBound;
use ascribe_workspace::{Kept, Listing, LoadError};

use crate::answer::{Loaded, Projects};

/// A file as the listing sees it: its path, size, and modification time.
pub(crate) use ascribe_workspace::Stamp;

/// A project kept loaded, and what the last call kept with its snapshot.
struct Entry {
    kept: Kept,
    loaded: Rc<Loaded>,
}

/// The projects loaded so far, by the path of their `ascribe.toml`.
#[derive(Default)]
pub struct Cache {
    entries: RefCell<HashMap<PathBuf, Entry>>,
    /// The folder the MCP server was started in, inside which a local
    /// source must be when no repository is around its project; `None`
    /// reads a source wherever the model says, as a command does.
    opened: Option<Vec<PathBuf>>,
}

impl Cache {
    /// A cache whose projects read a local source, when there's no
    /// repository around them, only inside `dir` (decision 77).
    pub fn opened_at(dir: &Path) -> Cache {
        Cache {
            entries: RefCell::default(),
            opened: Some(vec![dir.to_owned()]),
        }
    }

    /// Where a source may be, for [`ascribe_check::load_bounded`].
    pub fn bound(&self) -> SourceBound<'_> {
        match &self.opened {
            Some(opened) => SourceBound::Within(opened),
            None => SourceBound::Anywhere,
        }
    }
}

impl Projects for Cache {
    fn load(&self, config: &Path) -> Result<Rc<Loaded>, LoadError> {
        let key = absolute(config);
        let mut entries = self.entries.borrow_mut();
        if let Some(entry) = entries.get_mut(&key) {
            if let Err(e) = entry.kept.refresh(self.bound()) {
                entries.remove(&key);
                return Err(e);
            }
            let snapshot = entry.kept.snapshot();
            if !entry.loaded.is_of(&snapshot) {
                entry.loaded = Rc::new(Loaded::new(snapshot));
            }
            return Ok(Rc::clone(&entry.loaded));
        }
        let kept = Kept::load(LOADER, &key, self.bound())?;
        let loaded = Rc::new(Loaded::new(kept.snapshot()));
        entries.insert(
            key,
            Entry {
                kept,
                loaded: Rc::clone(&loaded),
            },
        );
        Ok(loaded)
    }
}

/// The project's files, as the MCP server lists them, then each code file
/// its snippets have read: what `ascribe preview` watches, rendering the
/// build again when any of it changes.
pub(crate) fn watched(project: &Project) -> Vec<Stamp> {
    let root = project.root();
    let code = project
        .code_files()
        .all()
        .into_iter()
        .map(|file| ascribe_workspace::stamp(&normalize(&root.join(file.path.as_str()))));
    Listing::of(root, project.layout(), project.model(), LOADER.side_files)
        .with(code)
        .stamps()
        .to_vec()
}

/// `path` made absolute from the current directory, and normalized.
fn absolute(path: &Path) -> PathBuf {
    normalize(&std::path::absolute(path).unwrap_or_else(|_| path.to_owned()))
}
