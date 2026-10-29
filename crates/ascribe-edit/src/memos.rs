//! The edits to memo files (`memos/` and the personal memos) that a rename
//! makes: a memo's `about` follows what it names when that's renamed or
//! moves. The server's lenses, links, and completion for memos are
//! `ascribe-lsp`'s.

use std::path::PathBuf;

use ascribe_core::path::normalize;
use ascribe_core::{LineIndex, RelPath, Span};
use ascribe_resolve::{
    Committed, EntryKind, FileIndex, Key, MemoFile, MemoStore, Named, PREFIXES, Personal,
};
use lsp_types::{TextEdit, Uri};

use crate::nav::Ctx;
use crate::uri::path_to_uri;

/// The committed memos, then the personal ones, as the editor has them.
pub fn read(ctx: &Ctx) -> Vec<MemoFile> {
    let fs: &dyn ascribe_resolve::FileSystem = &*ctx.fs;
    let mut memos = Committed::new(fs, ctx.snapshot.layout()).memos();
    memos.extend(Personal::new(fs).memos());
    memos
}

/// The project's folder, where `ascribe.toml` is.
pub fn root(ctx: &Ctx) -> PathBuf {
    ctx.config
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_default()
}

/// The `file:` URI of a path from the project root.
pub fn uri_of(ctx: &Ctx, path: &RelPath) -> Option<Uri> {
    path_to_uri(&normalize(&root(ctx).join(path.as_str())))
}

/// `value` as a YAML scalar: as it is when YAML reads it back as the same
/// text, and in double quotes otherwise, so `42`, `true`, and `null` stay
/// text.
pub fn yaml(value: &str) -> String {
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

/// Where a key's value is written: after the key's `:`, on its line or the
/// lines under it (a block scalar, or a value on the next line), inside its
/// quotes when it has them. `None` when it's written in another form, such
/// as with an escape, or folded over lines.
pub fn value_span(memo: &MemoFile, key: &Key) -> Option<Span> {
    let written = written_value(memo, key)?;
    let at = memo.text.get(written.range())?.find(key.value.as_str())?;
    let start = written.start() + at;
    Some(Span::new(start, start + key.value.len()))
}

/// All that's written for a key's value: from after its `:` to the end of
/// the last line before the next key, or the end of the frontmatter.
pub fn written_value(memo: &MemoFile, key: &Key) -> Option<Span> {
    let text = &memo.text;
    let start = key.line.start() + text.get(key.line.range())?.find(':')? + 1;
    let mut end = key.line.end();
    // The lines under the key are its value's while they're indented: a
    // line at the start of its line is the next key, or the `---` that
    // closes the frontmatter.
    let mut at = text
        .get(end..)?
        .find('\n')
        .map_or(text.len(), |n| end + n + 1);
    for line in text.get(at..)?.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        if !content.trim().is_empty() {
            if !content.starts_with([' ', '\t']) {
                break;
            }
            end = at + content.len();
        }
        at += line.len();
    }
    let end = start + text.get(start..end.max(start))?.trim_end().len();
    Some(Span::new(start, end))
}

/// The edits that keep each memo's `about` naming what it names through a
/// rename: an id that changes, a file that moves, or a content model entry
/// that's renamed. `renamed` gives, for what a memo names, what it names
/// after, or `None` when that doesn't change. The value is replaced where
/// it's written, inside its quotes when it has them; when it's written in a
/// form that can't be found in the text, such as with an escape, the whole
/// value is written again, on the key's line. The memo's `quote` and text
/// are left as they are. Committed and personal memos alike.
fn about_edits(
    ctx: &Ctx,
    mut renamed: impl FnMut(&Named) -> Option<String>,
) -> Vec<(Uri, TextEdit)> {
    let mut edits = Vec::new();
    for memo in read(ctx) {
        let (Some(about), Some(named)) = (&memo.about, &memo.named) else {
            continue;
        };
        let Some(new) = renamed(named) else {
            continue;
        };
        let edit = match value_span(&memo, about) {
            Some(span) => Some((span, new)),
            None => written_value(&memo, about).map(|span| (span, format!(" {}", yaml(&new)))),
        };
        let (Some((span, new_text)), Some(uri)) = (edit, uri_of(ctx, &memo.path)) else {
            continue;
        };
        edits.push((
            uri,
            TextEdit {
                range: ctx.encoding.range(&LineIndex::new(&memo.text), span),
                new_text,
            },
        ));
    }
    edits
}

/// The edits to the memos about the heading `old_id` of `file` when its id
/// becomes `new_id`: those that name it `file#old_id`, those that name it
/// through a page that includes `file` (`page.md#old_id`, as a link does),
/// and, when it's a glossary term's heading, those that name the term,
/// `term:old_id`.
pub fn id_edits(ctx: &Ctx, file: &FileIndex, old_id: &str, new_id: &str) -> Vec<(Uri, TextEdit)> {
    let term = file
        .headings
        .iter()
        .find(|h| h.source_id == old_id)
        .is_some_and(|h| {
            ctx.snapshot
                .glossary()
                .term_at(file.file, h.span)
                .is_some_and(|t| t.id == old_id)
        });
    about_edits(ctx, |named| match named {
        Named::Path { path, id: Some(id) }
            if id == old_id
                && (*path == file.path
                    || ctx
                        .snapshot
                        .page_heading(path, old_id)
                        .is_some_and(|(written_in, _)| written_in == file.path)) =>
        {
            Some(
                Named::Path {
                    path: path.clone(),
                    id: Some(new_id.to_owned()),
                }
                .written(),
            )
        }
        Named::Entry {
            kind: EntryKind::Term,
            name,
        } if term && name == old_id => Some(format!("term:{new_id}")),
        _ => None,
    })
}

/// The edits to the memos about the content model entry `old` of `kind`
/// when it's renamed `new`, such as `phrase:product`.
pub fn entry_edits(ctx: &Ctx, kind: EntryKind, old: &str, new: &str) -> Vec<(Uri, TextEdit)> {
    let Some((prefix, _)) = PREFIXES.iter().find(|(_, k)| *k == kind) else {
        return Vec::new();
    };
    about_edits(ctx, |named| match named {
        Named::Entry { kind: k, name } if *k == kind && name == old => {
            Some(format!("{prefix}:{new}"))
        }
        _ => None,
    })
}

/// The edits to the memos about a file that moves, or a heading in it, as
/// `moved` gives each file's new path: the path changes, and `#id` stays.
pub fn move_edits(ctx: &Ctx, moved: impl Fn(&RelPath) -> RelPath) -> Vec<(Uri, TextEdit)> {
    about_edits(ctx, |named| {
        let Named::Path { path, id } = named else {
            return None;
        };
        let new = moved(path);
        (new != *path).then(|| {
            Named::Path {
                path: new,
                id: id.clone(),
            }
            .written()
        })
    })
}
