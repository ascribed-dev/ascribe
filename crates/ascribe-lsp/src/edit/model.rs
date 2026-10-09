//! The content model: making the selected text a phrase, declaring a
//! glossary term, and changing a feature's availability. Each edits
//! `ascribe.toml` in place ([`ModelFile`]); making a phrase edits pages too.
//!
//! What these edits make is checked as a whole: `ascribe.toml` as it would
//! be must load, and the project, checked with that model and those pages,
//! must have no diagnostic it didn't have before.

use std::collections::{BTreeMap, HashMap};

use ascribe_core::availability::parse_availability;
use ascribe_core::{FileId, LineIndex, RelPath, Span, TextEdit, apply_edits};
use ascribe_resolve::FileIndex;
use ascribe_syntax::{Block, BlockKind, DirectiveLine, Inline, InlineKind, LinkForm, PrimaryValue};
use lsp_types::{Uri, WorkspaceEdit};
use serde_json::Value as Json;
use toml_edit::Value;

use super::{EditResult, Outcome, Page, Plan};
use crate::context::SelectionKind;
use crate::model_file::{ModelFile, path};
use crate::nav::{Ctx, Lines};

pub(crate) fn make_phrase(page: &Page<'_>) -> Outcome {
    let ctx = page.ctx;
    let (span, value) = phrase_selection(ctx, page.file, page.start, page.end)?;
    let key = page.line_arg("key")?;
    if !ascribe_model::is_key(&key) {
        return Err(format!(
            "`{key}` can't be a phrase key: {}.",
            ascribe_model::KEY_RULE
        ));
    }
    if let Some(existing) = ctx.model.phrase(&key) {
        return Err(format!("`{key}` is already a phrase, for “{existing}”."));
    }
    let everywhere = match page.args.get("everywhere") {
        None | Some(Json::Null) => false,
        Some(Json::Bool(b)) => *b,
        Some(_) => return Err("`everywhere` must be true or false.".to_owned()),
    };
    let file = model_file(ctx)?;
    let phrase = format!("{{{key}}}");
    let mut plan = Plan::new(vec![TextEdit::replace(span, phrase.clone())]);
    plan.model
        .push(file.add_entry(&["phrases"], &key, Value::from(value.as_str()))?);
    if everywhere {
        for (path, at) in occurrences(ctx, &value, Some((&ctx.path, span))) {
            let edit = TextEdit::replace(at, phrase.clone());
            if path == ctx.path {
                plan.edits.push(edit);
            } else {
                plan.files.entry(path).or_default().push(edit);
            }
        }
    }
    Ok(plan)
}

pub(crate) fn add_glossary_term(page: &Page<'_>) -> Outcome {
    let ctx = page.ctx;
    let id = page.line_arg("id")?;
    if !ascribe_model::is_key(&id) {
        return Err(format!(
            "`{id}` can't be a term's id: {}.",
            ascribe_model::KEY_RULE
        ));
    }
    if ctx.model.glossary.terms.iter().any(|t| t.id == id) {
        return Err(format!(
            "The glossary already has a term with the id `{id}`."
        ));
    }
    let term = page.line_arg("term")?;
    let aliases: Vec<String> = match page.args.get("aliases") {
        None | Some(Json::Null) => Vec::new(),
        Some(_) => page.strings_arg("aliases")?,
    }
    .into_iter()
    .map(|a| a.trim().to_owned())
    .filter(|a| !a.is_empty())
    .collect();
    if aliases.iter().any(|a| a.contains(['\n', '\r'])) {
        return Err("Each alias must be one line.".to_owned());
    }
    let definition = page.line_arg("definition")?;
    let link = page
        .opt_str("link")?
        .map(|l| l.trim().to_owned())
        .filter(|l| !l.is_empty());
    if let Some(link) = &link {
        check_glossary_link(ctx, link)?;
    }
    let mut entries = vec![("term", Value::from(term.as_str()))];
    if !aliases.is_empty() {
        entries.push((
            "aliases",
            Value::Array(aliases.iter().map(String::as_str).collect()),
        ));
    }
    entries.push(("definition", Value::from(definition.as_str())));
    if let Some(link) = &link {
        entries.push(("link", Value::from(link.as_str())));
    }
    let file = model_file(ctx)?;
    let mut plan = Plan::new(Vec::new());
    plan.model
        .push(file.add_table(&["glossary", "terms", &id], &entries, &["glossary"])?);
    Ok(plan)
}

/// A glossary term's link names a heading its page has: the content model
/// checks the page, but nothing else checks the id.
fn check_glossary_link(ctx: &Ctx, link: &str) -> Result<(), String> {
    if link.contains(['\n', '\r']) {
        return Err("`link` must be one line.".to_owned());
    }
    let (page, id) = link.split_once('#').unwrap_or((link, ""));
    let page = page.strip_prefix('/').unwrap_or(page);
    let Some(file) = RelPath::parse(page)
        .ok()
        .and_then(|p| ctx.snapshot.file(&p))
    else {
        // The content model reports a page that isn't there.
        return Ok(());
    };
    if !id.is_empty()
        && !crate::nav::link_headings(&ctx.snapshot, file)
            .iter()
            .any(|h| h.source_id == id)
    {
        return Err(format!("`{page}` has no heading with the id `{id}`."));
    }
    Ok(())
}

pub(crate) fn promote_feature(page: &Page<'_>) -> Outcome {
    let ctx = page.ctx;
    let key = page.line_arg("key")?;
    let feature = ctx.model.feature(&key).ok_or_else(|| {
        let keys: Vec<String> = ctx
            .model
            .features
            .iter()
            .map(|f| format!("`{}`", f.key))
            .collect();
        if keys.is_empty() {
            "The content model declares no features.".to_owned()
        } else {
            format!(
                "`{key}` isn't a feature. The features are {}.",
                keys.join(", ")
            )
        }
    })?;
    let spec = page.line_arg("spec")?;
    if let Err(e) = parse_availability(&spec, 0) {
        return Err(format!("`{spec}` isn't an availability spec: {e}."));
    }
    if spec == feature.available_text.trim() {
        return Err(format!("`{key}` is already available as `{spec}`."));
    }
    let file = model_file(ctx)?;
    let mut plan = Plan::new(Vec::new());
    plan.model.push(file.replace_value(
        &path(&["features", &key, "available"]),
        Value::from(spec.as_str()),
    )?);
    Ok(plan)
}

/// `ascribe.toml`, parsed, when it's the model the project has now.
fn model_file(ctx: &Ctx) -> Result<ModelFile<'_>, String> {
    if ctx.model_problem {
        return Err("`ascribe.toml` has a problem. Fix it, then try again.".to_owned());
    }
    ModelFile::parse(&ctx.model_text)
}

/// The selected text a phrase can take the place of: inside one run of
/// plain text in prose, on one line, written as it reads (no escapes or
/// entities). Its span, without whitespace at either end, and its text.
pub(crate) fn phrase_selection(
    ctx: &Ctx,
    file: &FileIndex,
    start: usize,
    end: usize,
) -> Result<(Span, String), String> {
    if start == end {
        return Err("Select the text to make a phrase.".to_owned());
    }
    let (kind, _) = crate::context::selection(file, start, end);
    if kind != SelectionKind::Prose {
        return Err("Select text inside one paragraph or heading.".to_owned());
    }
    let source: &str = &file.source;
    let text = source.get(start..end).unwrap_or_default();
    let s = start + (text.len() - text.trim_start().len());
    let e = end - (text.len() - text.trim_end().len());
    let span = Span::new(s, e);
    let mut inside = false;
    prose_text(ctx, file, &mut |text, _| {
        inside |= text.contains_span(span);
    });
    let value = source.get(span.range()).unwrap_or_default();
    if !inside || value.contains(['\n', '\r', '\\', '&']) {
        return Err(
            "Select plain text on one line, without formatting, links, or escapes.".to_owned(),
        );
    }
    Ok((span, value.to_owned()))
}

/// Every whole-word occurrence of `text` in the project's prose, but not
/// `except`: what making it a phrase "everywhere" replaces. Prose is the text
/// of paragraphs, table cells, links, titles, and text primaries, and of
/// headings that have an `@id` (another heading's id could change); never
/// code, destinations, or alt text. An occurrence right after a backslash or
/// between braces is left alone, since the phrase would read differently
/// there, and so is one in a URL written as text.
pub(crate) fn occurrences(
    ctx: &Ctx,
    text: &str,
    except: Option<(&RelPath, Span)>,
) -> Vec<(RelPath, Span)> {
    let mut out = Vec::new();
    if text.is_empty() {
        return out;
    }
    let word = |c: char| c.is_alphanumeric() || c == '_';
    for file in ctx.snapshot.files() {
        let source: &str = &file.source;
        prose_text(ctx, file, &mut |span, heading_without_id| {
            if heading_without_id {
                return;
            }
            let Some(run) = source.get(span.range()) else {
                return;
            };
            for (i, _) in run.match_indices(text) {
                let at = Span::new(span.start() + i, span.start() + i + text.len());
                let before = source[..at.start()].chars().next_back();
                let after = source[at.end()..].chars().next();
                let cut = |edge: Option<char>, end: Option<char>| {
                    end.is_some_and(word) && edge.is_some_and(word)
                };
                if cut(before, text.chars().next())
                    || cut(after, text.chars().next_back())
                    || matches!(before, Some('\\' | '{'))
                    || after == Some('}')
                    || in_url(run, i, i + text.len())
                {
                    continue;
                }
                if except.is_some_and(|(p, s)| {
                    *p == file.path && s.start() < at.end() && at.start() < s.end()
                }) {
                    continue;
                }
                out.push((file.path.clone(), at));
            }
        });
    }
    out
}

/// Whether `run[start..end]` is part of a URL written as plain text, such as
/// `https://example.com/Quill`: the run of non-space characters around it
/// has `://` in it.
fn in_url(run: &str, start: usize, end: usize) -> bool {
    let from = run[..start].rfind(char::is_whitespace).map_or(0, |i| i + 1);
    let to = run[end..]
        .find(char::is_whitespace)
        .map_or(run.len(), |i| end + i);
    run[from..to].contains("://")
}

/// Calls `f` with the span of each run of plain text in a file's prose, and
/// whether it's in a heading with no `@id`. A link's text counts only when
/// it's just text: not the label a shortcut or collapsed reference link is
/// looked up by, and not the destination an autolink or a bare URL shows.
fn prose_text(_ctx: &Ctx, file: &FileIndex, f: &mut dyn FnMut(Span, bool)) {
    fn inlines(source: &str, list: &[Inline], heading: bool, f: &mut dyn FnMut(Span, bool)) {
        for inline in list {
            match &inline.kind {
                InlineKind::Text(_) => f(inline.span, heading),
                InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                    inlines(source, children, heading, f);
                }
                InlineKind::Link(link) => {
                    let bracketed = source
                        .get(inline.span.range())
                        .is_some_and(|text| text.starts_with('['));
                    if bracketed && matches!(link.form, LinkForm::Inline | LinkForm::Full) {
                        inlines(source, &link.children, heading, f);
                    }
                }
                _ => {}
            }
        }
    }
    fn line(source: &str, line: &DirectiveLine, f: &mut dyn FnMut(Span, bool)) {
        if let Some(title) = &line.title {
            inlines(source, &title.inlines, false, f);
        }
        if let Some(PrimaryValue::Text(text)) = &line.primary {
            inlines(source, &text.inlines, false, f);
        }
    }
    fn blocks(file: &FileIndex, list: &[Block], f: &mut dyn FnMut(Span, bool)) {
        let source: &str = &file.source;
        for block in list {
            match &block.kind {
                BlockKind::Paragraph(p) => inlines(source, &p.inlines, false, f),
                BlockKind::Heading(h) => {
                    let has_id = file
                        .headings
                        .iter()
                        .any(|x| x.span == block.span && x.explicit_id.is_some());
                    inlines(source, &h.inlines, !has_id, f);
                }
                BlockKind::Table(t) => {
                    for cell in t.rows.iter().flat_map(|r| &r.cells) {
                        inlines(source, &cell.inlines, false, f);
                    }
                }
                BlockKind::Directive(l) => line(source, l, f),
                BlockKind::BlockQuote(q) => blocks(file, &q.children, f),
                BlockKind::List(l) => {
                    for item in &l.items {
                        blocks(file, &item.children, f);
                    }
                }
                BlockKind::Container(c) => {
                    line(source, &c.opener, f);
                    blocks(file, &c.children, f);
                }
                BlockKind::Group(g) => {
                    for arm in &g.arms {
                        line(source, &arm.opener, f);
                        blocks(file, &arm.children, f);
                    }
                }
                _ => {}
            }
        }
    }
    blocks(file, &file.document.blocks, f);
}

/// The result of a plan that changes the content model: the model as it
/// would be must load, and the project must have no new diagnostic with it.
// `Uri` hashes by its text; the map is built once and never mutated through
// a key.
#[allow(clippy::mutable_key_type)]
pub(super) fn finish(page: &Page<'_>, uri: &Uri, plan: Plan) -> Result<EditResult, String> {
    let ctx = page.ctx;
    let file = model_file(ctx)?;
    let (model_edits, model_text) = file.finish(plan.model)?;
    let root = ctx
        .config
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_default();
    let model =
        ascribe_model::load_str_in(&model_text, FileId::new(0), &root).map_err(|issues| {
            let problem = issues
                .iter()
                .map(ascribe_check::Diagnostic::from_issue)
                .find(|d| d.severity == ascribe_check::Severity::Error)
                .map_or_else(|| "it wouldn't load".to_owned(), |d| d.message);
            format!("The change would make a problem in `ascribe.toml`: {problem}")
        })?;

    let mut page_edits = plan.edits;
    page_edits.sort_by_key(|e| (e.span.start(), e.span.end()));
    let overlap = || "The action's edits overlap, so it can't be performed here.".to_owned();
    let mut texts: Vec<(RelPath, String)> = Vec::new();
    if !page_edits.is_empty() {
        texts.push((
            ctx.path.clone(),
            apply_edits(page.source, &page_edits).map_err(|_| overlap())?,
        ));
    }
    let mut files = plan.files;
    for (path, edits) in &mut files {
        edits.sort_by_key(|e| (e.span.start(), e.span.end()));
        let source = ctx.snapshot.file(path).ok_or_else(overlap)?;
        texts.push((
            path.clone(),
            apply_edits(&source.source, edits).map_err(|_| overlap())?,
        ));
    }
    let replaced: Vec<(&RelPath, &str)> = texts.iter().map(|(p, t)| (p, t.as_str())).collect();
    check_project(ctx, &root, &model, &model_text, &replaced)?;

    let mut changes = HashMap::new();
    let convert = |index: &LineIndex, edits: &[TextEdit]| -> Vec<lsp_types::TextEdit> {
        edits
            .iter()
            .map(|e| lsp_types::TextEdit {
                range: ctx.encoding.range(index, e.span),
                new_text: e.new_text.clone(),
            })
            .collect()
    };
    if !page_edits.is_empty() {
        changes.insert(uri.clone(), convert(page.index, &page_edits));
    }
    let mut lines = Lines::new(ctx);
    for (path, edits) in files {
        let file_uri = ctx.uri_of(&path).ok_or_else(overlap)?;
        let converted = edits
            .iter()
            .map(|e| {
                Some(lsp_types::TextEdit {
                    range: lines.range(&path, e.span)?,
                    new_text: e.new_text.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(overlap)?;
        changes.insert(file_uri, converted);
    }
    let model_uri = crate::uri::path_to_uri(&ctx.config)
        .ok_or_else(|| "`ascribe.toml` can't be named as a URI.".to_owned())?;
    changes.insert(
        model_uri,
        convert(&LineIndex::new(&ctx.model_text), &model_edits),
    );
    Ok(EditResult::Edit {
        edit: WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        },
        select: None,
    })
}

/// Refuses a change to the content model, and to some pages, that would add
/// a diagnostic anywhere in the project, as `ascribe check` checks the
/// editor's build.
fn check_project(
    ctx: &Ctx,
    root: &std::path::Path,
    model: &ascribe_model::ContentModel,
    model_text: &str,
    replaced: &[(&RelPath, &str)],
) -> Result<(), String> {
    let project = |model, model_text, replaced| {
        crate::compute::checked_project(
            &ctx.snapshot,
            root.to_path_buf(),
            model,
            model_text,
            &ctx.fs,
            replaced,
        )
    };
    let before = project(&ctx.model, &ctx.model_text, &[]);
    let after = project(model, model_text, replaced);
    let diagnostics = |p: &ascribe_check::Project| {
        ascribe_check::check_project(p, p.model().editor_default_build())
    };
    let own = after
        .source_at(&ctx.path)
        .and_then(|f| after.file(f.id))
        .map(|f| f.display_path.to_owned());
    match super::new_problem(
        &before,
        &diagnostics(&before),
        &after,
        &diagnostics(&after),
        own.as_deref(),
    ) {
        Some(problem) => Err(problem),
        None => Ok(()),
    }
}

/// The occurrences, by file, of a valid phrase selection: what
/// `ascribe/targets` lists for `occurrences`. Empty when the selection isn't
/// one a phrase can take the place of.
pub(crate) fn selection_occurrences(
    ctx: &Ctx,
    start: usize,
    end: usize,
) -> BTreeMap<RelPath, Vec<Span>> {
    let mut out: BTreeMap<RelPath, Vec<Span>> = BTreeMap::new();
    let Some(file) = ctx.file() else {
        return out;
    };
    let Ok((span, text)) = phrase_selection(ctx, file, start, end) else {
        return out;
    };
    for (path, at) in occurrences(ctx, &text, Some((&ctx.path, span))) {
        out.entry(path).or_default().push(at);
    }
    out
}
