//! Renaming without an editor, for `ascribe rename`: what the editor's
//! rename (`F2`) and its edits when a file moves do, asked by name instead of
//! at a position, and returned as each file's text after. The project is
//! loaded as the server loads it, and the edits are the server's own
//! (`refactor.rs`), so the command and the editor change the same places.
//!
//! Nothing is written here: the caller writes the files, and moves the one
//! that moves.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Coded, LineIndex, RelPath, TextEdit as ByteEdit};
use lsp_types::{
    FileRename, Position, RenameFilesParams, RenameParams, TextDocumentIdentifier,
    TextDocumentPositionParams, WorkspaceEdit,
};

use crate::core::Core;
use crate::nav::Ctx;
use crate::position::Encoding;
use crate::refactor;
use crate::uri::{path_to_uri, uri_to_path};

/// What to rename, and what to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Renaming {
    /// A phrase's key under `[phrases]`, and every `{key}` that uses it: in
    /// pages, frontmatter fields with `phrases = true`, and the code
    /// `@snippet {phrases=true}` shows.
    Phrase {
        /// The key now.
        key: String,
        /// The new key.
        new: String,
    },
    /// A dimension's value, in `ascribe.toml` and everywhere a page uses it.
    Value {
        /// The value now.
        value: String,
        /// The new value.
        new: String,
    },
    /// A heading's id, and every link and include that names it. A heading
    /// with an `@id` has it changed; one whose id is its slug gets an `@id`.
    Id {
        /// The page or fragment the heading is in, as a content path.
        file: RelPath,
        /// The id now.
        id: String,
        /// The new id.
        new: String,
    },
    /// A page or fragment that moves, and every link and include that names
    /// it, or that it has, whose path changes.
    File {
        /// Where it is, as a content path.
        from: RelPath,
        /// Where it goes, as a content path.
        to: RelPath,
    },
}

/// What a rename changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Renamed {
    /// Each file the rename changes, in path order, with its text after.
    pub files: Vec<Rewritten>,
    /// The file that moves, absolute: where it is, and where it goes. Its
    /// changes, if any, are in `files` under where it is.
    pub moved: Option<(PathBuf, PathBuf)>,
}

/// A file a rename changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rewritten {
    /// The file, absolute.
    pub path: PathBuf,
    /// Its text after the rename.
    pub text: String,
    /// How many places in it change.
    pub edits: usize,
}

/// Why a rename can't be made.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RenameError {
    /// What's to be renamed isn't in the project.
    #[error("{0}")]
    Missing(String),
    /// It can't have the new name, or can't be renamed at all.
    #[error("{0}")]
    Refused(String),
}

impl Coded for RenameError {
    fn code(&self) -> &'static str {
        match self {
            RenameError::Missing(_) => "rename_target_missing",
            RenameError::Refused(_) => "rename_refused",
        }
    }
}

/// Renames `renaming` in the project of `config`, an `ascribe.toml`: the
/// text each file has after it, with nothing written.
///
/// # Errors
///
/// What's named isn't in the project ([`RenameError::Missing`]), or the new
/// name can't be its name, or the files can't be changed in place
/// ([`RenameError::Refused`]).
pub fn rename(config: &Path, renaming: &Renaming) -> Result<Renamed, RenameError> {
    let (out, sent) = crossbeam_channel::unbounded();
    let mut core = Core::new(out);
    let config = normalize(config);
    core.folders = config.parent().map(Path::to_owned).into_iter().collect();
    // A command runs it, and a command reads a source wherever the model
    // says.
    core.bound_sources = false;
    core.encoding = Encoding::Utf8;
    core.config = Some(config);
    core.start();
    sent.try_iter().for_each(drop);
    if core.model_problem.is_some() {
        return Err(refused(
            "`ascribe.toml` has a problem. Fix it, then try again.",
        ));
    }
    let ctx = core
        .project_nav_target()
        .ok_or_else(|| refused("The project has no pages to rename in."))?;
    let (edit, moved) = match renaming {
        Renaming::Phrase { key, new } => (phrase(&ctx, key, new)?, None),
        Renaming::Value { value, new } => (value_edit(&ctx, value, new)?, None),
        Renaming::Id { file, id, new } => (heading_id(&core, &ctx, file, id, new)?, None),
        Renaming::File { from, to } => {
            let (edit, moved) = file_move(&ctx, from, to)?;
            (edit, Some(moved))
        }
    };
    Ok(Renamed {
        files: rewritten(&ctx, edit)?,
        moved,
    })
}

fn refused(message: &str) -> RenameError {
    RenameError::Refused(message.to_owned())
}

fn cant_change() -> RenameError {
    refused(
        "The files can't be changed in place: two of the changes overlap, or `ascribe.toml` writes something in a form this can't edit.",
    )
}

fn phrase(ctx: &Ctx, key: &str, new: &str) -> Result<WorkspaceEdit, RenameError> {
    if !ctx.model.has_phrase(key) {
        return Err(RenameError::Missing(format!(
            "`phrase:{key}` doesn't exist: `ascribe.toml` declares no phrase `{key}`."
        )));
    }
    if new == key {
        return Err(refused("The new name is the same."));
    }
    if !refactor::valid_phrase_key(new) {
        return Err(RenameError::Refused(format!(
            "`{new}` can't be a phrase's key: a key starts with a lowercase letter, and has only lowercase letters, digits, and hyphens."
        )));
    }
    if ctx.model.has_phrase(new) {
        return Err(RenameError::Refused(format!(
            "`{new}` is a phrase already."
        )));
    }
    let span = refactor::phrase_entry_span(&ctx.model_text, key).ok_or_else(cant_change)?;
    let position = position(&ctx.model_text, span.start())?;
    refactor::prepare_model(ctx, position).map_err(RenameError::Refused)?;
    refactor::rename_model_key(ctx, position, new).ok_or_else(cant_change)
}

fn value_edit(ctx: &Ctx, value: &str, new: &str) -> Result<WorkspaceEdit, RenameError> {
    let Some(found) = refactor::values::named(ctx, value) else {
        return Err(RenameError::Missing(format!(
            "`value:{value}` doesn't exist: no dimension in `ascribe.toml` has the value `{value}`."
        )));
    };
    if let Some(message) = refactor::values::refusal(ctx, &found, new) {
        return Err(RenameError::Refused(message));
    }
    refactor::values::rename(ctx, &found, new).ok_or_else(cant_change)
}

fn heading_id(
    core: &Core,
    project: &Ctx,
    file: &RelPath,
    id: &str,
    new: &str,
) -> Result<WorkspaceEdit, RenameError> {
    let missing = || RenameError::Missing(format!("`{file}#{id}` doesn't exist."));
    let path = normalize(&project.content_dir.join(file.as_str()));
    let ctx = path_to_uri(&path)
        .and_then(|uri| core.nav_target(&uri))
        .ok_or_else(|| RenameError::Missing(format!("`{file}` isn't a page or fragment.")))?;
    let index = ctx.file().ok_or_else(missing)?;
    let at = index
        .headings
        .iter()
        .position(|h| h.source_id == id && !id.is_empty())
        .ok_or_else(missing)?;
    let heading = &index.headings[at];
    if new == id {
        return Err(refused("The new name is the same."));
    }
    if !refactor::valid_id_or_phrase(new) {
        return Err(RenameError::Refused(format!(
            "`{new}` can't be an id: an id has only letters, digits, hyphens, underscores, and periods."
        )));
    }
    if index.heading_by_id(new).is_some() {
        return Err(RenameError::Refused(format!(
            "`{file}#{new}` is another heading's id already."
        )));
    }
    let Some(explicit) = &heading.explicit_id else {
        return refactor::add_id(&ctx, at, new).ok_or_else(|| {
            RenameError::Refused(format!(
                "`#{id}` is made from the heading's text, and the heading isn't on a line of its own, so it can't be given an `@id` here. Give it one, then rename that."
            ))
        });
    };
    if explicit.permalink {
        return Err(RenameError::Refused(format!(
            "`#{id}` is a permalink, which links from outside the docs use. To end it, change the `@id` by hand, then retire the old name with `ascribe permalinks retire`."
        )));
    }
    // The id is the end of its `@id` line.
    let line = index
        .source
        .get(explicit.span.range())
        .ok_or_else(cant_change)?;
    let start = explicit.span.start() + line.rfind(id).ok_or_else(cant_change)?;
    let params = RenameParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: path_to_uri(&path).ok_or_else(cant_change)?,
            },
            position: position(&index.source, start)?,
        },
        new_name: new.to_owned(),
        work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
    };
    refactor::rename(&ctx, params).ok_or_else(cant_change)
}

fn file_move(
    ctx: &Ctx,
    from: &RelPath,
    to: &RelPath,
) -> Result<(WorkspaceEdit, (PathBuf, PathBuf)), RenameError> {
    if ctx.snapshot.file(from).is_none() {
        return Err(RenameError::Missing(format!(
            "`{from}` isn't a page or fragment."
        )));
    }
    if !to.is_inside() {
        return Err(RenameError::Refused(format!(
            "`{to}` is outside the content root."
        )));
    }
    let extension = |path: &RelPath| {
        Path::new(path.as_str())
            .extension()
            .map(|e| e.to_ascii_lowercase())
    };
    if extension(from) != extension(to) {
        return Err(RenameError::Refused(format!(
            "`{to}` would have another extension than `{from}`, so it wouldn't be the same kind of file."
        )));
    }
    let old = normalize(&ctx.content_dir.join(from.as_str()));
    let new = normalize(&ctx.content_dir.join(to.as_str()));
    if old == new {
        return Err(refused("The new name is the same."));
    }
    // Outside FileSystem: whether the place a file moves to is taken, by
    // anything at all, as the editor's move asks.
    if new.exists() {
        return Err(RenameError::Refused(format!("`{to}` is there already.")));
    }
    // Nothing Ascribe writes goes through a link a repository commits
    // (decision 77): not a folder on the way to where the file moves, nor
    // what's there.
    let root = ctx.config.parent().unwrap_or(Path::new("."));
    if let Some(rel) = std::path::absolute(&new)
        .ok()
        .and_then(|new| new.strip_prefix(root).ok().map(Path::to_owned))
        && let Some(link) = ascribe_resolve::LinkCheck::new(root).link(&rel)
    {
        let link = relative_path(root, &link)
            .map_or_else(|| link.display().to_string(), |l| l.to_string());
        return Err(RenameError::Refused(format!(
            "`{to}` is through `{link}`, a link, which a rename doesn't write through. Move the page to a folder that isn't a link."
        )));
    }
    let params = RenameFilesParams {
        files: vec![FileRename {
            old_uri: path_to_uri(&old).ok_or_else(cant_change)?.to_string(),
            new_uri: path_to_uri(&new).ok_or_else(cant_change)?.to_string(),
        }],
    };
    let edit = refactor::will_rename(ctx, params).ok_or_else(cant_change)?;
    Ok((edit, (old, new)))
}

/// The position of `offset` in `text`, in UTF-8, as the edits here are
/// made.
fn position(text: &str, offset: usize) -> Result<Position, RenameError> {
    Encoding::Utf8
        .position(&LineIndex::new(text), offset)
        .ok_or_else(cant_change)
}

/// Each file `edit` changes, with its text after.
#[allow(clippy::mutable_key_type)]
fn rewritten(ctx: &Ctx, edit: WorkspaceEdit) -> Result<Vec<Rewritten>, RenameError> {
    let root = ctx.config.parent().unwrap_or(Path::new("."));
    let mut files = BTreeMap::new();
    for (uri, edits) in edit.changes.unwrap_or_default() {
        if edits.is_empty() {
            continue;
        }
        let path = uri_to_path(&uri)
            .map(|p| normalize(&p))
            .ok_or_else(cant_change)?;
        let text = text_of(ctx, root, &path)
            .ok_or_else(|| RenameError::Refused(format!("`{}` can't be read.", path.display())))?;
        let index = LineIndex::new(&text);
        let edits = edits
            .into_iter()
            .map(|e| {
                let start = Encoding::Utf8.offset(&index, e.range.start)?;
                let end = Encoding::Utf8.offset(&index, e.range.end)?;
                Some(ByteEdit::replace(
                    ascribe_core::Span::new(start, end),
                    e.new_text,
                ))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(cant_change)?;
        let after = ascribe_core::apply_edits(&text, &edits).map_err(|_| cant_change())?;
        if after != text {
            files.insert(
                path.clone(),
                Rewritten {
                    path,
                    text: after,
                    edits: edits.len(),
                },
            );
        }
    }
    Ok(files.into_values().collect())
}

/// The text of a file an edit changes: `ascribe.toml`, a source file as the
/// project was loaded, or another file of the project, such as the code a
/// `@snippet` shows.
fn text_of(ctx: &Ctx, root: &Path, path: &Path) -> Option<String> {
    if *path == normalize(&ctx.config) {
        return Some(ctx.model_text.clone());
    }
    if let Some(content) = relative_path(&ctx.content_dir, path)
        && let Some(file) = ctx.snapshot.file(&content)
    {
        return Some(file.source.to_string());
    }
    let project = relative_path(root, path).filter(RelPath::is_inside)?;
    let bytes = ascribe_resolve::FileSystem::read_file(&*ctx.fs, &project).ok()?;
    String::from_utf8(bytes).ok()
}
