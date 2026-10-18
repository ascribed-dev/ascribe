//! Memo files in the editor (SPEC §7.7): the lens on what a memo is about,
//! the memos This page lists, adding a memo from the cursor, and, in a memo
//! file, completion of `about` and a link from it to what it names.
//!
//! The memos are read each time they're asked for, committed and personal,
//! through the project's file system, which holds an open memo file's text
//! ([`crate::fsx::LayerFs`]), and placed against the current snapshot, as
//! `ascribe outline` places them. Only This page keeps them, until the
//! snapshot or a memo file changes ([`crate::page::PageCache`]). A memo is written only
//! when **Add a memo** creates it, and when a rename changes what its
//! `about` names ([`id_edits`], [`move_edits`]).

use std::collections::BTreeMap;
use std::path::PathBuf;

use ascribe_core::path::normalize;
use ascribe_core::{LineIndex, RelPath, Span};
use ascribe_resolve::{
    Committed, EntryKind, FileIndex, Key, MEMOS_FOLDER, MemoFile, MemoState, MemoStore, Named,
    PERSONAL_MEMOS_FOLDER, PREFIXES, Personal, Usable, folder_in_content, place,
};
use ascribe_syntax::{Block, BlockKind, raw_text};
use lsp_types::{
    CodeLens, Command, CompletionItem, CompletionItemKind, CompletionTextEdit, CreateFile,
    CreateFileOptions, DocumentChangeOperation, DocumentChanges, DocumentLink, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, ResourceOp, TextDocumentEdit,
    TextEdit, Uri, WorkspaceEdit,
};
use serde::Serialize;
use serde_json::json;

use crate::definition::{Entry, find_entry};
use crate::edit::EditResult;
use crate::links::OPEN_FILE;
use crate::nav::{Ctx, Lines};
use crate::page::PageMemo;
use crate::uri::path_to_uri;

/// The client command a lens on several memos carries: the client lists
/// them to pick one from, and opens it. Its one argument is the list, each
/// a [`Listed`].
pub(crate) const SHOW_MEMOS: &str = "ascribe.showMemos";

/// How many words of a quote a new memo file's name is made from.
const NAME_WORDS: usize = 5;

/// The committed memos, then the personal ones, as the editor has them.
pub(crate) fn read(ctx: &Ctx) -> Vec<MemoFile> {
    let fs: &dyn ascribe_resolve::FileSystem = &*ctx.fs;
    let mut memos = Committed::new(fs, ctx.snapshot.layout()).memos();
    memos.extend(Personal::new(fs).memos());
    memos
}

/// The project's folder, where `ascribe.toml` is.
fn root(ctx: &Ctx) -> PathBuf {
    ctx.config
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_default()
}

/// The `file:` URI of a path from the project root.
pub(crate) fn uri_of(ctx: &Ctx, path: &RelPath) -> Option<Uri> {
    path_to_uri(&normalize(&root(ctx).join(path.as_str())))
}

/// A memo about something in the requested file: what it's about, its
/// state, and the block its quote is in.
struct Here<'m> {
    memo: &'m MemoFile,
    target: Usable,
    state: MemoState,
    block: Option<Span>,
}

/// The memos about the requested file, a heading in it, or a block in it.
fn here<'m>(ctx: &Ctx, memos: &'m [MemoFile]) -> Vec<Here<'m>> {
    let project = ctx.snapshot.project();
    memos
        .iter()
        .filter_map(|memo| {
            let placement = place(memo, project)?;
            let target = placement.target?;
            let in_file = match &target {
                Usable::File(file) | Usable::Heading { file, .. } => *file == ctx.path,
                _ => false,
            };
            in_file.then_some(Here {
                memo,
                target,
                state: placement.state,
                block: placement.block,
            })
        })
        .collect()
}

/// A memo as a client lists it to pick one from.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Listed {
    /// The memo file's `file:` URI.
    pub uri: String,
    /// Its path from the project root.
    pub path: String,
    /// The first line of its text that isn't blank.
    pub first_line: String,
    /// Whether its quote is no longer found.
    pub outdated: bool,
    /// Whether it's kept on this machine only.
    pub personal: bool,
    /// Where its text starts, to open it at.
    pub body: Range,
}

fn listed(ctx: &Ctx, memo: &MemoFile, state: MemoState) -> Option<Listed> {
    let index = LineIndex::new(&memo.text);
    let at = ctx.encoding.position(&index, memo.body.start())?;
    Some(Listed {
        uri: uri_of(ctx, &memo.path)?.as_str().to_owned(),
        path: memo.path.to_string(),
        first_line: memo.first_line().to_owned(),
        outdated: state == MemoState::Outdated,
        personal: memo.personal(),
        body: Range::new(at, at),
    })
}

/// A lens over each heading and block that memos are about, and over the
/// file's first line for the memos about the whole file: "1 memo", or "3
/// memos, 1 outdated". It opens the memo, or lists them to pick one.
pub(crate) fn code_lenses(ctx: &Ctx) -> Vec<CodeLens> {
    let Some(file) = ctx.file() else {
        return Vec::new();
    };
    let memos = read(ctx);
    let mut at: BTreeMap<(usize, usize), Vec<Here<'_>>> = BTreeMap::new();
    for memo in here(ctx, &memos) {
        let span = match (&memo.block, &memo.target) {
            (Some(block), _) => *block,
            (None, Usable::Heading { id, .. }) => match file.heading_by_id(id) {
                Some(heading) => heading.span,
                None => continue,
            },
            _ => Span::new(0, 0),
        };
        at.entry((span.start(), span.end())).or_default().push(memo);
    }
    let mut lines = Lines::new(ctx);
    let mut out = Vec::new();
    for ((start, end), memos) in at {
        let Some(range) = lines.range(&ctx.path, Span::new(start, end)) else {
            continue;
        };
        let outdated = memos
            .iter()
            .filter(|m| m.state == MemoState::Outdated)
            .count();
        let mut title = match memos.len() {
            1 => "1 memo".to_owned(),
            n => format!("{n} memos"),
        };
        if outdated > 0 {
            title.push_str(&format!(", {outdated} outdated"));
        }
        let list: Vec<Listed> = memos
            .iter()
            .filter_map(|m| listed(ctx, m.memo, m.state))
            .collect();
        let command = match list.as_slice() {
            [one] => Command {
                title,
                command: OPEN_FILE.to_owned(),
                arguments: Some(vec![
                    json!(one.uri),
                    serde_json::to_value(one.body).unwrap_or_default(),
                ]),
            },
            _ => Command {
                title,
                command: SHOW_MEMOS.to_owned(),
                arguments: Some(vec![serde_json::to_value(&list).unwrap_or_default()]),
            },
        };
        out.push(CodeLens {
            range: Range::new(range.start, range.start),
            command: Some(command),
            data: None,
        });
    }
    out
}

/// The memos about the requested file, a heading in it, or a block in it,
/// each with what it's about (the file or the heading): committed then
/// personal, each in path order.
pub(crate) fn in_file(ctx: &Ctx) -> Vec<(Usable, PageMemo)> {
    let memos = read(ctx);
    here(ctx, &memos)
        .into_iter()
        .filter_map(|m| {
            let listed = listed(ctx, m.memo, m.state)?;
            let memo = PageMemo {
                uri: listed.uri,
                path: listed.path,
                first_line: listed.first_line,
                state: m.state.as_str().to_owned(),
                personal: listed.personal,
                body: listed.body,
            };
            Some((m.target, memo))
        })
        .collect()
}

/// `addMemo` and `addPersonalMemo`: a memo file about the section the
/// cursor is in, by its id, or the page when there's no heading, quoting
/// the selection or the first line of the block at the cursor. It's created
/// in `memos/`, or `.ascribe/memos/` when `personal`, named from the quote's
/// first words and never over a file that's there, and the client opens it
/// at its body.
pub(crate) fn add(
    ctx: &Ctx,
    file: &FileIndex,
    start: usize,
    end: usize,
    personal: bool,
) -> Result<EditResult, String> {
    let layout = ctx.snapshot.layout();
    if !personal && folder_in_content(layout).is_some() {
        return Err("`memos/` is in the content root, where its files are pages. Set `[project] content-root` to a folder below the project's, or add a personal memo.".to_owned());
    }
    let heading = file
        .headings
        .iter()
        .filter(|h| !h.source_id.is_empty())
        .filter(|h| h.section.start() <= start && start <= h.section.end())
        .min_by_key(|h| h.section.len());
    let about = match heading {
        Some(h) => format!("{}#{}", ctx.path, h.source_id),
        None => ctx.path.to_string(),
    };
    let quote = quote_at(file, start, end).filter(|quote| {
        // A quote the memo wouldn't find isn't written.
        let memo = memo_text(&about, Some(quote));
        let memo = MemoFile::parse(RelPath::root(), memo);
        memo.problems.is_empty()
            && memo.quote.is_some()
            && place(&memo, ctx.snapshot.project()).is_some_and(|p| p.state == MemoState::Placed)
    });
    let folder = RelPath::parse(if personal {
        PERSONAL_MEMOS_FOLDER
    } else {
        MEMOS_FOLDER
    })
    .map_err(|_| "The memos' folder isn't a path.".to_owned())?;
    let stem = name(quote.as_deref())
        .or_else(|| heading.map(|h| h.source_id.clone()))
        .unwrap_or_else(|| {
            ctx.path
                .file_name()
                .unwrap_or("memo")
                .trim_end_matches(".md")
                .to_owned()
        });
    let fs: &dyn ascribe_resolve::FileSystem = &*ctx.fs;
    let taken = fs.files_in(&folder);
    let path = (1..)
        .map(|n| match n {
            1 => format!("{stem}.md"),
            n => format!("{stem}-{n}.md"),
        })
        .filter_map(|name| folder.join(&name).ok())
        .find(|path| {
            let lower = path.as_str().to_lowercase();
            // Outside FileSystem: whether anything at all is at the name,
            // which a new file mustn't be written over.
            !taken.iter().any(|t| t.as_str().to_lowercase() == lower)
                && !root(ctx).join(path.as_str()).exists()
        })
        .ok_or_else(|| "No name is free for the memo.".to_owned())?;
    let uri = uri_of(ctx, &path).ok_or_else(|| format!("`{path}` can't be named by a URI."))?;
    let text = memo_text(&about, quote.as_deref());
    let index = LineIndex::new(&text);
    let body = ctx
        .encoding
        .position(&index, text.len())
        .unwrap_or_default();
    let operations = vec![
        DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
            uri: uri.clone(),
            options: Some(CreateFileOptions {
                overwrite: Some(false),
                ignore_if_exists: Some(false),
            }),
            annotation_id: None,
        })),
        DocumentChangeOperation::Edit(TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None },
            edits: vec![OneOf::Left(TextEdit {
                range: Range::default(),
                new_text: text,
            })],
        }),
    ];
    Ok(EditResult::Edit {
        edit: WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Operations(operations)),
            change_annotations: None,
        },
        select: Some(Range::new(body, body)),
    })
}

/// A new memo file's text: its frontmatter, then a blank line for the memo.
fn memo_text(about: &str, quote: Option<&str>) -> String {
    let mut text = format!("---\nabout: {}\n", yaml(about));
    if let Some(quote) = quote {
        text.push_str(&format!("quote: {}\n", yaml(quote)));
    }
    text.push_str("---\n\n");
    text
}

/// `value` as a YAML scalar: as it is when YAML reads it back as the same
/// text, and in double quotes otherwise, so `42`, `true`, and `null` stay
/// text.
fn yaml(value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '#'))
        && !value.starts_with(['-', '#', '.'])
        && matches!(
            serde_yaml_ng::from_str::<serde_yaml_ng::Value>(value),
            Ok(serde_yaml_ng::Value::String(read)) if read == value
        );
    if plain {
        value.to_owned()
    } else {
        // A JSON string is a YAML double-quoted scalar.
        serde_json::to_string(value).unwrap_or_default()
    }
}

/// What a memo at `start`..`end` quotes: the selection, its whitespace runs
/// as one space, or the first line of the block at the cursor. None on a
/// heading, whose memo is about its section, or where there's no block.
fn quote_at(file: &FileIndex, start: usize, end: usize) -> Option<String> {
    if end > start {
        let selected = spaced(file.source.get(start..end)?);
        return (!selected.is_empty()).then_some(selected);
    }
    let block = leaf_at(&file.document.blocks, start)?;
    if matches!(block.kind, BlockKind::Heading(_)) {
        return None;
    }
    let text = raw_text(&file.source, block.span);
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("```") && !line.starts_with("~~~"))
        .map(spaced)
}

/// The innermost block at `offset` that holds text rather than blocks.
fn leaf_at(blocks: &[Block], offset: usize) -> Option<&Block> {
    let block = blocks
        .iter()
        .find(|b| b.span.start() <= offset && offset <= b.span.end())?;
    match &block.kind {
        BlockKind::BlockQuote(quote) => leaf_at(&quote.children, offset),
        BlockKind::Container(container) => leaf_at(&container.children, offset),
        BlockKind::Group(group) => group
            .arms
            .iter()
            .find_map(|arm| leaf_at(&arm.children, offset)),
        BlockKind::List(list) => list
            .items
            .iter()
            .find_map(|item| leaf_at(&item.children, offset)),
        _ => Some(block),
    }
}

/// `text` with each run of whitespace read as one space, and none at
/// either end.
fn spaced(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A file name from a quote's first words, lowercase and joined by `-`.
fn name(quote: Option<&str>) -> Option<String> {
    let words: Vec<String> = quote?
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .take(NAME_WORDS)
        .map(str::to_lowercase)
        .collect();
    (!words.is_empty()).then(|| words.join("-"))
}

/// The value of a memo file's `about`, where it's written: the line's span
/// after `about:` and the spaces after it. `None` when the offset isn't on
/// an `about` line of the frontmatter.
fn about_value(text: &str, offset: usize) -> Option<Span> {
    let rest = text.strip_prefix("---")?;
    let close = rest.find("\n---").map(|at| at + 3)?;
    if offset < 3 || offset > close {
        return None;
    }
    let start = text.get(..offset)?.rfind('\n').map_or(0, |at| at + 1);
    let end = text
        .get(start..)?
        .find(['\n', '\r'])
        .map_or(text.len(), |at| start + at);
    let line = text.get(start..end)?;
    let after = line.strip_prefix("about:")?;
    let value = start + "about:".len() + (after.len() - after.trim_start().len());
    (offset >= value).then(|| Span::new(value, end))
}

/// Completion of `about` in a memo file whose text is `text`: every page
/// and fragment, each heading with an id as `page.md#id`, and each content
/// model entry as `phrase:<key>` and the like, as `ascribe refs` takes them.
pub(crate) fn complete(ctx: &Ctx, text: &str, offset: usize) -> Option<Vec<CompletionItem>> {
    let value = about_value(text, offset)?;
    let index = LineIndex::new(text);
    let range = ctx.encoding.range(&index, value);
    let item = |label: String, kind: CompletionItemKind, detail: Option<String>| CompletionItem {
        label: label.clone(),
        kind: Some(kind),
        detail,
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range,
            new_text: label,
        })),
        ..CompletionItem::default()
    };
    let mut files: Vec<&FileIndex> = ctx.snapshot.files().collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let mut out = Vec::new();
    for file in files {
        out.push(item(
            file.path.to_string(),
            CompletionItemKind::FILE,
            file.title.clone(),
        ));
        for heading in file.headings.iter().filter(|h| !h.source_id.is_empty()) {
            out.push(item(
                format!("{}#{}", file.path, heading.source_id),
                CompletionItemKind::REFERENCE,
                Some(heading.text.clone()),
            ));
        }
    }
    let model = &ctx.model;
    let glossary = ctx.snapshot.glossary();
    for (prefix, kind) in PREFIXES {
        let names: Vec<&str> = match kind {
            EntryKind::Phrase => model.phrases.iter().map(|p| p.key.as_str()).collect(),
            EntryKind::Feature => model.features.iter().map(|f| f.key.as_str()).collect(),
            EntryKind::Term => glossary.terms.iter().map(|t| t.id.as_str()).collect(),
            EntryKind::Dimension => model.dimensions.iter().map(|d| d.name.as_str()).collect(),
            EntryKind::Note => model.notes.iter().map(|n| n.name.as_str()).collect(),
            EntryKind::Widget => model
                .widgets
                .iter()
                .map(|w| w.schema.name.as_str())
                .collect(),
        };
        for name in names {
            out.push(item(
                format!("{prefix}:{name}"),
                CompletionItemKind::CONSTANT,
                None,
            ));
        }
    }
    Some(out)
}

/// Where a key's value is written on its line: after the key's `:`, inside
/// its quotes when it has them. `None` when it's written in another form,
/// such as with an escape.
fn value_span(memo: &MemoFile, key: &Key) -> Option<Span> {
    let line = memo.text.get(key.line.range())?;
    let colon = line.find(':')? + 1;
    let at = colon + line.get(colon..)?.find(key.value.as_str())?;
    let start = key.line.start() + at;
    Some(Span::new(start, start + key.value.len()))
}

/// A link from a memo file's `about` to what it names: the page, the
/// heading, or the entry in `ascribe.toml`. None when it names nothing.
pub(crate) fn links(ctx: &Ctx, memo: &MemoFile) -> Vec<DocumentLink> {
    let (Some(about), Some(named)) = (&memo.about, &memo.named) else {
        return Vec::new();
    };
    let Some(span) = value_span(memo, about) else {
        return Vec::new();
    };
    let target = match named.find(ctx.snapshot.project()) {
        Some(Usable::File(path)) => ctx.uri_of(&path),
        Some(Usable::Heading { file, id }) => {
            let mut lines = Lines::new(ctx);
            let heading = ctx.snapshot.file(&file).and_then(|f| f.heading_by_id(&id));
            let line = heading
                .and_then(|h| lines.position(&file, h.span.start()))
                .map_or(0, |p| p.line);
            ctx.uri_of(&file).and_then(|uri| with_line(&uri, line))
        }
        Some(_) => entry_uri(ctx, named),
        None => None,
    };
    let Some(target) = target else {
        return Vec::new();
    };
    vec![DocumentLink {
        range: ctx.encoding.range(&LineIndex::new(&memo.text), span),
        target: Some(target),
        tooltip: Some(format!("Open {}", about.value)),
        data: None,
    }]
}

/// The edits that keep each memo's `about` naming its page or heading
/// through a rename: an id that changes, or a file that moves. `renamed`
/// gives, for the file and id a memo names, what it names after, or `None`
/// when that doesn't change. The value is replaced where it's written,
/// inside its quotes when it has them; the memo's `quote` and text are left
/// as they are. Committed and personal memos alike.
fn about_edits(
    ctx: &Ctx,
    mut renamed: impl FnMut(&RelPath, Option<&str>) -> Option<String>,
) -> Vec<(Uri, TextEdit)> {
    let mut edits = Vec::new();
    for memo in read(ctx) {
        let (Some(about), Some(Named::Path { path, id })) = (&memo.about, &memo.named) else {
            continue;
        };
        let Some(new) = renamed(path, id.as_deref()) else {
            continue;
        };
        let Some(span) = value_span(&memo, about) else {
            continue;
        };
        let Some(uri) = uri_of(ctx, &memo.path) else {
            continue;
        };
        edits.push((
            uri,
            TextEdit {
                range: ctx.encoding.range(&LineIndex::new(&memo.text), span),
                new_text: new,
            },
        ));
    }
    edits
}

/// The edits to the memos about the heading `old_id` of `file` when its id
/// becomes `new_id`. A heading in a fragment is named in the fragment, so
/// only `file#old_id` is.
pub(crate) fn id_edits(
    ctx: &Ctx,
    file: &RelPath,
    old_id: &str,
    new_id: &str,
) -> Vec<(Uri, TextEdit)> {
    about_edits(ctx, |path, id| {
        (path == file && id == Some(old_id)).then(|| format!("{path}#{new_id}"))
    })
}

/// The edits to the memos about a file that moves, or a heading in it, as
/// `moved` gives each file's new path: the path changes, and `#id` stays.
pub(crate) fn move_edits(ctx: &Ctx, moved: impl Fn(&RelPath) -> RelPath) -> Vec<(Uri, TextEdit)> {
    about_edits(ctx, |path, id| {
        let new = moved(path);
        (new != *path).then(|| match id {
            Some(id) => format!("{new}#{id}"),
            None => new.to_string(),
        })
    })
}

/// `ascribe.toml` at the entry's declaration, or a glossary term's page at
/// its heading.
fn entry_uri(ctx: &Ctx, named: &Named) -> Option<Uri> {
    let Named::Entry { kind, name } = named else {
        return None;
    };
    let entry = match kind {
        EntryKind::Phrase => Entry::Phrase(name),
        EntryKind::Feature => Entry::Feature(name),
        EntryKind::Term => {
            let glossary = ctx.snapshot.glossary();
            let term = glossary.term(name)?;
            let file = ctx.snapshot.file(&term.written_in)?;
            let line = ctx
                .encoding
                .position(&LineIndex::new(&file.source), term.heading.start())
                .map_or(0, |p| p.line);
            return with_line(&ctx.uri_of(&term.written_in)?, line);
        }
        EntryKind::Dimension => Entry::Dimension(name),
        EntryKind::Note => Entry::Note(name),
        EntryKind::Widget => Entry::Widget(name),
    };
    let uri = path_to_uri(&ctx.config)?;
    let line = find_entry(&ctx.model_text, &entry)
        .and_then(|span| {
            ctx.encoding
                .position(&LineIndex::new(&ctx.model_text), span.start())
        })
        .map_or(0, |p| p.line);
    with_line(&uri, line)
}

/// A `#L` fragment makes VS Code open a file at a line.
fn with_line(uri: &Uri, line: u32) -> Option<Uri> {
    if line == 0 {
        return Some(uri.clone());
    }
    std::str::FromStr::from_str(&format!("{}#L{}", uri.as_str(), line + 1)).ok()
}

/// Where `position` is in a memo file whose text is `text`.
pub(crate) fn offset(ctx: &Ctx, text: &str, position: Position) -> usize {
    ctx.encoding
        .offset_lenient(&LineIndex::new(text), text, position)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_the_quotes_first_words() {
        assert_eq!(
            name(Some("Install the agent package, then run it")).as_deref(),
            Some("install-the-agent-package-then")
        );
        assert_eq!(name(Some("`--`")), None);
    }

    #[test]
    fn a_value_is_quoted_when_yaml_would_read_it_otherwise() {
        assert_eq!(yaml("install-agent.md#install"), "install-agent.md#install");
        assert_eq!(yaml("Run it: now"), "\"Run it: now\"");
        assert_eq!(yaml("#heading"), "\"#heading\"");
        for scalar in ["42", "1.0", "true", "null", "1e3"] {
            assert_eq!(yaml(scalar), format!("\"{scalar}\""));
        }
    }

    #[test]
    fn the_about_value_is_found_on_its_line_in_the_frontmatter() {
        let text = "---\nabout: a.md\nquote: x\n---\n\nabout: not here\n";
        let at = text.find("a.md").unwrap_or_default();
        assert_eq!(about_value(text, at), Some(Span::new(at, at + 4)));
        assert_eq!(
            about_value(text, text.rfind("not").unwrap_or_default()),
            None
        );
        assert_eq!(about_value(text, text.find("x").unwrap_or_default()), None);
    }
}
