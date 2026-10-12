//! The files a content type names: its `template` and its `guide`.
//!
//! The content model's loader checks what's written (a path from the folder
//! `ascribe.toml` is in); this checks where each path leads, through the
//! project's [`FileSystem`]: the file is there, with that exact name, inside
//! the project's folder and outside its content root and its output
//! directory, symbolic links followed. A template is read and parsed too, so
//! one that isn't a page is the content model's problem, at the type, once,
//! and not a problem on every page made from it.
//!
//! The reads are [`FileSystem::probe`], [`FileSystem::real_path`], and
//! [`FileSystem::read_file`], the reads of a file outside the content root
//! that a snippet's code file and `ascribe.lock` already take, so a template
//! needs no read of its own.

use std::path::Path;

use ascribe_core::{FileId, Issue, LineIndex, Location, RelPath, diagnostics};
use ascribe_model::{ContentModel, SourceBound, TypeFile};
use ascribe_resolve::{DiskFs, FileSystem, Layout, Probe};
use ascribe_syntax::{ParseOptions, parse};

use crate::diagnostic::Severity;
use crate::registry::Registry;

/// Loads the content model in `text`, the `ascribe.toml` of the project in
/// `root`, as [`ascribe_model::load_str_bounded`] does, and then [`check`]s
/// the files its types name, on disk. A problem with one fails the load, as
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
    let model = ascribe_model::load_str_bounded(text, FileId::new(0), root, bound)?;
    let disk = DiskFs::new(root, &Layout::from_model(&model));
    let problems = check(&model, &disk);
    if problems.is_empty() {
        return Ok(model);
    }
    Err(model.warnings.into_iter().chain(problems).collect())
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
    out
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
    let issue = |variant: Option<&'static str>| {
        let issue = Issue::new(
            diagnostics::MODEL_TYPE_FILE,
            Location::new(FileId::new(0), file.span),
        )
        .with_arg("path", file.path.as_str())
        .with_arg("key", key)
        .with_arg("type", type_name);
        match variant {
            Some(variant) => issue.with_variant(variant),
            None => issue,
        }
    };
    // The loader has turned away an absolute path, and a relative one with
    // no NUL always parses.
    let Ok(path) = RelPath::parse(&file.path) else {
        out.push(issue(Some("outside")));
        return None;
    };
    if let Some(problem) = where_it_is(&path, layout) {
        out.push(problem.issue(issue, layout));
        return None;
    }
    match fs.probe(&path) {
        Probe::File => {}
        Probe::Missing => {
            out.push(issue(None));
            return None;
        }
        Probe::CaseMismatch(actual) => {
            out.push(issue(Some("case")).with_arg("actual", actual.as_str()));
            return None;
        }
    }
    // Through a symbolic link, the file is where the link leads.
    let Some(real) = fs.real_path(&path) else {
        out.push(issue(Some("link")));
        return None;
    };
    let real_layout = Layout {
        content_root: fs
            .real_path(&layout.content_root)
            .unwrap_or_else(|| layout.content_root.clone()),
        output_dir: fs
            .real_path(&layout.output_dir)
            .unwrap_or_else(|| layout.output_dir.clone()),
    };
    if let Some(problem) = where_it_is(&real, &real_layout) {
        let problem = match problem {
            Place::Outside => Place::Link,
            other => other,
        };
        out.push(problem.issue(issue, layout));
        return None;
    }
    Some(path)
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
    fn issue(self, issue: impl Fn(Option<&'static str>) -> Issue, layout: &Layout) -> Issue {
        match self {
            Place::Outside => issue(Some("outside")),
            Place::Link => issue(Some("link")),
            Place::Content => {
                issue(Some("inside-content")).with_arg("content", layout.content_root.as_str())
            }
            Place::Output => issue(Some("output")).with_arg("output", layout.output_dir.as_str()),
        }
    }
}

/// Why no type can name a file in `folder`, a path from the project's
/// folder, in words, when none can: it's above the project's folder, a link
/// on the way leads there, or it's in the content root or the output
/// directory. The rule [`check`] holds each file to, for a folder that
/// needn't exist yet: `ascribe templates eject` asks it before writing one.
pub fn folder_problem(folder: &RelPath, layout: &Layout, fs: &dyn FileSystem) -> Option<String> {
    let words = |place: Place| match place {
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
    };
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
