//! [`SourceBoundary`]: the content root's boundary, for a crate that finds
//! files on disk itself, and [`Visited`]: the folders a walk has been in.

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

/// Which files a symbolic link may lead to (SPEC §2.1): a link on the way to a
/// file in the content root must lead to a source file of the content root.
/// `ascribe_resolve::FileSystem` keeps the rule; this trait hands it to a
/// crate that walks the disk itself and can't depend on `ascribe-resolve`
/// (`ascribe-fmt`), and `ascribe_resolve::DiskFs` implements it.
pub trait SourceBoundary {
    /// Whether the file at `path` may be read as a source file. A file in the
    /// content root that's a symbolic link, or is in a linked folder, may be
    /// read only when the link leads to a source file of the content root;
    /// otherwise this says why not, in the words `check` reports it with
    /// (`source-unreadable`). A file reached through no link, and one outside
    /// the content root, may be read.
    ///
    /// # Errors
    ///
    /// The file is reached through a symbolic link that leads to a file that
    /// isn't a source file of the content root: the reason.
    fn check_link(&self, path: &Path) -> Result<(), String>;
}

/// The folders a walk of the disk has gone into, each known by its real path,
/// so a walk that follows symbolic links goes into each folder once: a link
/// to a folder it has already been in, such as a link to its own folder,
/// isn't followed again. The source index's walk (`ascribe_resolve::DiskFs`)
/// keeps this rule, and so does every other walk of a project's folders that
/// follows links (the language server's, and `ascribe-fmt`'s).
#[derive(Clone, Debug, Default)]
pub struct Visited(BTreeSet<PathBuf>);

impl Visited {
    /// Records that a walk reached `dir`. Its real path the first time, so the
    /// walk goes in; `None` after that, so it doesn't.
    ///
    /// # Errors
    ///
    /// The folder's real path can't be found: it's gone, or a link on the way
    /// leads nowhere.
    pub fn enter(&mut self, dir: &Path) -> io::Result<Option<PathBuf>> {
        // Outside FileSystem: where a folder really is, which the walks inside
        // FileSystem and outside it ask alike; it reads no file.
        let real = std::fs::canonicalize(dir)?;
        Ok(self.0.insert(real.clone()).then_some(real))
    }
}
