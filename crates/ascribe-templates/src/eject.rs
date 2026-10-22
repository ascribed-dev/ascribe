//! Writing templates into a project: `ascribe templates eject`.
//!
//! Each template of the set named is written as the project's own file, with
//! its guide and Ascribe's notes on its type, under a folder beside
//! `ascribe.toml` (`templates/`, unless the caller names another), and its
//! type is added to `ascribe.toml`, at its end, so the rest of the file stays
//! as it was, comments and order with it.
//! `templates.lock`, in the same folder, records what each file was copied
//! from and a hash of it as written, with Ascribe's notes apart from their
//! files; a second eject adds to it.
//!
//! Everything is checked before anything is written: a type the project
//! declares already, or a file in the way, is refused, and then nothing is
//! written. From then on the files are the project's, and nothing here writes
//! over them.
//!
//! Asked to keep a declared type ([`Eject::keep_type`]), eject takes the
//! template for it instead of refusing it: it writes the same files and adds
//! only `template` and `guide` to the type's table, in place, after its last
//! entry, leaving its other keys and every comment as they were. A type
//! written as an inline or dotted table, one that names a template or a guide
//! already, and one whose frontmatter doesn't take the template's are
//! refused.
//!
//! A type added takes its frontmatter from the template, and each field
//! another of the project's types declares as a string is declared as the
//! first such type declares it, `inline` and `role` with it, rather than as
//! a plain string. A page says what type it is in its frontmatter (SPEC
//! §7.2), so a type added applies to no page until a page names it; eject
//! then lists the project's pages that name no type, which `ascribe check`
//! reports once the project declares types.

use std::path::Path;

use ascribe_check::MODEL_FILE;
use ascribe_core::RelPath;
use ascribe_core::{TextEdit, apply_edits};
use ascribe_model::{ContentModel, file_hash};
use ascribe_model::{ContentType, FieldType, TYPE_KEY};
use ascribe_resolve::{FileSystem, Layout, Probe, in_nested_project};
use serde::Serialize;
use toml_edit::{ArrayOfTables, Document, DocumentMut, Item, Table, value};

use crate::{Set, Template, set};

/// The folder eject writes into by default, from the project's folder.
pub const TEMPLATES_DIR: &str = "templates";

/// The file, in the folder eject writes into, that records what each file
/// was copied from.
pub const LOCK_FILE: &str = "templates.lock";

/// The version of `templates.lock`'s format.
pub const LOCK_VERSION: i64 = 1;

/// The version of [`EjectAnswer`]'s schema.
pub const SCHEMA_VERSION: u32 = 2;

/// The version of Ascribe, which the answer carries.
pub(crate) const ASCRIBE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The first lines of a new `templates.lock`.
const LOCK_HEADER: &str = "# Written by `ascribe templates eject`, which adds to it. Don't edit it: it\n# records what each template was copied from, and a hash of it as written.\n";

/// What `ascribe templates eject` did: the files it wrote and the types it
/// added, or, when it refused anything, what it refused, and then it wrote
/// nothing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct EjectAnswer {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The files written, from the folder `ascribe.toml` is in,
    /// `/`-separated, in the order they were written: each template, its
    /// guide, and Ascribe's notes on its type, then `templates.lock`, then
    /// `ascribe.toml`. Empty when anything was refused.
    pub written: Vec<String>,
    /// The types added to `ascribe.toml`, in the order they were named.
    /// Empty when anything was refused.
    pub added: Vec<Added>,
    /// The types `ascribe.toml` declared already and kept, with `--keep-type`,
    /// in the order they were named: each was given its `template` and
    /// `guide`, and nothing else in it changed. Empty when anything was
    /// refused.
    pub kept: Vec<Kept>,
    /// What was refused, in the order it was named. When it isn't empty,
    /// nothing was written.
    pub refused: Vec<Refusal>,
    /// Whether `ascribe.toml` declared no types, so that every page was of
    /// the implicit type `page`, and `[types.page]` was written out first,
    /// as it was, so the pages that aren't of a type added can name it.
    pub page_declared: bool,
    /// The project's pages that name no type, from the folder `ascribe.toml`
    /// is in, in order. With the types added the project declares types, so
    /// each needs `type:` in its frontmatter, naming its type.
    pub untyped: Vec<String>,
}

/// A type eject added to `ascribe.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Added {
    /// The type's name, `[types.<name>]`, which is the template's.
    pub name: String,
    /// Their index's summary of it.
    pub summary: String,
    /// Its `template`, from the folder `ascribe.toml` is in.
    pub template: String,
    /// Its `guide`, from the folder `ascribe.toml` is in, in the order to
    /// read them: their guide, then Ascribe's notes on the type.
    pub guide: Vec<String>,
    /// The fields of its frontmatter declared as another of the project's
    /// types declares them, rather than as plain strings, in order.
    pub copied: Vec<String>,
}

/// A type `ascribe.toml` declared already, which eject kept, adding only
/// its `template` and `guide`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Kept {
    /// The type's name, `[types.<name>]`, which is the template's.
    pub name: String,
    /// Their index's summary of it.
    pub summary: String,
    /// The `template` added, from the folder `ascribe.toml` is in.
    pub template: String,
    /// The `guide` added, from the folder `ascribe.toml` is in, in the order
    /// to read them: their guide, then Ascribe's notes on the type.
    pub guide: Vec<String>,
    /// The type's keys left as they were, in the order `ascribe.toml` has
    /// them, such as `purpose` and `frontmatter`.
    pub left: Vec<String>,
}

/// A template eject refused, and why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Refusal {
    /// The name given.
    pub name: String,
    /// Why it was refused.
    pub reason: RefusalReason,
    /// The file in the way, from the folder `ascribe.toml` is in, when that's
    /// the reason; otherwise null.
    pub path: Option<String>,
    /// Why, in a sentence.
    pub message: String,
}

/// Why a template was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum RefusalReason {
    /// The set has no template of that name.
    Unknown,
    /// `ascribe.toml` declares a type of that name already.
    Declared,
    /// A file is where one of its files would be written.
    FileInTheWay,
    /// Asked to keep the type, `ascribe.toml` writes it as an inline or a
    /// dotted table, which eject doesn't add to.
    TypeNotTable,
    /// Asked to keep the type, it names a `template` or a `guide` already.
    TypeHasTemplate,
    /// Asked to keep the type, its frontmatter doesn't declare a field the
    /// template's has, or declares it as something other than a string.
    FrontmatterDiffers,
}

/// Why eject couldn't do its work. Nothing was written, except after a
/// [`EjectError::Write`]: the files written before it stay.
#[derive(Debug, thiserror::Error)]
pub enum EjectError {
    /// The folder can't hold a type's files.
    #[error(
        "eject can't write templates in `{folder}`: {reason}; pass --to with a folder outside it"
    )]
    Folder {
        /// The folder, from the folder `ascribe.toml` is in.
        folder: String,
        /// Why.
        reason: String,
    },
    /// `templates.lock` is there, and isn't one eject can add to.
    #[error("can't add to `{path}`: {reason}")]
    Lock {
        /// The lock, from the folder `ascribe.toml` is in.
        path: String,
        /// Why.
        reason: String,
    },
    /// The types can't be added to `ascribe.toml`.
    #[error(
        "can't add {types} to ascribe.toml: {reason}; add it by hand, as `ascribe templates eject` would"
    )]
    Model {
        /// The types, as `[types.<name>]`.
        types: String,
        /// Why.
        reason: String,
    },
    /// A file couldn't be written.
    #[error("can't write `{path}`: {reason}")]
    Write {
        /// The file, from the folder `ascribe.toml` is in.
        path: String,
        /// Why.
        reason: String,
    },
}

impl ascribe_core::Coded for EjectError {
    fn code(&self) -> &'static str {
        match self {
            EjectError::Folder { .. } => "templates_folder_refused",
            EjectError::Lock { .. } => "templates_lock_invalid",
            EjectError::Model { .. } => "model_edit_refused",
            EjectError::Write { .. } => "templates_unwritable",
        }
    }
}

/// What eject is asked to do.
#[derive(Clone, Debug)]
pub struct Eject<'a> {
    /// The project's folder, where `ascribe.toml` is.
    pub root: &'a Path,
    /// The project's content model.
    pub model: &'a ContentModel,
    /// `ascribe.toml`'s text, which is written again with the types added at
    /// its end.
    pub model_text: &'a str,
    /// The templates to write, by name; all of them when it's empty.
    pub names: &'a [String],
    /// The folder to write them in, from the project's folder.
    pub to: &'a RelPath,
    /// The version of Ascribe `templates.lock` records.
    pub ascribe_version: &'a str,
    /// Whether a type `ascribe.toml` declares already is kept, given only
    /// its `template` and `guide`, rather than refused.
    pub keep_type: bool,
}

/// Writes the templates `request` names into the project, with their guides,
/// Ascribe's notes on their types, and `templates.lock`, and adds their types
/// to `ascribe.toml`, or, for a type it declares already and `request` keeps,
/// adds the type's `template` and `guide`. What's there is looked at through
/// `fs`, the project's [`FileSystem`].
///
/// # Errors
///
/// The folder is in the content root or the output directory, or outside the
/// project's folder; `templates.lock` is there and can't be read as one; the
/// types can't be added to `ascribe.toml`; or a file can't be written. A
/// template refused isn't an error: the answer lists it, and nothing is
/// written.
pub fn eject(request: &Eject<'_>, fs: &dyn FileSystem) -> Result<EjectAnswer, EjectError> {
    let set = set();
    let mut answer = EjectAnswer {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        written: Vec::new(),
        added: Vec::new(),
        kept: Vec::new(),
        refused: Vec::new(),
        page_declared: false,
        untyped: Vec::new(),
    };
    let chosen = chosen(&set, request.names, &mut answer.refused);
    let layout = Layout::from_model(request.model);
    if let Some(reason) = ascribe_check::type_files::folder_problem(request.to, &layout, fs) {
        return Err(EjectError::Folder {
            folder: shown(request.to),
            reason,
        });
    }
    let lock_path = at(request.to, LOCK_FILE);
    let mut lock = read_lock(&lock_path, fs)?;
    let doc = Document::parse(request.model_text).ok();
    for template in &chosen {
        refusals(request, doc.as_ref(), fs, template, &mut answer.refused);
    }
    if !answer.refused.is_empty() {
        return Ok(answer);
    }
    let defaults = Defaults::of(request.model, request.model_text);
    let mut placed: Vec<(String, Vec<String>)> = Vec::new();
    let mut new: Vec<Template> = Vec::new();
    for template in &chosen {
        if request.model.types.iter().any(|t| t.name == template.name) {
            let kept = kept(template, request.to, doc.as_ref());
            placed.push((kept.template.clone(), kept.guide.clone()));
            answer.kept.push(kept);
        } else {
            let added = added(template, request.to, &defaults);
            placed.push((added.template.clone(), added.guide.clone()));
            answer.added.push(added);
            new.push(template.clone());
        }
    }
    answer.page_declared = !declares_types(request.model_text);
    let model_text = with_kept(request.model_text, doc.as_ref(), &answer.kept)?;
    let (model_text, _) = with_types(
        &model_text,
        answer.page_declared,
        &new,
        &answer.added,
        &Before {
            model: request.model,
            kept: &answer.kept,
            defaults: &defaults,
        },
    )?;
    answer.untyped = untyped(request.model, &layout, fs);
    let mut files: Vec<(String, &'static str)> = Vec::new();
    for (template, (template_path, guide)) in chosen.iter().zip(&placed) {
        let theirs = [
            (template_path.clone(), template.text(&template.template)),
            (guide[0].clone(), template.text(&template.guide)),
        ];
        let ours: Vec<(String, &'static str)> = guide[1..]
            .iter()
            .cloned()
            .zip(template.notes().map(|n| n.text))
            .collect();
        lock_entry(
            &mut lock,
            &set,
            template,
            &theirs,
            &ours,
            request.ascribe_version,
        );
        files.extend(theirs);
        files.extend(ours);
    }
    for (path, text) in &files {
        write(request.root, path, text)?;
        answer.written.push(path.clone());
    }
    write(request.root, lock_path.as_str(), &lock.to_string())?;
    answer.written.push(lock_path.to_string());
    write(request.root, MODEL_FILE, &model_text)?;
    answer.written.push(MODEL_FILE.to_owned());
    Ok(answer)
}

/// The templates named, each once, in the order named; every template of
/// the set when none is. A name the set doesn't have is refused.
fn chosen(set: &Set, names: &[String], refused: &mut Vec<Refusal>) -> Vec<Template> {
    if names.is_empty() {
        return set.templates.clone();
    }
    let mut out: Vec<Template> = Vec::new();
    for name in names {
        if out.iter().any(|t| &t.name == name) || refused.iter().any(|r| &r.name == name) {
            continue;
        }
        match set.templates.iter().find(|t| &t.name == name) {
            Some(template) => out.push(template.clone()),
            None => {
                let known: Vec<String> = set
                    .templates
                    .iter()
                    .map(|t| format!("`{}`", t.name))
                    .collect();
                refused.push(Refusal {
                    name: name.clone(),
                    reason: RefusalReason::Unknown,
                    path: None,
                    message: format!(
                        "there's no template named `{name}`; the set has {}",
                        known.join(", ")
                    ),
                });
            }
        }
    }
    out
}

/// Why `template` can't be written, if it can't, in `refused`. `doc` is
/// `ascribe.toml`, parsed with its spans.
fn refusals(
    request: &Eject<'_>,
    doc: Option<&Document<&str>>,
    fs: &dyn FileSystem,
    template: &Template,
    refused: &mut Vec<Refusal>,
) {
    let name = &template.name;
    if let Some(declared) = request.model.types.iter().find(|t| &t.name == name) {
        let refuse = |reason: RefusalReason, message: String| Refusal {
            name: name.clone(),
            reason,
            path: None,
            message,
        };
        if !request.keep_type {
            refused.push(refuse(
                RefusalReason::Declared,
                format!(
                    "ascribe.toml declares [types.{name}] already; `ascribe templates eject {name} --keep-type` takes the template for it"
                ),
            ));
        } else if let Some((reason, message)) = not_kept(declared, template, doc) {
            refused.push(refuse(reason, message));
        }
    }
    let notes = template.notes().map(|n| n.name);
    for file in [template.template.as_str(), template.guide.as_str()]
        .into_iter()
        .chain(notes)
    {
        let path = at(request.to, file);
        if fs.probe(&path) != Probe::Missing || fs.real_path(&path).is_some() {
            refused.push(Refusal {
                name: name.clone(),
                reason: RefusalReason::FileInTheWay,
                path: Some(path.to_string()),
                message: format!(
                    "`{path}` is there already, and eject never writes over a file of the project's"
                ),
            });
        }
    }
}

/// Why `declared` can't be kept and given `template`'s `template` and
/// `guide`, if it can't: it isn't a `[types.<name>]` table of its own, it
/// names a template or a guide already, or its frontmatter doesn't take the
/// template's.
fn not_kept(
    declared: &ContentType,
    template: &Template,
    doc: Option<&Document<&str>>,
) -> Option<(RefusalReason, String)> {
    let name = &declared.name;
    if header_table(doc, name).is_none() {
        return Some((
            RefusalReason::TypeNotTable,
            format!(
                "ascribe.toml doesn't write [types.{name}] as a table of its own, so eject can't add `template` and `guide` to it; write it as one, under a [types.{name}] header, or add them by hand"
            ),
        ));
    }
    let named = if declared.template.is_some() {
        Some("template")
    } else if !declared.guide.is_empty() {
        Some("guide")
    } else {
        None
    };
    if let Some(key) = named {
        return Some((
            RefusalReason::TypeHasTemplate,
            format!(
                "[types.{name}] names a `{key}` already, and eject never changes one; take it out first to replace it"
            ),
        ));
    }
    frontmatter_differs(declared, template)
        .map(|message| (RefusalReason::FrontmatterDiffers, message))
}

/// Why a page started from `template` wouldn't be of `declared`, if it
/// wouldn't: the template's frontmatter has a field the type doesn't
/// declare, or declares as something other than a string.
fn frontmatter_differs(declared: &ContentType, template: &Template) -> Option<String> {
    let name = &declared.name;
    for field in frontmatter_fields(template.text(&template.template)) {
        let found = declared.frontmatter.fields.iter().find(|f| f.name == field);
        match found {
            None => {
                return Some(format!(
                    "the template's frontmatter has `{field}`, which [types.{name}] doesn't declare, so no page started from it would check; declare `{field}` as a string first"
                ));
            }
            Some(f) if f.ty != FieldType::String => {
                return Some(format!(
                    "[types.{name}] declares `{field}` as {}, and the template's is a string, so no page started from it would check",
                    f.ty.describe()
                ));
            }
            Some(_) => {}
        }
    }
    None
}

/// The `[types.<name>]` table in `doc`, when it's written with a header of
/// its own: not inline, not dotted, and not only the parent of
/// `[types.<name>.frontmatter]`.
fn header_table<'d>(doc: Option<&'d Document<&str>>, name: &str) -> Option<&'d Table> {
    match doc?.get("types")?.get(name)? {
        Item::Table(t) if !t.is_dotted() && !t.is_implicit() => Some(t),
        _ => None,
    }
}

/// The type `template` keeps, given its `template` and `guide`.
fn kept(template: &Template, to: &RelPath, doc: Option<&Document<&str>>) -> Kept {
    Kept {
        name: template.name.clone(),
        summary: template.summary.clone(),
        template: at(to, &template.template).to_string(),
        guide: guide(template, to),
        left: header_table(doc, &template.name)
            .map(|t| t.iter().map(|(k, _)| k.to_owned()).collect())
            .unwrap_or_default(),
    }
}

/// The paths of `template`'s guide and Ascribe's notes, in `to`.
fn guide(template: &Template, to: &RelPath) -> Vec<String> {
    std::iter::once(template.guide.as_str())
        .chain(template.notes().map(|n| n.name))
        .map(|file| at(to, file).to_string())
        .collect()
}

/// The type `template` is added as.
fn added(template: &Template, to: &RelPath, defaults: &Defaults) -> Added {
    Added {
        name: template.name.clone(),
        summary: template.summary.clone(),
        template: at(to, &template.template).to_string(),
        guide: guide(template, to),
        copied: frontmatter(template, defaults)
            .into_iter()
            .filter(|(_, declared)| declared.is_some())
            .map(|(field, _)| field)
            .collect(),
    }
}

/// The project's pages that name no type in their frontmatter, from the
/// folder `ascribe.toml` is in, in order. A fragment, and a file in a project
/// nested in the content root, is no page; a page that can't be read, or
/// whose frontmatter isn't YAML, is left to `ascribe check`.
fn untyped(model: &ContentModel, layout: &Layout, fs: &dyn FileSystem) -> Vec<String> {
    let sources = fs.sources();
    let mut pages: Vec<&RelPath> = sources
        .paths
        .iter()
        .filter(|p| !in_nested_project(p, &sources.nested) && !model.is_fragment(p.as_str()))
        .collect();
    pages.sort();
    pages
        .into_iter()
        .filter_map(|p| {
            let path = layout
                .content_root
                .join(p.as_str())
                .unwrap_or_else(|_| p.clone());
            let text = String::from_utf8(fs.read_file(&path).ok()?).ok()?;
            names_no_type(&text).then(|| path.to_string())
        })
        .collect()
}

/// Whether a page's text names no type: it has no frontmatter, or its
/// frontmatter, read as YAML, has no `type`.
fn names_no_type(text: &str) -> bool {
    let Some(body) = frontmatter_text(text) else {
        return true;
    };
    match serde_yaml_ng::from_str::<serde_yaml_ng::Value>(body) {
        Ok(value) => value.get(TYPE_KEY).is_none(),
        Err(_) => false,
    }
}

/// The project's types, as `ascribe.toml` declares them, which a type added
/// takes its frontmatter's declarations from.
struct Defaults {
    /// The types `ascribe.toml` declares, in order; none when it declares
    /// none.
    types: Vec<ContentType>,
    /// The names of the fields `[frontmatter]` declares, which every type
    /// has already.
    shared: Vec<String>,
    /// `ascribe.toml`, parsed, when it parses.
    doc: Option<DocumentMut>,
}

impl Defaults {
    fn of(model: &ContentModel, text: &str) -> Defaults {
        Defaults {
            types: model
                .types
                .iter()
                .filter(|t| !t.implicit)
                .cloned()
                .collect(),
            shared: model.frontmatter.iter().map(|f| f.name.clone()).collect(),
            doc: text.parse().ok(),
        }
    }

    /// How the first of the project's types to declare `field` as a string,
    /// which every field of a template's frontmatter is, declares it, as
    /// TOML, when that's other than as a plain `"string"`.
    fn declared(&self, field: &str) -> Option<String> {
        self.types.iter().find_map(|t| self.declared_by(t, field))
    }

    /// How `declared` declares `field`, as [`Defaults::declared`] gives it.
    fn declared_by(&self, declared: &ContentType, field: &str) -> Option<String> {
        let found = declared
            .frontmatter
            .fields
            .iter()
            .find(|f| f.name == field)?;
        if found.ty != FieldType::String {
            return None;
        }
        let item = self
            .doc
            .as_ref()?
            .get("types")?
            .get(&declared.name)?
            .get("frontmatter")?
            .get(field)?;
        let text = match item {
            Item::Value(v) => {
                let mut v = v.clone();
                v.decor_mut().clear();
                v.to_string()
            }
            Item::Table(t) => {
                let mut t = t.clone().into_inline_table();
                t.fmt();
                t.to_string()
            }
            _ => return None,
        };
        (text != toml_string("string")).then_some(text)
    }
}

/// The fields of the frontmatter of the type `template` is added as, in
/// order, each with how another of the project's types declares it, when
/// it's copied from there. A field `[frontmatter]` declares is left out,
/// since the type has it already, and `title` is every type's.
fn frontmatter(template: &Template, defaults: &Defaults) -> Vec<(String, Option<String>)> {
    frontmatter_fields(template.text(&template.template))
        .into_iter()
        .filter(|field| !defaults.shared.contains(field))
        .map(|field| {
            let declared = defaults.declared(&field);
            (field, declared)
        })
        .collect()
}

/// `file` in the folder `to`.
fn at(to: &RelPath, file: &str) -> RelPath {
    to.join(file).unwrap_or_else(|_| to.clone())
}

/// A folder from the project's folder, as a person reads it.
fn shown(folder: &RelPath) -> String {
    if folder.is_root() {
        ".".to_owned()
    } else {
        folder.to_string()
    }
}

/// Whether `ascribe.toml`'s text declares a type: a `types` table with an
/// entry. An empty `[types]` declares none, and then, as with no `types` at
/// all, the loader gives every page the implicit type `page`.
fn declares_types(text: &str) -> bool {
    text.parse::<DocumentMut>().map_or(true, |doc| {
        doc.get("types")
            .is_some_and(|types| types.as_table_like().is_none_or(|t| !t.is_empty()))
    })
}

/// The lines of `[types.page]` as a content model with no types has it: a
/// type with nothing of its own, whose fields are `title` and
/// `[frontmatter]`'s (SPEC §7.2).
const PAGE_TYPE: &[&str] = &["[types.page]"];

/// `ascribe.toml`'s text with the `template` and `guide` of each type kept
/// added to its table, after its last entry, each a line of its own, and
/// nothing else changed. [`with_types`] checks the result.
fn with_kept(
    text: &str,
    doc: Option<&Document<&str>>,
    kept: &[Kept],
) -> Result<String, EjectError> {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut edits = Vec::new();
    for kept in kept {
        let refuse = |reason: &str| EjectError::Model {
            types: format!("`template` and `guide` of [types.{}]", kept.name),
            reason: reason.to_owned(),
        };
        let table =
            header_table(doc, &kept.name).ok_or_else(|| refuse("its table can't be found"))?;
        let (at, indent) = after_entries(text, table);
        let lines = format!(
            "{indent}template = {}{newline}{indent}guide = {}{newline}",
            toml_string(&kept.template),
            toml_list(&kept.guide)
        );
        let edit = if at == text.len() && !text.is_empty() && !text.ends_with('\n') {
            TextEdit::insert(at, format!("{newline}{}", lines.trim_end()))
        } else {
            TextEdit::insert(at, lines)
        };
        edits.push(edit);
    }
    apply_edits(text, &edits).map_err(|_| EjectError::Model {
        types: "`template` and `guide`".to_owned(),
        reason: "the edits overlap".to_owned(),
    })
}

/// Where a line goes after `table`'s last value, the start of the line after
/// it, or after the header when it has no values; and the white space
/// before that value's key, when it's all white space.
fn after_entries<'t>(text: &'t str, table: &Table) -> (usize, &'t str) {
    let last = table
        .iter()
        .filter_map(|(k, _)| table.get_key_value(k))
        .filter(|(_, item)| item.is_value())
        .max_by_key(|(_, item)| item.span().map_or(0, |s| s.end));
    let end = last
        .and_then(|(_, item)| item.span())
        .or_else(|| table.span())
        .map_or(text.len(), |s| s.end);
    let at = text
        .get(end..)
        .and_then(|rest| rest.find('\n'))
        .map_or(text.len(), |i| end + i + 1);
    let indent = last
        .and_then(|(key, _)| key.span())
        .and_then(|span| {
            let start = text[..span.start].rfind('\n').map_or(0, |i| i + 1);
            text.get(start..span.start)
        })
        .filter(|indent| indent.trim().is_empty())
        .unwrap_or("");
    (at, indent)
}

/// What the content model was before eject, to check the types kept against.
struct Before<'a> {
    /// The content model `ascribe.toml` loaded as.
    model: &'a ContentModel,
    /// The types kept.
    kept: &'a [Kept],
    /// The project's types, which the types added take their frontmatter
    /// from.
    defaults: &'a Defaults,
}

/// `ascribe.toml`'s text with a table for each type added at its end, and
/// nothing before it changed; with `page` first, when the file declared no
/// types. The result must load as a content model with the types in it, and
/// with each type kept as it was, but for its `template` and `guide`; it's
/// returned with the text.
fn with_types(
    text: &str,
    page: bool,
    templates: &[Template],
    added: &[Added],
    before: &Before<'_>,
) -> Result<(String, ContentModel), EjectError> {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = text.to_owned();
    let mut tables: Vec<Vec<String>> = Vec::new();
    if page {
        tables.push(PAGE_TYPE.iter().map(|l| (*l).to_owned()).collect());
    }
    for (template, added) in templates.iter().zip(added) {
        tables.push(type_table(template, added, before.defaults));
    }
    for table in tables {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push_str(newline);
        }
        if !out.is_empty() && !out.ends_with(&format!("{newline}{newline}")) {
            out.push_str(newline);
        }
        for line in table {
            out.push_str(&line);
            out.push_str(newline);
        }
    }
    let types = added
        .iter()
        .map(|a| format!("[types.{}]", a.name))
        .chain(
            before
                .kept
                .iter()
                .map(|k| format!("`template` and `guide` of [types.{}]", k.name)),
        )
        .collect::<Vec<_>>()
        .join(", ");
    let refuse = |reason: String| EjectError::Model {
        types: types.clone(),
        reason,
    };
    let model = ascribe_model::load_str(&out, ascribe_core::FileId::new(0)).map_err(|issues| {
        let slugs: Vec<&str> = issues.iter().map(|i| i.slug.as_str()).collect();
        refuse(format!(
            "the file wouldn't load with them ({})",
            slugs.join(", ")
        ))
    })?;
    if let Some(missing) = added
        .iter()
        .find(|a| !model.types.iter().any(|t| t.name == a.name))
    {
        return Err(refuse(format!(
            "[types.{}] wouldn't be read as a type",
            missing.name
        )));
    }
    for kept in before.kept {
        let was = before.model.types.iter().find(|t| t.name == kept.name);
        let is = model.types.iter().find(|t| t.name == kept.name);
        if !is
            .zip(was)
            .is_some_and(|(is, was)| same_but_template(is, was, kept))
        {
            return Err(refuse(format!(
                "[types.{}] wouldn't be read as it was with them added",
                kept.name
            )));
        }
    }
    for (template, added) in templates.iter().zip(added) {
        let is = model.types.iter().find(|t| t.name == added.name);
        if let Some(message) = is.and_then(|is| frontmatter_differs(is, template)) {
            return Err(refuse(format!(
                "with the frontmatter [types.{}] takes from the project's other types, {message}",
                added.name,
            )));
        }
    }
    Ok((out, model))
}

/// Whether `is` is `was` with `kept`'s `template` and `guide`, and nothing
/// else changed.
fn same_but_template(is: &ContentType, was: &ContentType, kept: &Kept) -> bool {
    is.frontmatter == was.frontmatter
        && is.purpose == was.purpose
        && is.template.as_ref().map(|t| t.path.as_str()) == Some(kept.template.as_str())
        && is
            .guide
            .iter()
            .map(|g| g.path.as_str())
            .eq(kept.guide.iter().map(String::as_str))
}

/// `s` as a TOML string.
fn toml_string(s: &str) -> String {
    toml_edit::Value::from(s).to_string()
}

/// `items` as a TOML array of strings, on one line.
fn toml_list(items: &[String]) -> String {
    let items: Vec<String> = items.iter().map(|s| toml_string(s)).collect();
    format!("[{}]", items.join(", "))
}

/// The lines of a type's tables: its template and guide, then its
/// frontmatter, which is the template's own: each field the template's
/// frontmatter has, as a string the page must have, `title` first, declared
/// as another of the project's types declares it when it's a string there.
fn type_table(template: &Template, added: &Added, defaults: &Defaults) -> Vec<String> {
    let key = |k: &str| toml_edit::Key::new(k).to_string();
    let name = key(&added.name);
    let mut lines = vec![
        format!("[types.{name}]"),
        format!("template = {}", toml_string(&added.template)),
        format!("guide = {}", toml_list(&added.guide)),
    ];
    let fields = frontmatter(template, defaults);
    if !fields.is_empty() {
        lines.push(String::new());
        lines.push(format!("[types.{name}.frontmatter]"));
    }
    for (field, declared) in fields {
        let declared = declared.unwrap_or_else(|| toml_string("string"));
        lines.push(format!("{} = {declared}", key(&field)));
    }
    lines
}

/// The names of the fields in a page's frontmatter, in order, but for
/// `type`, which names the page's type rather than declaring a field.
fn frontmatter_fields(page: &str) -> Vec<String> {
    let Some(Ok(serde_yaml_ng::Value::Mapping(map))) =
        frontmatter_text(page).map(serde_yaml_ng::from_str::<serde_yaml_ng::Value>)
    else {
        return Vec::new();
    };
    map.keys()
        .filter_map(|k| k.as_str().map(str::to_owned))
        .filter(|k| k != TYPE_KEY)
        .collect()
}

/// A page's frontmatter, between its `---` lines, when it has one.
fn frontmatter_text(page: &str) -> Option<&str> {
    let page = page.strip_prefix('\u{feff}').unwrap_or(page);
    let rest = page
        .strip_prefix("---\n")
        .or_else(|| page.strip_prefix("---\r\n"))?;
    if rest.starts_with("---") {
        return Some("");
    }
    rest.find("\n---\n")
        .or_else(|| rest.find("\n---\r\n"))
        .or_else(|| rest.find("\n---").filter(|at| at + 4 == rest.len()))
        .map(|end| &rest[..=end])
}

/// `templates.lock` as it is, or a new one when there's none.
fn read_lock(path: &RelPath, fs: &dyn FileSystem) -> Result<DocumentMut, EjectError> {
    let invalid = |reason: String| EjectError::Lock {
        path: path.to_string(),
        reason,
    };
    if fs.probe(path) == Probe::Missing && fs.real_path(path).is_none() {
        return format!("{LOCK_HEADER}version = {LOCK_VERSION}\n")
            .parse()
            .map_err(|e: toml_edit::TomlError| invalid(e.message().to_owned()));
    }
    let bytes = fs.read_file(path).map_err(|e| invalid(e.to_string()))?;
    parse_lock(bytes).map_err(invalid)
}

/// A `templates.lock`'s bytes, as a document of the version eject writes,
/// or why they aren't one.
pub(crate) fn parse_lock(bytes: Vec<u8>) -> Result<DocumentMut, String> {
    let text = String::from_utf8(bytes).map_err(|_| "it isn't valid UTF-8".to_owned())?;
    let doc: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| format!("it isn't TOML: {}", e.message()))?;
    match doc.get("version").and_then(Item::as_integer) {
        Some(LOCK_VERSION) => {}
        Some(other) => {
            return Err(format!(
                "its version is {other}, and this Ascribe writes version {LOCK_VERSION}"
            ));
        }
        None => return Err("it has no `version`".to_owned()),
    }
    if doc
        .get("template")
        .is_some_and(|t| t.as_array_of_tables().is_none())
    {
        return Err("its `template` isn't a list of tables".to_owned());
    }
    Ok(doc)
}

/// Records `template`'s files, as written, in the lock: in place of an entry
/// of the same name, or at the end. Their files are `files`, and Ascribe's
/// notes on the type are `ascribe-files`, so a later release of theirs is
/// compared with their files alone.
fn lock_entry(
    lock: &mut DocumentMut,
    set: &Set,
    template: &Template,
    theirs: &[(String, &str)],
    ours: &[(String, &str)],
    ascribe_version: &str,
) {
    let mut entry = Table::new();
    entry.insert("name", value(&template.name));
    entry.insert("summary", value(&template.summary));
    entry.insert("repository", value(&set.repository));
    entry.insert("tag", value(&set.tag));
    entry.insert("commit", value(&set.commit));
    entry.insert("license", value(&set.license));
    entry.insert("ascribe", value(ascribe_version));
    for (key, written) in [("files", theirs), ("ascribe-files", ours)] {
        if written.is_empty() {
            continue;
        }
        let mut files = Table::new();
        for (path, text) in written {
            files.insert(path, value(file_hash(text.as_bytes())));
        }
        entry.insert(key, Item::Table(files));
    }
    let item = lock
        .entry("template")
        .or_insert(Item::ArrayOfTables(ArrayOfTables::new()));
    if let Some(entries) = item.as_array_of_tables_mut() {
        entries.retain(|t| t.get("name").and_then(Item::as_str) != Some(&template.name));
        entries.push(entry);
    }
}

/// Writes `text` to `path`, from the project's folder, making its folder.
fn write(root: &Path, path: &str, text: &str) -> Result<(), EjectError> {
    let file = path.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    let failed = |e: std::io::Error| EjectError::Write {
        path: path.to_owned(),
        reason: e.to_string(),
    };
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(failed)?;
    }
    std::fs::write(&file, text).map_err(failed)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn a_template_s_frontmatter_names_its_type_s_fields() {
        assert_eq!(
            frontmatter_fields("---\ntitle: \"Title\"\ndescription: \"{x}\"\n---\n\nText\n"),
            ["title", "description"]
        );
        assert_eq!(frontmatter_fields("---\ntitle: T\n---"), ["title"]);
        assert!(frontmatter_fields("No frontmatter\n").is_empty());
    }

    #[test]
    fn a_crlf_file_is_added_to_with_crlf() {
        let set = set();
        let template = &set.templates[0];
        let model =
            ascribe_model::load_str("spec = \"0.1\"\n", ascribe_core::FileId::new(0)).unwrap();
        let defaults = Defaults::of(&model, "spec = \"0.1\"\n");
        let added = added(template, &RelPath::parse("templates").unwrap(), &defaults);
        let (text, _) = with_types(
            "spec = \"0.1\"\r\n",
            false,
            std::slice::from_ref(template),
            &[added],
            &Before {
                model: &model,
                kept: &[],
                defaults: &defaults,
            },
        )
        .unwrap();
        assert!(
            text.starts_with("spec = \"0.1\"\r\n\r\n[types."),
            "{text:?}"
        );
        assert!(!text.replace("\r\n", "").contains('\n'), "{text:?}");
    }
}
