//! Loads and validates a documentation set's content model, `ascribe.toml`.
//!
//! [`load`] reads the file, checks it against every loading rule (each a
//! `model-` diagnostic in the registry), and returns a typed
//! [`ContentModel`], or the [`Issue`]s that stopped it. Every issue is
//! reported by registry slug, at the span of the offending key or value in
//! `ascribe.toml`.
//!
//! | Module | Contents |
//! |---|---|
//! | [`model`] | [`ContentModel`], its parts, and the queries other crates ask |
//! | [`types`] | Field types, and [`validate_frontmatter`] |
//! | [`inline`] | Code spans in a field that sets `inline = "code"` |
//! | [`pattern`] | [`Pattern`]: the glob syntax of `files` and fragment patterns |
//! | [`lock`] | [`Lock`]: `ascribe.lock`, the pins of sources in other repositories |
//!
//! The availability-spec parser is in `ascribe_core::availability`; this
//! crate checks specs against the model ([`ContentModel::check_availability`]).
//!
//! # Loading
//!
//! ```
//! let model = ascribe_model::load_str("spec = \"0.1\"\n", ascribe_core::FileId::new(0))
//!     .expect("the minimal model is valid");
//! assert_eq!(model.project.content_root, "docs");
//! assert_eq!(model.builds[0].name, "site");
//! ```
//!
//! Warnings (`model-name-case`, `model-build-filter-excluded`) don't stop
//! loading; they are in [`ContentModel::warnings`]. When loading fails, the
//! returned issues include the warnings found before the failure.

mod checks;
mod fields;
pub mod inline;
mod intended;
mod loader;
pub mod lock;
pub mod model;
mod names;
pub mod pattern;
mod sections;
mod toml_util;
pub mod types;

use std::path::Path;

pub use ascribe_core::Issue;
pub use inline::{InlineMarkup, Segment};
pub use lock::{LOCK_FILE, LOCK_VERSION, Lock, LockedFile, LockedSource, file_hash, short_commit};
pub use model::*;
pub use names::{KEY_RULE, NAME_WORD_RULE, edit_distance, is_key, is_name_word, suggest};
pub use pattern::{Pattern, PatternError};
pub use types::{Field, FieldType, FrontmatterSchema, SchemaOwner, validate_frontmatter};

use ascribe_core::FileId;

/// The content model's file name, at the project root.
pub const MODEL_FILE: &str = "ascribe.toml";

/// Reads and validates `ascribe.toml` at `path`.
///
/// Issues are located in file `FileId::new(0)`; use [`load_with_file`] when
/// the caller owns a file table. The filesystem rules run relative to the
/// file's directory (the project root): the content root must exist, and
/// glossary links must name existing files.
///
/// An unreadable file is reported as `model-toml-syntax` at offset 0.
pub fn load(path: impl AsRef<Path>) -> Result<ContentModel, Vec<Issue>> {
    load_with_file(path, FileId::new(0))
}

/// [`load`], locating issues in `file`.
pub fn load_with_file(path: impl AsRef<Path>, file: FileId) -> Result<ContentModel, Vec<Issue>> {
    let path = path.as_ref();
    // Outside FileSystem: the content model is read before there's a project,
    // since it says where the content root is.
    let text = match std::fs::read(path) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(e) => {
                return Err(vec![syntax_issue(
                    file,
                    format!("the file isn't valid UTF-8 ({e})"),
                )]);
            }
        },
        Err(e) => {
            return Err(vec![syntax_issue(
                file,
                format!("can't read {}: {e}", path.display()),
            )]);
        }
    };
    let dir = path.parent().map(|p| {
        if p.as_os_str().is_empty() {
            Path::new(".")
        } else {
            p
        }
    });
    loader::load(&text, file, dir)
}

/// Validates model text without touching the file system: the filesystem
/// rules (`model-content-root-missing`, and the existence half of
/// `model-glossary-link`) are skipped. For unsaved editor buffers and tests.
pub fn load_str(text: &str, file: FileId) -> Result<ContentModel, Vec<Issue>> {
    loader::load(text, file, None)
}

/// [`load_str`], with the project directory, so the filesystem rules run.
pub fn load_str_in(
    text: &str,
    file: FileId,
    project_dir: &Path,
) -> Result<ContentModel, Vec<Issue>> {
    loader::load(text, file, Some(project_dir))
}

fn syntax_issue(file: FileId, detail: String) -> Issue {
    Issue::new(
        ascribe_core::diagnostics::MODEL_TOML_SYNTAX,
        ascribe_core::Location::new(file, 0..0),
    )
    .with_arg("detail", detail)
}
