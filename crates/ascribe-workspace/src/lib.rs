//! Finding and loading an Ascribe project, once.
//!
//! [`locate`] finds the content model a command works on, and a
//! [`Workspace`] is the project around it, loaded once: the content model and
//! its text, every source file under the content root, indexed, and the side
//! files beside `ascribe.toml`, all read through one
//! `ascribe_resolve::FileSystem`. What it gives out is a
//! [`WorkspaceSnapshot`], which every check and command reads. What loading
//! needs of the checks, the content model's own checks and the list of side
//! files, comes in as a [`Loader`]'s fields, so this crate doesn't depend on
//! `ascribe-check`. It reads, and writes nothing.

mod load;
mod locate;
mod workspace;

pub use load::{LoadError, Loader, ModelFile, ModelLoader, ReadFailure, SourceFile, root_of};
pub use locate::{LocateError, find_below, find_config, locate};
pub use workspace::{SharedFs, Workspace, WorkspaceSnapshot};
