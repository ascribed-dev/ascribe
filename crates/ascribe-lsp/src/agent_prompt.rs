//! The `ascribe/agentPrompt` request: a prompt for the user's agent about a
//! problem, a file's problems, or the project's, or, while review is on,
//! about a changed page or a changed fragment's pages, for **Prompt agent**.
//!
//! The answer comes from the current snapshot, so it includes unsaved
//! edits, and from the checks the server runs: the file-level ones and the
//! page-level ones of the editor's build, through
//! `ascribe_check::diagnose_editor_build`, which `ascribe check
//! --editor-build` calls too; a prompt about one problem that the checks of
//! its file don't find, such as a `page-orphan`, comes from the whole
//! project's. The prompt itself is built by
//! `ascribe_check::prompt`, so for a saved file the command line and the
//! editor give the same one.
//!
//! A prompt about changes compares the snapshot with the review base the
//! server holds (`review.rs`), as `ascribe/review/changes` does, and is built
//! by `ascribe_diff::prompt`, as `ascribe diff --format prompt` builds it.

use std::sync::Arc;

use ascribe_check::prompt::{self, Builds, Context};
use ascribe_check::{Reported, Scope, diagnose_editor_build};
use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{LineIndex, RelPath};
use ascribe_diff::Side;
use ascribe_diff::prompt::Review;
use lsp_types::{Diagnostic, NumberOrString, TextDocumentIdentifier, Uri};
use serde::{Deserialize, Serialize};

use crate::compute::checked_project;
use crate::nav::Ctx;
use crate::review::ReviewBase;
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
    /// What changed on the page `textDocument`, against the review base.
    PageChanges,
    /// The pages that changed through `fragment`, against the review base.
    FragmentReach,
}

/// The parameters of `ascribe/agentPrompt`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPromptParams {
    /// What the prompt is about.
    pub kind: PromptKind,
    /// The file: a source file of the project for `problem` and `file`, and
    /// the page for `pageChanges`; for `project` and `fragmentReach`, any file
    /// of it, or, for `project`, none for the server's project.
    #[serde(default)]
    pub text_document: Option<TextDocumentIdentifier>,
    /// For `problem`: the diagnostic, as the server published it.
    #[serde(default)]
    pub diagnostic: Option<Diagnostic>,
    /// The documents with unsaved changes, which a prompt about one says to
    /// save first.
    #[serde(default)]
    pub unsaved: Vec<Uri>,
    /// For `pageChanges` and `fragmentReach`: the build whose pages are
    /// compared. Without one, the editor's build (`[editor] build`).
    #[serde(default)]
    pub build: Option<String>,
    /// For `fragmentReach`: the fragment, by content path, as `ascribe diff`
    /// names it in a page's `because`.
    #[serde(default)]
    pub fragment: Option<String>,
}

/// The answer to `ascribe/agentPrompt`, or `null` when there's nothing to
/// prompt about: the file has no problem, the diagnostic is no longer
/// reported, review is off, or the page or fragment didn't change.
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
/// another file. `review` is the review base, while review is on.
pub(crate) fn agent_prompt(
    ctx: &Ctx,
    params: &AgentPromptParams,
    review: Option<Arc<ReviewBase>>,
) -> Option<AgentPromptResult> {
    let root = normalize(ctx.config.parent()?);
    if matches!(
        params.kind,
        PromptKind::PageChanges | PromptKind::FragmentReach
    ) {
        return change_prompt(ctx, params, &root, review.as_deref()?);
    }
    let project = checked_project(
        &ctx.snapshot,
        root.clone(),
        &ctx.model,
        &ctx.model_text,
        &ctx.fs,
        &[],
    );
    let unsaved = unsaved_paths(&root, params);
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
    let reported = Scope::new([shown.clone()]).report(&project, diagnosed.diagnostics);
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
                .or_else(|| reported.iter().find(|r| same(r, false)));
            if let Some(found) = found {
                return Some(AgentPromptResult {
                    prompt: prompt::problem(&project, &context, found, &shown_on),
                });
            }
            // The content checks across the project aren't run for one
            // file; they need every page.
            let whole = diagnose_editor_build(&project, None);
            let reported = Scope::new([shown]).report(&project, whole.diagnostics);
            let found = reported
                .iter()
                .find(|r| same(r, true))
                .or_else(|| reported.iter().find(|r| same(r, false)))?;
            prompt::problem(&project, &context, found, &shown_on)
        }
    };
    Some(AgentPromptResult { prompt })
}

/// The documents with unsaved changes, relative to the project root.
fn unsaved_paths(root: &std::path::Path, params: &AgentPromptParams) -> Vec<String> {
    params
        .unsaved
        .iter()
        .filter_map(uri_to_path)
        .filter_map(|path| relative_path(root, &normalize(&path)))
        .filter(RelPath::is_inside)
        .map(|rel| rel.to_string())
        .collect()
}

/// A prompt about changes: the page `ctx.path`'s, or the reach of
/// `params.fragment`, in the build asked for, against `review`.
fn change_prompt(
    ctx: &Ctx,
    params: &AgentPromptParams,
    root: &std::path::Path,
    review: &ReviewBase,
) -> Option<AgentPromptResult> {
    let build = match &params.build {
        Some(name) => ctx.model.build(name)?,
        None => ctx.model.editor_default_build(),
    };
    let context = Context::of_project(root, Builds::Named(vec![build.name.clone()]))
        .with_unsaved(unsaved_paths(root, params));
    let now = Side {
        project: ctx.snapshot.project(),
        model_text: &ctx.model_text,
    };
    let shown = Review {
        base: &review.info,
        content_root: &ctx.snapshot.project().layout().content_root,
    };
    let prompt = if params.kind == PromptKind::FragmentReach {
        let fragment = RelPath::parse(params.fragment.as_deref()?).ok()?;
        let diffs = ascribe_diff::compare_builds(review.side(), now, &[build.name.as_str()]);
        ascribe_diff::prompt::fragment_reach(&context, &shown, diffs.first()?, &fragment)?
    } else {
        let page = ascribe_diff::compare_page_in(review.side(), now, &build.name, &ctx.path)?;
        ascribe_diff::prompt::page(&context, &shown, &build.name, &page)
    };
    Some(AgentPromptResult { prompt })
}
