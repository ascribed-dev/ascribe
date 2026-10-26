//! `ascribe init`: starts a project, writing its first `ascribe.toml` and
//! `ascribe.addresses`, and with `--templates`, the page templates too
//! (`ascribe_templates::init`).

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_core::path::normalize;
use ascribe_model::ADDRESSES_FILE;
use ascribe_templates::{InitAnswer, InitError};
use clap::Args as ClapArgs;

use crate::answer::{self, Format, from_here, path_from_here};
use crate::cli::Global;
use crate::commands::templates::{write_added, write_refused};
use crate::exit;

/// Arguments of `ascribe init`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The folder to start the project in, from the current folder. It's
    /// made if it isn't there. By default, the current folder.
    #[arg(value_name = "DIR")]
    pub dir: Option<PathBuf>,

    /// Take the page templates too, as `ascribe templates eject` with no
    /// name does: six types, beside the type `page`.
    #[arg(long)]
    pub templates: bool,

    /// How to show what was written.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Runs the command. Exit codes: 0 when the project is there; 2 when it
/// wrote nothing (there's a project at or above the folder, or a template
/// was refused), or it couldn't run.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = run_init(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn run_init(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    // The project is the one being started, so a `--config` would be
    // ignored; refuse it rather than look as if it were used.
    if global.config.is_some() {
        let _ = writeln!(
            err,
            "error: `ascribe init` doesn't take --config: it starts a project in the current folder, or in the folder DIR names"
        );
        return exit::FAILURE;
    }
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(e) => return answer::fail(err, &ascribe_check::LocateError::CurrentDir(e)),
    };
    let root = normalize(&cwd.join(args.dir.as_deref().unwrap_or(Path::new("."))));
    let request = ascribe_templates::Init {
        root: &root,
        templates: args.templates,
        ascribe_version: env!("ASCRIBE_VERSION"),
    };
    let started = match ascribe_templates::init(&request) {
        Ok(started) => started,
        // The project found, as it's typed from here.
        Err(InitError::Project { found }) => {
            let found = PathBuf::from(path_from_here(&found));
            return answer::fail(err, &InitError::Project { found });
        }
        Err(e) => return answer::fail(err, &e),
    };
    let refused = started
        .templates
        .as_ref()
        .is_some_and(|t| !t.refused.is_empty());
    let result = match args.format {
        Format::Json => answer::write_json(out, &started),
        Format::Text => write_init(out, err, &root, &started),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if refused { exit::FAILURE } else { exit::OK }
}

fn write_init(
    out: &mut dyn Write,
    err: &mut dyn Write,
    root: &Path,
    started: &InitAnswer,
) -> io::Result<()> {
    if let Some(ejected) = started.templates.as_ref().filter(|t| !t.refused.is_empty()) {
        return write_refused(err, ejected);
    }
    for path in &started.written {
        if path == ADDRESSES_FILE {
            writeln!(
                out,
                "wrote {}, where a page's move is recorded, so the addresses it published keep working",
                from_here(root, path)
            )?;
        } else {
            writeln!(out, "wrote {}", from_here(root, path))?;
        }
    }
    if started.content_root_made {
        writeln!(
            out,
            "made {}/, the folder the pages go in",
            from_here(root, &started.content_root)
        )?;
    }
    if let Some(ejected) = &started.templates {
        write_added(out, root, ejected, false)?;
    }
    writeln!(
        out,
        "Next, write a first page, {}, and run `ascribe check`",
        from_here(root, &format!("{}/index.md", started.content_root))
    )?;
    writeln!(
        out,
        "Then, to publish the pages as a site with Astro, read {}",
        started.next
    )
}
