//! The copies in `sources/<name>/`: what's there, and writing and removing
//! them without leaving the folder.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tessera_core::RelPath;
use tessera_resolve::{DiskFs, FileSystem};

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

/// A source's copies folder.
pub(crate) struct Folder {
    /// The project root.
    root: PathBuf,
    /// The project's files.
    files: DiskFs,
    /// The folder, relative to the root: `sources/<name>`.
    rel: RelPath,
}

impl Folder {
    pub(crate) fn new(root: &Path, files: &DiskFs, rel: RelPath) -> Folder {
        Folder {
            root: root.to_path_buf(),
            files: files.clone(),
            rel,
        }
    }

    fn dir(&self) -> PathBuf {
        on_disk(&self.root, &self.rel)
    }

    /// Every file in the folder, by path relative to it, without following
    /// links.
    pub(crate) fn files(&self) -> Vec<String> {
        self.files
            .files_in(&self.rel)
            .iter()
            .filter_map(|path| path.relative_to(&self.rel))
            .map(|path| path.as_str().to_owned())
            .collect()
    }

    /// The bytes of a copy, if it's a file (not a link).
    pub(crate) fn read(&self, path: &str) -> Option<Vec<u8>> {
        let full = self.path(path)?;
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
        // Outside FileSystem: writing a copy, which the FileSystem doesn't do.
        // From the project root down, every folder that exists is a real
        // one.
        let mut dir = self.root.clone();
        let segments: Vec<&str> = self.rel.segments().chain(path.split('/')).collect();
        let (_, folders) = segments.split_last().unwrap_or((&"", &[]));
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
        if fs::symlink_metadata(&full).is_ok_and(|m| !m.is_file()) {
            fs::remove_file(&full)?;
        }
        fs::write(&full, bytes)?;
        Ok(true)
    }

    /// Removes a copy, then each folder it leaves empty, up to and including
    /// the source's folder and `sources/`.
    pub(crate) fn remove(&self, path: &str) -> io::Result<()> {
        let Some(full) = self.path(path) else {
            return Ok(());
        };
        match fs::remove_file(&full) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        let mut dir = full;
        let stop = self.root.clone();
        while dir.pop() && dir != stop {
            // Only an empty folder is removed.
            if fs::remove_dir(&dir).is_err() {
                break;
            }
        }
        Ok(())
    }
}

fn on_disk(base: &Path, rel: &RelPath) -> PathBuf {
    rel.segments().fold(base.to_path_buf(), |p, s| p.join(s))
}
