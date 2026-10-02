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
///
/// It reports no nested projects' folders, so `Project::is_source` on an index
/// built from it goes by the path alone: fine for the page checks, which only
/// read the sources listed here, and those already leave such folders out.
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
            // See the type's documentation.
            nested: Vec::new(),
            own_folder: None,
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

/// The source index a [`PageChecker`](super::PageChecker) reads: built from a
/// checked project's texts, or one the caller already has (the language server's
/// incremental index).
pub(super) enum IndexRef<'p> {
    Owned(Box<tessera_resolve::Project>),
    Borrowed(&'p tessera_resolve::Project),
}

impl std::ops::Deref for IndexRef<'_> {
    type Target = tessera_resolve::Project;

    fn deref(&self) -> &tessera_resolve::Project {
        match self {
            IndexRef::Owned(index) => index,
            IndexRef::Borrowed(index) => index,
        }
    }
}

/// The source index of `project`, and how its file ids map to the project's.
///
/// `tessera-check` and `tessera-resolve` number source files the same way when
/// they're in path order, but a project built from parts may not be, so every
/// location the index reports goes through [`Indexed::file`].
pub(super) struct Indexed<'p> {
    pub index: IndexRef<'p>,
    ids: HashMap<FileId, FileId>,
}

impl<'p> Indexed<'p> {
    pub fn new(project: &Project) -> Indexed<'p> {
        let index = tessera_resolve::Project::load(
            Arc::new(project.model().clone()),
            project.layout().clone(),
            &Held(project),
        );
        let ids = index
            .files()
            .filter_map(|f| Some((f.file, project.source_at(&f.path)?.id)))
            .collect();
        Indexed {
            index: IndexRef::Owned(Box::new(index)),
            ids,
        }
    }

    /// An index the caller already has, whose file ids are the checked
    /// project's (no renumbering).
    pub fn shared(index: &'p tessera_resolve::Project) -> Indexed<'p> {
        Indexed {
            index: IndexRef::Borrowed(index),
            ids: HashMap::new(),
        }
    }

    /// The checked project's id for a file of the index.
    pub fn file(&self, id: FileId) -> FileId {
        self.ids.get(&id).copied().unwrap_or(id)
    }
}
