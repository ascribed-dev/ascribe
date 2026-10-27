//! The Ascribe language server, run as `ascribe lsp`.
//!
//! It keeps a project in memory ([`ascribe_resolve::IncrementalProject`]),
//! follows every change to it (open documents, files changed on disk, the
//! content model), and publishes the diagnostics `ascribe check` reports for
//! the editor's build, as the author types. See the README for the design, the
//! semantic token legend, and how stale results are kept from the editor.

mod addresses;
mod agent_prompt;
mod baseline;
mod build_view;
mod code_action;
mod complete;
mod compute;
mod context;
mod core;
mod definition;
mod docs;
mod edit;
mod examples;
mod folding;
mod formatting;
mod headless;
mod hover;
mod intended;
mod inventory;
mod links;
mod log;
mod memos;
mod nav;
mod page;
mod page_rows;
mod preview;
mod project_edit;
mod prose;
mod references;
mod review;
#[cfg(feature = "json-schema")]
mod schema;
mod search;
mod server;
mod settings;
mod shown;
mod summary;
mod targets;
mod tokens;

use std::sync::Arc;

// The edits the server shares with the commands, under the names the
// server's modules use for them.
use ascribe_edit::{fsx, model_file, nav_file, position, refactor, uri};
use std::sync::atomic::AtomicBool;

use lsp_server::Connection;

pub use addresses::{AddressesParams, NOTIFICATION as ADDRESSES_NOTIFICATION, PageOldAddress};
pub use agent_prompt::{
    AgentPromptParams, AgentPromptResult, METHOD as AGENT_PROMPT_METHOD, PromptKind,
};
pub use ascribe_edit::Encoding;
pub use build_view::{
    BuildViewParams, BuildViewResult, Excluded, ExclusionReason, METHOD as BUILD_VIEW_METHOD,
};
pub use context::{
    AttributePair, ContextNode, ContextParams, ContextProject, ContextResult, ContextToken, Form,
    FrontmatterValue, METHOD as CONTEXT_METHOD, Selection, SelectionKind, VariantArm,
};
pub use edit::{EditAction, EditParams, EditResult, METHOD as EDIT_METHOD};
pub use headless::{Problem, Watched};
pub use inventory::{
    InventoryEntry, InventoryFragment, InventoryPage, InventoryParams, InventoryResult,
    InventoryValue, METHOD as INVENTORY_METHOD, ModelKind,
};
pub use page::{
    METHOD as PAGE_METHOD, PageAbout, PageHeading, PageMemo, PageParams, PagePlace, PageResult,
};
pub use page_rows::{
    ExampleState, IncludeKind, LeftOut, PageBuild, PageDescription, PageExample, PageHealth,
    PageInclude, PageListing, PagePermalink, PageRecheck, PageSize, PageTemplate,
};
pub use preview::{
    AssetKind, METHOD as PREVIEW_METHOD, PreviewAsset, PreviewBuild, PreviewLink, PreviewPage,
    PreviewParams, PreviewProblem, PreviewResult, PreviewReview, PreviewSection, ProblemSeverity,
};
pub use project_edit::{
    METHOD as PROJECT_EDIT_METHOD, ProjectEditKind, ProjectEditParams, ProjectEditResult,
};
pub use review::{
    BASE_FILE_METHOD as REVIEW_BASE_FILE_METHOD, BaseFileParams, BaseFileResult,
    CHANGES_METHOD as REVIEW_CHANGES_METHOD, ChangesParams, ChangesResult,
    SET_BASE_METHOD as REVIEW_SET_BASE_METHOD, SetBaseParams, SetBaseResult,
};
pub use search::{
    METHOD as SEARCH_METHOD, SearchIn, SearchPage, SearchParams, SearchPlace, SearchResult,
    SearchThrough,
};
pub use server::{Exit, ServeError, serve};
pub use settings::{
    CheckEntry, CheckLimit, METHOD as SETTINGS_METHOD, SettingEntry, SettingPiece, SettingPlace,
    SettingProblem, SettingValue, SettingsParams, SettingsResult,
};
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

/// A hook called with the commit of a base whose addresses were listed.
pub type BaseListedHook = Arc<dyn Fn(&str) + Send + Sync>;

/// What gives the day the checks run on, asked at each round: the server
/// doesn't read the clock itself.
pub type Clock = Arc<dyn Fn() -> Option<ascribe_core::Date> + Send + Sync>;

/// Settings for [`serve`]. `ascribe lsp` sets only [`Options::today`] and
/// [`Options::version`].
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
    /// The day the checks run on, which `recheck-overdue` compares a page's
    /// recheck date with. Without one, no recheck is overdue.
    pub today: Option<Clock>,
    /// What checks the prose on open and on save, for a project with
    /// `[checks.vale]`: Vale itself when `None`, or a test's stand-in.
    pub linter: Option<Arc<dyn ascribe_report::prose::Linter>>,
    /// Called with the commit each time the server lists a base's
    /// addresses (`addresses.rs`), so a test can count the listings.
    pub base_listed: Option<BaseListedHook>,
    /// The version the server gives in its `serverInfo` and first log line:
    /// what `ascribe --version` prints, such as `0.4.1+dev (7ae1340440fc)`.
    /// The crate's version when `None`.
    pub version: Option<String>,
}

/// Runs the server over standard input and output until the client ends the
/// session. Nothing but LSP messages is ever written to standard output;
/// logging goes to standard error.
///
/// # Errors
///
/// The client broke the protocol, or the streams failed.
pub fn run_stdio(options: Options) -> Result<Exit, Box<dyn std::error::Error + Send + Sync>> {
    let (connection, io_threads) = Connection::stdio();
    let exit = serve(connection, options)?;
    io_threads.join()?;
    Ok(exit)
}
