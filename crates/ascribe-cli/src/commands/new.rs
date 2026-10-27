//! `ascribe new`: starts a page from its type's template, with its type and
//! title set, as a draft, as the editor's New Page does
//! (`ascribe_templates::start`), and writes it. It never writes over a
//! file: `start` refuses a path where anything is, and the file is opened
//! only if it isn't there.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::Project;
use ascribe_core::{Coded, RelPath};
use ascribe_templates::{NewPage, StartError};
use clap::{Args as ClapArgs, ValueEnum};
use serde::Serialize;

use crate::answer::{self, from_here};
use crate::cli::Global;
use crate::context::Failure;
use crate::exit;

/// Arguments of `ascribe new`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The page's type, as `ascribe model --section types` names it.
    #[arg(value_name = "TYPE")]
    pub type_name: String,

    /// Where the page goes, from the current folder, in the content root:
    /// a page's path, ending in .md, or a folder, where the page is named
    /// for its title.
    #[arg(value_name = "PATH")]
    pub path: PathBuf,

    /// The page's title, set in its frontmatter. Without one, the page
    /// keeps the template's, to fill in with the rest, and PATH names the
    /// page itself.
    #[arg(long, value_name = "TITLE")]
    pub title: Option<String>,

    /// Say where the page would go, and print its text, writing nothing.
    #[arg(long)]
    pub dry_run: bool,

    /// Taken as by every command that writes. `new` changes no file that's
    /// there, only adds one, so the git guard has nothing to refuse.
    #[arg(long)]
    pub force: bool,

    /// How to show what was written.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// How to show what `new` wrote.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Where the page went, and what to do next, for people and agents.
    #[default]
    #[value(alias = "concise")]
    Text,
    /// One JSON document, for tools.
    Json,
}

/// What `ascribe new` wrote, or with `--dry-run` would write.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct NewJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The page, from the folder `ascribe.toml` is in, `/`-separated.
    pub path: String,
    /// Its type.
    #[serde(rename = "type")]
    pub type_name: String,
    /// The title given, or `null` when the page keeps the template's.
    pub title: Option<String>,
    /// The type's template, from the folder `ascribe.toml` is in, or `null`
    /// when it has none and the page is its frontmatter alone.
    pub template: Option<String>,
    /// The type's guide, from the folder `ascribe.toml` is in, in the order
    /// to read it.
    pub guide: Vec<String>,
    /// How many places the page has that its template left to fill in, as
    /// `template-placeholder-left` finds them: its placeholders, and the
    /// frontmatter values still the template's.
    pub placeholders: usize,
    /// Whether the page was written: `false` with `--dry-run`.
    pub written: bool,
    /// The page's text.
    pub text: String,
}

/// The version of [`NewJson`].
const SCHEMA_VERSION: u32 = 1;

/// Why `ascribe new` couldn't start the page, before `start`'s own errors.
#[derive(Debug, thiserror::Error)]
pub enum NewError {
    /// PATH isn't in the content root.
    #[error("`{}` isn't in the content root, `{content}`: a page goes there", given.display())]
    Outside {
        /// The path, as given.
        given: PathBuf,
        /// The content root, from the current folder.
        content: String,
    },
    /// A file is where the page would go.
    #[error("`{path}` is there already, and `new` never writes over a file")]
    Exists {
        /// The file, from the current folder.
        path: String,
    },
    /// PATH is a folder, and no title was given to name the page in it.
    #[error("`{}` is a folder, where a page is named for its title: give --title, or a page's path ending in .md", given.display())]
    NoTitle {
        /// The path, as given.
        given: PathBuf,
    },
    /// The page couldn't be written.
    #[error("can't write `{path}`: {message}")]
    Write {
        /// The page, from the current folder.
        path: String,
        /// Why.
        message: String,
    },
}

impl Coded for NewError {
    fn code(&self) -> &'static str {
        match self {
            NewError::Outside { .. } => "page_outside_content",
            NewError::Exists { .. } => "page_exists",
            NewError::NoTitle { .. } => "page_path_needed",
            NewError::Write { .. } => "page_unwritable",
        }
    }
}

/// Runs the command. Exit codes: 0 when the page was written, or with
/// `--dry-run` would be; 2 when it wasn't: a type the model doesn't have, a
/// path outside the content root or where a file is, or a page that can't
/// be written.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = run_new(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn run_new(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let config = match answer::locate(global, Some(&args.path)) {
        Ok(config) => config,
        Err(e) => return answer::report_failure(err, &Failure::Config(e)),
    };
    let project = match ascribe_check::load(&config) {
        Ok(workspace) => Project::new(workspace.snapshot()),
        Err(e) => return answer::report_failure(err, &Failure::Load(e)),
    };
    // PATH is from the current folder, as every command's is.
    let [from_cwd, _] = answer::content_path_readings(&project, &args.path);
    let Some(path) = from_cwd else {
        let error = NewError::Outside {
            given: args.path.clone(),
            content: from_here(project.root(), project.content_root().as_str()),
        };
        return answer::fail(err, &error);
    };
    // The content root itself: the page goes in it, named for its title.
    let path = (!path.is_root()).then_some(path);
    let title = args.title.as_deref().map(str::trim);
    let page = match ascribe_templates::start(&project, &args.type_name, title, path.as_ref()) {
        Ok(page) => page,
        Err(StartError::FileInTheWay { path }) => {
            let error = NewError::Exists {
                path: from_here(
                    project.root(),
                    project
                        .content_root()
                        .join(&path)
                        .map_or(path.clone(), |p| p.to_string())
                        .as_str(),
                ),
            };
            return answer::fail(err, &error);
        }
        Err(StartError::NoPath { .. }) if title.is_none() => {
            let error = NewError::NoTitle {
                given: args.path.clone(),
            };
            return answer::fail(err, &error);
        }
        Err(e) => return answer::fail(err, &e),
    };
    let in_project = in_project(&project, &page.path);
    if !args.dry_run
        && let Err(e) = write_page(project.root(), &in_project, &page.text)
    {
        return answer::fail(err, &e);
    }
    let answer = NewJson {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("ASCRIBE_VERSION"),
        path: in_project.to_string(),
        type_name: args.type_name.clone(),
        title: title.map(str::to_owned),
        template: page.template.clone(),
        guide: page.guide.clone(),
        placeholders: project.template_places(&args.type_name, &page.text).len(),
        written: !args.dry_run,
        text: page.text.clone(),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &answer),
        Format::Text => write_text(out, project.root(), &answer, &page),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

/// The page's path from the folder `ascribe.toml` is in.
fn in_project(project: &Project, page: &RelPath) -> RelPath {
    project
        .content_root()
        .join(page.as_str())
        .unwrap_or_else(|_| page.clone())
}

/// Writes `text` at `path`, from `root`, making its folders, and only if
/// nothing is there: one that appeared since `start` looked is refused.
fn write_page(root: &Path, path: &RelPath, text: &str) -> Result<(), NewError> {
    let file = root.join(path.as_str());
    let shown = from_here(root, path.as_str());
    let failed = |e: io::Error| match e.kind() {
        io::ErrorKind::AlreadyExists => NewError::Exists {
            path: shown.clone(),
        },
        _ => NewError::Write {
            path: shown.clone(),
            message: e.to_string(),
        },
    };
    if let Some(folder) = file.parent() {
        std::fs::create_dir_all(folder).map_err(failed)?;
    }
    let mut written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file)
        .map_err(failed)?;
    written.write_all(text.as_bytes()).map_err(failed)
}

fn write_text(
    out: &mut dyn Write,
    root: &Path,
    answer: &NewJson,
    page: &NewPage,
) -> io::Result<()> {
    let shown = from_here(root, &answer.path);
    if answer.written {
        write!(out, "wrote {shown}, a page of type `{}`", answer.type_name)?;
    } else {
        write!(
            out,
            "would write {shown}, a page of type `{}`",
            answer.type_name
        )?;
    }
    match &page.template {
        Some(template) => writeln!(out, ", from {}", from_here(root, template))?,
        None => writeln!(out, ", which has no template")?,
    }
    if !page.guide.is_empty() {
        let guide: Vec<String> = page.guide.iter().map(|g| from_here(root, g)).collect();
        writeln!(out, "Read its guide before you write: {}", guide.join(", "))?;
    }
    if answer.placeholders > 0 {
        writeln!(
            out,
            "It has {} left to fill in, which `ascribe check {shown}` lists",
            if answer.placeholders == 1 {
                "1 placeholder".to_owned()
            } else {
                format!("{} placeholders", answer.placeholders)
            }
        )?;
    }
    writeln!(
        out,
        "It's a draft, with `draft: true`, which no build publishes: take the line out to publish it"
    )?;
    if !answer.written {
        writeln!(out)?;
        write!(out, "{}", answer.text)?;
    }
    Ok(())
}
