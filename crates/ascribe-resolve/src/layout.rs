//! Where the content root and the output directory sit, and the boundary rule
//! for local files: a file must be inside the content root or the project
//! root, and not inside the output directory.

use ascribe_core::RelPath;
use ascribe_model::ContentModel;

/// The project's directories, as the boundary rule needs them.
///
/// Every source path in this crate is a **content path**, relative to the
/// content root. The layout says where that root is, relative to the
/// **project root** (the directory containing `ascribe.toml`), and where the
/// output directory is, so that a reference can be checked against the
/// boundary of what a build may copy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The content root, relative to the project root. Empty when they are the
    /// same directory.
    pub content_root: RelPath,
    /// `[project] output-dir`, relative to the project root.
    pub output_dir: RelPath,
}

impl Layout {
    /// The layout `[project]` in the content model declares.
    ///
    /// A path that isn't valid (an absolute path, or one with a NUL) reads as
    /// the project root; the content-model loader rejects such values before
    /// a model exists.
    pub fn from_model(model: &ContentModel) -> Layout {
        Layout {
            content_root: RelPath::parse(&model.project.content_root).unwrap_or_default(),
            output_dir: RelPath::parse(&model.project.output_dir).unwrap_or_default(),
        }
    }

    /// A content path as a path relative to the project root.
    pub fn project_path(&self, content_path: &RelPath) -> RelPath {
        // A content path never has a NUL or a leading `/`, so `join` can't fail;
        // the fallback keeps the content path itself.
        self.content_root
            .join(content_path.as_str())
            .unwrap_or_else(|_| content_path.clone())
    }

    /// Whether a build may copy the file at this content path: it's inside the
    /// content root or inside the project root, and not inside the output
    /// directory (SPEC §9.4).
    pub(crate) fn is_allowed(&self, content_path: &RelPath) -> bool {
        let in_project = self.project_path(content_path);
        let inside = content_path.is_inside() || in_project.is_inside();
        let in_output = !self.output_dir.is_root() && in_project.starts_with(&self.output_dir);
        inside && !in_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> RelPath {
        RelPath::parse(s).expect("valid path")
    }

    fn layout() -> Layout {
        Layout {
            content_root: p("docs"),
            output_dir: p(".ascribe/build"),
        }
    }

    #[test]
    fn inside_the_content_root_or_the_project_is_allowed() {
        let l = layout();
        assert!(l.is_allowed(&p("guides/a.png")));
        assert!(l.is_allowed(&p("../shared/logo.png")));
        assert!(l.is_allowed(&p("../README.md")));
    }

    #[test]
    fn outside_the_project_and_the_output_directory_are_not() {
        let l = layout();
        assert!(!l.is_allowed(&p("../../outside.png")));
        assert!(!l.is_allowed(&p("../.ascribe/build/site/p.png")));
        assert!(l.is_allowed(&p("../.ascribe/other.png")));
    }

    #[test]
    fn content_root_at_the_project_root() {
        let l = Layout {
            content_root: RelPath::root(),
            output_dir: p(".ascribe/build"),
        };
        assert!(l.is_allowed(&p("a.png")));
        assert!(!l.is_allowed(&p("../a.png")));
        assert!(!l.is_allowed(&p(".ascribe/build/x")));
    }
}
