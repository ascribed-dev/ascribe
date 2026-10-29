//! Where `ascribe report` keeps what it remembers between runs: the link
//! answers it reuses and its histories, under the project's `.ascribe/`.
//!
//! The library doesn't write these files itself. Its caller hands it a
//! [`Store`]: the CLI one on the disk, under the project's folder, and a
//! test a [`MemoryStore`]. A path is relative to the project's folder,
//! with `/` between its parts.
//!
//! - **Written whole or not at all:** a reader sees the old text or the
//!   new, never part of one.
//! - **A write that stopped part way** may leave something behind on the
//!   disk; [`Store::tidy`] deletes it once it's over a day old.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::sync::Mutex;

use ascribe_core::Date;

/// Files the library reads and writes through its caller.
pub trait Store: Debug + Send + Sync {
    /// The text of the file at `path`, when there's one that reads.
    fn read(&self, path: &str) -> Option<String>;

    /// Writes `text` as the file at `path`, whole or not at all, making
    /// the folders on the way.
    fn write(&self, path: &str, text: &str) -> std::io::Result<()>;

    /// Deletes the file at `path`, when there's one.
    fn remove(&self, path: &str);

    /// What's directly in `folder`: the names of its files, and of its
    /// folders. What a write left behind isn't listed.
    fn list(&self, folder: &str) -> Listing;

    /// Deletes `folder` when it's empty.
    fn remove_folder(&self, folder: &str);

    /// Deletes what a write that stopped part way left in `folder`, when
    /// it's over a day old by `today`, not the clock.
    fn tidy(&self, folder: &str, today: Date);
}

/// What's directly in a folder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing {
    /// The names of its files.
    pub files: Vec<String>,
    /// The names of its folders.
    pub folders: Vec<String>,
}

/// A [`Store`] held in memory, for a test. A folder is there while a file
/// is in it.
#[derive(Debug, Default)]
pub struct MemoryStore {
    files: Mutex<BTreeMap<String, String>>,
}

impl MemoryStore {
    /// An empty store.
    pub fn new() -> MemoryStore {
        MemoryStore::default()
    }

    /// The path of every file, sorted.
    pub fn paths(&self) -> Vec<String> {
        self.held().keys().cloned().collect()
    }

    fn held(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, String>> {
        // A test that panicked while holding the lock left whole entries.
        self.files
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Store for MemoryStore {
    fn read(&self, path: &str) -> Option<String> {
        self.held().get(path).cloned()
    }

    fn write(&self, path: &str, text: &str) -> std::io::Result<()> {
        self.held().insert(path.to_owned(), text.to_owned());
        Ok(())
    }

    fn remove(&self, path: &str) {
        self.held().remove(path);
    }

    fn list(&self, folder: &str) -> Listing {
        let prefix = format!("{folder}/");
        let mut files = Vec::new();
        let mut folders = BTreeSet::new();
        for path in self.held().keys() {
            let Some(rest) = path.strip_prefix(&prefix) else {
                continue;
            };
            match rest.split_once('/') {
                Some((name, _)) => {
                    folders.insert(name.to_owned());
                }
                None => files.push(rest.to_owned()),
            }
        }
        Listing {
            files,
            folders: folders.into_iter().collect(),
        }
    }

    fn remove_folder(&self, _folder: &str) {}

    fn tidy(&self, _folder: &str, _today: Date) {}
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn a_memory_store_lists_files_and_the_folders_they_re_in() {
        let store = MemoryStore::new();
        store.write("a/b/c.json", "1").unwrap();
        store.write("a/d.json", "2").unwrap();
        store.write("ab/e.json", "3").unwrap();
        assert_eq!(
            store.list("a"),
            Listing {
                files: vec!["d.json".to_owned()],
                folders: vec!["b".to_owned()],
            }
        );
        assert_eq!(store.read("a/d.json").as_deref(), Some("2"));
        store.remove("a/b/c.json");
        assert_eq!(store.list("a").folders, Vec::<String>::new());
        assert_eq!(store.paths(), ["a/d.json", "ab/e.json"]);
    }
}
