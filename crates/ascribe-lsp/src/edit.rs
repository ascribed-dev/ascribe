//! The `ascribe/edit` request: the edit that performs an action on a page,
//! written in canonical form.
//!
//! A client names an action (`wrapNote`, `insertVariantGroup`, …), the range
//! it was invoked on, and the arguments its wizard collected; the answer is a
//! `WorkspaceEdit` of plain text edits to the page, and the range of any
//! placeholder text it wrote, for the client to leave selected. An action
//! that doesn't apply where it was invoked, or whose arguments the content
//! model doesn't allow, gets a plain-language error instead, never a guess.
//!
//! Each operation finds its target among the nodes `ascribe/context` reports
//! ([`crate::context::found_at`]), so the two never disagree about what is at
//! a position. Every edit is checked before it's returned: the page as it
//! would be is checked as `ascribe check` checks a file, and an edit that
//! would add a diagnostic is refused with the diagnostic's message.
//!
//! The operations are in `edit/blocks.rs` (notes, details, steps, ids,
//! variant arms), `edit/insert.rs` (blocks inserted on a blank line),
//! `edit/inline.rs` (links, phrases, images), `edit/page.rs`
//! (availability and the frontmatter), and `edit/model.rs` (the content
//! model: phrases, glossary terms, and features). An operation on the content
//! model edits `ascribe.toml` in place ([`crate::model_file`]), and may edit
//! other pages too; it's checked as the whole project, with the model as it
//! would be.

mod blocks;
mod inline;
mod insert;
mod model;
mod page;

pub(crate) use model::selection_occurrences;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use ascribe_core::{LineIndex, RelPath, Span, TextEdit, apply_edits};
use ascribe_resolve::FileIndex;
use ascribe_syntax::{Block, BlockKind, Inline, InlineKind, PrimaryValue};
use lsp_types::{Range, TextDocumentIdentifier, Uri, WorkspaceEdit};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::context::{ContextNode, found_at};
use crate::nav::Ctx;

/// The request's method name.
pub const METHOD: &str = "ascribe/edit";

/// The parameters of `ascribe/edit`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditParams {
    /// The page.
    pub text_document: TextDocumentIdentifier,
    /// The cursor or selection the action was invoked on.
    pub range: Range,
    /// The action.
    pub action: EditAction,
    /// What the action needs, by name; each action documents its own.
    #[serde(default)]
    pub args: Map<String, Value>,
    /// The document version the client saw, when the document is open.
    #[serde(default)]
    pub version: Option<i32>,
}

/// The actions `ascribe/edit` performs.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EditAction {
    /// Puts a paragraph, or whole blocks, in a `@note`.
    WrapNote,
    /// Changes a note's type.
    SetNoteType,
    /// Takes a note's content out of it.
    UnwrapNote,
    /// Turns a note into `@details`, keeping its content.
    NoteToDetails,
    /// Puts a block, or whole blocks, in `@details`.
    WrapDetails,
    /// Takes a `@details`' content out of it.
    UnwrapDetails,
    /// Marks an ordered list as steps.
    MakeSteps,
    /// Removes `@steps` from a list.
    RemoveSteps,
    /// Gives a heading an `@id`.
    AddHeadingId,
    /// Inserts a note.
    InsertNote,
    /// Inserts `@steps` and a numbered list.
    InsertSteps,
    /// Inserts a group of `@variant` arms.
    InsertVariantGroup,
    /// Adds an arm to a variant group.
    AddVariantArm,
    /// Removes an arm from a variant group.
    RemoveVariantArm,
    /// Inserts `@details`.
    InsertDetails,
    /// Inserts `@include`.
    InsertInclude,
    /// Inserts `@snippet`.
    InsertSnippet,
    /// Inserts an image.
    InsertImage,
    /// Inserts a project widget.
    InsertWidget,
    /// Marks a section, a block, or a table row with its availability.
    MarkAvailable,
    /// Sets one dimension of the page's `variant`.
    SetPageVariant,
    /// Sets the page's `available`.
    SetPageAvailable,
    /// Links the selected text.
    LinkSelection,
    /// Inserts a link that takes its target's title.
    InsertLink,
    /// Inserts a phrase.
    InsertPhrase,
    /// Changes a link's destination.
    SetLinkTarget,
    /// Empties a link's text, so it takes its target's title.
    UseTargetTitle,
    /// Sets an image's `width`.
    SetImageWidth,
    /// Changes an image's alt text.
    SetImageAlt,
    /// Declares the selected text a phrase and puts the phrase in its place.
    MakePhrase,
    /// Declares a glossary term.
    AddGlossaryTerm,
    /// Changes a feature's availability.
    PromoteFeature,
}

impl EditAction {
    /// Every action, in the order the README documents them.
    pub const ALL: [EditAction; 32] = [
        EditAction::WrapNote,
        EditAction::SetNoteType,
        EditAction::UnwrapNote,
        EditAction::NoteToDetails,
        EditAction::WrapDetails,
        EditAction::UnwrapDetails,
        EditAction::MakeSteps,
        EditAction::RemoveSteps,
        EditAction::AddHeadingId,
        EditAction::InsertNote,
        EditAction::InsertSteps,
        EditAction::InsertVariantGroup,
        EditAction::AddVariantArm,
        EditAction::RemoveVariantArm,
        EditAction::InsertDetails,
        EditAction::InsertInclude,
        EditAction::InsertSnippet,
        EditAction::InsertImage,
        EditAction::InsertWidget,
        EditAction::MarkAvailable,
        EditAction::SetPageVariant,
        EditAction::SetPageAvailable,
        EditAction::LinkSelection,
        EditAction::InsertLink,
        EditAction::InsertPhrase,
        EditAction::SetLinkTarget,
        EditAction::UseTargetTitle,
        EditAction::SetImageWidth,
        EditAction::SetImageAlt,
        EditAction::MakePhrase,
        EditAction::AddGlossaryTerm,
        EditAction::PromoteFeature,
    ];
}

/// The answer to `ascribe/edit`: the edit, or why there is none.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(untagged, rename_all_fields = "camelCase")]
pub enum EditResult {
    /// The edit that performs the action.
    Edit {
        /// Plain text edits under `changes`, in canonical form: to the
        /// requested document, and, for an action on the content model, to
        /// `ascribe.toml` and any other pages it changes.
        #[cfg_attr(
            feature = "json-schema",
            schemars(with = "crate::schema::LspWorkspaceEdit")
        )]
        edit: WorkspaceEdit,
        /// The placeholder text the edit wrote, in the document as it is
        /// after the edit, for the client to leave selected; `null` when it
        /// wrote none.
        #[cfg_attr(
            feature = "json-schema",
            schemars(with = "Option<crate::schema::LspRange>")
        )]
        select: Option<Range>,
    },
    /// Why the action can't be performed: the document changed under it, it
    /// doesn't apply where it was invoked, or an argument is invalid. A
    /// plain-language sentence, for the client to show.
    Failed {
        /// The message.
        error: String,
    },
}

impl Default for EditResult {
    fn default() -> EditResult {
        EditResult::Failed {
            error: "This file isn't a page of the project.".to_owned(),
        }
    }
}

/// The edit for `params`.
pub(crate) fn edit(ctx: &Ctx, uri: &Uri, params: &EditParams) -> EditResult {
    match plan_edit(ctx, uri, params) {
        Ok(result) => result,
        Err(error) => EditResult::Failed { error },
    }
}

fn plan_edit(ctx: &Ctx, uri: &Uri, params: &EditParams) -> Result<EditResult, String> {
    let Some(file) = ctx.file() else {
        return Ok(EditResult::default());
    };
    if let (Some(seen), Some(now)) = (params.version, ctx.version)
        && seen != now
    {
        return Err("The page changed after the action was chosen. Choose it again.".to_owned());
    }
    let index = LineIndex::new(&file.source);
    let source: &str = &file.source;
    let start = ctx
        .encoding
        .offset(&index, params.range.start)
        .ok_or_else(|| "The range isn't in the page.".to_owned())?;
    let end = ctx
        .encoding
        .offset(&index, params.range.end)
        .ok_or_else(|| "The range isn't in the page.".to_owned())?
        .max(start);
    let mut found = found_at(ctx, file, &index, start);
    found.reverse();
    let page = Page {
        ctx,
        file,
        source,
        index: &index,
        start,
        end,
        found,
        args: &params.args,
        nl: if source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        },
    };
    let plan = match params.action {
        EditAction::WrapNote => blocks::wrap_note(&page),
        EditAction::SetNoteType => blocks::set_note_type(&page),
        EditAction::UnwrapNote => blocks::unwrap(&page, Unwrap::Note),
        EditAction::NoteToDetails => blocks::note_to_details(&page),
        EditAction::WrapDetails => blocks::wrap_details(&page),
        EditAction::UnwrapDetails => blocks::unwrap(&page, Unwrap::Details),
        EditAction::MakeSteps => blocks::make_steps(&page),
        EditAction::RemoveSteps => blocks::unwrap(&page, Unwrap::Steps),
        EditAction::AddHeadingId => blocks::add_heading_id(&page),
        EditAction::AddVariantArm => blocks::add_variant_arm(&page),
        EditAction::RemoveVariantArm => blocks::remove_variant_arm(&page),
        EditAction::InsertNote => insert::note(&page),
        EditAction::InsertSteps => insert::steps(&page),
        EditAction::InsertVariantGroup => insert::variant_group(&page),
        EditAction::InsertDetails => insert::details(&page),
        EditAction::InsertInclude => insert::include(&page),
        EditAction::InsertSnippet => insert::snippet(&page),
        EditAction::InsertImage => insert::image(&page),
        EditAction::InsertWidget => insert::widget(&page),
        EditAction::MarkAvailable => page::mark_available(&page),
        EditAction::SetPageVariant => page::set_variant(&page),
        EditAction::SetPageAvailable => page::set_available(&page),
        EditAction::LinkSelection => inline::link_selection(&page),
        EditAction::InsertLink => inline::insert_link(&page),
        EditAction::InsertPhrase => inline::insert_phrase(&page),
        EditAction::SetLinkTarget => inline::set_link_target(&page),
        EditAction::UseTargetTitle => inline::use_target_title(&page),
        EditAction::SetImageWidth => inline::set_image_width(&page),
        EditAction::SetImageAlt => inline::set_image_alt(&page),
        EditAction::MakePhrase => model::make_phrase(&page),
        EditAction::AddGlossaryTerm => model::add_glossary_term(&page),
        EditAction::PromoteFeature => model::promote_feature(&page),
    }?;
    finish(&page, uri, plan)
}

/// The result of a plan: the edits applied, checked, and converted.
// `Uri` hashes by its text; the map is built once and never mutated through
// a key.
#[allow(clippy::mutable_key_type)]
fn finish(page: &Page<'_>, uri: &Uri, plan: Plan) -> Result<EditResult, String> {
    if !plan.model.is_empty() || !plan.files.is_empty() {
        return model::finish(page, uri, plan);
    }
    let Plan {
        mut edits, select, ..
    } = plan;
    // Keep the placeholder's edit findable once the edits are sorted.
    let marked = select.map(|s| (edits[s.edit].span, edits[s.edit].new_text.clone(), s));
    edits.sort_by_key(|e| (e.span.start(), e.span.end()));
    let after = apply_edits(page.source, &edits)
        .map_err(|_| "The action's edits overlap, so it can't be performed here.".to_owned())?;
    check_diagnostics(page, &after)?;
    let new_index = LineIndex::new(&after);
    let select = marked.and_then(|(span, text, s)| {
        let mut shift: isize = 0;
        for e in &edits {
            if e.span == span && e.new_text == text {
                break;
            }
            shift += e.new_text.len().cast_signed() - e.span.len().cast_signed();
        }
        let from = span.start().checked_add_signed(shift)? + s.from;
        Some(
            page.ctx
                .encoding
                .range(&new_index, Span::new(from, from + s.len)),
        )
    });
    let text_edits = edits
        .iter()
        .map(|e| lsp_types::TextEdit {
            range: page.ctx.encoding.range(page.index, e.span),
            new_text: e.new_text.clone(),
        })
        .collect();
    let mut changes = HashMap::new();
    changes.insert(uri.clone(), text_edits);
    Ok(EditResult::Edit {
        edit: WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        },
        select,
    })
}

/// Refuses an edit that would add a diagnostic, as `ascribe check` checks
/// the editor's build: the file-level checks of the page, and the page-level
/// checks of the page and the pages that include it. What's added is
/// compared by code, so a diagnostic that only moves isn't counted.
fn check_diagnostics(page: &Page<'_>, after: &str) -> Result<(), String> {
    let ctx = page.ctx;
    let snapshot = &ctx.snapshot;
    let root = ctx
        .config
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_default();
    let build = ctx.model.editor_default_build();
    let index = snapshot.project();
    // The pages this file is part of, and the pages that link to them or
    // to it: an edit can break a link written elsewhere.
    let mut targets = index.including_pages(&ctx.path);
    if page.file.kind == ascribe_resolve::FileKind::Page {
        targets.push(ctx.path.clone());
    }
    let mut pages: BTreeSet<RelPath> = targets.iter().cloned().collect();
    for target in targets.iter().chain(std::iter::once(&ctx.path)) {
        for link in index.links_to(target) {
            if index
                .file(&link.file)
                .is_some_and(|f| f.kind == ascribe_resolve::FileKind::Page)
            {
                pages.insert(link.file.clone());
            }
            pages.extend(index.including_pages(&link.file));
        }
    }
    let pages: Vec<RelPath> = pages.into_iter().collect();
    let project = |text: Option<&str>| {
        crate::compute::checked_project(
            snapshot,
            root.clone(),
            &ctx.model,
            &ctx.model_text,
            &ctx.fs,
            &text.map(|t| (&ctx.path, t)).into_iter().collect::<Vec<_>>(),
        )
    };
    let file_level = |project: &ascribe_check::Project| {
        project
            .source_at(&ctx.path)
            .map(|file| ascribe_check::check_file(project, file))
            .unwrap_or_default()
    };
    let before_project = project(None);
    let mut before = file_level(&before_project);
    before.extend(
        ascribe_check::PageChecker::with_index(&before_project, index).check_pages(build, &pages),
    );
    let after_project = project(Some(after));
    let own = after_project
        .source_at(&ctx.path)
        .and_then(|f| after_project.file(f.id))
        .map(|f| f.display_path.to_owned());
    let mut after = file_level(&after_project);
    after.extend(ascribe_check::PageChecker::new(&after_project).check_pages(build, &pages));
    match new_problem(
        &before_project,
        &before,
        &after_project,
        &after,
        own.as_deref(),
    ) {
        Some(problem) => Err(problem),
        None => Ok(()),
    }
}

/// The first diagnostic of `after` that `before` doesn't have, as the
/// message refusing the edit. What's added is compared by file and code, so
/// a diagnostic that only moves isn't counted. `own` is the display path of
/// the page the edit was asked for, whose problems don't name it.
pub(crate) fn new_problem(
    before_project: &ascribe_check::Project,
    before: &[ascribe_check::Diagnostic],
    after_project: &ascribe_check::Project,
    after: &[ascribe_check::Diagnostic],
    own: Option<&str>,
) -> Option<String> {
    // Where a diagnostic is, by the file's display path.
    let file_of = |project: &ascribe_check::Project, d: &ascribe_check::Diagnostic| {
        project
            .file(d.location.file)
            .map(|f| f.display_path.to_owned())
            .unwrap_or_default()
    };
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for d in before {
        let key = (file_of(before_project, d), d.code.to_owned());
        *counts.entry(key).or_default() += 1;
    }
    for d in after {
        let file = file_of(after_project, d);
        let count = counts.entry((file.clone(), d.code.to_owned())).or_default();
        if *count == 0 {
            return Some(if own == Some(file.as_str()) || file.is_empty() {
                format!("The edit would make a problem: {}", d.message)
            } else {
                format!("The edit would make a problem in `{file}`: {}", d.message)
            });
        }
        *count -= 1;
    }
    None
}

/// What an operation works from.
pub(crate) struct Page<'a> {
    pub ctx: &'a Ctx,
    pub file: &'a FileIndex,
    pub source: &'a str,
    pub index: &'a LineIndex,
    /// The range, in bytes.
    pub start: usize,
    pub end: usize,
    /// What contains the start of the range, innermost first, as
    /// `ascribe/context` reports it.
    pub found: Vec<(Span, ContextNode)>,
    pub args: &'a Map<String, Value>,
    /// The page's line ending.
    pub nl: &'static str,
}

/// The edits an operation makes.
pub(crate) struct Plan {
    /// To the page.
    pub edits: Vec<TextEdit>,
    pub select: Option<Placeholder>,
    /// To `ascribe.toml`.
    pub model: Vec<crate::model_file::Edit>,
    /// To other source files, by content path.
    pub files: BTreeMap<RelPath, Vec<TextEdit>>,
}

impl Plan {
    pub(crate) fn new(edits: Vec<TextEdit>) -> Plan {
        Plan {
            edits,
            select: None,
            model: Vec::new(),
            files: BTreeMap::new(),
        }
    }
}

/// Placeholder text in one of a plan's edits: `len` bytes from `from` in
/// `edits[edit].new_text`.
#[derive(Clone, Copy)]
pub(crate) struct Placeholder {
    pub edit: usize,
    pub from: usize,
    pub len: usize,
}

/// What `unwrap` takes away.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unwrap {
    Note,
    Details,
    Steps,
}

pub(crate) type Outcome = Result<Plan, String>;

impl<'a> Page<'a> {
    /// The innermost node `pick` accepts, with its span.
    pub(crate) fn innermost<T>(
        &self,
        pick: impl Fn(&ContextNode) -> Option<T>,
    ) -> Option<(Span, T)> {
        self.found
            .iter()
            .find_map(|(span, node)| pick(node).map(|t| (*span, t)))
    }

    /// The block starting at `start`, with its siblings and its index among
    /// them.
    pub(crate) fn block_at(&self, start: usize) -> Option<(&'a [Block], usize)> {
        find_block(&self.file.document.blocks, start)
    }

    /// The inline node with exactly `span`.
    pub(crate) fn inline_at(&self, span: Span) -> Option<&'a Inline> {
        find_inline_in(&self.file.document.blocks, span)
    }

    /// What the next line needs before its text to stay where the text at
    /// `offset` is: the line's prefix up to `offset`, with list markers and
    /// anything else that isn't a block quote's `>` turned into spaces.
    pub(crate) fn prefix(&self, offset: usize) -> String {
        let start = line_start(self.source, offset);
        self.source[start..offset]
            .chars()
            .map(|c| if c == '>' || c == '\t' { c } else { ' ' })
            .collect()
    }

    /// The text of `span`.
    pub(crate) fn text(&self, span: Span) -> &'a str {
        self.source.get(span.range()).unwrap_or_default()
    }

    /// A string argument, if it was given.
    pub(crate) fn opt_str(&self, key: &str) -> Result<Option<String>, String> {
        match self.args.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(Value::Number(n)) => Ok(Some(n.to_string())),
            Some(Value::Bool(b)) => Ok(Some(b.to_string())),
            Some(_) => Err(format!("`{key}` must be text.")),
        }
    }

    /// A string argument that must be given and not be empty.
    pub(crate) fn str_arg(&self, key: &str) -> Result<String, String> {
        match self.opt_str(key)? {
            Some(s) if !s.trim().is_empty() => Ok(s.trim().to_owned()),
            _ => Err(format!("`{key}` is missing.")),
        }
    }

    /// A one-line string argument that must be given.
    pub(crate) fn line_arg(&self, key: &str) -> Result<String, String> {
        let value = self.str_arg(key)?;
        if value.contains(['\n', '\r']) {
            return Err(format!("`{key}` must be one line."));
        }
        Ok(value)
    }

    /// A list of strings.
    pub(crate) fn strings_arg(&self, key: &str) -> Result<Vec<String>, String> {
        match self.args.get(key) {
            Some(Value::Array(items)) => items
                .iter()
                .map(|v| match v {
                    Value::String(s) => Ok(s.clone()),
                    _ => Err(format!("`{key}` must be a list of text.")),
                })
                .collect(),
            Some(Value::String(s)) => Ok(s.split(',').map(|v| v.trim().to_owned()).collect()),
            None | Some(Value::Null) => Err(format!("`{key}` is missing.")),
            Some(_) => Err(format!("`{key}` must be a list of text.")),
        }
    }

    /// An object of attributes, key to value as text, in the order given.
    pub(crate) fn attributes_arg(&self, key: &str) -> Result<Vec<(String, String)>, String> {
        match self.args.get(key) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(Value::Object(map)) => map
                .iter()
                .map(|(k, v)| {
                    let value = match v {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => {
                            return Err(format!(
                                "`{key}.{k}` must be text, a number, or true or false."
                            ));
                        }
                    };
                    if value.contains(['\n', '\r']) {
                        return Err(format!("`{key}.{k}` must be one line."));
                    }
                    Ok((k.clone(), value))
                })
                .collect(),
            Some(_) => Err(format!("`{key}` must be an object of attributes.")),
        }
    }

    /// The parse options of the project, which the formatter's helpers take.
    pub(crate) fn options(&self) -> ascribe_syntax::ParseOptions {
        ascribe_fmt::options_from_model(&self.ctx.model)
    }
}

/// The block whose span starts at `start`, with its siblings and its index.
pub(crate) fn find_block(blocks: &[Block], start: usize) -> Option<(&[Block], usize)> {
    for (i, block) in blocks.iter().enumerate() {
        if block.span.start() == start {
            return Some((blocks, i));
        }
        if !(block.span.start() <= start && start <= block.span.end()) {
            continue;
        }
        let found = match &block.kind {
            BlockKind::BlockQuote(q) => find_block(&q.children, start),
            BlockKind::List(list) => list
                .items
                .iter()
                .find_map(|item| find_block(&item.children, start)),
            BlockKind::Container(c) => find_block(&c.children, start),
            BlockKind::Group(g) => g.arms.iter().find_map(|a| find_block(&a.children, start)),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

/// The inline node with exactly `span`, anywhere in `blocks`.
fn find_inline_in(blocks: &[Block], span: Span) -> Option<&Inline> {
    blocks.iter().find_map(|block| {
        if !block.span.contains_span(span) {
            return None;
        }
        match &block.kind {
            BlockKind::Heading(h) => find_inline(&h.inlines, span),
            BlockKind::Paragraph(p) => find_inline(&p.inlines, span),
            BlockKind::Table(t) => t
                .rows
                .iter()
                .flat_map(|r| &r.cells)
                .find_map(|c| find_inline(&c.inlines, span)),
            BlockKind::Directive(line) => line_inline(line, span),
            BlockKind::BlockQuote(q) => find_inline_in(&q.children, span),
            BlockKind::List(list) => list
                .items
                .iter()
                .find_map(|item| find_inline_in(&item.children, span)),
            BlockKind::Container(c) => {
                line_inline(&c.opener, span).or_else(|| find_inline_in(&c.children, span))
            }
            BlockKind::Group(g) => g.arms.iter().find_map(|a| {
                line_inline(&a.opener, span).or_else(|| find_inline_in(&a.children, span))
            }),
            _ => None,
        }
    })
}

fn line_inline(line: &ascribe_syntax::DirectiveLine, span: Span) -> Option<&Inline> {
    line.title
        .as_ref()
        .and_then(|t| find_inline(&t.inlines, span))
        .or_else(|| match &line.primary {
            Some(PrimaryValue::Text(text)) => find_inline(&text.inlines, span),
            _ => None,
        })
}

fn find_inline(inlines: &[Inline], span: Span) -> Option<&Inline> {
    inlines.iter().find_map(|inline| {
        if inline.span == span {
            return Some(inline);
        }
        if !inline.span.contains_span(span) {
            return None;
        }
        match &inline.kind {
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                find_inline(children, span)
            }
            InlineKind::Link(link) => find_inline(&link.children, span),
            InlineKind::Image(image) => find_inline(&image.children, span),
            _ => None,
        }
    })
}

/// The byte offset of the start of the line holding `offset`.
pub(crate) fn line_start(source: &str, offset: usize) -> usize {
    source[..offset].rfind('\n').map_or(0, |i| i + 1)
}

/// The byte offset of the end of the line holding `offset`, before its line
/// ending.
pub(crate) fn line_end(source: &str, offset: usize) -> usize {
    let end = source[offset..]
        .find('\n')
        .map_or(source.len(), |i| offset + i);
    if end > offset && source[..end].ends_with('\r') {
        end - 1
    } else {
        end
    }
}

/// Whether a line is blank: nothing but whitespace and block quote markers.
pub(crate) fn is_blank(line: &str) -> bool {
    line.chars().all(|c| c.is_whitespace() || c == '>')
}

/// A note's or a details' title line, its `.` and text, with a text that
/// starts with a dot escaped (SPEC §3.7).
pub(crate) fn title_line(title: &str) -> String {
    if title.starts_with('.') {
        format!(".\\{title}")
    } else {
        format!(".{title}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_that_start_with_a_dot_are_escaped() {
        assert_eq!(title_line("Setup"), ".Setup");
        assert_eq!(title_line(".NET"), ".\\.NET");
    }

    #[test]
    fn line_ends_leave_out_a_carriage_return() {
        let text = "one\r\ntwo";
        assert_eq!(line_end(text, 1), 3);
        assert_eq!(line_end(text, 6), text.len());
        assert_eq!(line_start(text, 6), 5);
    }
}
