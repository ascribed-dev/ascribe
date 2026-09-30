//! File ids: how they're assigned, and what happens to them when files are
//! created, deleted, or renamed.
//!
//! Every location in every diagnostic and every tree names its file by a
//! [`FileId`] ([`Span`](tessera_core::Span)s don't record their file). So the
//! rules for ids are part of the contract between the crates that number
//! files (`tessera-resolve`, `tessera-check`) and the tools that key results
//! by id (the language server):
//!
//! 1. **Id 0 is `ascribe.toml`**, always. Source files have ids from 1.
//! 2. **A fresh load numbers files in path order**, from 1
//!    ([`Project::load`](crate::Project::load), `tessera_check::Project::load`).
//! 3. **An id names a path** for as long as the [`FileIds`] table lives (in an
//!    [`IncrementalProject`](crate::IncrementalProject), its whole life): a path
//!    is given an id the first time a file is there and keeps it. Editing a
//!    file doesn't change its id.
//! 4. **Ids are never reused for another path.** A new path gets the next
//!    unused number, so a diagnostic located in file *n* can only ever mean the
//!    path *n* named when it was made.
//! 5. **Deleting a file retires its id's file, not its number**: no live file
//!    has that id, so [`Project::path_of`](crate::Project::path_of) returns
//!    `None`, and a result located there is stale. If a file comes back at the
//!    same path (an editor's save-by-rename, a `git checkout`), it gets its old
//!    id back.
//! 6. **A rename is a deletion and a creation.** The new path has its own id
//!    (its old one if a file was ever there), because relative references
//!    resolve from the path, so everything about the file is re-derived
//!    anyway.
//! 7. **A new load starts over.** Ids are stable within one table, not across
//!    loads. A consumer that keeps ids across a restart must key by path.
//!
//! `tessera_check::Project` accepts any ids (it finds a file by id, not by
//! position), so the language server builds one from a snapshot's ids, and the
//! two agree.

use std::collections::BTreeMap;

use tessera_core::{FileId, RelPath};

/// The ids of the paths a project has had files at. See the [module
/// documentation](self) for the rules.
// Ids are per path, never reused, and a rename is a
// deletion plus a creation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileIds {
    by_path: BTreeMap<RelPath, FileId>,
    next: u32,
}

impl Default for FileIds {
    fn default() -> FileIds {
        FileIds {
            by_path: BTreeMap::new(),
            next: 1,
        }
    }
}

impl FileIds {
    /// An empty table: the first path assigned gets id 1.
    pub fn new() -> FileIds {
        FileIds::default()
    }

    /// The id of a path a file has been at.
    pub fn get(&self, path: &RelPath) -> Option<FileId> {
        self.by_path.get(path).copied()
    }

    /// The id of a path: its own if it has one, else the next unused number.
    pub fn assign(&mut self, path: &RelPath) -> FileId {
        if let Some(id) = self.by_path.get(path) {
            return *id;
        }
        let id = FileId::new(self.next);
        // 2^32 distinct paths is not a project; saturate rather than wrap, so
        // an id is never reused for another path except at that limit.
        self.next = self.next.saturating_add(1);
        self.by_path.insert(path.clone(), id);
        id
    }

    /// Every path that has an id, with it, in path order. Includes the paths
    /// of files that were deleted.
    pub fn iter(&self) -> impl Iterator<Item = (&RelPath, FileId)> {
        self.by_path.iter().map(|(p, id)| (p, *id))
    }

    /// How many paths have ids.
    pub fn len(&self) -> usize {
        self.by_path.len()
    }

    /// Whether no path has an id.
    pub fn is_empty(&self) -> bool {
        self.by_path.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> RelPath {
        RelPath::parse(s).expect("a path")
    }

    #[test]
    fn ids_start_at_one_and_are_stable_per_path() {
        let mut ids = FileIds::new();
        assert_eq!(ids.assign(&p("b.md")), FileId::new(1));
        assert_eq!(ids.assign(&p("a.md")), FileId::new(2));
        assert_eq!(ids.assign(&p("b.md")), FileId::new(1));
        assert_eq!(ids.get(&p("c.md")), None);
        assert_eq!(ids.len(), 2);
    }
}
