//! `ascribe preview`: one build's pages as a site, with Ascribe alone.
//!
//! The build is checked as `ascribe check --build` checks it, and its
//! problems printed as that prints them; then its pages are rendered with
//! the archive's renderer ([`ascribe_emit::preview`]), errors or not, into
//! `.ascribe/preview/`, which is served at `127.0.0.1` ([`serve`]). The
//! project's files are listed again every few hundred milliseconds, as the
//! MCP server lists them ([`crate::mcp::cache::watched`]), and any change
//! renders the build again. It runs until it's stopped.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use ascribe_core::Coded;
use ascribe_emit::OutputDir;
use clap::Args as ClapArgs;

use crate::answer::{self, FromDisk};
use crate::cli::Global;
use crate::commands::check;
use crate::context::{stdout_is_terminal, use_color};
use crate::exit;
use crate::mcp::cache::{self, Stamp};

mod serve;

/// The folder the preview is written in, from the project's root.
pub const FOLDER: &str = ".ascribe/preview";

/// The port asked for without `--port`. When it's taken, the system picks
/// a free one.
pub const PORT: u16 = 4880;

/// How long between two listings of the project's files.
const POLL: Duration = Duration::from_millis(300);

/// Arguments of `ascribe preview`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The build to preview.
    ///
    /// Needed when `ascribe.toml` has more than one build. Of a versioned
    /// build, its current line.
    #[arg(long, value_name = "NAME")]
    pub build: Option<String>,

    /// The port to serve the preview on, at 127.0.0.1.
    ///
    /// By default 4880, or a free port the system picks when 4880 is taken.
    #[arg(long, value_name = "PORT")]
    pub port: Option<u16>,
}

/// Why the preview can't be served.
#[derive(Debug, thiserror::Error)]
pub enum PreviewError {
    /// The port can't be listened on.
    #[error("can't serve the preview at {address}: {message}")]
    Listen {
        /// The address asked for.
        address: String,
        /// Why.
        message: String,
    },
}

impl Coded for PreviewError {
    fn code(&self) -> &'static str {
        match self {
            PreviewError::Listen { .. } => "preview_port_unavailable",
        }
    }
}

/// Runs the command, until it's stopped. Exit codes: 2 when it can't start:
/// no project, a content model with errors, a build that isn't the model's
/// or none named where one is needed, a preview that can't be written, or a
/// port that can't be listened on. Problems in the pages don't stop it.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let stderr = io::stderr();
    let mut err = stderr.lock();
    exit::code(preview(global, &args, &mut out, &mut err))
}

fn preview(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let color = use_color(global, stdout_is_terminal());
    let mut rendered = match render(global, args, color, out, err) {
        Ok(rendered) => rendered,
        Err(code) => return code,
    };
    let server = match serve::Server::bind(rendered.folder.clone(), args.port) {
        Ok(server) => server,
        Err(e) => return exit::fail(err, &e),
    };
    let _ = writeln!(
        out,
        "previewing build `{}` at {}\nit renders again as the project's files change; Ctrl-C stops it",
        rendered.build,
        server.address()
    );
    let _ = out.flush();
    server.spawn();
    loop {
        thread::sleep(POLL);
        let mut listing = cache::watched(rendered.outcome.project());
        if listing == rendered.listing {
            continue;
        }
        // An editor's save can be more than one write: wait for the files
        // to stay as they are for one listing.
        loop {
            thread::sleep(POLL);
            let again = cache::watched(rendered.outcome.project());
            if again == listing {
                break;
            }
            listing = again;
        }
        let _ = writeln!(out);
        match render(global, args, color, out, err) {
            Ok(again) => rendered = again,
            // What's served stays as it was, until the next change.
            Err(_) => rendered.listing = listing,
        }
        let _ = out.flush();
    }
}

/// A build rendered into the preview's folder.
struct Rendered {
    /// The check of the project it was rendered from.
    outcome: check::Outcome,
    /// The project's files, as they were checked.
    listing: Vec<Stamp>,
    /// The folder it's in.
    folder: PathBuf,
    /// The build.
    build: String,
}

/// Checks the project, prints the problems as `ascribe check` does, and
/// renders the build into the preview's folder; or the exit code of what
/// stopped it, once it's reported.
fn render(
    global: &Global,
    args: &Args,
    color: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<Rendered, u8> {
    let check_args = check_args(args);
    let outcome = check::run_check(&FromDisk, global, &check_args, &mut io::empty())
        .map_err(|stopped| check::report_stopped(stopped, &check_args, color, out, err))?;
    if let Err(e) = check::write(&outcome, global, &check_args, color, out)
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        let _ = writeln!(err, "error: can't write the report: {e}");
        return Err(exit::FAILURE);
    }
    let project = outcome.project();
    let build = answer::one_build(project.model(), args.build.as_deref())
        .map_err(|e| answer::fail(err, &e))?;
    let index = project.index();
    let preview =
        ascribe_emit::preview(&index, project.root(), build).map_err(|e| exit::fail(err, &e))?;
    let folder = project.root().join(FOLDER);
    OutputDir::lock_archive(&folder)
        .and_then(|dir| dir.replace_archive(&build.name, &preview.files))
        .map_err(|e| exit::fail(err, &e))?;
    let assets = preview
        .files
        .iter()
        .filter(|f| f.kind == ascribe_emit::FileKind::Asset)
        .count();
    let _ = writeln!(
        out,
        "rendered build `{}`: {} page{}, {assets} asset{}, in {}",
        build.name,
        preview.pages,
        plural(preview.pages),
        plural(assets),
        answer::path_from_here(&folder)
    );
    let build = build.name.clone();
    let listing = cache::watched(project);
    Ok(Rendered {
        outcome,
        listing,
        folder,
        build,
    })
}

/// The arguments of the `ascribe check` the preview runs: `ascribe check
/// --build <NAME>`, or of every build without `--build`.
fn check_args(args: &Args) -> check::Args {
    check::Args {
        paths: Vec::new(),
        stdin: false,
        path: None,
        build: args.build.iter().cloned().collect(),
        editor_build: false,
        code: Vec::new(),
        summary: false,
        format: check::Format::Text,
        limit: None,
        deny_warnings: false,
        base: None,
        base_exact: false,
        addresses: check::Addresses::NotCompared,
        fix: false,
        vale: false,
        no_baseline: false,
        vale_timeout: None,
        vale_runs: check::ValeRuns::Any,
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
