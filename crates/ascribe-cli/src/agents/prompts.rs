//! The named prompts: what `ascribe agents prompt <NAME>` prints, and the
//! MCP server's prompts, which hosts show as slash commands. Each follows
//! the agent prompt format (`ascribe_check::prompt`).
//!
//! - `new-page`: write a page of a type: where its file goes, the guide to
//!   read first, and the type's template with the title set, or, for a type
//!   with none, the frontmatter it requires;
//! - `fix`: fix what `ascribe check` reports: its prompt about the problems
//!   when there are any, else the loop of checking and fixing;
//! - `review`: review what the branch does to its pages, as readers see
//!   them: the prompt `ascribe diff --format prompt` writes.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use ascribe_check::Project;
use ascribe_check::prompt::{self, Builds, Context};
use ascribe_core::RelPath;
use ascribe_core::path::{normalize, relative_path};
use ascribe_model::ContentType;
use ascribe_query::Section;
use ascribe_query::model::ModelType;
use ascribe_templates::StartError;
use ascribe_templates::start;

use crate::answer::{self, Projects};
use crate::cli::Global;
use crate::commands::{check, diff};

/// A named prompt.
pub struct Named {
    /// Its name.
    pub name: &'static str,
    /// A short name for people.
    pub title: &'static str,
    /// What it asks an agent to do.
    pub description: &'static str,
    /// Its arguments: name, what it is, and whether it's required.
    pub arguments: &'static [(&'static str, &'static str, bool)],
    build: fn(&dyn Projects, &Arguments) -> Result<String, String>,
}

/// A prompt's arguments, by name.
pub type Arguments = std::collections::BTreeMap<String, String>;

/// The `path` argument every prompt takes.
const PATH: (&str, &str, bool) = (
    "path",
    "A file or folder in the project; by default, the current directory.",
    false,
);

/// Every named prompt, in order.
pub const PROMPTS: &[Named] = &[
    Named {
        name: "new-page",
        title: "New page",
        description: "Write a new page of a type: where its file goes, the guide to read \
            first, and the type's template to start from.",
        arguments: &[
            (
                "type",
                "The page type, as `ascribe model` lists them.",
                true,
            ),
            ("title", "The page's title.", true),
            (
                "path",
                "Where the page goes: its file, or a folder for it, in the content root, from \
                 the current directory or the content root. Any other file or folder names \
                 the project. By default, the type's first folder in the project at or above \
                 the current directory.",
                false,
            ),
        ],
        build: new_page,
    },
    Named {
        name: "fix",
        title: "Fix problems",
        description: "Check the project, fix what's reported, and check again.",
        arguments: &[(
            "path",
            "A file or folder to fix; by default, the project at or above the current \
             directory.",
            false,
        )],
        build: fix,
    },
    Named {
        name: "review",
        title: "Review changed pages",
        description: "Review what this branch does to its pages, as readers see them, and \
            report what reads wrongly.",
        arguments: &[
            PATH,
            (
                "base",
                "The git revision to compare with; by default, the default branch.",
                false,
            ),
        ],
        build: review,
    },
];

impl Named {
    /// The prompt for `arguments`.
    ///
    /// # Errors
    ///
    /// An argument it doesn't take, a required one missing, or a project
    /// that can't be read: the message says which.
    pub fn text(&self, projects: &dyn Projects, arguments: &Arguments) -> Result<String, String> {
        if let Some(unknown) = arguments
            .keys()
            .find(|k| !self.arguments.iter().any(|(name, _, _)| name == k))
        {
            return Err(format!(
                "`{}` takes no argument `{unknown}`; it takes {}",
                self.name,
                self.argument_names()
            ));
        }
        if let Some((missing, _, _)) = self
            .arguments
            .iter()
            .find(|(name, _, required)| *required && !arguments.contains_key(*name))
        {
            return Err(format!("`{}` needs `{missing}`", self.name));
        }
        (self.build)(projects, arguments)
    }

    fn argument_names(&self) -> String {
        self.arguments
            .iter()
            .map(|(name, _, _)| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The prompt named `name`.
pub fn find(name: &str) -> Option<&'static Named> {
    PROMPTS.iter().find(|p| p.name == name)
}

/// What the commands share when a prompt runs one: no `--config`, and no
/// color.
fn global() -> Global {
    Global {
        config: None,
        color: crate::cli::Color::Never,
    }
}

/// The folder the prompt is for: where the command, or the MCP server, was
/// run.
fn here() -> Option<PathBuf> {
    std::env::current_dir().ok()
}

fn path_of(arguments: &Arguments) -> Option<PathBuf> {
    arguments.get("path").map(PathBuf::from)
}

fn new_page(projects: &dyn Projects, arguments: &Arguments) -> Result<String, String> {
    let given = path_of(arguments);
    let loaded = answer::load(projects, &global(), given.as_deref())
        .map_err(|f| answer::Stop::Load(f).message())?;
    let project = &loaded.project;
    let type_name = arguments
        .get("type")
        .map(String::as_str)
        .unwrap_or_default();
    let title = arguments
        .get("title")
        .map(String::as_str)
        .unwrap_or_default();
    let types = ascribe_query::model(project.model(), Some(Section::Types), None)
        .types
        .unwrap_or_default();
    let (Some(page_type), Some(content_type)) = (
        types.iter().find(|t| t.name == type_name),
        project.model().types.iter().find(|t| t.name == type_name),
    ) else {
        return Err(format!(
            "the content model has no page type `{type_name}`; its types are {}",
            types
                .iter()
                .map(|t| format!("`{}`", t.name))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    };
    let context = Context::seen_from(project.root(), here().as_deref(), Builds::All);
    let content = project.content_root();
    let shown = |path: &RelPath| {
        let in_project = content.join(path.as_str()).unwrap_or_else(|_| path.clone());
        context.shown(in_project.as_str())
    };
    let content_dir = context.shown(content.as_str());
    let content_dir = if content_dir.is_empty() {
        ".".to_owned()
    } else {
        content_dir
    };
    let elsewhere = where_it_goes(&content_dir);
    let asked = given.as_deref().and_then(|given| page_path(project, given));
    // Where the page goes, unless a file is there or the title names no
    // file; then the agent picks another path of the type's, and starts from
    // the same text.
    let (place, text) = match ascribe_templates::start(project, type_name, title, asked.as_ref()) {
        Ok(page) if asked.is_some() => (format!("Where: `{}`.", shown(&page.path)), page.text),
        Ok(page) => (
            format!(
                "Where: where the request says the page goes, {elsewhere}; when it doesn't say, `{}`.",
                shown(&page.path)
            ),
            page.text,
        ),
        Err(StartError::FileInTheWay { path }) => (
            format!(
                "Where: not `{}`, which is there already, but {elsewhere}.",
                RelPath::parse(&path).map_or(path.clone(), |p| shown(&p))
            ),
            start::text(project, content_type, title).map_err(|e| e.to_string())?,
        ),
        Err(StartError::NoPath { .. }) => (
            format!("Where: {elsewhere}."),
            start::text(project, content_type, title).map_err(|e| e.to_string())?,
        ),
        Err(e) => return Err(e.to_string()),
    };
    let mut known = Vec::new();
    if let Some(guide) = guide(content_type, &context) {
        known.push(guide);
    }
    known.push(start_with(content_type, &text));
    if !content_type.implicit {
        known.push(format!(
            "The page says what type it is with `type: {}` in its frontmatter, as the text \
             to start from has it: keep it, wherever the page goes.",
            content_type.name
        ));
    }
    known.push(format!("The type's fields:\n{}", fields(page_type)));
    known.push(
        "A file or folder whose name starts with `_` is a fragment, not a page: don't name \
         the page that way."
            .to_owned(),
    );
    let cut = match &content_type.template {
        Some(template) => format!(
            "(cut: the type's template is `{}`, and `ascribe model --section types` lists \
             its fields)",
            context.shown(&template.path)
        ),
        None => "(cut: run `ascribe model --section types` for the rest)".to_owned(),
    };
    Ok(prompt::compose(
        &context,
        &format!(
            "Write a new page of type `{}` titled {}.",
            page_type.name,
            quoted_title(title)
        ),
        vec![place],
        &known,
        &cut,
        &[content_dir],
    ))
}

/// Where `given`, the `path` argument, asks for the page to go, from the
/// content root: a file or a folder in the content root, from the current
/// directory, or, when nothing is there, from the content root. `None` when
/// it names the project, or the content root itself.
fn page_path(project: &Project, given: &Path) -> Option<RelPath> {
    let absolute = |p: &Path| normalize(&std::path::absolute(p).unwrap_or_else(|_| p.to_owned()));
    let content = absolute(project.root()).join(project.content_root().as_str());
    let content = normalize(&content);
    let inside = |path: &Path| {
        relative_path(&content, path)
            .filter(RelPath::is_inside)
            .filter(|rel| !rel.is_root())
    };
    let from_here = absolute(given);
    if let Some(rel) = inside(&from_here) {
        return Some(rel);
    }
    // Outside FileSystem: whether the argument names something that's there,
    // which makes it the project's file or folder, not the page's.
    if given.is_absolute() || from_here.exists() {
        return None;
    }
    inside(&normalize(&content.join(given)))
}

/// A title in double quotes, its own double quotes escaped.
fn quoted_title(title: &str) -> String {
    format!("\"{}\"", title.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Where a page of any type can go: anywhere in the content root, since a
/// page names its type in its frontmatter (SPEC §7.2).
fn where_it_goes(content_dir: &str) -> String {
    format!("anywhere in the content root, `{content_dir}/`")
}

/// What to read before writing: the type's `guide`, in order, when it has
/// one. The files aren't put in the prompt: they're long, and a file the
/// agent reads arrives whole.
fn guide(page_type: &ContentType, context: &Context) -> Option<String> {
    if page_type.guide.is_empty() {
        return None;
    }
    let paths: Vec<String> = page_type
        .guide
        .iter()
        .map(|g| format!("`{}`", context.shown(&g.path)))
        .collect();
    Some(format!(
        "Before you write, read the type's guide, in this order: {}. It says what belongs in \
         each part of the page.",
        paths.join(", then ")
    ))
}

/// What the page starts with: the type's template with the title set, or,
/// for a type with none, its frontmatter.
fn start_with(page_type: &ContentType, text: &str) -> String {
    let fence = "`".repeat(longest_run(text, '`').max(2) + 1);
    let lead = if page_type.template.is_some() {
        "Start the file with this text, the type's template with the title set. Write over \
         each placeholder, the text in `{curly brackets}`, and each quoted instruction, and \
         leave out a part that's marked optional and doesn't apply: `ascribe check` reports \
         what's left of the template (`template-placeholder-left`)."
    } else {
        "Start the file with this frontmatter, and fill in each empty field:"
    };
    let info = if page_type.template.is_some() {
        "markdown"
    } else {
        "yaml"
    };
    format!("{lead}\n{fence}{info}\n{text}{fence}")
}

/// The most times `c` is repeated in a row in `text`.
fn longest_run(text: &str, c: char) -> usize {
    let (mut longest, mut run) = (0, 0);
    for ch in text.chars() {
        run = if ch == c { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    longest
}

/// One line per field: its name, type, whether it's required, the values it
/// allows, and what it's for.
fn fields(page_type: &ModelType) -> String {
    if page_type.fields.is_empty() {
        return "- none".to_owned();
    }
    page_type
        .fields
        .iter()
        .map(|f| {
            let mut line = format!("- `{}` ({}", f.name, f.field_type);
            if f.required {
                line.push_str(", required");
            }
            line.push(')');
            if let Some(description) = &f.description {
                line.push_str(&format!(": {description}"));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn fix(projects: &dyn Projects, arguments: &Arguments) -> Result<String, String> {
    let given = path_of(arguments);
    let args = check::Args {
        paths: given.iter().cloned().collect(),
        stdin: false,
        path: None,
        build: Vec::new(),
        editor_build: false,
        code: Vec::new(),
        summary: false,
        format: check::Format::Prompt,
        deny_warnings: false,
        fix: false,
        vale: false,
        vale_timeout: None,
        // The MCP server serves this prompt too, and starts no program.
        vale_runs: check::ValeRuns::Never,
    };
    let global = global();
    let outcome = check::run_check(projects, &global, &args, &mut Cursor::new(""));
    if let Ok(outcome) = &outcome
        && outcome.counts.total() > 0
    {
        let mut out = Vec::new();
        check::write(outcome, &global, &args, false, &mut out).map_err(|e| e.to_string())?;
        return Ok(String::from_utf8_lossy(&out).into_owned());
    }
    // No problems, or none that could be checked: the loop, which reports
    // what's wrong when it's run.
    let loaded = answer::load(projects, &global, given.as_deref())
        .map_err(|f| answer::Stop::Load(f).message())?;
    let context = Context::seen_from(loaded.project.root(), here().as_deref(), Builds::All);
    // The path as seen from the prompt's base, where its commands run.
    let in_project = given.as_deref().and_then(|path| {
        let absolute = |p: &std::path::Path| normalize(&std::path::absolute(p).unwrap_or_default());
        relative_path(&absolute(loaded.project.root()), &absolute(path))
            .filter(ascribe_core::RelPath::is_inside)
            .filter(|rel| !rel.is_root())
    });
    let targets: Vec<String> = match in_project {
        Some(rel) => vec![context.shown(rel.as_str())],
        None => context.folder.iter().cloned().collect(),
    };
    let concise = {
        let mut words = vec!["ascribe".to_owned(), "check".to_owned()];
        words.extend(targets.iter().map(|t| prompt::shell_word(t)));
        words.extend(["--format".to_owned(), "concise".to_owned()]);
        words.join(" ")
    };
    Ok(prompt::compose(
        &context,
        "Fix the problems `ascribe check` reports.",
        Vec::new(),
        &[
            format!("Run `{concise}`: it prints one line per problem, with what to change."),
            "Fix every error, and run it again. `ascribe explain <code>` says how to fix a \
             diagnostic, with an example. If the same errors remain after three rounds, stop \
             and say what's left."
                .to_owned(),
        ],
        "",
        &targets,
    ))
}

fn review(projects: &dyn Projects, arguments: &Arguments) -> Result<String, String> {
    let args = diff::Args {
        page: None,
        base: arguments.get("base").cloned(),
        base_exact: false,
        build: Vec::new(),
        format: diff::Format::Prompt,
        exit_code: false,
        pages_only: false,
    };
    let mut global = global();
    if let Some(path) = path_of(arguments) {
        global.config = Some(answer::locate(&global, Some(&path)).map_err(|e| e.to_string())?);
    }
    let (loaded, mut compared) = diff::answer(projects, &global, &args)?;
    if let Some(here) = here() {
        compared.seen_from(here);
    }
    Ok(
        diff::prompt(&loaded.project, &compared, &args)?.unwrap_or_else(|| {
            format!(
                "No page changed against {}, so there's nothing to review.\n",
                compared.report.base.requested
            )
        }),
    )
}
