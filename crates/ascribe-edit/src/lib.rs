//! The edits Ascribe's commands share with its language server: renaming
//! (`rename`), a release (`release`), and retiring a line (`retire_line`),
//! with the edits to `ascribe.toml` (`model_file`), `ascribe.nav`
//! (`navigation`), and the memos (`memos`) they make, and the editor's
//! rename and file moves they're made of (`refactor`).
//!
//! An edit needs no client: it works from a [`nav::Ctx`], which the language
//! server makes for each request and a command loads from the disk
//! ([`project::load`]), and returns each file's text, or its edits, rather
//! than writing. The caller writes the files.

pub mod fsx;
pub mod memos;
pub mod model_file;
pub mod nav;
pub mod nav_file;
pub mod navigation;
pub mod position;
pub mod project;
pub mod refactor;
pub mod release;
pub mod rename;
pub mod retire_line;
pub mod uri;

pub use model_file::{TakenFile, read_taken};
pub use position::Encoding;
pub use release::{
    ChangedOutput, ClearedFile, ClearedMark, MovedFeature, ReleaseError, Released, ReleasedFile,
    Releasing, RewrittenMark, release,
};
pub use rename::{RenameError, Renamed, Renaming, Rewritten, rename};
pub use retire_line::{
    Redirect, RedirectWhy, RetireError, RetiredLine, RetiringLine, Unused, retire_line, unused_in,
};
