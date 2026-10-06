//! Errors an emitter can return.

use crate::store::StoreError;

/// Why a build's output couldn't be produced.
#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    /// A file the output needs (an asset) couldn't be read.
    #[error("can't read {path}: {message}")]
    Read {
        /// The file.
        path: String,
        /// What went wrong.
        message: String,
    },
    /// A page couldn't be turned into this output's form.
    #[error("can't render {page}: {message}")]
    Render {
        /// The page's content path.
        page: String,
        /// What went wrong.
        message: String,
    },
    /// The build can't be written in this output's form, whatever the page.
    #[error("{message}")]
    Invalid {
        /// What's wrong.
        message: String,
    },
    /// Replacing the previous output failed.
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl ascribe_core::Coded for EmitError {
    fn code(&self) -> &'static str {
        match self {
            EmitError::Read { .. } => "asset_unreadable",
            EmitError::Render { .. } => "render_failed",
            EmitError::Invalid { .. } => "build_invalid",
            EmitError::Store(e) => ascribe_core::Coded::code(e),
        }
    }
}
