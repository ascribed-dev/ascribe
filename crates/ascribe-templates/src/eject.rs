//! Writing templates into a project: `ascribe templates eject`.
//!
//! Each template of the set named is written as the project's own file, with
//! its guide, under a folder beside `ascribe.toml` (`templates/`, unless the
//! caller names another), and its type is added to `ascribe.toml`, at its
//! end, so the rest of the file stays as it was, comments and order with it.
//! `templates.lock`, in the same folder, records what each file was copied
//! from and a hash of it as written; a second eject adds to it.
//!
//! Everything is checked before anything is written: a type the project
//! declares already, or a file in the way, is refused, and then nothing is
//! written. From then on the files are the project's, and nothing here writes
//! over them.

use std::path::Path;

use ascribe_check::MODEL_FILE;
use ascribe_core::RelPath;
use ascribe_model::{ContentModel, file_hash};
use ascribe_resolve::{FileSystem, Layout, Probe};
use serde::Serialize;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};

use crate::{Set, Template, set};

/// The folder eject writes into by default, from the project's folder.
pub const TEMPLATES_DIR: &str = "templates";

/// The file, in the folder eject writes into, that records what each file
/// was copied from.
pub const LOCK_FILE: &str = "templates.lock";

/// The version of `templates.lock`'s format.
pub const LOCK_VERSION: i64 = 1;

/// The version of [`EjectAnswer`]'s schema.
pub const SCHEMA_VERSION: u32 = 1;

/// The version of Ascribe, which the answer carries.
const ASCRIBE_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    /// `/`-separated, in the order they were written: each template and its
    /// guide, then `templates.lock`, then `ascribe.toml`. Empty when anything
    /// was refused.
    pub written: Vec<String>,
    /// The types added to `ascribe.toml`, in the order they were named.
    /// Empty when anything was refused.
    pub added: Vec<Added>,
    /// What was refused, in the order it was named. When it isn't empty,
    /// nothing was written.
    pub refused: Vec<Refusal>,
    /// Whether `ascribe.toml` declared no types, so that every page was of
    /// the implicit type `page`, and `[types.page]` was written out first,
    /// as it was, so the pages outside the types added keep it.
    pub page_declared: bool,
}

/// A type eject added to `ascribe.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Added {
    /// The type's name, `[types.<name>]`, which is the template's.
    pub name: String,
    /// Their index's summary of it.
    pub summary: String,
    /// The pattern its `files` has, from the content root.
    pub files: String,
    /// The folder of the pages it covers, from the folder `ascribe.toml` is
    /// in.
    pub folder: String,
    /// Its `template`, from the folder `ascribe.toml` is in.
    pub template: String,
    /// Its `guide`, from the folder `ascribe.toml` is in.
    pub guide: Vec<String>,
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
}

/// Writes the templates `request` names into the project, with their guides
/// and `templates.lock`, and adds their types to `ascribe.toml`. What's there
/// is looked at through `fs`, the project's [`FileSystem`].
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
        refused: Vec::new(),
        page_declared: false,
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
    let lock = read_lock(&lock_path, fs)?;
    for template in &chosen {
        refusals(request, fs, template, &mut answer.refused);
    }
    if !answer.refused.is_empty() {
        return Ok(answer);
    }
    answer.added = chosen
        .iter()
        .map(|t| added(t, request.to, &layout))
        .collect();
    answer.page_declared = !declares_types(request.model_text);
    let model_text = with_types(
        request.model_text,
        answer.page_declared,
        &chosen,
        &answer.added,
    )?;
    let mut lock = lock;
    let mut files: Vec<(String, &'static str)> = Vec::new();
    for (template, added) in chosen.iter().zip(&answer.added) {
        let written = [
            (added.template.clone(), template.text(&template.template)),
            (added.guide[0].clone(), template.text(&template.guide)),
        ];
        lock_entry(&mut lock, &set, template, &written, request.ascribe_version);
        files.extend(written);
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

/// Why `template` can't be written, if it can't, in `refused`.
fn refusals(
    request: &Eject<'_>,
    fs: &dyn FileSystem,
    template: &Template,
    refused: &mut Vec<Refusal>,
) {
    let name = &template.name;
    if request.model.types.iter().any(|t| &t.name == name) {
        refused.push(Refusal {
            name: name.clone(),
            reason: RefusalReason::Declared,
            path: None,
            message: format!("ascribe.toml declares [types.{name}] already"),
        });
    }
    for file in [&template.template, &template.guide] {
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

/// The type `template` is added as.
fn added(template: &Template, to: &RelPath, layout: &Layout) -> Added {
    let folder = layout
        .content_root
        .join(&template.name)
        .unwrap_or_else(|_| layout.content_root.clone());
    Added {
        name: template.name.clone(),
        summary: template.summary.clone(),
        files: format!("{}/**", template.name),
        folder: folder.to_string(),
        template: at(to, &template.template).to_string(),
        guide: vec![at(to, &template.guide).to_string()],
    }
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

/// Whether `ascribe.toml`'s text declares a type, or has a `types` key at
/// all. Without one, every page is of the implicit type `page`.
fn declares_types(text: &str) -> bool {
    text.parse::<DocumentMut>()
        .map_or(true, |doc| doc.contains_key("types"))
}

/// The lines of `[types.page]` as a content model with no types has it: the
/// default, with a required title (contracts/content-model/validation.md,
/// decision 18).
const PAGE_TYPE: &[&str] = &[
    "[types.page]",
    "default = true",
    "",
    "[types.page.frontmatter]",
    "title = \"string\"",
];

/// `ascribe.toml`'s text with a table for each type added at its end, and
/// nothing before it changed; with `page` first, when the file declared no
/// types. The result must load as a content model with the types in it.
fn with_types(
    text: &str,
    page: bool,
    templates: &[Template],
    added: &[Added],
) -> Result<String, EjectError> {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = text.to_owned();
    let mut tables: Vec<Vec<String>> = Vec::new();
    if page {
        tables.push(PAGE_TYPE.iter().map(|l| (*l).to_owned()).collect());
    }
    for (template, added) in templates.iter().zip(added) {
        tables.push(type_table(template, added));
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
    Ok(out)
}

/// The lines of a type's tables: its files, template, and guide, then its
/// frontmatter, which is the template's own: each field the template's
/// frontmatter has, as a string the page must have, `title` first.
fn type_table(template: &Template, added: &Added) -> Vec<String> {
    let key = |k: &str| toml_edit::Key::new(k).to_string();
    let string = |s: &str| toml_edit::Value::from(s).to_string();
    let list = |items: &[String]| {
        let items: Vec<String> = items.iter().map(|s| string(s)).collect();
        format!("[{}]", items.join(", "))
    };
    let name = key(&added.name);
    let mut lines = vec![
        format!("[types.{name}]"),
        format!("files = {}", list(std::slice::from_ref(&added.files))),
        format!("template = {}", string(&added.template)),
        format!("guide = {}", list(&added.guide)),
        String::new(),
        format!("[types.{name}.frontmatter]"),
    ];
    let mut fields = frontmatter_fields(template.text(&template.template));
    if !fields.iter().any(|f| f == "title") {
        fields.insert(0, "title".to_owned());
    }
    for field in fields {
        lines.push(format!("{} = \"string\"", key(&field)));
    }
    lines
}

/// The names of the fields in a page's frontmatter, in order.
fn frontmatter_fields(page: &str) -> Vec<String> {
    let body = page.strip_prefix("---\n").and_then(|rest| {
        rest.find("\n---\n")
            .or_else(|| rest.find("\n---").filter(|at| at + 4 == rest.len()))
            .map(|end| &rest[..=end])
    });
    let Some(Ok(serde_yaml_ng::Value::Mapping(map))) =
        body.map(serde_yaml_ng::from_str::<serde_yaml_ng::Value>)
    else {
        return Vec::new();
    };
    map.keys()
        .filter_map(|k| k.as_str().map(str::to_owned))
        .collect()
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
    let text = String::from_utf8(bytes).map_err(|_| invalid("it isn't valid UTF-8".to_owned()))?;
    let doc: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| invalid(format!("it isn't TOML: {}", e.message())))?;
    match doc.get("version").and_then(Item::as_integer) {
        Some(LOCK_VERSION) => {}
        Some(other) => {
            return Err(invalid(format!(
                "its version is {other}, and this Ascribe writes version {LOCK_VERSION}"
            )));
        }
        None => return Err(invalid("it has no `version`".to_owned())),
    }
    if doc
        .get("template")
        .is_some_and(|t| t.as_array_of_tables().is_none())
    {
        return Err(invalid("its `template` isn't a list of tables".to_owned()));
    }
    Ok(doc)
}

/// Records `template`'s files, as written, in the lock: in place of an entry
/// of the same name, or at the end.
fn lock_entry(
    lock: &mut DocumentMut,
    set: &Set,
    template: &Template,
    written: &[(String, &str)],
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
    let mut files = Table::new();
    for (path, text) in written {
        files.insert(path, value(file_hash(text.as_bytes())));
    }
    entry.insert("files", Item::Table(files));
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
        let layout = Layout {
            content_root: RelPath::parse("docs").unwrap(),
            output_dir: RelPath::parse("dist").unwrap(),
        };
        let added = added(template, &RelPath::parse("templates").unwrap(), &layout);
        let text = with_types(
            "spec = \"0.1\"\r\n",
            false,
            std::slice::from_ref(template),
            &[added],
        )
        .unwrap();
        assert!(
            text.starts_with("spec = \"0.1\"\r\n\r\n[types."),
            "{text:?}"
        );
        assert!(!text.replace("\r\n", "").contains('\n'), "{text:?}");
    }
}
