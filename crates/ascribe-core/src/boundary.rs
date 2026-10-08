//! [`SourceBoundary`]: the content root's boundary, for a crate that finds
//! files on disk itself.

use std::path::Path;

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
