//! The Tessera language server, run as `tessera lsp`.
//!
//! It keeps a project in memory ([`tessera_resolve::IncrementalProject`]),
//! follows every change to it (open documents, files changed on disk, the
//! content model), and publishes the diagnostics `tessera check` reports for
//! the editor's build, as the author types. See the README for the design, the
//! semantic token legend, and how stale results are kept from the editor.

mod compute;
mod core;
mod docs;
mod fsx;
mod position;
mod server;
mod tokens;
mod uri;

use std::sync::Arc;

use lsp_server::Connection;

pub use position::Encoding;
pub use server::{Exit, ServeError, serve};
pub use tokens::{MODIFIERS as TOKEN_MODIFIERS, TYPES as TOKEN_TYPES, legend as token_legend};

/// What a round of diagnostics is about to publish, for tests to observe and
/// hold.
#[derive(Clone, Debug)]
pub struct PublishInfo {
    /// The snapshot version the round computed from.
    pub version: u64,
    /// The content paths of the files it computed.
    pub files: Vec<String>,
}

/// Settings for [`serve`]; the default is what `tessera lsp` uses.
#[derive(Clone, Default)]
pub struct Options {
    /// Called on the worker thread after a round has computed and before it
    /// publishes, so a test can hold the round while newer edits arrive.
    pub before_publish: Option<Arc<dyn Fn(&PublishInfo) + Send + Sync>>,
}

/// Runs the server over standard input and output until the client ends the
/// session. Nothing but LSP messages is ever written to standard output;
/// logging goes to standard error.
///
/// # Errors
///
/// The client broke the protocol, or the streams failed.
pub fn run_stdio() -> Result<Exit, Box<dyn std::error::Error + Send + Sync>> {
    let (connection, io_threads) = Connection::stdio();
    let exit = serve(connection, Options::default())?;
    io_threads.join()?;
    Ok(exit)
}
