//! The message loop and the worker.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;

use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
    Notification as _,
};
use lsp_types::request::{
    CodeActionRequest, CodeLensRequest, Completion, DocumentLinkRequest, ExecuteCommand,
    Formatting, GotoDefinition, HoverRequest, InlayHintRequest, Rename, Request as _,
    SemanticTokensFullRequest, SemanticTokensRangeRequest, WillRenameFiles,
};
use lsp_types::{
    CodeActionProviderCapability, CodeLensOptions, CompletionOptions, DocumentFormattingParams,
    DocumentLinkOptions, ExecuteCommandOptions, GotoDefinitionResponse, HoverProviderCapability,
    InitializeParams, InitializeResult, OneOf, RenameOptions, SemanticTokens,
    SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensRangeParams, SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo,
    ShowDocumentParams, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    Uri, WorkspaceFileOperationsServerCapabilities, WorkspaceServerCapabilities,
};
use tessera_core::{LineIndex, Span};

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

impl tessera_core::Coded for ServeError {
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
                prepare_provider: Some(false),
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
        options,
        idle,
        show_document,
    });
    let worker = {
        let shared = shared.clone();
        thread::spawn(move || worker(&shared))
    };
    {
        let mut core = shared.lock();
        guarded("start", || {
            core.start();
            core.register_watchers();
            if core.has_work() {
                shared.idle.store(false, Ordering::SeqCst);
            }
        });
    }
    shared.wake.notify_all();

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
            }
            Message::Response(_) => {}
        }
    };
    shared.lock().shutdown = true;
    shared.wake.notify_all();
    let _ = worker.join();
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
    folders.iter().map(|p| crate::uri::normalize(p)).collect()
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
            // action (`didSave`, `$/cancelRequest`, `$/setTrace`, …).
            _ => {}
        }
        if core.has_work() {
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
            WillRenameFiles::METHOD => will_rename_request(shared, &request),
            ExecuteCommand::METHOD => execute_command(shared, &request),
            crate::preview::METHOD => preview_request(shared, &request),
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
            .is_some_and(|(config, path)| {
                crate::uri::normalize(config) == crate::uri::normalize(&path)
            });
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
                    shared.idle.store(false, Ordering::SeqCst);
                    break job;
                }
                shared.idle.store(true, Ordering::SeqCst);
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

#[cfg(test)]
mod tests {
    use lsp_server::Connection;
    use tessera_core::Coded;

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
