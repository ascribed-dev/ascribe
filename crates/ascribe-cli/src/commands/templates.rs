//! `ascribe templates`: the page templates Ascribe carries. `list` shows the
//! set, and `eject` writes templates into the project, as its own files, and
//! declares their types (`ascribe_templates::eject`).

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::{LocateError, Project};
use ascribe_core::Coded;
use ascribe_core::RelPath;
use ascribe_core::path::{normalize, relative_path};
use ascribe_resolve::{DiskFs, Layout};
use ascribe_templates::{EjectAnswer, TEMPLATES_DIR};
use clap::{Args as ClapArgs, Subcommand};

use crate::answer::{self, Format, from_here};
use crate::cli::Global;
use crate::context::{Failure, locate};
use crate::exit;

/// Arguments of `ascribe templates`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe templates`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show the templates Ascribe carries, each with what it's for. Reads
    /// nothing in the project.
    #[command(after_help = docs_link!("cli-templates-list"))]
    List(ListArgs),
    /// Write templates into the project, with their guides, as its own
    /// files, and declare each one's type in ascribe.toml.
    ///
    /// Writes nothing when a type is declared already, or a file is where
    /// one would be written.
    #[command(after_help = docs_link!("cli-templates-eject"))]
    Eject(EjectArgs),
}

/// Arguments of `ascribe templates list`.
#[derive(Debug, ClapArgs)]
pub struct ListArgs {}

/// Arguments of `ascribe templates eject`.
#[derive(Debug, ClapArgs)]
pub struct EjectArgs {
    /// The templates to write, by name, as `ascribe templates list` shows
    /// them. Without a name, every one.
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,

    /// The folder to write them in, from the current folder: inside the
    /// folder ascribe.toml is in, and outside the content root. By default,
    /// templates/ beside ascribe.toml.
    #[arg(long, value_name = "DIR")]
    pub to: Option<PathBuf>,

    /// How to show what was written.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Why `ascribe templates eject` couldn't do its work, before the library's
/// own errors.
#[derive(Debug, thiserror::Error)]
pub enum TemplatesError {
    /// There's no `ascribe.toml` to add to.
    #[error(
        "{0}\neject adds templates to a project, and this folder isn't in one: to start a project here with the templates, run `ascribe init --templates`"
    )]
    NoProject(LocateError),
    /// `--to` names a folder outside the project's.
    #[error("--to names `{}`, which isn't inside the folder ascribe.toml is in, `{}`", to.display(), root.display())]
    ToOutside {
        /// The folder, as given.
        to: PathBuf,
        /// The project's folder.
        root: PathBuf,
    },
}

impl Coded for TemplatesError {
    fn code(&self) -> &'static str {
        match self {
            TemplatesError::NoProject(e) => e.code(),
            TemplatesError::ToOutside { .. } => "templates_folder_refused",
        }
    }
}

/// Runs the command.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = match args.command {
        Command::List(_) => run_list(&mut out, &mut err),
        Command::Eject(args) => run_eject(global, &args, &mut out, &mut err),
    };
    let _ = out.flush();
    exit::code(code)
}

/// Exit codes: 0, or 2 when the list can't be written.
fn run_list(out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let set = ascribe_templates::set();
    let width = set
        .templates
        .iter()
        .map(|t| t.name.len())
        .max()
        .unwrap_or(0);
    let result = (|| {
        for template in &set.templates {
            writeln!(out, "{:width$}  {}", template.name, template.summary)?;
        }
        writeln!(
            out,
            "From The Good Docs Project's templates, {}, at {}, under the {}.",
            set.repository, set.tag, set.license
        )
    })();
    answer::written(result, err).unwrap_or(exit::OK)
}

/// Exit codes: 0 when every template named was written and its type added;
/// 2 when anything was refused, and nothing was written, or it couldn't run.
fn run_eject(global: &Global, args: &EjectArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let config = match locate(global) {
        Ok(config) => config,
        Err(e @ LocateError::NotFound { .. }) => {
            return answer::fail(err, &TemplatesError::NoProject(e));
        }
        Err(e) => return answer::fail(err, &Failure::Config(e)),
    };
    let file = match Project::load_model(&config) {
        Ok(file) => file,
        Err(e) => return answer::report_failure(err, &Failure::Load(e)),
    };
    let to = match &args.to {
        None => RelPath::parse(TEMPLATES_DIR).unwrap_or_default(),
        Some(to) => match folder_in_project(&file.root, to) {
            Some(rel) => rel,
            None => {
                let error = TemplatesError::ToOutside {
                    to: to.clone(),
                    root: file.root.clone(),
                };
                return answer::fail(err, &error);
            }
        },
    };
    let fs = DiskFs::new(&file.root, &Layout::from_model(&file.model));
    let request = ascribe_templates::Eject {
        root: &file.root,
        model: &file.model,
        model_text: &file.text,
        names: &args.names,
        to: &to,
        ascribe_version: env!("ASCRIBE_VERSION"),
    };
    let ejected = match ascribe_templates::eject(&request, &fs) {
        Ok(ejected) => ejected,
        Err(e) => return answer::fail(err, &e),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &ejected),
        Format::Text => write_eject(out, err, &file.root, &ejected),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if ejected.refused.is_empty() {
        exit::OK
    } else {
        exit::FAILURE
    }
}

/// `to`, a folder from the current folder, as a path from the project's
/// folder, when it's inside it.
fn folder_in_project(root: &Path, to: &Path) -> Option<RelPath> {
    let cwd = std::env::current_dir().ok()?;
    let root = normalize(&std::path::absolute(root).ok()?);
    relative_path(&root, &normalize(&cwd.join(to))).filter(RelPath::is_inside)
}

fn write_eject(
    out: &mut dyn Write,
    err: &mut dyn Write,
    root: &Path,
    ejected: &EjectAnswer,
) -> io::Result<()> {
    if !ejected.refused.is_empty() {
        return write_refused(err, ejected);
    }
    for path in &ejected.written {
        writeln!(out, "wrote {}", from_here(root, path))?;
    }
    write_added(out, root, ejected)
}

/// What eject refused, on standard error.
pub(crate) fn write_refused(err: &mut dyn Write, ejected: &EjectAnswer) -> io::Result<()> {
    for refusal in &ejected.refused {
        writeln!(err, "error: {}: {}", refusal.name, refusal.message)?;
    }
    writeln!(err, "wrote nothing")
}

/// The types eject added, and how to start a page of one.
pub(crate) fn write_added(
    out: &mut dyn Write,
    root: &Path,
    ejected: &EjectAnswer,
) -> io::Result<()> {
    if ejected.page_declared {
        writeln!(
            out,
            "added [types.page] to {}: it declared no types, so every page was of this one, and the pages outside the new types keep it",
            from_here(root, ascribe_check::MODEL_FILE)
        )?;
    }
    for added in &ejected.added {
        writeln!(
            out,
            "added [types.{}] to {}, for pages under {}/",
            added.name,
            from_here(root, ascribe_check::MODEL_FILE),
            from_here(root, &added.folder)
        )?;
    }
    if !ejected.added.is_empty() {
        writeln!(
            out,
            "A type's folder is only a start: to keep its pages elsewhere, change the type's `files` in {}.",
            from_here(root, ascribe_check::MODEL_FILE)
        )?;
    }
    match ejected.added.as_slice() {
        [] => {}
        [one] => writeln!(
            out,
            "To start a page, copy {} into {}/, and read {} first.",
            from_here(root, &one.template),
            from_here(root, &one.folder),
            one.guide
                .iter()
                .map(|g| from_here(root, g))
                .collect::<Vec<_>>()
                .join(" and ")
        )?,
        _ => writeln!(
            out,
            "To start a page, copy its type's template into the type's folder, and read its guide and Ascribe's notes on it first."
        )?,
    }
    Ok(())
}
