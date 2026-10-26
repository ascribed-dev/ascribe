//! The source index over a checked [`Project`]: the same files, in memory.

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use ascribe_core::{FileId, RelPath};
use ascribe_resolve::{FileSystem, Probe, Sources};

use crate::Project;

/// The files of a checked project as the source index reads them: the source
/// texts held in memory (the language server's buffers as much as the files
/// on disk), and everything else, such as images, from the project's own file
/// system.
///
/// A loaded project's sources are listed as loading listed them, with the
/// unreadable entries, the nested projects' folders, and the project's own
/// folder, so an index built from it is the one the files on disk give. A
/// project built from parts lists its readable sources and no folders, so
/// `Project::is_source` on an index built from it goes by the path alone:
/// fine for the page checks, which only read the sources listed here, and
/// those already leave such folders out.
struct Held<'p>(&'p Project);

impl FileSystem for Held<'_> {
    fn sources(&self) -> Sources {
        if let Some(listing) = self.0.listing() {
            return listing.clone();
        }
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
        let source = self
            .0
            .source_at(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such source file"))?;
        match &source.unreadable {
            // Failing as loading's read failed, for the index to list it as
            // unreadable for the same reason.
            Some(failure) => {
                let kind = if failure.not_utf8 {
                    io::ErrorKind::InvalidData
                } else {
                    io::ErrorKind::Other
                };
                Err(io::Error::new(kind, failure.reason.clone()))
            }
            None => Ok(source.text.clone()),
        }
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        self.0.file_system().probe(project_path)
    }

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        self.0.file_system().read_file(project_path)
    }

    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        self.0.file_system().real_path(project_path)
    }

    fn size(&self, project_path: &RelPath) -> Option<u64> {
        self.0.file_system().size(project_path)
    }

    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        self.0.file_system().files_in(project_dir)
    }
}

/// The source index over a checked project's texts, in the project's
/// own order.
pub(crate) fn held_index(project: &Project) -> ascribe_resolve::Project {
    ascribe_resolve::Project::load(
        Arc::new(project.model().clone()),
        project.layout().clone(),
        &Held(project),
    )
}

/// The source index a [`PageChecker`](super::PageChecker) reads: built from a
/// checked project's texts, or one the caller already has (the language server's
/// incremental index).
pub(super) enum IndexRef<'p> {
    Owned(Arc<ascribe_resolve::Project>),
    Borrowed(&'p ascribe_resolve::Project),
}

impl std::ops::Deref for IndexRef<'_> {
    type Target = ascribe_resolve::Project;

    fn deref(&self) -> &ascribe_resolve::Project {
        match self {
            IndexRef::Owned(index) => index,
            IndexRef::Borrowed(index) => index,
        }
    }
}

/// The source index of `project`, and how its file ids map to the project's.
///
/// `ascribe-check` and `ascribe-resolve` number source files the same way when
/// they're in path order, but a project built from parts may not be, so every
/// location the index reports goes through [`Indexed::file`].
pub(super) struct Indexed<'p> {
    pub index: IndexRef<'p>,
    ids: HashMap<FileId, FileId>,
}

impl<'p> Indexed<'p> {
    /// The project's own source index, which its file-level checks share
    /// (a broken link's closest targets are looked for in it), so a check
    /// of the whole project indexes it once.
    pub fn new(project: &Project) -> Indexed<'p> {
        let index = project.index().clone();
        let ids = index
            .files()
            .filter_map(|f| Some((f.file, project.source_at(&f.path)?.id)))
            .collect();
        Indexed {
            index: IndexRef::Owned(index),
            ids,
        }
    }

    /// An index the caller already has, whose file ids are the checked
    /// project's (no renumbering).
    pub fn shared(index: &'p ascribe_resolve::Project) -> Indexed<'p> {
        Indexed {
            index: IndexRef::Borrowed(index),
            ids: HashMap::new(),
        }
    }

    /// The same index, borrowed, for another checker.
    pub fn borrowed(&self) -> Indexed<'_> {
        Indexed {
            index: IndexRef::Borrowed(&self.index),
            ids: self.ids.clone(),
        }
    }

    /// The checked project's id for a file of the index.
    pub fn file(&self, id: FileId) -> FileId {
        self.ids.get(&id).copied().unwrap_or(id)
    }
}
