//! The file systems the server hands to the project and to the checks.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::sync::{Mutex, PoisonError, RwLock};

use ascribe_core::RelPath;
use ascribe_resolve::{DiskFs, FileSystem, Probe, Sources, in_nested_project};

/// The disk with the editor's open source buffers over it, for loading a
/// project: what the project reads for a source is the buffer when there is
/// one (an open document's contents win over the file on disk). A buffer in a
/// nested project's folder isn't a source, as the file on disk isn't.
///
/// It answers each probe from the disk once and remembers the answer. The
/// incremental project asks its base whether a non-source file was there
/// *before* an update ("the base plus the changes reported so far"), and by the
/// time the editor reports a file created or deleted the disk already shows the
/// new state; remembering what the disk said when a reference first looked
/// keeps the base what it was when the project read it, and the reported
/// changes layered over it (`ascribe_resolve::incremental`) do the rest.
pub(crate) struct BufferFs {
    disk: DiskFs,
    buffers: BTreeMap<RelPath, String>,
    probes: Mutex<HashMap<RelPath, Probe>>,
}

impl BufferFs {
    pub(crate) fn new(disk: DiskFs, buffers: BTreeMap<RelPath, String>) -> BufferFs {
        BufferFs {
            disk,
            buffers,
            probes: Mutex::new(HashMap::new()),
        }
    }
}

impl FileSystem for BufferFs {
    fn sources(&self) -> Sources {
        let mut sources = self.disk.sources();
        for path in self.buffers.keys() {
            if !sources.paths.contains(path) && !in_nested_project(path, &sources.nested) {
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

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        self.disk.read_file(project_path)
    }

    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        self.disk.real_path(project_path)
    }

    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        self.disk.files_in(project_dir)
    }

    fn files_named(
        &self,
        project_dir: &RelPath,
        name: &str,
        enter: &dyn Fn(&RelPath) -> bool,
    ) -> Vec<RelPath> {
        self.disk.files_named(project_dir, name, enter)
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        let mut probes = self.probes.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(answer) = probes.get(project_path) {
            return answer.clone();
        }
        let answer = self.disk.probe(project_path);
        probes.insert(project_path.clone(), answer.clone());
        answer
    }
}

/// What the file-level checks probe for files that aren't sources: the disk,
/// with the files the editor and the file watcher have reported appearing or
/// disappearing layered over it. It answers exactly as the incremental
/// project's own overlay does (`ascribe_resolve::incremental`), so the checks
/// and the source index agree on which images exist.
///
/// It's shared and updated as changes arrive; a result computed from an older
/// snapshot is dropped by the currency check before it's published, so a
/// probe that sees a newer state than its snapshot can't reach the editor.
///
/// It also holds the buffers of the open files that aren't sources but are
/// read as the editor has them: memo files (SPEC §7.7), which a memo store
/// reads through it, and `ascribe.nav` (SPEC §7.8), which the checks do.
pub(crate) struct LayerFs {
    disk: DiskFs,
    layers: RwLock<Layers>,
    buffers: RwLock<BTreeMap<RelPath, String>>,
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
            buffers: RwLock::new(BTreeMap::new()),
        }
    }

    /// An open memo file's or `ascribe.nav`'s text, at a path relative to
    /// the project root, or `None` once it's closed and the disk has it
    /// again.
    pub(crate) fn set_buffer(&self, project_path: &RelPath, text: Option<String>) {
        let mut buffers = self.buffers.write().unwrap_or_else(PoisonError::into_inner);
        match text {
            Some(text) => buffers.insert(project_path.clone(), text),
            None => buffers.remove(project_path),
        };
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

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        let buffers = self.buffers.read().unwrap_or_else(PoisonError::into_inner);
        match buffers.get(project_path) {
            Some(text) => Ok(text.clone().into_bytes()),
            None => self.disk.read_file(project_path),
        }
    }

    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        self.disk.real_path(project_path)
    }

    /// The disk's files, and an open file that isn't saved yet.
    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        let mut files = self.disk.files_in(project_dir);
        let buffers = self.buffers.read().unwrap_or_else(PoisonError::into_inner);
        for path in buffers.keys() {
            if path.starts_with(project_dir) && path != project_dir && !files.contains(path) {
                files.push(path.clone());
            }
        }
        files.sort();
        files
    }

    /// The disk's files: an open file that isn't saved yet is looked for by
    /// name only when it's asked about, as `ascribe.examples.toml` is.
    fn files_named(
        &self,
        project_dir: &RelPath,
        name: &str,
        enter: &dyn Fn(&RelPath) -> bool,
    ) -> Vec<RelPath> {
        self.disk.files_named(project_dir, name, enter)
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
