//! The message loop and the worker.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;

use ascribe_core::path::normalize;
use ascribe_core::{LineIndex, Span};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
    DidSaveTextDocument, Notification as _,
};
use lsp_types::request::{
    CodeActionRequest, CodeLensRequest, Completion, DocumentLinkRequest, ExecuteCommand,
    Formatting, GotoDefinition, HoverRequest, InlayHintRequest, PrepareRenameRequest, References,
    Rename, Request as _, SemanticTokensFullRequest, SemanticTokensRangeRequest, WillRenameFiles,
};
use lsp_types::{
    CodeActionProviderCapability, CodeLensOptions, CompletionOptions, DocumentFormattingParams,
    DocumentLinkOptions, ExecuteCommandOptions, GotoDefinitionResponse, HoverProviderCapability,
    InitializeParams, InitializeResult, OneOf, RenameOptions, SemanticTokens,
    SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensRangeParams, SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo,
    ShowDocumentParams, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions, Uri, WorkspaceFileOperationsServerCapabilities, WorkspaceServerCapabilities,
};

use crate::compute::{Outcome, compute};
use crate::core::Core;
use crate::links::OPEN_FILE;
use crate::nav::Ctx;
use crate::position::Encoding;
use crate::tokens::{legend, semantic_tokens};
use crate::{Options, PublishInfo};

// A handler's panic is caught (`guarded` and the worker below), so one bad
// request doesn't end the server. That needs unwinding: with `panic = "abort"`
// in a profile, the first panic would end the process.
#[cfg(panic = "abort")]
compile_error!(
    "the language server catches a handler's panic, which needs `panic = \"unwind\"`: remove `panic = \"abort\"` from the profile"
);

/// Why the server stopped.
#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    /// The client broke the protocol.
    #[error("protocol error: {0}")]
    Protocol(#[from] lsp_server::ProtocolError),
    /// The client's `initialize` parameters weren't understood.
    #[error("can't read the initialize parameters: {0}")]
    Params(#[from] serde_json::Error),
}

impl ascribe_core::Coded for ServeError {
    fn code(&self) -> &'static str {
        match self {
            ServeError::Protocol(_) => "lsp_protocol",
            ServeError::Params(_) => "lsp_initialize_params",
        }
    }
}

/// How the session ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    /// `shutdown`, then `exit`, or the client closed the connection after
    /// `shutdown`.
    Clean,
    /// `exit` without `shutdown`, or the connection closed without either:
    /// the protocol says the exit code is 1.
    Abrupt,
}

struct Shared {
    core: Mutex<Core>,
    wake: Condvar,
    /// Wakes the thread that runs Vale (`prose.rs`).
    prose_wake: Condvar,
    options: Options,
    idle: Arc<AtomicBool>,
    /// Whether the client can be asked to show a document
    /// (`window/showDocument`), which the open-file command needs.
    show_document: bool,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, Core> {
        // A panic in a handler doesn't stop the server (it's caught), so a
        // poisoned lock is still a usable state.
        self.core.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Runs the server over a connection until the client ends the session.
///
/// # Errors
///
/// The client's `initialize` isn't understood, or the client breaks the
/// protocol before the session starts.
// Longer than the lint allows from before it was on. Split it only while
// changing it for another reason.
#[allow(clippy::too_many_lines)]
pub fn serve(connection: Connection, options: Options) -> Result<Exit, ServeError> {
    let (id, params) = connection.initialize_start()?;
    let params: InitializeParams = serde_json::from_value(params)?;
    let encoding = Encoding::negotiate(
        params
            .capabilities
            .general
            .as_ref()
            .and_then(|g| g.position_encodings.as_deref()),
    );
    let result = InitializeResult {
        capabilities: ServerCapabilities {
            position_encoding: Some(encoding.kind()),
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::INCREMENTAL),
                    // Vale checks a document's prose when it's saved.
                    save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                    ..TextDocumentSyncOptions::default()
                },
            )),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    legend: legend(),
                    range: Some(true),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    work_done_progress_options: Default::default(),
                }),
            ),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(
                    ["@", "{", "(", "#", "/", "=", ",", "|", " "]
                        .map(str::to_owned)
                        .to_vec(),
                ),
                ..CompletionOptions::default()
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            document_link_provider: Some(DocumentLinkOptions {
                resolve_provider: Some(false),
                work_done_progress_options: Default::default(),
            }),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false),
            }),
            inlay_hint_provider: Some(OneOf::Left(true)),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Right(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: Default::default(),
            })),
            workspace: Some(WorkspaceServerCapabilities {
                file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                    will_rename: Some(crate::refactor::file_operation_capability()),
                    ..WorkspaceFileOperationsServerCapabilities::default()
                }),
                ..WorkspaceServerCapabilities::default()
            }),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec![OPEN_FILE.to_owned()],
                work_done_progress_options: Default::default(),
            }),
            ..ServerCapabilities::default()
        },
        server_info: Some(ServerInfo {
            name: "ascribe".to_owned(),
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
        }),
    };
    connection.initialize_finish(id, serde_json::to_value(result)?)?;

    let mut core = Core::new(connection.sender.clone());
    core.encoding = encoding;
    core.can_watch = params
        .capabilities
        .workspace
        .as_ref()
        .and_then(|w| w.did_change_watched_files.as_ref())
        .and_then(|c| c.dynamic_registration)
        .unwrap_or(false);
    core.folders = workspace_folders(&params);
    let idle = options.idle.clone().unwrap_or_default();
    let show_document = params
        .capabilities
        .window
        .as_ref()
        .and_then(|w| w.show_document.as_ref())
        .is_some_and(|s| s.support);
    let shared = Arc::new(Shared {
        core: Mutex::new(core),
        wake: Condvar::new(),
        prose_wake: Condvar::new(),
        options,
        idle,
        show_document,
    });
    let worker = {
        let shared = shared.clone();
        thread::spawn(move || worker(&shared))
    };
    let prose_worker = {
        let shared = shared.clone();
        thread::spawn(move || prose_worker(&shared))
    };
    {
        let mut core = shared.lock();
        guarded("start", || {
            core.start();
            core.register_watchers();
            if core.busy() {
                shared.idle.store(false, Ordering::SeqCst);
            }
        });
    }
    shared.wake.notify_all();
    shared.prose_wake.notify_all();

    let mut shutdown_requested = false;
    let exit = loop {
        let Ok(message) = connection.receiver.recv() else {
            break if shutdown_requested {
                Exit::Clean
            } else {
                Exit::Abrupt
            };
        };
        match message {
            Message::Request(request) => {
                if shutdown_requested {
                    connection
                        .sender
                        .send(
                            Response::new_err(
                                request.id,
                                ErrorCode::InvalidRequest as i32,
                                "the server is shutting down".to_owned(),
                            )
                            .into(),
                        )
                        .ok();
                } else if request.method == "shutdown" {
                    shutdown_requested = true;
                    connection
                        .sender
                        .send(Response::new_ok(request.id, ()).into())
                        .ok();
                } else {
                    let response = handle_request(&shared, request);
                    connection.sender.send(response.into()).ok();
                }
            }
            Message::Notification(notification) => {
                if notification.method == "exit" {
                    break if shutdown_requested {
                        Exit::Clean
                    } else {
                        Exit::Abrupt
                    };
                }
                handle_notification(&shared, notification);
                shared.wake.notify_all();
                shared.prose_wake.notify_all();
            }
            Message::Response(_) => {}
        }
    };
    shared.lock().shutdown = true;
    shared.wake.notify_all();
    shared.prose_wake.notify_all();
    let _ = worker.join();
    // A check running now ends within its time limit.
    let _ = prose_worker.join();
    Ok(exit)
}

fn workspace_folders(params: &InitializeParams) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = params
        .workspace_folders
        .iter()
        .flatten()
        .filter_map(|f| crate::uri::uri_to_path(&f.uri))
        .collect();
    if folders.is_empty() {
        #[allow(deprecated)]
        let root = params.root_uri.clone();
        if let Some(path) = root.as_ref().and_then(crate::uri::uri_to_path) {
            folders.push(path);
        }
    }
    folders.iter().map(|p| normalize(p)).collect()
}

/// Runs a handler, logging a panic instead of letting it end the server.
fn guarded(what: &str, f: impl FnOnce()) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(f)) {
        crate::log::line(format_args!("panic in {what}: {}", panic_message(&*panic)));
    }
}

fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    panic
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "(no message)".to_owned())
}

fn handle_notification(shared: &Shared, notification: Notification) {
    let method = notification.method.clone();
    guarded(&method, || {
        let mut core = shared.lock();
        match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                if let Some(p) = parse::<lsp_types::DidOpenTextDocumentParams>(&notification) {
                    core.did_open(
                        p.text_document.uri,
                        p.text_document.version,
                        p.text_document.text,
                    );
                }
            }
            DidChangeTextDocument::METHOD => {
                if let Some(p) = parse::<lsp_types::DidChangeTextDocumentParams>(&notification) {
                    core.did_change(
                        &p.text_document.uri,
                        p.text_document.version,
                        p.content_changes,
                    );
                }
            }
            DidSaveTextDocument::METHOD => {
                if let Some(p) = parse::<lsp_types::DidSaveTextDocumentParams>(&notification) {
                    core.did_save(&p.text_document.uri);
                }
            }
            DidCloseTextDocument::METHOD => {
                if let Some(p) = parse::<lsp_types::DidCloseTextDocumentParams>(&notification) {
                    core.did_close(&p.text_document.uri);
                }
            }
            DidChangeWatchedFiles::METHOD => {
                if let Some(p) = parse::<lsp_types::DidChangeWatchedFilesParams>(&notification) {
                    core.did_change_watched(p.changes);
                }
            }
            // `initialized` was consumed by the handshake; the rest need no
            // action (`$/cancelRequest`, `$/setTrace`, …).
            _ => {}
        }
        if core.busy() {
            shared.idle.store(false, Ordering::SeqCst);
        }
    });
}

fn parse<T: serde::de::DeserializeOwned>(notification: &Notification) -> Option<T> {
    match serde_json::from_value(notification.params.clone()) {
        Ok(params) => Some(params),
        Err(e) => {
            crate::log::line(format_args!(
                "bad parameters for {}: {e}",
                notification.method
            ));
            None
        }
    }
}

fn handle_request(shared: &Shared, request: Request) -> Response {
    let id = request.id.clone();
    let result = catch_unwind(AssertUnwindSafe(|| {
        if let Some(hook) = &shared.options.before_request {
            hook(&request.method);
        }
        match request.method.as_str() {
            SemanticTokensFullRequest::METHOD => semantic_tokens_request(shared, &request, false),
            SemanticTokensRangeRequest::METHOD => semantic_tokens_request(shared, &request, true),
            Completion::METHOD => navigation(shared, &request, |p: lsp_types::CompletionParams| {
                let at = p.text_document_position;
                (at.text_document.uri, move |ctx: &Ctx| {
                    crate::complete::complete(ctx, at.position)
                })
            }),
            HoverRequest::METHOD => navigation(shared, &request, |p: lsp_types::HoverParams| {
                let at = p.text_document_position_params;
                (at.text_document.uri, move |ctx: &Ctx| {
                    crate::hover::hover(ctx, at.position)
                })
            }),
            GotoDefinition::METHOD => {
                navigation(shared, &request, |p: lsp_types::GotoDefinitionParams| {
                    let at = p.text_document_position_params;
                    (at.text_document.uri, move |ctx: &Ctx| {
                        crate::definition::definition(ctx, at.position)
                            .map(GotoDefinitionResponse::Scalar)
                    })
                })
            }
            References::METHOD => navigation(shared, &request, |p: lsp_types::ReferenceParams| {
                let at = p.text_document_position;
                (at.text_document.uri, move |ctx: &Ctx| {
                    crate::references::references(ctx, at.position, p.context.include_declaration)
                })
            }),
            DocumentLinkRequest::METHOD => {
                navigation(shared, &request, |p: lsp_types::DocumentLinkParams| {
                    (p.text_document.uri, |ctx: &Ctx| {
                        Some(crate::links::document_links(ctx))
                    })
                })
            }
            CodeLensRequest::METHOD => {
                navigation(shared, &request, |p: lsp_types::CodeLensParams| {
                    (p.text_document.uri, |ctx: &Ctx| {
                        Some(crate::links::code_lenses(ctx))
                    })
                })
            }
            InlayHintRequest::METHOD => {
                navigation(shared, &request, |p: lsp_types::InlayHintParams| {
                    (p.text_document.uri, move |ctx: &Ctx| {
                        Some(crate::links::inlay_hints(ctx, p.range))
                    })
                })
            }
            CodeActionRequest::METHOD => {
                navigation(shared, &request, |p: lsp_types::CodeActionParams| {
                    (p.text_document.uri.clone(), move |ctx: &Ctx| {
                        Some(crate::code_action::actions(ctx, p))
                    })
                })
            }
            Formatting::METHOD => navigation(shared, &request, |p: DocumentFormattingParams| {
                (p.text_document.uri.clone(), move |ctx: &Ctx| {
                    Some(crate::formatting::format(ctx, p))
                })
            }),
            Rename::METHOD => rename_request(shared, &request),
            PrepareRenameRequest::METHOD => prepare_rename_request(shared, &request),
            WillRenameFiles::METHOD => will_rename_request(shared, &request),
            ExecuteCommand::METHOD => execute_command(shared, &request),
            crate::preview::METHOD => preview_request(shared, &request),
            crate::build_view::METHOD => {
                answer(shared, &request, |p: crate::build_view::BuildViewParams| {
                    (p.text_document.uri, move |ctx: &Ctx| {
                        crate::build_view::build_view(ctx, p.build.as_deref())
                    })
                })
            }
            crate::context::METHOD => {
                answer(shared, &request, |p: crate::context::ContextParams| {
                    (p.text_document.uri, move |ctx: &Ctx| {
                        crate::context::context(ctx, p.range)
                    })
                })
            }
            crate::edit::METHOD => answer(shared, &request, |p: crate::edit::EditParams| {
                let uri = p.text_document.uri.clone();
                (uri.clone(), move |ctx: &Ctx| {
                    crate::edit::edit(ctx, &uri, &p)
                })
            }),
            crate::targets::METHOD => {
                project_answer(shared, &request, |p: crate::targets::TargetsParams| {
                    (p.text_document.uri, move |ctx: &Ctx| {
                        crate::targets::targets(ctx, &p.kinds, p.range)
                    })
                })
            }
            crate::inventory::METHOD => {
                project_answer(shared, &request, |p: crate::inventory::InventoryParams| {
                    (p.text_document.uri, crate::inventory::inventory)
                })
            }
            crate::agent_prompt::METHOD => agent_prompt_request(shared, &request),
            crate::review::SET_BASE_METHOD => set_base_request(shared, &request),
            crate::review::CHANGES_METHOD => changes_request(shared, &request),
            method => Err(Response::new_err(
                request.id.clone(),
                ErrorCode::MethodNotFound as i32,
                format!("the server doesn't handle {method}"),
            )),
        }
    }));
    match result {
        Ok(Ok(value)) => Response::new_ok(id, value),
        Ok(Err(response)) => response,
        Err(panic) => {
            crate::log::line(format_args!(
                "panic in {}: {}",
                request.method,
                panic_message(&*panic)
            ));
            Response::new_err(
                id,
                ErrorCode::InternalError as i32,
                format!("internal error handling {}", request.method),
            )
        }
    }
}

fn invalid(id: &RequestId, message: String) -> Response {
    Response::new_err(id.clone(), ErrorCode::InvalidParams as i32, message)
}

/// Answers a request about one document from the project as it is now: the
/// snapshot is taken under the lock, and the answer computed without it. A
/// document that isn't a source file of the project has no answer (`null`).
fn navigation<P, R, F>(
    shared: &Shared,
    request: &Request,
    read: impl FnOnce(P) -> (Uri, F),
) -> Result<serde_json::Value, Response>
where
    P: serde::de::DeserializeOwned,
    R: serde::Serialize,
    F: FnOnce(&Ctx) -> Option<R>,
{
    let params: P = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    let (uri, answer) = read(params);
    let target = shared.lock().nav_target(&uri);
    let Some(ctx) = target else {
        return Ok(serde_json::Value::Null);
    };
    match answer(&ctx) {
        Some(value) => serde_json::to_value(value).map_err(|e| {
            Response::new_err(
                request.id.clone(),
                ErrorCode::InternalError as i32,
                e.to_string(),
            )
        }),
        None => Ok(serde_json::Value::Null),
    }
}

/// Answers a custom request about one document as [`navigation`] does, but
/// with the empty answer (`R::default()`), not `null`, for a document that
/// isn't a source file of the project.
fn answer<P, R, F>(
    shared: &Shared,
    request: &Request,
    read: impl FnOnce(P) -> (Uri, F),
) -> Result<serde_json::Value, Response>
where
    P: serde::de::DeserializeOwned,
    R: serde::Serialize + Default,
    F: FnOnce(&Ctx) -> R,
{
    let params: P = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    let (uri, compute) = read(params);
    let target = shared.lock().nav_target(&uri);
    let result = target.map(|ctx| compute(&ctx)).unwrap_or_default();
    to_json(request, result)
}

/// Answers a custom request about the whole project as [`answer`] does,
/// asked through any of the project's files ([`Core::project_target`]).
fn project_answer<P, R, F>(
    shared: &Shared,
    request: &Request,
    read: impl FnOnce(P) -> (Uri, F),
) -> Result<serde_json::Value, Response>
where
    P: serde::de::DeserializeOwned,
    R: serde::Serialize + Default,
    F: FnOnce(&Ctx) -> R,
{
    let params: P = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    let (uri, compute) = read(params);
    let target = shared.lock().project_target(&uri);
    let result = target.map(|ctx| compute(&ctx)).unwrap_or_default();
    to_json(request, result)
}

fn rename_request(shared: &Shared, request: &Request) -> Result<serde_json::Value, Response> {
    let params: lsp_types::RenameParams = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    let uri = params.text_document_position.text_document.uri.clone();
    let context = {
        let core = shared.lock();
        let is_model = core
            .config
            .as_ref()
            .zip(crate::uri::uri_to_path(&uri))
            .is_some_and(|(config, path)| normalize(config) == normalize(&path));
        if is_model {
            core.project_nav_target().and_then(|ctx| {
                crate::refactor::rename_model_key(
                    &ctx,
                    params.text_document_position.position,
                    &params.new_name,
                )
            })
        } else {
            core.nav_target(&uri)
                .and_then(|ctx| crate::refactor::rename(&ctx, params))
        }
    };
    match context {
        Some(edit) => serde_json::to_value(edit).map_err(|e| {
            Response::new_err(
                request.id.clone(),
                ErrorCode::InternalError as i32,
                e.to_string(),
            )
        }),
        None => Ok(serde_json::Value::Null),
    }
}

/// What a rename at a position would rename, or, as an error the client
/// shows, why nothing there can be renamed.
fn prepare_rename_request(
    shared: &Shared,
    request: &Request,
) -> Result<serde_json::Value, Response> {
    let params: lsp_types::TextDocumentPositionParams =
        serde_json::from_value(request.params.clone())
            .map_err(|e| invalid(&request.id, e.to_string()))?;
    let uri = params.text_document.uri;
    let answer = {
        let core = shared.lock();
        let is_model = core
            .config
            .as_ref()
            .zip(crate::uri::uri_to_path(&uri))
            .is_some_and(|(config, path)| normalize(config) == normalize(&path));
        if is_model {
            core.project_nav_target()
                .map(|ctx| crate::refactor::prepare_model(&ctx, params.position))
        } else {
            core.nav_target(&uri)
                .map(|ctx| crate::refactor::prepare(&ctx, params.position))
        }
    };
    match answer {
        None => Ok(serde_json::Value::Null),
        Some(Err(message)) => Err(Response::new_err(
            request.id.clone(),
            ErrorCode::RequestFailed as i32,
            message,
        )),
        Some(Ok(answer)) => serde_json::to_value(answer).map_err(|e| {
            Response::new_err(
                request.id.clone(),
                ErrorCode::InternalError as i32,
                e.to_string(),
            )
        }),
    }
}

fn will_rename_request(shared: &Shared, request: &Request) -> Result<serde_json::Value, Response> {
    let params: lsp_types::RenameFilesParams = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    let edit = shared
        .lock()
        .project_nav_target()
        .and_then(|ctx| crate::refactor::will_rename(&ctx, params))
        .unwrap_or_default();
    serde_json::to_value(edit).map_err(|e| {
        Response::new_err(
            request.id.clone(),
            ErrorCode::InternalError as i32,
            e.to_string(),
        )
    })
}

/// `ascribe.openFile`, the command a CodeLens carries: asks the client to show
/// a file (and a range of it).
fn execute_command(shared: &Shared, request: &Request) -> Result<serde_json::Value, Response> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let params: lsp_types::ExecuteCommandParams = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    if params.command != OPEN_FILE {
        return Err(invalid(
            &request.id,
            format!("unknown command {}", params.command),
        ));
    }
    let uri: Option<Uri> = params
        .arguments
        .first()
        .and_then(|v| v.as_str())
        .and_then(|s| std::str::FromStr::from_str(s).ok());
    let Some(uri) = uri.filter(|u| crate::uri::uri_to_path(u).is_some()) else {
        return Err(invalid(&request.id, "expected a file: URI".to_owned()));
    };
    let selection: Option<lsp_types::Range> = params
        .arguments
        .get(1)
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    if shared.show_document {
        let id = format!("ascribe-show-{}", NEXT.fetch_add(1, Ordering::Relaxed));
        shared.lock().send(Request::new(
            RequestId::from(id),
            "window/showDocument".to_owned(),
            ShowDocumentParams {
                uri,
                external: None,
                take_focus: Some(true),
                selection,
            },
        ));
    }
    Ok(serde_json::Value::Null)
}

fn semantic_tokens_request(
    shared: &Shared,
    request: &Request,
    range: bool,
) -> Result<serde_json::Value, Response> {
    let (uri, range): (Uri, Option<lsp_types::Range>) = if range {
        let p: SemanticTokensRangeParams = serde_json::from_value(request.params.clone())
            .map_err(|e| invalid(&request.id, e.to_string()))?;
        (p.text_document.uri, Some(p.range))
    } else {
        let p: SemanticTokensParams = serde_json::from_value(request.params.clone())
            .map_err(|e| invalid(&request.id, e.to_string()))?;
        (p.text_document.uri, None)
    };
    let (target, encoding) = {
        let core = shared.lock();
        (core.tokens_target(&uri), core.encoding)
    };
    let Some((snapshot, path, model)) = target else {
        return Ok(serde_json::Value::Null);
    };
    let Some(file) = snapshot.file(&path) else {
        return Ok(serde_json::Value::Null);
    };
    let only: Option<Span> = range.map(|range| {
        let index = LineIndex::new(&file.source);
        let start = encoding.offset_lenient(&index, &file.source, range.start);
        let end = encoding.offset_lenient(&index, &file.source, range.end);
        Span::new(start, end.max(start))
    });
    let data = semantic_tokens(file, &model, encoding, only);
    serde_json::to_value(SemanticTokens {
        result_id: None,
        data,
    })
    .map_err(|e| {
        Response::new_err(
            request.id.clone(),
            ErrorCode::InternalError as i32,
            e.to_string(),
        )
    })
}

fn preview_request(shared: &Shared, request: &Request) -> Result<serde_json::Value, Response> {
    let params: crate::preview::PreviewParams = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    let target = shared.lock().preview_target(&params.text_document.uri);
    let result = match target {
        Ok(target) => crate::preview::preview(&target, params.build.as_deref(), params.review),
        Err(result) => *result,
    };
    serde_json::to_value(result).map_err(|e| {
        Response::new_err(
            request.id.clone(),
            ErrorCode::InternalError as i32,
            e.to_string(),
        )
    })
}

/// Answers `ascribe/agentPrompt`. A prompt about a problem, a file, or a
/// page's changes needs a source file of the project; one about the project
/// or a fragment's reach takes any of its files, or none.
fn agent_prompt_request(shared: &Shared, request: &Request) -> Result<serde_json::Value, Response> {
    use crate::agent_prompt::{AgentPromptParams, PromptKind, agent_prompt};
    let params: AgentPromptParams = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    let (target, review) = {
        let core = shared.lock();
        // Any file of the project: a source the index can't read is still
        // one whose problems a prompt is about.
        let target = match (&params.text_document, params.kind) {
            (Some(document), _) => core.project_target(&document.uri),
            (None, PromptKind::Project | PromptKind::FragmentReach) => core
                .config
                .as_deref()
                .and_then(crate::uri::path_to_uri)
                .and_then(|uri| core.project_target(&uri)),
            (None, _) => None,
        };
        (target, core.review.clone())
    };
    let result = target.and_then(|ctx| agent_prompt(&ctx, &params, review));
    to_json(request, result)
}

fn set_base_request(shared: &Shared, request: &Request) -> Result<serde_json::Value, Response> {
    use crate::review::{SetBaseParams, SetBaseResult, info_of, read_base, resolve_base};
    let params: SetBaseParams = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    let result = match params.base {
        // Dropping the base frees it.
        Some(None) => {
            shared.lock().review = None;
            SetBaseResult::default()
        }
        Some(Some(rev)) => set_base(shared, Some(&rev)),
        None => set_base(shared, None),
    };
    fn set_base(shared: &Shared, requested: Option<&str>) -> SetBaseResult {
        let Some(root) = shared.lock().project_root() else {
            return SetBaseResult {
                base: None,
                problem: Some(
                    "There is no project loaded, so there is nothing to review.".to_owned(),
                ),
            };
        };
        // git runs without the lock. The same base again (the preview
        // checking whether it moved) keeps the one read, and what it holds.
        let current = shared.lock().review.clone();
        let read = resolve_base(&root, requested).and_then(|(repo, base)| match current {
            Some(current) if current.info == info_of(&base) => Ok(current),
            _ => read_base(&repo, &base).map(std::sync::Arc::new),
        });
        match read {
            Ok(base) => {
                let info = base.info.clone();
                let mut core = shared.lock();
                if core.project_root().as_deref() != Some(root.as_path()) {
                    return SetBaseResult {
                        base: None,
                        problem: Some(
                            "The project changed while its base was read; start the review again."
                                .to_owned(),
                        ),
                    };
                }
                core.review = Some(base);
                SetBaseResult {
                    base: Some(info),
                    problem: None,
                }
            }
            Err(problem) => SetBaseResult {
                base: None,
                problem: Some(problem.to_string()),
            },
        }
    }
    to_json(request, result)
}

fn changes_request(shared: &Shared, request: &Request) -> Result<serde_json::Value, Response> {
    let params: crate::review::ChangesParams = serde_json::from_value(request.params.clone())
        .map_err(|e| invalid(&request.id, e.to_string()))?;
    let target = shared.lock().changes_target();
    to_json(
        request,
        crate::review::changes(target.as_ref(), params.build.as_deref()),
    )
}

fn to_json(request: &Request, value: impl serde::Serialize) -> Result<serde_json::Value, Response> {
    serde_json::to_value(value).map_err(|e| {
        Response::new_err(
            request.id.clone(),
            ErrorCode::InternalError as i32,
            e.to_string(),
        )
    })
}

/// The worker: takes what is queued, computes it without the lock, and
/// publishes what is still current.
fn worker(shared: &Shared) {
    loop {
        let job = {
            let mut core = shared.lock();
            loop {
                if core.shutdown {
                    return;
                }
                if let Some(job) = core.plan() {
                    core.computing = true;
                    shared.idle.store(false, Ordering::SeqCst);
                    break job;
                }
                core.computing = false;
                shared.idle.store(!core.busy(), Ordering::SeqCst);
                core = shared
                    .wake
                    .wait(core)
                    .unwrap_or_else(PoisonError::into_inner);
            }
        };
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let outcome = compute(&job, &|| job.snapshot.is_current());
            if let (Some(hook), Outcome::Done(results)) = (&shared.options.before_publish, &outcome)
            {
                hook(&PublishInfo {
                    version: job.snapshot.version().get(),
                    files: results.iter().map(|(p, _)| p.to_string()).collect(),
                });
            }
            outcome
        }));
        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(panic) => {
                // The round is dropped, not retried: the same input would
                // panic again. The next change to these files queues them.
                crate::log::line(format_args!(
                    "panic computing diagnostics: {}",
                    panic_message(&*panic)
                ));
                continue;
            }
        };
        let mut core = shared.lock();
        guarded("publishing diagnostics", || core.finish(&job, outcome));
    }
}

/// The thread that runs Vale: takes one document at a time, checks it
/// without the lock, and keeps what's still current (`prose.rs`).
fn prose_worker(shared: &Shared) {
    let program = ascribe_check::prose::Program;
    let linter: &dyn ascribe_check::prose::Linter = match &shared.options.linter {
        Some(linter) => &**linter,
        None => &program,
    };
    loop {
        let job = {
            let mut core = shared.lock();
            loop {
                if core.shutdown {
                    return;
                }
                if let Some(job) = core.take_prose() {
                    shared.idle.store(false, Ordering::SeqCst);
                    break job;
                }
                shared.idle.store(!core.busy(), Ordering::SeqCst);
                core = shared
                    .prose_wake
                    .wait(core)
                    .unwrap_or_else(PoisonError::into_inner);
            }
        };
        let results = match catch_unwind(AssertUnwindSafe(|| job.run(linter))) {
            Ok(results) => results,
            Err(panic) => {
                crate::log::line(format_args!(
                    "panic checking the prose: {}",
                    panic_message(&*panic)
                ));
                Vec::new()
            }
        };
        let mut core = shared.lock();
        guarded("publishing the prose's diagnostics", || {
            core.finish_prose(&job, results);
        });
    }
}

#[cfg(test)]
mod tests {
    use ascribe_core::Coded;
    use lsp_server::Connection;

    /// The one error code a test outside this crate can't reach: lsp-server
    /// makes its protocol errors itself. The CLI's tests list the rest.
    #[test]
    fn protocol_error_code() {
        let (client, server) = Connection::memory();
        drop(client);
        let error = super::serve(server, Default::default()).unwrap_err();
        assert!(matches!(error, super::ServeError::Protocol(_)), "{error}");
        assert_eq!(error.code(), "lsp_protocol");
    }
}
