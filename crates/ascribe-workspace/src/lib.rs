//! Finding and loading an Ascribe project.
//!
//! [`locate`] finds the content model a command works on, and a [`Loader`]
//! reads the project around it: the content model and its text, every
//! source file under the content root, and the side files beside
//! `ascribe.toml`, all through one `ascribe_resolve::FileSystem`. What
//! loading needs of the checks, the content model's own checks and the list
//! of side files, comes in as the [`Loader`]'s fields, so this crate doesn't
//! depend on `ascribe-check`. It reads, and writes nothing.

mod load;
mod locate;

pub use load::{
    LoadError, Loaded, Loader, ModelFile, ModelLoader, ReadFailure, SourceFile, read_side_files,
    read_sources,
};
pub use locate::{LocateError, find_below, find_config, locate};
