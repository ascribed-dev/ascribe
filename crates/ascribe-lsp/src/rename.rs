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
    let core = load(config);
    if core.model_problem.is_some() {
        return Err(model_problem());
    }
    let ctx = core
        .project_nav_target()
        .ok_or_else(|| refused("The project has no pages to rename in."))?;
    let mut outside = BTreeMap::new();
    let (edit, moved) = match renaming {
        Renaming::Phrase { key, new } => (phrase(&ctx, key, new, &mut outside)?, None),
        Renaming::Value { value, new } => (value_edit(&ctx, value, new, &mut outside)?, None),
        Renaming::Id { file, id, new } => (heading_id(&core, &ctx, file, id, new)?, None),
        Renaming::File { from, to } => {
            let (edit, moved) = file_move(&ctx, from, to)?;
            (edit, Some(moved))
        }
    };
    let files = changes(&ctx, edit, &outside)?
        .into_iter()
        .map(|(path, (text, edits))| {
            let after = ascribe_core::apply_edits(&text, &edits).map_err(|_| cant_change())?;
            Ok((after != text).then_some(Rewritten {
                path,
                text: after,
                edits: edits.len(),
            }))
        })
        .filter_map(Result::transpose)
        .collect::<Result<_, _>>()?;
    Ok(Renamed { files, moved })
}

/// The project of `config`, an `ascribe.toml`, loaded as the server loads
/// it.
fn load(config: &Path) -> Core {
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
    core
}

fn model_problem() -> RenameError {
    refused("`ascribe.toml` has a problem. Fix it, then try again.")
}

thread_local! {
    /// The `ascribe.toml` of each project a rename is being made in, so that
    /// a project isn't renamed in twice.
    static RENAMING_IN: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// `edit`, a rename made in `ctx`'s project, with the same rename made in
/// the project of each file `[from]` takes from that it changes (SPEC
/// §7.9): that project declares what's renamed, so its pages use it too, and
/// they're renamed with it. `outside` gets the text of each file the other
/// projects change, in UTF-8, as `edit` is made.
///
/// # Errors
///
/// The other project can't take the rename: [`RenameError::Refused`],
/// naming the file.
#[allow(clippy::mutable_key_type)]
pub(crate) fn in_holders(
    ctx: &Ctx,
    renaming: &Renaming,
    mut edit: WorkspaceEdit,
    outside: &mut BTreeMap<PathBuf, String>,
) -> Result<WorkspaceEdit, RenameError> {
    let here = normalize(&ctx.config);
    let pushed = RENAMING_IN.with(|r| {
        let mut r = r.borrow_mut();
        let fresh = !r.contains(&here);
        if fresh {
            r.push(here.clone());
        }
        fresh
    });
    let result = (|| {
        for taken in ctx.taken() {
            let holder = normalize(&taken.path);
            let Some(uri) = path_to_uri(&holder) else {
                continue;
            };
            let changed = edit.changes.as_ref().is_some_and(|c| c.contains_key(&uri));
            if !changed || RENAMING_IN.with(|r| r.borrow().contains(&holder)) {
                continue;
            }
            let theirs = match holder_changes(&holder, renaming) {
                Ok(theirs) => theirs,
                // It doesn't declare it as such: what's taken is all it has.
                Err(RenameError::Missing(_)) => continue,
                Err(RenameError::Refused(why)) => {
                    return Err(RenameError::Refused(format!(
                        "`{}` writes `{}`, and its project uses it too, so it's renamed there as well, but it can't be: {why}",
                        taken.file,
                        renaming.name(),
                    )));
                }
            };
            let changes = edit.changes.get_or_insert_with(Default::default);
            for (path, (text, edits)) in theirs {
                let uri_there = path_to_uri(&path).ok_or_else(cant_change)?;
                // The other project's edits to its own `ascribe.toml` are this
                // project's and more; anything else both change can't be
                // both.
                if path != holder && changes.contains_key(&uri_there) {
                    return Err(cant_change());
                }
                let index = LineIndex::new(&text);
                let edits = edits
                    .into_iter()
                    .map(|e| lsp_types::TextEdit {
                        range: ctx.encoding.range(&index, e.span),
                        new_text: e.new_text,
                    })
                    .collect();
                changes.insert(uri_there, edits);
                outside.insert(path, text);
            }
        }
        Ok(edit)
    })();
    if pushed {
        RENAMING_IN.with(|r| r.borrow_mut().retain(|p| *p != here));
    }
    result
}

/// What renaming `renaming` changes in the project of `config`, as each
/// file's text before and its edits; nothing when it has no pages.
fn holder_changes(config: &Path, renaming: &Renaming) -> Result<Changes, RenameError> {
    let core = load(config);
    if let Some(problem) = &core.model_problem {
        // A file that only holds what other projects take has no content
        // root, and so no pages.
        let no_pages = problem
            .diagnostics
            .iter()
            .all(|d| d.slug == ascribe_core::diagnostics::MODEL_CONTENT_ROOT_MISSING);
        return if no_pages {
            Ok(Changes::new())
        } else {
            Err(model_problem())
        };
    }
    let Some(ctx) = core.project_nav_target() else {
        return Ok(Changes::new());
    };
    let mut outside = BTreeMap::new();
    let edit = match renaming {
        Renaming::Phrase { key, new } => phrase(&ctx, key, new, &mut outside)?,
        Renaming::Value { value, new } => value_edit(&ctx, value, new, &mut outside)?,
        Renaming::Id { .. } | Renaming::File { .. } => return Ok(Changes::new()),
    };
    changes(&ctx, edit, &outside)
}

impl Renaming {
    /// What's renamed, as `ascribe rename` names it.
    fn name(&self) -> String {
        match self {
            Renaming::Phrase { key, .. } => format!("phrase:{key}"),
            Renaming::Value { value, .. } => format!("value:{value}"),
            Renaming::Id { file, id, .. } => format!("{file}#{id}"),
            Renaming::File { from, .. } => from.to_string(),
        }
    }
}

fn refused(message: &str) -> RenameError {
    RenameError::Refused(message.to_owned())
}

fn cant_change() -> RenameError {
    refused(
        "The files can't be changed in place: two of the changes overlap, or `ascribe.toml` writes something in a form this can't edit.",
    )
}

fn phrase(
    ctx: &Ctx,
    key: &str,
    new: &str,
    outside: &mut BTreeMap<PathBuf, String>,
) -> Result<WorkspaceEdit, RenameError> {
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
    let edit = match refactor::phrase_entry_span(&ctx.model_text, key) {
        Some(span) => {
            let position = position(&ctx.model_text, span.start())?;
            refactor::prepare_model(ctx, position).map_err(RenameError::Refused)?;
            refactor::rename_model_key_here(ctx, position, new)
        }
        // A phrase `[from]` takes is renamed where it's written.
        None => refactor::rename_phrase_here(ctx, key, new, None),
    };
    let renaming = Renaming::Phrase {
        key: key.to_owned(),
        new: new.to_owned(),
    };
    in_holders(ctx, &renaming, edit.ok_or_else(cant_change)?, outside)
}

fn value_edit(
    ctx: &Ctx,
    value: &str,
    new: &str,
    outside: &mut BTreeMap<PathBuf, String>,
) -> Result<WorkspaceEdit, RenameError> {
    let Some(found) = refactor::values::named(ctx, value) else {
        return Err(RenameError::Missing(format!(
            "`value:{value}` doesn't exist: no dimension in `ascribe.toml` has the value `{value}`."
        )));
    };
    if let Some(message) = refactor::values::refusal(ctx, &found, new) {
        return Err(RenameError::Refused(message));
    }
    let edit = refactor::values::rename_here(ctx, &found, new).ok_or_else(cant_change)?;
    let renaming = Renaming::Value {
        value: value.to_owned(),
        new: new.to_owned(),
    };
    in_holders(ctx, &renaming, edit, outside)
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
    // As the editor's move asks: a name that differs only in case is the
    // file's own on a case-insensitive file system.
    if refactor::taken(&old, &new) {
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

/// Each file a rename changes, by path: its text before, and its edits.
type Changes = BTreeMap<PathBuf, (String, Vec<ByteEdit>)>;

/// Each file `edit` changes, with its text before and its edits in bytes.
/// `outside` has the text of the files another project's rename changes.
#[allow(clippy::mutable_key_type)]
fn changes(
    ctx: &Ctx,
    edit: WorkspaceEdit,
    outside: &BTreeMap<PathBuf, String>,
) -> Result<Changes, RenameError> {
    let root = ctx.config.parent().unwrap_or(Path::new("."));
    let mut files = BTreeMap::new();
    for (uri, edits) in edit.changes.unwrap_or_default() {
        if edits.is_empty() {
            continue;
        }
        let path = uri_to_path(&uri)
            .map(|p| normalize(&p))
            .ok_or_else(cant_change)?;
        let text = outside
            .get(&path)
            .cloned()
            .or_else(|| text_of(ctx, root, &path))
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
        files.insert(path, (text, edits));
    }
    Ok(files)
}

/// The text of a file an edit changes: `ascribe.toml`, a file its `[from]`
/// takes from, a source file as the project was loaded, or another file of
/// the project, such as the code a `@snippet` shows.
fn text_of(ctx: &Ctx, root: &Path, path: &Path) -> Option<String> {
    if *path == normalize(&ctx.config) {
        return Some(ctx.model_text.clone());
    }
    if let Some(taken) = ctx.taken().iter().find(|t| t.path == *path) {
        return Some(taken.text.clone());
    }
    if let Some(content) = relative_path(&ctx.content_dir, path)
        && let Some(file) = ctx.snapshot.file(&content)
    {
        return Some(file.source.to_string());
    }
    let project = relative_path(root, path)?;
    let bytes = ascribe_resolve::FileSystem::read_file(&*ctx.fs, &project).ok()?;
    String::from_utf8(bytes).ok()
}
