//! The named prompts: what `ascribe agents prompt <NAME>` prints, and the
//! MCP server's prompts, which hosts show as slash commands. Each follows
//! the agent prompt format (`ascribe_check::prompt`).
//!
//! - `new-page`: write a page of a type, with the frontmatter the type
//!   requires and where its file goes;
//! - `fix`: fix what `ascribe check` reports: its prompt about the problems
//!   when there are any, else the loop of checking and fixing;
//! - `review`: review what the branch does to its pages, as readers see
//!   them: the prompt `ascribe diff --format prompt` writes.

use std::io::Cursor;
use std::path::PathBuf;

use ascribe_check::prompt::{self, Builds, Context};
use ascribe_core::path::{normalize, relative_path};
use ascribe_query::Section;
use ascribe_query::model::ModelType;

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
        description: "Write a new page of a type: the frontmatter the type requires, and \
            where its file goes.",
        arguments: &[
            (
                "type",
                "The page type, as `ascribe model` lists them.",
                true,
            ),
            ("title", "The page's title.", true),
            PATH,
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
    let types = ascribe_query::model(project.model(), Some(Section::Types))
        .types
        .unwrap_or_default();
    let Some(page_type) = types.iter().find(|t| t.name == type_name) else {
        return Err(format!(
            "the content model has no page type `{type_name}`; its types are {}",
            types
                .iter()
                .map(|t| format!("`{}`", t.name))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    };
    let context = Context::of_project(project.root(), Builds::All);
    let content = project.content_root().as_str();
    let content_dir = context.in_repository(content);
    let content_dir = if content_dir.is_empty() {
        ".".to_owned()
    } else {
        content_dir
    };
    let place = vec![where_it_goes(page_type, &types, &content_dir)];
    let mut known = vec![frontmatter(page_type, title)];
    known.push(format!("The type's fields:\n{}", fields(page_type)));
    known.push(
        "A file or folder whose name starts with `_` is a fragment, not a page: don't name \
         the page that way."
            .to_owned(),
    );
    Ok(prompt::compose(
        &context,
        &format!(
            "Write a new page of type `{}` titled {}.",
            page_type.name,
            quoted_title(title)
        ),
        place,
        &known,
        "(cut: run `ascribe model --section types` for the rest)",
        &[content_dir],
    ))
}

/// A title in double quotes, its own double quotes escaped.
fn quoted_title(title: &str) -> String {
    format!("\"{}\"", title.replace('\\', "\\\\").replace('"', "\\\""))
}

/// `Where:`, the folder and the patterns the page's path must match.
fn where_it_goes(page_type: &ModelType, types: &[ModelType], content_dir: &str) -> String {
    if !page_type.files.is_empty() {
        let patterns = page_type
            .files
            .iter()
            .map(|p| format!("`{p}`"))
            .collect::<Vec<_>>()
            .join(", ");
        return format!(
            "Where: in the content root, `{content_dir}/`, at a path matching {patterns} \
             (relative to it)."
        );
    }
    let others: Vec<String> = types
        .iter()
        .filter(|t| t.name != page_type.name)
        .flat_map(|t| t.files.iter().map(|p| format!("`{p}`")))
        .collect();
    if others.is_empty() {
        format!("Where: anywhere in the content root, `{content_dir}/`.")
    } else {
        format!(
            "Where: in the content root, `{content_dir}/`, at a path that matches none of \
             {} (those are other types' pages).",
            others.join(", ")
        )
    }
}

/// The page's frontmatter to start from: the title, and each other required
/// field to fill in.
fn frontmatter(page_type: &ModelType, title: &str) -> String {
    let mut lines = vec!["Start the file with this frontmatter, and fill in each `…`:".to_owned()];
    lines.push("```yaml".to_owned());
    lines.push("---".to_owned());
    let has_title = page_type.fields.iter().any(|f| f.name == "title");
    if has_title {
        lines.push(format!("title: {}", quoted_title(title)));
    }
    for field in page_type
        .fields
        .iter()
        .filter(|f| f.required && f.name != "title")
    {
        lines.push(format!("{}: …", field.name));
    }
    lines.push("---".to_owned());
    lines.push("```".to_owned());
    if !has_title {
        lines.push(format!(
            "The type has no `title` field: start the page with the heading `# {title}`."
        ));
    }
    lines.join("\n")
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
        summary: false,
        format: check::Format::Prompt,
        deny_warnings: false,
        vale: false,
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
    let context = Context::of_project(loaded.project.root(), Builds::All);
    // The path as seen from the repository's root, where the prompt's
    // commands run.
    let in_project = given.as_deref().and_then(|path| {
        let absolute = |p: &std::path::Path| normalize(&std::path::absolute(p).unwrap_or_default());
        relative_path(&absolute(loaded.project.root()), &absolute(path))
            .filter(ascribe_core::RelPath::is_inside)
            .filter(|rel| !rel.is_root())
    });
    let targets: Vec<String> = match in_project {
        Some(rel) => vec![context.in_repository(rel.as_str())],
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
    let (loaded, compared) = diff::answer(projects, &global, &args)?;
    Ok(
        diff::prompt(&loaded.project, &compared, &args)?.unwrap_or_else(|| {
            format!(
                "No page changed against {}, so there's nothing to review.\n",
                compared.report.base.requested
            )
        }),
    )
}
