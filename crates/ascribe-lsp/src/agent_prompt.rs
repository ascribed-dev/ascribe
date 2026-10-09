//! The `ascribe/agentPrompt` request: a prompt for the user's agent about a
//! problem, a file's problems, or the project's, for **Prompt agent**.
//!
//! The answer comes from the current snapshot, so it includes unsaved
//! edits, and from the checks the server runs: the file-level ones and the
//! page-level ones of the editor's build, through
//! `ascribe_check::diagnose_editor_build`, which `ascribe check
//! --editor-build` calls too. The prompt itself is built by
//! `ascribe_check::prompt`, so for a saved file the command line and the
//! editor give the same one.

use ascribe_check::prompt::{self, Builds, Context};
use ascribe_check::{Reported, Scope, diagnose_editor_build};
use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{LineIndex, RelPath};
use lsp_types::{Diagnostic, NumberOrString, TextDocumentIdentifier, Uri};
use serde::{Deserialize, Serialize};

use crate::compute::checked_project;
use crate::nav::Ctx;
use crate::uri::uri_to_path;

/// The request's method name.
pub const METHOD: &str = "ascribe/agentPrompt";

/// What the prompt is about.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PromptKind {
    /// One problem: `diagnostic`, in `textDocument`.
    Problem,
    /// The problems of `textDocument`.
    File,
    /// The problems of the project.
    Project,
}

/// The parameters of `ascribe/agentPrompt`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPromptParams {
    /// What the prompt is about.
    pub kind: PromptKind,
    /// The file: a source file of the project for `problem` and `file`; for
    /// `project`, any file of it, or none for the server's project.
    #[serde(default)]
    pub text_document: Option<TextDocumentIdentifier>,
    /// For `problem`: the diagnostic, as the server published it.
    #[serde(default)]
    pub diagnostic: Option<Diagnostic>,
    /// The documents with unsaved changes, which a prompt about one says to
    /// save first.
    #[serde(default)]
    pub unsaved: Vec<Uri>,
}

/// The answer to `ascribe/agentPrompt`, or `null` when there's no problem to
/// prompt about: the file has none, or the diagnostic is no longer reported.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AgentPromptResult {
    /// The prompt: plain text with Markdown, short enough for an agent's
    /// link to carry.
    pub prompt: String,
}

/// Builds the prompt from the project as `ctx` has it. `ctx.path` is the
/// file asked about, or [`Ctx::PROJECT`] for a project asked about through
/// another file.
pub(crate) fn agent_prompt(ctx: &Ctx, params: &AgentPromptParams) -> Option<AgentPromptResult> {
    let root = normalize(ctx.config.parent()?);
    let project = checked_project(
        &ctx.snapshot,
        root.clone(),
        &ctx.model,
        &ctx.model_text,
        &ctx.fs,
        &[],
    );
    let unsaved = params
        .unsaved
        .iter()
        .filter_map(uri_to_path)
        .filter_map(|path| relative_path(&root, &normalize(&path)))
        .filter(RelPath::is_inside)
        .map(|rel| rel.to_string())
        .collect();
    let build = ctx.model.editor_default_build().name.clone();
    let context = Context::of_project(&root, Builds::Editor(build)).with_unsaved(unsaved);
    let shown_on = |fragment: &RelPath| ctx.snapshot.including_pages(fragment);

    if params.kind == PromptKind::Project {
        let diagnosed = diagnose_editor_build(&project, None);
        let reported = Reported::all(diagnosed.diagnostics);
        let prompt = prompt::project(&project, &context, &[], &reported)?;
        return Some(AgentPromptResult { prompt });
    }

    let file = project.source_at(&ctx.path)?;
    let shown = project.content_root().join(ctx.path.as_str()).ok()?;
    let diagnosed = diagnose_editor_build(&project, Some(std::slice::from_ref(&ctx.path)));
    let reported = Scope::new([shown]).report(&project, diagnosed.diagnostics);
    let prompt = match params.kind {
        PromptKind::File => prompt::file(&project, &context, file.id, &reported, &shown_on)?,
        _ => {
            let asked = params.diagnostic.as_ref()?;
            let index = LineIndex::new(&file.text);
            let same = |r: &&Reported, message: bool| {
                let d = &r.diagnostic;
                d.location.file == file.id
                    && asked.code == Some(NumberOrString::String(d.code.to_owned()))
                    && ctx.encoding.range(&index, d.location.span) == asked.range
                    && (!message || d.message == asked.message)
            };
            // The message names what's wrong, so two problems at one place
            // differ by it; when an edit has changed it since, the place
            // still finds the problem.
            let found = reported
                .iter()
                .find(|r| same(r, true))
                .or_else(|| reported.iter().find(|r| same(r, false)))?;
            prompt::problem(&project, &context, found, &shown_on)
        }
    };
    Some(AgentPromptResult { prompt })
}
