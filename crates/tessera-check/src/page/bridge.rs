//! The source index over a checked [`Project`]: the same files, in memory.

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use tessera_core::{FileId, RelPath};
use tessera_resolve::{FileSystem, Probe, Sources};

use crate::Project;

/// The files of a checked project as the source index reads them: the source
/// texts held in memory (the language server's buffers as much as the files
/// on disk), and everything else, such as images, from the project's own file
/// system.
struct Held<'p>(&'p Project);

impl FileSystem for Held<'_> {
    fn sources(&self) -> Sources {
        Sources {
            // A file that couldn't be read isn't a source another file can
            // name, and is reported as `source-unreadable` at file level.
            paths: self
                .0
                .sources()
                .iter()
                .filter(|s| s.unreadable.is_none())
                .map(|s| s.path.clone())
                .collect(),
            unreadable: Vec::new(),
        }
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        self.0
            .source_at(path)
            .filter(|s| s.unreadable.is_none())
            .map(|s| s.text.clone())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such source file"))
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        self.0.file_system().probe(project_path)
    }
}

/// The source index of `project`, and how its file ids map to the project's.
///
/// `tessera-check` and `tessera-resolve` number source files the same way when
/// they're in path order, but a project built from parts may not be, so every
/// location the index reports goes through [`Indexed::file`].
pub(super) struct Indexed {
    pub index: tessera_resolve::Project,
    ids: HashMap<FileId, FileId>,
}

impl Indexed {
    pub fn new(project: &Project) -> Indexed {
        let index = tessera_resolve::Project::load(
            Arc::new(project.model().clone()),
            project.layout().clone(),
            &Held(project),
        );
        let ids = index
            .files()
            .filter_map(|f| Some((f.file, project.source_at(&f.path)?.id)))
            .collect();
        Indexed { index, ids }
    }

    /// The checked project's id for a file of the index.
    pub fn file(&self, id: FileId) -> FileId {
        self.ids.get(&id).copied().unwrap_or(id)
    }
}
