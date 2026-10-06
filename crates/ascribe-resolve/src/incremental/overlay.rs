//! The file system an incremental project probes: a base (the disk, or
//! nothing) and the changes applied to it since.

use std::collections::BTreeSet;
use std::io;

use ascribe_core::RelPath;

use crate::fs::{FileSystem, Probe, Sources};

/// A base file system with files added and removed on top of it, by path
/// relative to the project root.
///
/// The language server hears about files appearing and disappearing from the
/// editor and the file watcher; this is how the project sees them without
/// rereading the disk. Only what a reference can probe matters, so it holds
/// paths, not contents.
pub(crate) struct Overlay {
    base: Box<dyn FileSystem + Send>,
    added: BTreeSet<RelPath>,
    removed: BTreeSet<RelPath>,
}

impl Overlay {
    pub(crate) fn new(base: Box<dyn FileSystem + Send>) -> Overlay {
        Overlay {
            base,
            added: BTreeSet::new(),
            removed: BTreeSet::new(),
        }
    }

    /// Whether a file is there, with exactly this name.
    pub(crate) fn has(&self, project_path: &RelPath) -> bool {
        self.probe(project_path) == Probe::File
    }

    /// A file appeared.
    pub(crate) fn add(&mut self, project_path: &RelPath) {
        self.removed.remove(project_path);
        self.added.insert(project_path.clone());
    }

    /// A file disappeared.
    pub(crate) fn remove(&mut self, project_path: &RelPath) {
        self.added.remove(project_path);
        self.removed.insert(project_path.clone());
    }
}

impl FileSystem for Overlay {
    fn sources(&self) -> Sources {
        self.base.sources()
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        self.base.read(path)
    }

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        self.base.read_file(project_path)
    }

    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        self.base.real_path(project_path)
    }

    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        self.base.files_in(project_dir)
    }

    /// Exact names win; among names that differ only in case, files added
    /// here come before the base's, first in path order (as
    /// [`MemoryFs`](crate::MemoryFs) does).
    fn probe(&self, project_path: &RelPath) -> Probe {
        if self.added.contains(project_path) {
            return Probe::File;
        }
        let from_base = self.base.probe(project_path);
        if from_base == Probe::File && !self.removed.contains(project_path) {
            return Probe::File;
        }
        let folded = project_path.as_str().to_lowercase();
        if let Some(twin) = self
            .added
            .iter()
            .find(|p| p.as_str().to_lowercase() == folded)
        {
            return Probe::CaseMismatch(twin.clone());
        }
        match from_base {
            Probe::CaseMismatch(actual) if !self.removed.contains(&actual) => {
                Probe::CaseMismatch(actual)
            }
            _ => Probe::Missing,
        }
    }
}
