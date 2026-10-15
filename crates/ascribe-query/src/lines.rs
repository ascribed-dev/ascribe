//! Lines and columns of source files, counted as `ascribe check` counts
//! them, and the paths answers show.

use std::collections::HashMap;

use ascribe_core::{LineIndex, RelPath, WideEncoding};
use ascribe_resolve::Project;

/// Line indexes of a project's source files, built once each.
pub(crate) struct Lines<'a> {
    project: &'a Project,
    cache: HashMap<RelPath, LineIndex>,
}

impl<'a> Lines<'a> {
    pub(crate) fn new(project: &'a Project) -> Lines<'a> {
        Lines {
            project,
            cache: HashMap::new(),
        }
    }

    /// The line and column, from 1, of a byte offset of a source file;
    /// columns count Unicode characters.
    pub(crate) fn at(&mut self, path: &RelPath, offset: usize) -> (u32, u32) {
        let index = match self.cache.get(path) {
            Some(index) => index,
            None => {
                let text = self.project.file(path).map_or("", |f| &f.source);
                self.cache
                    .entry(path.clone())
                    .or_insert_with(|| LineIndex::new(text))
            }
        };
        index
            .wide_line_col(WideEncoding::Utf32, offset)
            .map_or((1, 1), |p| (p.line + 1, p.col + 1))
    }
}

/// A content path as a path from the project root, `/`-separated: what
/// `ascribe check` reports as a diagnostic's `file`.
pub(crate) fn project_path(project: &Project, path: &RelPath) -> String {
    project.layout().project_path(path).to_string()
}
