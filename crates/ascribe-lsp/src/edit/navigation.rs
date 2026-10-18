//! The edits the Pages view makes to `ascribe.nav` (SPEC §7.8): a section
//! put in order among its siblings, a page among its section's, a page the
//! file leaves out placed, and a section's label; and what a page or a
//! folder moving changes in the file.
//!
//! Each is made of the file's own pieces (`ascribe_model::nav_text`), one
//! at a time, with the file read again after each, so its comments and
//! layout stay: a table is added after the last, a `pages` list is written
//! once, the order the view shows, and then an entry moves within it. What
//! the file doesn't list keeps its place: a section listed for the first
//! time is listed after the siblings that come before it, which are listed
//! too when they aren't, since a listed section comes before every one
//! that isn't. The result is the file's text after; [`text_edit`] makes it
//! the one edit that changes the part that differs.

use std::collections::HashMap;

use ascribe_core::{FileId, LineIndex, RelPath, Span, TextEdit};
use ascribe_model::nav_text::{
    add_to_table, append_names, append_tables, insert_before_item, key_line, line_end, line_start,
    pages_line, remove_item_span, section_table, table_span, table_span_with_comments, toml_string,
};
use ascribe_model::{NAV_FILE, NAV_VERSION, NavFile, NavSection};
use ascribe_resolve::{Navigation, Section};
use lsp_types::{
    CreateFile, CreateFileOptions, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Range, ResourceOp, TextDocumentEdit, WorkspaceEdit,
};
use serde_json::{Map, Value};

use super::{EditAction, EditResult};
use crate::nav::Ctx;

/// What the Pages view asks of `ascribe.nav`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NavChange {
    /// A section put before a sibling, or after them all.
    MoveSection {
        folder: String,
        before: Option<String>,
    },
    /// A page put before another of its section's, or after them all.
    MovePage {
        page: RelPath,
        before: Option<RelPath>,
    },
    /// A page the section's `pages` leaves out, appended to it.
    PlacePage { page: RelPath },
    /// A section's label set, or taken out with `None`.
    SetLabel {
        folder: String,
        label: Option<String>,
    },
}

/// The text of `ascribe.nav` after `change`, from `text`, the file as it is
/// (`None` when there's none), and `navigation`, the project's as it is.
pub(crate) fn change(
    text: Option<&str>,
    navigation: &Navigation,
    change: &NavChange,
) -> Result<String, String> {
    let mut file = File::new(text)?;
    match change {
        NavChange::MoveSection { folder, before } => {
            file.move_section(navigation, folder, before.as_deref())?;
        }
        NavChange::MovePage { page, before } => {
            file.move_page(navigation, page, before.as_ref())?;
        }
        NavChange::PlacePage { page } => file.place_page(navigation, page)?,
        NavChange::SetLabel { folder, label } => {
            file.set_label(navigation, folder, label.as_deref())?;
        }
    }
    Ok(file.finish())
}

/// The text of `ascribe.nav` after the files and folders `remap` names move:
/// a page's entry in `pages` renamed with it, or taken out when it leaves
/// its folder, and a folder's `folder` and its nested sections' following
/// it. `None` when the file doesn't read, or nothing in it changes.
pub(crate) fn moved(text: &str, remap: &dyn Fn(&RelPath) -> RelPath) -> Option<String> {
    let mut file = File::new(Some(text)).ok()?;
    // Each change reads the file again: the entries' places move. Each
    // takes an entry out or names it as it ends up, so they come to an end;
    // the bound is for a mapping that moves a folder into itself.
    for _ in 0..10_000 {
        let nav = file.read().ok()?;
        let Some(edit) = next_move(&file.text, &nav, remap) else {
            break;
        };
        file.apply(edit);
    }
    let after = file.finish();
    (after != text).then_some(after)
}

/// The first change a move makes to the file, as it reads now.
fn next_move(text: &str, nav: &NavFile, remap: &dyn Fn(&RelPath) -> RelPath) -> Option<TextEdit> {
    for entry in &nav.sections {
        let new_folder = moved_folder(&entry.folder, remap);
        if let Some(new) = &new_folder {
            return Some(TextEdit::replace(entry.folder_span, toml_string(new)));
        }
        let Some(pages) = &entry.pages else {
            continue;
        };
        for (i, (name, span)) in pages.iter().enumerate() {
            let Ok(path) = RelPath::parse(&format!("{}{name}", entry.folder)) else {
                continue;
            };
            let new = remap(&path);
            if new == path {
                continue;
            }
            if parent(new.as_str()) == entry.folder {
                return Some(TextEdit::replace(
                    *span,
                    toml_string(file_name(new.as_str())),
                ));
            }
            return remove_item_span(text, pages, i).map(TextEdit::delete);
        }
    }
    None
}

/// Where a section's `folder` goes when `remap` moves it, or a folder above
/// it; `None` when it stays.
fn moved_folder(folder: &str, remap: &dyn Fn(&RelPath) -> RelPath) -> Option<String> {
    let path = RelPath::parse(folder.strip_suffix('/')?).ok()?;
    let new = remap(&path);
    (new != path).then(|| format!("{new}/"))
}

/// The one edit that makes `before` into `after`: the part between what
/// they start and end with alike.
pub(crate) fn text_edit(before: &str, after: &str) -> TextEdit {
    let mut start = before
        .bytes()
        .zip(after.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while !before.is_char_boundary(start) || !after.is_char_boundary(start) {
        start -= 1;
    }
    let mut end = before[start..]
        .bytes()
        .rev()
        .zip(after[start..].bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    while !before.is_char_boundary(before.len() - end) || !after.is_char_boundary(after.len() - end)
    {
        end -= 1;
    }
    TextEdit::replace(
        Span::new(start, before.len() - end),
        &after[start..after.len() - end],
    )
}

/// The file's text, changed one piece at a time.
struct File {
    text: String,
}

impl File {
    fn new(text: Option<&str>) -> Result<File, String> {
        let file = File {
            text: text.map_or_else(|| format!("version = {NAV_VERSION}\n"), str::to_owned),
        };
        let nav = file.read()?;
        if nav
            .sections
            .iter()
            .any(|entry| table_span(&file.text, entry.folder_span).is_none())
        {
            return Err("`ascribe.nav` writes its sections inline. Write each as a `[[sections]]` table to put them in order here.".to_owned());
        }
        Ok(file)
    }

    fn read(&self) -> Result<NavFile, String> {
        NavFile::parse(&self.text, FileId::new(0))
            .map_err(|_| "`ascribe.nav` has a problem. Fix it, then try again.".to_owned())
    }

    fn apply(&mut self, edit: TextEdit) {
        let range = edit.span.range();
        self.text.replace_range(range, &edit.new_text);
    }

    /// The text, ending in one line break.
    fn finish(mut self) -> String {
        let trimmed = self.text.trim_end().len();
        self.text.truncate(trimmed);
        self.text.push('\n');
        self.text
    }

    /// `folder`'s table, made when the file has none, with its siblings'
    /// that come before it and have none, so the order stays as it is.
    fn table(&mut self, navigation: &Navigation, folder: &str) -> Result<NavSection, String> {
        let nav = self.read()?;
        if let Some(entry) = nav.section(folder) {
            return Ok(entry.clone());
        }
        let (_, parent) = find(&navigation.root, folder)
            .ok_or_else(|| format!("`{folder}` isn't a section of the project."))?;
        let mut folders: Vec<&str> = Vec::new();
        if let Some(parent) = parent {
            for sibling in parent.sections.iter().filter(|s| !parent.places(s)) {
                if sibling.folder == folder {
                    break;
                }
                if nav.section(&sibling.folder).is_none() {
                    folders.push(&sibling.folder);
                }
            }
        }
        folders.push(folder);
        let tables: Vec<String> = folders
            .iter()
            .map(|f| section_table(f, None, None))
            .collect();
        let edit = append_tables(&self.text, &tables);
        self.apply(edit);
        self.read()?
            .section(folder)
            .cloned()
            .ok_or_else(|| format!("`{folder}` couldn't be added to `ascribe.nav`."))
    }

    fn move_section(
        &mut self,
        navigation: &Navigation,
        folder: &str,
        before: Option<&str>,
    ) -> Result<(), String> {
        let (section, parent) = find(&navigation.root, folder)
            .ok_or_else(|| format!("`{folder}` isn't a section of the project."))?;
        let parent = parent.ok_or_else(|| "The root section comes first.".to_owned())?;
        if parent.places(section) {
            return Err(format!(
                "`{folder}` sits where its page beside the folder sits: move that page among `{}`'s pages.",
                parent.folder
            ));
        }
        let siblings: Vec<&Section> = parent
            .sections
            .iter()
            .filter(|s| !parent.places(s))
            .collect();
        if let Some(before) = before
            && !siblings.iter().any(|s| s.folder == before)
        {
            return Err(format!(
                "`{before}` isn't a section beside `{folder}` that it can go before."
            ));
        }
        // Every sibling listed, in the order they have now.
        if let Some(last) = siblings.last() {
            self.table(navigation, &last.folder)?;
        }
        for sibling in &siblings {
            self.table(navigation, &sibling.folder)?;
        }
        let order: Vec<&str> = siblings.iter().map(|s| s.folder.as_str()).collect();
        let at = order.iter().position(|f| *f == folder);
        let next = at.and_then(|i| order.get(i + 1)).copied();
        if before == Some(folder) || next == before {
            return Ok(());
        }
        let nav = self.read()?;
        let span = nav
            .section(folder)
            .and_then(|e| table_span_with_comments(&self.text, e.folder_span))
            .ok_or_else(|| format!("`{folder}`'s table can't be moved."))?;
        let table = format!("{}\n", self.text[span.range()].trim_end());
        self.apply(TextEdit::delete(span));
        let nav = self.read()?;
        let edit = match before {
            Some(before) => {
                let target = nav
                    .section(before)
                    .and_then(|e| table_span_with_comments(&self.text, e.folder_span))
                    .ok_or_else(|| format!("`{before}`'s table can't be found."))?;
                TextEdit::insert(target.start(), format!("{table}\n"))
            }
            None => {
                // After the last sibling's table.
                let last = order
                    .iter()
                    .rev()
                    .find(|f| **f != folder)
                    .and_then(|f| nav.section(f))
                    .and_then(|e| table_span(&self.text, e.folder_span))
                    .ok_or_else(|| "The sections' tables can't be found.".to_owned())?;
                let end = last.end();
                let head = &self.text[..end];
                let lead = if head.ends_with("\n\n") {
                    ""
                } else if head.ends_with('\n') {
                    "\n"
                } else {
                    "\n\n"
                };
                let tail = if end == self.text.len() { "" } else { "\n" };
                TextEdit::insert(end, format!("{lead}{table}{tail}"))
            }
        };
        self.apply(edit);
        Ok(())
    }

    /// The section `page` is a page of, and its table, with `pages` written
    /// as the navigation has them when it has none.
    fn listed_pages<'n>(
        &mut self,
        navigation: &'n Navigation,
        page: &RelPath,
    ) -> Result<(&'n Section, NavSection), String> {
        let folder = parent(page.as_str()).to_owned();
        let (section, _) = find(&navigation.root, &folder)
            .ok_or_else(|| format!("`{folder}` isn't a section of the project."))?;
        if !section.pages.contains(page) {
            return Err(format!("`{page}` isn't one of `{folder}`'s pages."));
        }
        let entry = self.table(navigation, &folder)?;
        if entry.pages.is_some() {
            return Ok((section, entry));
        }
        let names: Vec<&str> = section
            .pages
            .iter()
            .map(|p| file_name(p.as_str()))
            .collect();
        let edit = add_to_table(&self.text, entry.folder_span, &pages_line(&names))
            .ok_or_else(|| format!("`{folder}`'s table can't be changed."))?;
        self.apply(edit);
        let entry = self.table(navigation, &folder)?;
        Ok((section, entry))
    }

    fn move_page(
        &mut self,
        navigation: &Navigation,
        page: &RelPath,
        before: Option<&RelPath>,
    ) -> Result<(), String> {
        if before == Some(page) {
            return Ok(());
        }
        let (section, entry) = self.listed_pages(navigation, page)?;
        if let Some(before) = before
            && !section.pages.contains(before)
        {
            return Err(format!(
                "`{before}` isn't in `{}`, so `{page}` can't go before it: a page moves to another section by moving its file.",
                section.folder
            ));
        }
        let name = file_name(page.as_str());
        let written = |entry: &NavSection| -> Vec<String> {
            entry
                .pages
                .iter()
                .flatten()
                .map(|(n, _)| n.clone())
                .collect()
        };
        // The pages the list leaves out, which come after it, are placed up
        // to where the page goes, so the rest keep their order.
        let listed = written(&entry);
        let left: Vec<&str> = section
            .pages
            .iter()
            .map(|p| file_name(p.as_str()))
            .filter(|n| !listed.iter().any(|l| l == n))
            .collect();
        let target = before.map(|b| file_name(b.as_str()));
        let place: Vec<&str> = match target {
            Some(target) if listed.iter().any(|l| l == target) => Vec::new(),
            Some(target) => left
                .iter()
                .copied()
                .take_while(|n| *n != target)
                .filter(|n| *n != name)
                .collect(),
            None => left.iter().copied().filter(|n| *n != name).collect(),
        };
        let mut entry = entry;
        if !place.is_empty() {
            let edit = entry
                .pages_span
                .and_then(|list| append_names(&self.text, list, entry.pages.as_deref()?, &place))
                .ok_or_else(|| format!("`{}`'s pages can't be changed.", section.folder))?;
            self.apply(edit);
            entry = self.table(navigation, &section.folder)?;
        }
        // Out of the list, then back in where it goes, with its comment when
        // it's on a line of its own.
        let pages = entry.pages.clone().unwrap_or_default();
        let mut carried = None;
        if let Some(i) = pages.iter().position(|(n, _)| n == name) {
            carried =
                own_line(&self.text, pages[i].1).map(|line| self.text[line.range()].to_owned());
            let span = remove_item_span(&self.text, &pages, i)
                .ok_or_else(|| format!("`{name}` can't be moved."))?;
            self.apply(TextEdit::delete(span));
            entry = self.table(navigation, &section.folder)?;
        }
        let pages = entry.pages.clone().unwrap_or_default();
        let at = target.and_then(|t| pages.iter().position(|(n, _)| n == t));
        let line_at = match at {
            Some(i) => own_line(&self.text, pages[i].1).map(|l| l.start()),
            None => pages
                .last()
                .and_then(|(_, span)| own_line(&self.text, *span))
                .map(|l| l.end()),
        };
        let edit = match (carried, line_at, at) {
            (Some(line), Some(offset), _) => Some(TextEdit::insert(offset, line)),
            (_, _, Some(i)) => insert_before_item(&self.text, &pages, i, name),
            (_, _, None) => entry
                .pages_span
                .and_then(|list| append_names(&self.text, list, &pages, &[name])),
        }
        .ok_or_else(|| format!("`{name}` can't be moved."))?;
        self.apply(edit);
        Ok(())
    }

    fn place_page(&mut self, navigation: &Navigation, page: &RelPath) -> Result<(), String> {
        let folder = parent(page.as_str());
        let name = file_name(page.as_str());
        let nav = self.read()?;
        let entry = nav
            .section(folder)
            .filter(|e| e.pages.is_some())
            .ok_or_else(|| {
                format!("`{folder}` lists no pages, so its pages are all in their place already.")
            })?;
        if entry.pages.iter().flatten().any(|(n, _)| n == name) {
            return Err(format!("`{name}` is in `{folder}`'s pages already."));
        }
        let (section, _) = find(&navigation.root, folder)
            .ok_or_else(|| format!("`{folder}` isn't a section of the project."))?;
        if !section.pages.contains(page) {
            return Err(format!("`{page}` isn't one of `{folder}`'s pages."));
        }
        let edit = entry
            .pages_span
            .and_then(|list| append_names(&self.text, list, entry.pages.as_deref()?, &[name]))
            .ok_or_else(|| format!("`{folder}`'s pages can't be changed."))?;
        self.apply(edit);
        Ok(())
    }

    fn set_label(
        &mut self,
        navigation: &Navigation,
        folder: &str,
        label: Option<&str>,
    ) -> Result<(), String> {
        let label = label.map(str::trim).filter(|l| !l.is_empty());
        if label.is_some_and(|l| l.contains(['\n', '\r'])) {
            return Err("A label is one line.".to_owned());
        }
        let nav = self.read()?;
        if label.is_none() && nav.section(folder).is_none_or(|e| e.label.is_none()) {
            return Ok(());
        }
        let entry = self.table(navigation, folder)?;
        let line = key_line(&self.text, entry.folder_span, "label");
        let edit = match (line, label) {
            (Some(line), Some(label)) => {
                TextEdit::replace(line, format!("label = {}\n", toml_string(label)))
            }
            (Some(line), None) => TextEdit::delete(line),
            (None, Some(label)) => add_to_table(
                &self.text,
                entry.folder_span,
                &format!("label = {}", toml_string(label)),
            )
            .ok_or_else(|| format!("`{folder}`'s table can't be changed."))?,
            (None, None) => {
                return Err(format!(
                    "`{folder}`'s label isn't on a line of its own. Change it in `ascribe.nav`."
                ));
            }
        };
        self.apply(edit);
        Ok(())
    }
}

/// The `ascribe/edit` answer to a change the Pages view asks for: an edit
/// of `ascribe.nav`, or one that creates it with its text when the project
/// has none. The project's navigation with the file as it would be is the
/// one asked for, or the change is refused.
#[allow(clippy::mutable_key_type)]
pub(crate) fn edit(ctx: &Ctx, asked: &NavChange) -> Result<EditResult, String> {
    let path = RelPath::parse(NAV_FILE).map_err(|_| unnamed())?;
    let text = ascribe_resolve::FileSystem::read_file(&*ctx.fs, &path)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
    let project = ctx.snapshot.project();
    // An unlisted page is in no sidebar (SPEC §7.8): it has no place to take.
    if let NavChange::PlacePage { page } = asked
        && project
            .file(page)
            .is_some_and(|f| f.listing(project.model()) == ascribe_model::Listing::Unlisted)
    {
        return Err(format!(
            "`{}` is `unlisted`, so no sidebar shows it and it has no place to take.",
            file_name(page.as_str())
        ));
    }
    let navigation = project.navigation();
    let after = change(text.as_deref(), &navigation, asked)?;
    let nav = NavFile::parse(&after, FileId::new(0))
        .map_err(|_| "The change would make `ascribe.nav` unreadable.".to_owned())?;
    let made = Navigation::of(project, Some(&nav));
    if !follows(&navigation, &made, asked) {
        return Err("The change wouldn't give the order asked for.".to_owned());
    }
    let root = ctx.config.parent().ok_or_else(unnamed)?;
    let uri = crate::uri::path_to_uri(&root.join(NAV_FILE)).ok_or_else(unnamed)?;
    let edit = match text {
        Some(text) => {
            let edit = text_edit(&text, &after);
            let index = LineIndex::new(&text);
            let mut changes = HashMap::new();
            changes.insert(
                uri,
                vec![lsp_types::TextEdit {
                    range: ctx.encoding.range(&index, edit.span),
                    new_text: edit.new_text,
                }],
            );
            WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }
        }
        None => WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Operations(vec![
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
                    edits: vec![OneOf::Left(lsp_types::TextEdit {
                        range: Range::default(),
                        new_text: after,
                    })],
                }),
            ])),
            change_annotations: None,
        },
    };
    Ok(EditResult::Edit { edit, select: None })
}

/// The change `args` names for `action`.
pub(crate) fn asked(action: EditAction, args: &Map<String, Value>) -> Result<NavChange, String> {
    let text = |key: &str| match args.get(key) {
        Some(Value::String(s)) => Ok(Some(s.clone())),
        None | Some(Value::Null) => Ok(None),
        Some(_) => Err(format!("`{key}` must be text.")),
    };
    let required = |key: &str| text(key)?.ok_or_else(|| format!("`{key}` is missing."));
    let page = |key: &str| -> Result<Option<RelPath>, String> {
        text(key)?
            .map(|p| {
                RelPath::parse(&p)
                    .ok()
                    .filter(RelPath::is_inside)
                    .ok_or_else(|| format!("`{key}` must be a page's path from the content root."))
            })
            .transpose()
    };
    Ok(match action {
        EditAction::MoveSection => NavChange::MoveSection {
            folder: required("folder")?,
            before: text("before")?,
        },
        EditAction::MovePage => NavChange::MovePage {
            page: page("page")?.ok_or_else(|| "`page` is missing.".to_owned())?,
            before: page("before")?,
        },
        EditAction::PlacePage => NavChange::PlacePage {
            page: page("page")?.ok_or_else(|| "`page` is missing.".to_owned())?,
        },
        _ => NavChange::SetLabel {
            folder: required("folder")?,
            label: text("label")?,
        },
    })
}

/// Whether `made`, the navigation with the file as it would be, has what
/// `asked` asks of `before`, the navigation as it is.
fn follows(before: &Navigation, made: &Navigation, asked: &NavChange) -> bool {
    match asked {
        NavChange::MoveSection { folder, before: at } => {
            let order = |navigation: &Navigation| -> Option<Vec<String>> {
                let (_, parent) = find(&navigation.root, folder)?;
                let parent = parent?;
                Some(
                    parent
                        .sections
                        .iter()
                        .filter(|s| !parent.places(s))
                        .map(|s| s.folder.clone())
                        .collect(),
                )
            };
            let (Some(mut wanted), Some(got)) = (order(before), order(made)) else {
                return false;
            };
            wanted.retain(|f| f != folder);
            let at = at
                .as_ref()
                .and_then(|b| wanted.iter().position(|f| f == b))
                .unwrap_or(wanted.len());
            wanted.insert(at, folder.clone());
            wanted == got
        }
        NavChange::MovePage { page, before: at } => {
            let folder = parent(page.as_str());
            let pages = |navigation: &Navigation| {
                find(&navigation.root, folder).map(|(s, _)| s.pages.clone())
            };
            let (Some(mut wanted), Some(got)) = (pages(before), pages(made)) else {
                return false;
            };
            wanted.retain(|p| p != page);
            let at = at
                .as_ref()
                .and_then(|b| wanted.iter().position(|p| p == b))
                .unwrap_or(wanted.len());
            wanted.insert(at, page.clone());
            wanted == got
        }
        // It stays in its section, placed after the pages listed before it.
        NavChange::PlacePage { page } => {
            find(&made.root, parent(page.as_str())).is_some_and(|(s, _)| s.pages.contains(page))
        }
        NavChange::SetLabel { folder, label } => find(&made.root, folder).is_some_and(|(s, _)| {
            s.label.as_deref() == label.as_deref().map(str::trim).filter(|l| !l.is_empty())
        }),
    }
}

fn unnamed() -> String {
    "`ascribe.nav` can't be named by a URI.".to_owned()
}

/// The line of a `pages` entry at `span`, when it's on one of its own with
/// a comma after it: what moves with the entry, a comment after it
/// included.
fn own_line(text: &str, span: Span) -> Option<Span> {
    let start = line_start(text, span.start());
    if !text[start..span.start()].trim().is_empty() {
        return None;
    }
    let end = line_end(text, span.end());
    text[span.end()..end]
        .trim_start()
        .starts_with(',')
        .then(|| Span::new(start, end))
}

/// The section of `folder`, and its parent; `None` for the parent of the
/// root.
fn find<'a>(root: &'a Section, folder: &str) -> Option<(&'a Section, Option<&'a Section>)> {
    if root.folder == folder {
        return Some((root, None));
    }
    let mut stack = vec![root];
    while let Some(section) = stack.pop() {
        for nested in &section.sections {
            if nested.folder == folder {
                return Some((nested, Some(section)));
            }
            stack.push(nested);
        }
    }
    None
}

/// The folder a path is in, ending in `/`, or `""` at the content root.
fn parent(path: &str) -> &str {
    match path.rfind('/') {
        Some(at) => &path[..=at],
        None => "",
    }
}

/// A path's file name.
fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_one_edit_is_what_differs() {
        let edit = text_edit("a = 1\nb = 2\nc = 3\n", "a = 1\nb = 22\nc = 3\n");
        assert_eq!(edit.span, Span::new(11, 11));
        assert_eq!(edit.new_text, "2");
        let edit = text_edit("ab", "ab");
        assert!(edit.span.is_empty() && edit.new_text.is_empty());
        let edit = text_edit("é", "è");
        assert_eq!(edit.span, Span::new(0, 2));
    }
}
