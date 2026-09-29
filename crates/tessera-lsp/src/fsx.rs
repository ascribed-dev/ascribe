//! The file systems the server hands to the project and to the checks.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::sync::{PoisonError, RwLock};

use tessera_core::RelPath;
use tessera_resolve::{DiskFs, FileSystem, Probe, Sources};

/// The disk with the editor's open source buffers over it, for loading a
/// project: what the project reads for a source is the buffer when there is
/// one (an open document's contents win over the file on disk).
pub(crate) struct BufferFs {
    disk: DiskFs,
    buffers: BTreeMap<RelPath, String>,
}

impl BufferFs {
    pub(crate) fn new(disk: DiskFs, buffers: BTreeMap<RelPath, String>) -> BufferFs {
        BufferFs { disk, buffers }
    }
}

impl FileSystem for BufferFs {
    fn sources(&self) -> Sources {
        let mut sources = self.disk.sources();
        for path in self.buffers.keys() {
            if !sources.paths.contains(path) {
                sources.paths.push(path.clone());
            }
        }
        // A directory that couldn't be listed stays listed, but a file the
        // editor holds is readable whatever the disk says about it.
        sources
            .unreadable
            .retain(|u| !self.buffers.contains_key(&u.path));
        sources.paths.sort();
        sources
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        match self.buffers.get(path) {
            Some(text) => Ok(text.clone()),
            None => self.disk.read(path),
        }
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        self.disk.probe(project_path)
    }
}

/// What the file-level checks probe for files that aren't sources: the disk,
/// with the files the editor and the file watcher have reported appearing or
/// disappearing layered over it. It answers exactly as the incremental
/// project's own overlay does (`tessera_resolve::incremental`), so the checks
/// and the source index agree on which images exist.
///
/// It's shared and updated as changes arrive; a result computed from an older
/// snapshot is dropped by the currency check before it's published, so a
/// probe that sees a newer state than its snapshot can't reach the editor.
pub(crate) struct LayerFs {
    disk: DiskFs,
    layers: RwLock<Layers>,
}

#[derive(Default)]
struct Layers {
    added: BTreeSet<RelPath>,
    removed: BTreeSet<RelPath>,
}

impl LayerFs {
    pub(crate) fn new(disk: DiskFs) -> LayerFs {
        LayerFs {
            disk,
            layers: RwLock::new(Layers::default()),
        }
    }

    /// A file appeared (`present`) or disappeared, at a path relative to the
    /// project root.
    pub(crate) fn set(&self, project_path: &RelPath, present: bool) {
        let mut layers = self.layers.write().unwrap_or_else(PoisonError::into_inner);
        if present {
            layers.removed.remove(project_path);
            layers.added.insert(project_path.clone());
        } else {
            layers.added.remove(project_path);
            layers.removed.insert(project_path.clone());
        }
    }
}

impl FileSystem for LayerFs {
    fn sources(&self) -> Sources {
        self.disk.sources()
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        self.disk.read(path)
    }

    /// Exact names win; among names that differ only in case, files added here
    /// come before the disk's, first in path order.
    fn probe(&self, project_path: &RelPath) -> Probe {
        let layers = self.layers.read().unwrap_or_else(PoisonError::into_inner);
        if layers.added.contains(project_path) {
            return Probe::File;
        }
        let from_disk = self.disk.probe(project_path);
        if from_disk == Probe::File && !layers.removed.contains(project_path) {
            return Probe::File;
        }
        let folded = project_path.as_str().to_lowercase();
        if let Some(twin) = layers
            .added
            .iter()
            .find(|p| p.as_str().to_lowercase() == folded)
        {
            return Probe::CaseMismatch(twin.clone());
        }
        match from_disk {
            Probe::CaseMismatch(actual) if !layers.removed.contains(&actual) => {
                Probe::CaseMismatch(actual)
            }
            _ => Probe::Missing,
        }
    }
}
