//! The Ascribe language server, run as `ascribe lsp`.
//!
//! It keeps a project in memory ([`ascribe_resolve::IncrementalProject`]),
//! follows every change to it (open documents, files changed on disk, the
//! content model), and publishes the diagnostics `ascribe check` reports for
//! the editor's build, as the author types. See the README for the design, the
//! semantic token legend, and how stale results are kept from the editor.

mod code_action;
mod complete;
mod compute;
mod context;
mod core;
mod definition;
mod docs;
mod formatting;
mod fsx;
mod hover;
mod links;
mod log;
mod nav;
mod position;
mod preview;
mod refactor;
mod review;
#[cfg(feature = "json-schema")]
mod schema;
mod server;
mod targets;
mod tokens;
mod uri;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use lsp_server::Connection;

pub use context::{
    AttributePair, ContextNode, ContextParams, ContextProject, ContextResult, ContextToken, Form,
    FrontmatterValue, METHOD as CONTEXT_METHOD, Selection, SelectionKind, VariantArm,
};
pub use position::Encoding;
pub use preview::{
    AssetKind, METHOD as PREVIEW_METHOD, PreviewAsset, PreviewBuild, PreviewLink, PreviewPage,
    PreviewParams, PreviewProblem, PreviewResult, PreviewReview, PreviewSection, ProblemSeverity,
};
pub use review::{
    CHANGES_METHOD as REVIEW_CHANGES_METHOD, ChangesParams, ChangesResult,
    SET_BASE_METHOD as REVIEW_SET_BASE_METHOD, SetBaseParams, SetBaseResult,
};
pub use server::{Exit, ServeError, serve};
pub use targets::{
    AttributeKind, BindingKind, METHOD as TARGETS_METHOD, PrimaryKind, TargetAttribute,
    TargetBuild, TargetDimension, TargetDimensionValue, TargetFeature, TargetFragment,
    TargetHeading, TargetImage, TargetKind, TargetNote, TargetPage, TargetPhrase, TargetRegion,
    TargetSource, TargetSourceFile, TargetWidget, TargetsParams, TargetsResult,
};
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

/// A hook called with what a round is about to publish.
pub type PublishHook = Arc<dyn Fn(&PublishInfo) + Send + Sync>;

/// A hook called with a request's method.
pub type RequestHook = Arc<dyn Fn(&str) + Send + Sync>;

/// Settings for [`serve`]; the default is what `ascribe lsp` uses.
#[derive(Clone, Default)]
pub struct Options {
    /// Called on the worker thread after a round has computed and before it
    /// publishes, so a test can hold the round while newer edits arrive.
    pub before_publish: Option<PublishHook>,
    /// Called with the method of each request before it's handled, so a test
    /// can make a handler panic and see the server survive.
    pub before_request: Option<RequestHook>,
    /// Set to `true` while the worker has nothing queued and isn't computing,
    /// and to `false` as soon as a handler queues work: a test's way to wait
    /// until every result of the messages it sent has been published.
    pub idle: Option<Arc<AtomicBool>>,
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
