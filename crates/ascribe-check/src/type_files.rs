//! The files a content type names, its `template` and its `guide`, and the
//! pages `[glossary] pages` lists.
//!
//! The content model's loader checks what's written (a path from the folder
//! `ascribe.toml` is in); this checks where each path leads, through the
//! project's [`FileSystem`]: the file is there, with that exact name, inside
//! the project's folder and outside its content root and its output
//! directory, symbolic links followed. A template is read and parsed too, so
//! one that isn't a page is the content model's problem, at the type, once,
//! and not a problem on every page made from it.
//!
//! A type that names neither a template nor a guide takes them from
//! `templates/templates.lock`, which `ascribe templates eject` writes, when
//! the lock records a template under the type's name ([`from_lock`]). A file
//! the lock gives is held to the same rules, and a problem with one is the
//! type's error, `model-type-template`, at the type.
//!
//! The reads are [`FileSystem::probe`], [`FileSystem::real_path`], and
//! [`FileSystem::read_file`], the reads of a file outside the content root
//! that a snippet's code file and `ascribe.lock` already take, so a template
//! and the lock need no read of their own.

use std::path::Path;

use ascribe_core::{FileId, Issue, LineIndex, Location, RelPath, diagnostics};
use ascribe_model::{ContentModel, ContentType, GlossaryPage, SourceBound, TypeFile};
use ascribe_resolve::{DiskFs, FileSystem, Layout, Probe};
use ascribe_syntax::{ParseOptions, parse};

use crate::diagnostic::Severity;
use crate::registry::Registry;

/// The folder `ascribe templates eject` writes into unless it's given
/// another, from the project's folder.
pub const TEMPLATES_DIR: &str = "templates";

/// The file, in the folder eject writes into, that records what each file
/// was copied from.
pub const LOCK_FILE: &str = "templates.lock";

/// The version of `templates.lock`'s format.
pub const LOCK_VERSION: i64 = 1;

/// The lock a type that names no template or guide takes them from:
/// `templates/templates.lock`, from the project's folder.
pub fn lock_path() -> RelPath {
    // Two plain segments always parse.
    #[allow(clippy::expect_used)]
    RelPath::parse(&format!("{TEMPLATES_DIR}/{LOCK_FILE}")).expect("a relative path")
}

/// Loads the content model in `text`, the `ascribe.toml` of the project in
/// `root`, as [`ascribe_model::load_str_bounded`] does, with what `[from]`
/// takes read through the project's [`DiskFs`], gives its types the
/// files `templates.lock` records for them ([`from_lock`]), and then
/// [`check`]s the files its types name, on disk. A problem with one fails the load, as
/// any of the model's errors does, and the issues returned include the
/// model's warnings.
///
/// # Errors
///
/// The content model's problems, at their places in `text`.
pub fn load_model(
    text: &str,
    root: &Path,
    bound: SourceBound<'_>,
) -> Result<ContentModel, Vec<Issue>> {
    load_model_changed(text, root, bound, &[])
}

/// [`load_model`], with the files `[from]` takes from that an edit has
/// changed read as `changed` has them: each the path `[from]` writes, and
/// its text after the edit.
///
/// # Errors
///
/// The content model's problems, at their places in `text`.
pub fn load_model_changed(
    text: &str,
    root: &Path,
    bound: SourceBound<'_>,
    changed: &[(String, String)],
) -> Result<ContentModel, Vec<Issue>> {
    let taking = Taking {
        disk: taking_fs(root),
        changed,
    };
    let mut model = ascribe_model::load_str_taking(text, FileId::new(0), root, bound, &taking)?;
    let disk = DiskFs::new(root, &Layout::from_model(&model));
    from_lock(&mut model, &disk);
    let problems = check(&model, &disk);
    if problems.is_empty() {
        return Ok(model);
    }
    Err(model.warnings.into_iter().chain(problems).collect())
}

/// The project's `FileSystem` for what `[from]` takes. It's read as the
/// lock is, as a file outside the content root; where the content root is
/// isn't known yet, and a read from the project's folder doesn't need it.
fn taking_fs(root: &Path) -> DiskFs {
    DiskFs::new(
        root,
        &Layout {
            content_root: RelPath::root(),
            output_dir: RelPath::root(),
        },
    )
}

/// The text of the file `[from]` names as `file` in the content model of
/// the project in `root`, read as the loader reads it: for an edit of a key
/// that file holds.
///
/// # Errors
///
/// The path isn't one, or the file isn't there or can't be read as text.
pub fn read_taken(root: &Path, file: &str) -> std::io::Result<String> {
    let path = RelPath::parse(file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
    ascribe_model::TakenFiles::read_taken(&taking_fs(root), &path)
}

/// What `[from]` takes, read from the disk unless an edit changed it.
struct Taking<'a> {
    disk: DiskFs,
    changed: &'a [(String, String)],
}

impl ascribe_model::TakenFiles for Taking<'_> {
    fn read_taken(&self, project_path: &RelPath) -> std::io::Result<String> {
        let changed = self
            .changed
            .iter()
            .find(|(file, _)| RelPath::parse(file).is_ok_and(|p| p == *project_path));
        match changed {
            Some((_, text)) => Ok(text.clone()),
            None => self.disk.read_taken(project_path),
        }
    }
}

/// Gives each type of `model` that names neither a `template` nor a `guide`
/// the files `templates/templates.lock`, read through `fs`, records under
/// the type's name: its template, and its guide, theirs and then Ascribe's
/// notes on the type, in the order to read them. A type that names either
/// keeps what it names. A lock that isn't there, or that can't be read as
/// one of the version eject writes, gives nothing: `ascribe templates
/// status` says what's wrong with it. Whether each file is there is
/// [`check`]'s.
pub fn from_lock(model: &mut ContentModel, fs: &dyn FileSystem) {
    let takes = |t: &ContentType| !t.implicit && t.template.is_none() && t.guide.is_empty();
    if !model.types.iter().any(takes) {
        return;
    }
    let lock = lock_path();
    let Some(entries) = lock_entries(&lock, fs) else {
        return;
    };
    for t in model.types.iter_mut().filter(|t| takes(t)) {
        let Some(entry) = entries.iter().find(|e| e.name == t.name) else {
            continue;
        };
        let file = |path: &String| TypeFile {
            path: path.clone(),
            span: t.span,
            lock: Some(lock.to_string()),
        };
        t.template = entry.template.as_ref().map(file);
        t.guide = entry.guide.iter().map(file).collect();
    }
}

/// What a lock records of one template.
struct LockEntry {
    /// The type's name.
    name: String,
    /// The template.
    template: Option<String>,
    /// Their guide, then Ascribe's notes, in the lock's order.
    guide: Vec<String>,
}

/// The templates the lock at `path` records, when it's there and of the
/// version eject writes. Of an entry's `files`, the one ending `.guide.md`
/// is their guide, and the other the template; its `ascribe-files` are
/// Ascribe's notes, read after the guide.
fn lock_entries(path: &RelPath, fs: &dyn FileSystem) -> Option<Vec<LockEntry>> {
    if fs.probe(path) != Probe::File {
        return None;
    }
    let text = String::from_utf8(fs.read_file(path).ok()?).ok()?;
    let lock: toml_edit::DocumentMut = text.parse().ok()?;
    if lock.get("version")?.as_integer()? != LOCK_VERSION {
        return None;
    }
    let entries = lock
        .get("template")
        .and_then(toml_edit::Item::as_array_of_tables)
        .into_iter()
        .flat_map(|t| t.iter())
        .filter_map(|table| {
            let name = table.get("name")?.as_str()?.to_owned();
            let listed = |key: &str| -> Vec<String> {
                table
                    .get(key)
                    .and_then(toml_edit::Item::as_table_like)
                    .map(|files| files.iter().map(|(path, _)| path.to_owned()).collect())
                    .unwrap_or_default()
            };
            let (guide, template): (Vec<String>, Vec<String>) = listed("files")
                .into_iter()
                .partition(|p| p.ends_with(".guide.md"));
            Some(LockEntry {
                name,
                template: template.into_iter().next(),
                guide: guide.into_iter().chain(listed("ascribe-files")).collect(),
            })
        })
        .collect();
    Some(entries)
}

/// The problems with the files the content model's types name, located in
/// `ascribe.toml` (file id 0) at each path's string, in the order the types
/// and their keys are declared.
pub fn check(model: &ContentModel, fs: &dyn FileSystem) -> Vec<Issue> {
    let layout = Layout::from_model(model);
    let mut out = Vec::new();
    for t in &model.types {
        if let Some(template) = &t.template
            && let Some(path) = place(&t.name, "template", template, &layout, fs, &mut out)
        {
            read_template(model, &t.name, template, &path, fs, &mut out);
        }
        for guide in &t.guide {
            place(&t.name, "guide", guide, &layout, fs, &mut out);
        }
    }
    for page in &model.glossary.pages {
        glossary_page(page, &layout, fs, &mut out);
    }
    out
}

/// A problem with a page `[glossary] pages` lists, in `out`: it isn't there,
/// with that exact name. The loader has turned away a fragment.
fn glossary_page(page: &GlossaryPage, layout: &Layout, fs: &dyn FileSystem, out: &mut Vec<Issue>) {
    let Ok(path) = RelPath::parse(&page.path) else {
        return;
    };
    let issue = Issue::new(
        diagnostics::MODEL_GLOSSARY_PAGE,
        Location::new(FileId::new(0), page.span),
    )
    .with_arg("path", page.path.as_str());
    match fs.probe(&layout.project_path(&path)) {
        Probe::File => {}
        Probe::Missing => out.push(issue),
        Probe::CaseMismatch(actual) => {
            let actual = actual.relative_to(&layout.content_root).unwrap_or(actual);
            out.push(
                issue
                    .with_variant("case")
                    .with_arg("actual", actual.as_str()),
            );
        }
    }
}

/// Where `file` leads, when it's a file the type may name; otherwise the
/// problem, in `out`.
fn place(
    type_name: &str,
    key: &str,
    file: &TypeFile,
    layout: &Layout,
    fs: &dyn FileSystem,
    out: &mut Vec<Issue>,
) -> Option<RelPath> {
    match lead(file, layout, fs) {
        Ok(path) => Some(path),
        Err(problem) => {
            out.push(problem.issue(type_name, key, file, layout));
            None
        }
    }
}

/// Where `file` leads, or why the type can't name it.
fn lead(file: &TypeFile, layout: &Layout, fs: &dyn FileSystem) -> Result<RelPath, Problem> {
    // The loader has turned away an absolute path, and a relative one with
    // no NUL always parses.
    let Ok(path) = RelPath::parse(&file.path) else {
        return Err(Problem::Place(Place::Outside));
    };
    if let Some(place) = where_it_is(&path, layout) {
        return Err(Problem::Place(place));
    }
    match fs.probe(&path) {
        Probe::File => {}
        Probe::Missing => return Err(Problem::Missing),
        Probe::CaseMismatch(actual) => return Err(Problem::Case(actual)),
    }
    // Through a symbolic link, the file is where the link leads.
    let Some(real) = fs.real_path(&path) else {
        return Err(Problem::Place(Place::Link));
    };
    let real_layout = Layout {
        content_root: fs
            .real_path(&layout.content_root)
            .unwrap_or_else(|| layout.content_root.clone()),
        output_dir: fs
            .real_path(&layout.output_dir)
            .unwrap_or_else(|| layout.output_dir.clone()),
    };
    if let Some(place) = where_it_is(&real, &real_layout) {
        return Err(Problem::Place(match place {
            Place::Outside => Place::Link,
            other => other,
        }));
    }
    Ok(path)
}

/// Why a type can't name a file.
enum Problem {
    /// It isn't there.
    Missing,
    /// It isn't there, and this file's name differs from it only in case.
    Case(RelPath),
    /// It's somewhere a type can't name a file.
    Place(Place),
}

impl Problem {
    /// The issue, at `file`: `model-type-file` for a file `ascribe.toml`
    /// names, and `model-type-template`, the type's own error, for one a
    /// lock gives it.
    fn issue(self, type_name: &str, key: &str, file: &TypeFile, layout: &Layout) -> Issue {
        let at = Location::new(FileId::new(0), file.span);
        if let Some(lock) = &file.lock {
            return Issue::new(diagnostics::MODEL_TYPE_TEMPLATE, at)
                .with_variant("lock")
                .with_arg("lock", lock.as_str())
                .with_arg("path", file.path.as_str())
                .with_arg("key", key)
                .with_arg("type", type_name)
                .with_arg("reason", self.words(layout));
        }
        let issue = Issue::new(diagnostics::MODEL_TYPE_FILE, at)
            .with_arg("path", file.path.as_str())
            .with_arg("key", key)
            .with_arg("type", type_name);
        match self {
            Problem::Missing => issue,
            Problem::Case(actual) => issue
                .with_variant("case")
                .with_arg("actual", actual.as_str()),
            Problem::Place(place) => place.issue(issue, layout),
        }
    }

    /// Why, in words that follow "and".
    fn words(&self, layout: &Layout) -> String {
        match self {
            Problem::Missing => "it isn't there".to_owned(),
            Problem::Case(actual) => {
                format!("it isn't there; `{actual}` differs from it only in case")
            }
            Problem::Place(place) => place.words(layout),
        }
    }
}

/// Why a type can't name a file at `path`.
enum Place {
    /// It's above the project's folder.
    Outside,
    /// A link on the way leads above the project's folder.
    Link,
    /// It's in the content root, where it would be read as a page.
    Content,
    /// It's in the output directory, which a build writes and removes.
    Output,
}

impl Place {
    /// `issue`, with this place's variant.
    fn issue(&self, issue: Issue, layout: &Layout) -> Issue {
        match self {
            Place::Outside => issue.with_variant("outside"),
            Place::Link => issue.with_variant("link"),
            Place::Content => issue
                .with_variant("inside-content")
                .with_arg("content", layout.content_root.as_str()),
            Place::Output => issue
                .with_variant("output")
                .with_arg("output", layout.output_dir.as_str()),
        }
    }

    /// Why no type can name a file here, in words.
    fn words(&self, layout: &Layout) -> String {
        match self {
            Place::Outside => "it's outside the folder ascribe.toml is in".to_owned(),
            Place::Link => {
                "a symbolic link on the way leads outside the folder ascribe.toml is in".to_owned()
            }
            Place::Content => format!(
                "it's in the content root, `{}`, where its files would be read as pages",
                layout.content_root
            ),
            Place::Output => format!(
                "it's in the output directory, `{}`, which a build writes and removes",
                layout.output_dir
            ),
        }
    }
}

/// Why no type can name a file in `folder`, a path from the project's
/// folder, in words, when none can: it's above the project's folder, a link
/// on the way leads there, or it's in the content root or the output
/// directory. The rule [`check`] holds each file to, for a folder that
/// needn't exist yet: `ascribe templates eject` asks it before writing one.
pub fn folder_problem(folder: &RelPath, layout: &Layout, fs: &dyn FileSystem) -> Option<String> {
    let words = |place: Place| place.words(layout);
    if let Some(place) = where_it_is(folder, layout) {
        return Some(words(place));
    }
    let real = fs.real_path(folder)?;
    let real_layout = Layout {
        content_root: fs
            .real_path(&layout.content_root)
            .unwrap_or_else(|| layout.content_root.clone()),
        output_dir: fs
            .real_path(&layout.output_dir)
            .unwrap_or_else(|| layout.output_dir.clone()),
    };
    where_it_is(&real, &real_layout).map(|place| match place {
        Place::Outside => words(Place::Link),
        other => words(other),
    })
}

fn where_it_is(path: &RelPath, layout: &Layout) -> Option<Place> {
    if path.up_count() > 0 {
        return Some(Place::Outside);
    }
    // A content root of only `..`s holds the project's folder, and so every
    // path in it.
    if path.relative_to(&layout.content_root).is_some()
        || layout.content_root.segments().all(|s| s == "..")
    {
        return Some(Place::Content);
    }
    if path.relative_to(&layout.output_dir).is_some() {
        return Some(Place::Output);
    }
    None
}

/// Reads and parses the template at `path`: it's a page, with what a page
/// can have, and nothing that stops one being read.
fn read_template(
    model: &ContentModel,
    type_name: &str,
    file: &TypeFile,
    path: &RelPath,
    fs: &dyn FileSystem,
    out: &mut Vec<Issue>,
) {
    let issue = || {
        Issue::new(
            diagnostics::MODEL_TYPE_TEMPLATE,
            Location::new(FileId::new(0), file.span),
        )
        .with_arg("path", file.path.as_str())
        .with_arg("type", type_name)
    };
    let text = match fs.read_file(path) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => {
                out.push(issue().with_arg("reason", "it isn't valid UTF-8"));
                return;
            }
        },
        Err(e) => {
            out.push(issue().with_arg("reason", e.to_string()));
            return;
        }
    };
    let lines = LineIndex::new(&text);
    let line = |offset: usize| {
        lines
            .line_col(offset)
            .map_or(1, |at| at.line as usize + 1)
            .to_string()
    };
    let parse_problem = |at: usize, detail: String| {
        issue()
            .with_variant("parse")
            .with_arg("line", line(at))
            .with_arg("detail", detail)
    };
    let options = ParseOptions::new(model.directive_schemas())
        .with_note_types(model.notes.iter().map(|n| n.name.clone()).collect());
    let doc = parse(&text, &options);
    if let Some(fm) = &doc.frontmatter {
        let content = text.get(fm.content.range()).unwrap_or_default();
        if !content.trim().is_empty()
            && let Err(e) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(content)
        {
            let at = e
                .location()
                .map_or(fm.content.start(), |l| fm.content.start() + l.index());
            let detail = format!("the frontmatter isn't valid YAML: {e}");
            out.push(parse_problem(at, detail));
            return;
        }
    }
    let registry = Registry::global();
    let first_error = doc.issues.iter().find(|i| {
        registry
            .get(i.slug)
            .is_some_and(|entry| entry.severity == Severity::Error)
    });
    if let Some(problem) = first_error {
        let detail = registry.message(problem);
        out.push(parse_problem(problem.location.span.start(), detail));
    }
}
