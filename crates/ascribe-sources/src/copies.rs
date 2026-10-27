//! The copies in `sources/<name>/`: what's there, and writing and removing
//! them without leaving the folder.

use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ascribe_core::RelPath;
use ascribe_resolve::{DiskFs, FileSystem, FolderExamples, Probe, Sources};

/// The largest file Ascribe copies: a megabyte is generous for an example.
pub const SIZE_LIMIT: usize = 1024 * 1024;

/// Why a file isn't copied.
pub(crate) fn not_text(bytes: &[u8]) -> Option<&'static str> {
    if bytes.contains(&0) {
        Some("it isn't text: it has a NUL character")
    } else if std::str::from_utf8(bytes).is_err() {
        Some("it isn't text: it isn't UTF-8")
    } else {
        None
    }
}

/// A change `fetch` or `update` plans, to make once the plan is asked
/// about: a copy written, a copy removed, or the lock written. Each path is
/// absolute.
#[derive(Clone, Debug)]
pub(crate) enum Change {
    /// A copy written with these bytes.
    Copy { path: PathBuf, bytes: Vec<u8> },
    /// A copy removed.
    Remove { path: PathBuf },
    /// `ascribe.lock` written with this text.
    Lock { path: PathBuf, text: String },
}

impl Change {
    pub(crate) fn path(&self) -> &Path {
        match self {
            Change::Copy { path, .. } | Change::Remove { path } | Change::Lock { path, .. } => path,
        }
    }

    /// Makes the change, under the project `root`.
    pub(crate) fn make(&self, root: &Path) -> io::Result<()> {
        match self {
            Change::Copy { path, bytes } => write_copy(root, path, bytes),
            Change::Remove { path } => remove_copy(root, path),
            // Library write: the lock file, after a fetch that changed it.
            Change::Lock { path, text } => fs::write(path, text),
        }
    }
}

/// A source's copies folder.
pub(crate) struct Folder<'a> {
    /// The project root.
    root: PathBuf,
    /// The project's files.
    files: DiskFs,
    /// The folder, relative to the root: `sources/<name>`.
    rel: RelPath,
    /// When planning, where writing and removing a copy record the change
    /// instead, and where the folder is read from as the plan leaves it.
    planned: Option<&'a RefCell<Vec<Change>>>,
}

impl<'a> Folder<'a> {
    pub(crate) fn new(
        root: &Path,
        files: &DiskFs,
        rel: RelPath,
        planned: Option<&'a RefCell<Vec<Change>>>,
    ) -> Folder<'a> {
        Folder {
            root: root.to_path_buf(),
            files: files.clone(),
            rel,
            planned,
        }
    }

    fn dir(&self) -> PathBuf {
        on_disk(&self.root, &self.rel)
    }

    /// Every file in the folder, by path relative to it, without following
    /// links.
    pub(crate) fn files(&self) -> Vec<String> {
        self.view()
            .files_in(&self.rel)
            .iter()
            .filter_map(|path| path.relative_to(&self.rel))
            .map(|path| path.as_str().to_owned())
            .collect()
    }

    /// The tested examples' config and observations among the copies, as
    /// read.
    pub(crate) fn examples(&self) -> FolderExamples {
        FolderExamples::read(&self.view(), &self.rel)
    }

    /// The project's files, as the plan so far leaves them.
    fn view(&self) -> Planned<'_> {
        Planned {
            root: &self.root,
            base: &self.files,
            changes: self.planned,
        }
    }

    /// The last change planned to the file at `full`, if any.
    fn planned_at(&self, full: &Path) -> Option<Change> {
        planned_at(self.planned?, full)
    }

    /// The bytes of a copy, if it's a file (not a link).
    pub(crate) fn read(&self, path: &str) -> Option<Vec<u8>> {
        let full = self.path(path)?;
        match self.planned_at(&full) {
            Some(Change::Copy { bytes, .. }) => return Some(bytes),
            Some(_) => return None,
            None => {}
        }
        // Outside FileSystem: a copy is read only to compare it with what would
        // be written over it, and never through a link, which
        // `FileSystem::read_file` follows.
        let meta = fs::symlink_metadata(&full).ok()?;
        if !meta.is_file() {
            return None;
        }
        fs::read(full).ok()
    }

    /// Where a copy goes: inside the folder, by a path that can't leave it.
    fn path(&self, path: &str) -> Option<PathBuf> {
        let rel = RelPath::parse(path).ok()?;
        if !rel.is_inside() || rel.is_root() || rel.segments().any(|s| s == "..") {
            return None;
        }
        Some(on_disk(&self.dir(), &rel))
    }

    /// Writes a copy, unless it's already these bytes. Each folder on the way
    /// must be a folder, not a link, so nothing is written outside. Returns
    /// whether the file changed.
    pub(crate) fn write(&self, path: &str, bytes: &[u8]) -> io::Result<bool> {
        let full = self.path(path).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "not a path in the folder")
        })?;
        if self.read(path).as_deref() == Some(bytes) {
            return Ok(false);
        }
        match self.planned {
            Some(planned) => planned.borrow_mut().push(Change::Copy {
                path: full,
                bytes: bytes.to_vec(),
            }),
            None => write_copy(&self.root, &full, bytes)?,
        }
        Ok(true)
    }

    /// Removes a copy, then each folder it leaves empty, up to and including
    /// the source's folder and `sources/`.
    pub(crate) fn remove(&self, path: &str) -> io::Result<()> {
        let Some(full) = self.path(path) else {
            return Ok(());
        };
        match self.planned {
            Some(planned) => {
                if self.view().probe_full(&full) {
                    planned.borrow_mut().push(Change::Remove { path: full });
                }
            }
            None => remove_copy(&self.root, &full)?,
        }
        Ok(())
    }
}

/// Writes a copy at `full`, under `root`. Each folder on the way must be a
/// folder, not a link, so nothing is written outside.
fn write_copy(root: &Path, full: &Path, bytes: &[u8]) -> io::Result<()> {
    let segments: Vec<String> = full
        .strip_prefix(root)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "not a path in the project"))?
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    // Outside FileSystem: writing a copy, which the FileSystem doesn't do.
    // From the project root down, every folder that exists is a real
    // one.
    let mut dir = root.to_path_buf();
    let (_, folders) = segments.split_last().unwrap_or((&String::new(), &[]));
    for segment in folders {
        dir.push(segment);
        match fs::symlink_metadata(&dir) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("{} is in the way: it isn't a folder", dir.display()),
                ));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => fs::create_dir(&dir)?,
            Err(e) => return Err(e),
        }
    }
    // Outside FileSystem: as above.
    // A link where the copy goes is replaced, not written through.
    if fs::symlink_metadata(full).is_ok_and(|m| !m.is_file()) {
        fs::remove_file(full)?;
    }
    fs::write(full, bytes)
}

/// Removes the copy at `full`, then each folder it leaves empty, up to but
/// not including `root`.
fn remove_copy(root: &Path, full: &Path) -> io::Result<()> {
    // Outside FileSystem: removing a copy, which the FileSystem doesn't do.
    match fs::remove_file(full) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    let mut dir = full.to_path_buf();
    while dir.pop() && dir != root {
        // Only an empty folder is removed.
        if fs::remove_dir(&dir).is_err() {
            break;
        }
    }
    Ok(())
}

/// The last change in `changes` to the file at `full`, if any.
pub(crate) fn planned_at(changes: &RefCell<Vec<Change>>, full: &Path) -> Option<Change> {
    changes
        .borrow()
        .iter()
        .rev()
        .find(|c| c.path() == full)
        .cloned()
}

/// The project's files as a plan leaves them: the disk, with the copies it
/// writes there and the ones it removes gone. What `update` reads its tested
/// examples' status from, before writing anything.
struct Planned<'a> {
    root: &'a Path,
    base: &'a DiskFs,
    changes: Option<&'a RefCell<Vec<Change>>>,
}

impl Planned<'_> {
    fn at(&self, project_path: &RelPath) -> Option<Change> {
        planned_at(self.changes?, &on_disk(self.root, project_path))
    }

    /// Whether a file is at `full`, as the plan leaves it.
    fn probe_full(&self, full: &Path) -> bool {
        match self.changes.and_then(|c| planned_at(c, full)) {
            Some(Change::Remove { .. }) => false,
            Some(_) => true,
            // Outside FileSystem: whether a copy is there to remove.
            None => fs::symlink_metadata(full).is_ok(),
        }
    }
}

impl FileSystem for Planned<'_> {
    fn sources(&self) -> Sources {
        self.base.sources()
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        self.base.read(path)
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        match self.at(project_path) {
            Some(Change::Remove { .. }) => Probe::Missing,
            Some(_) => Probe::File,
            None => self.base.probe(project_path),
        }
    }

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        match self.at(project_path) {
            Some(Change::Copy { bytes, .. }) => Ok(bytes),
            Some(Change::Lock { text, .. }) => Ok(text.into_bytes()),
            Some(Change::Remove { .. }) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "removed by the plan",
            )),
            None => self.base.read_file(project_path),
        }
    }

    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        match self.at(project_path) {
            Some(Change::Remove { .. }) => None,
            Some(_) => Some(project_path.clone()),
            None => self.base.real_path(project_path),
        }
    }

    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        let mut files = self.base.files_in(project_dir);
        let Some(changes) = self.changes else {
            return files;
        };
        let dir = on_disk(self.root, project_dir);
        for change in changes.borrow().iter() {
            let Ok(rel) = change.path().strip_prefix(self.root) else {
                continue;
            };
            if !change.path().starts_with(&dir) {
                continue;
            }
            let rel: Vec<String> = rel
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect();
            let Ok(rel) = RelPath::parse(&rel.join("/")) else {
                continue;
            };
            match change {
                Change::Remove { .. } => files.retain(|f| f != &rel),
                _ => {
                    if !files.contains(&rel) {
                        files.push(rel);
                    }
                }
            }
        }
        files.sort();
        files
    }
}

fn on_disk(base: &Path, rel: &RelPath) -> PathBuf {
    rel.segments().fold(base.to_path_buf(), |p, s| p.join(s))
}
