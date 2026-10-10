//! `ascribe vale`: the project's Vale setup (`[checks.vale]`). `eject`
//! writes a preset out as a configuration the project owns.

use std::io::Write;
use std::process::ExitCode;

use ascribe_check::Project;
use ascribe_check::prose::eject;
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::Global;
use crate::context::{Failure, locate};
use crate::exit;

/// Arguments of `ascribe vale`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe vale`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Write the preset `[checks.vale]` names out as the project's own
    /// `.vale.ini` and `.vale/` styles folder, with the rules `off` names
    /// turned off, and switch `[checks.vale]` to `config = ".vale.ini"`.
    ///
    /// Writes nothing when `.vale.ini` or `.vale` is there already.
    #[command(after_help = docs_page!("reference/cli/#ascribe-vale-eject"))]
    Eject(EjectArgs),
}

/// Arguments of `ascribe vale eject`.
#[derive(Debug, ClapArgs)]
pub struct EjectArgs {}

/// Runs the command. Exit codes: 0 when the preset was written out, 2 when
/// it wasn't.
pub fn run(global: &Global, args: Args) -> ExitCode {
    match args.command {
        Command::Eject(_) => run_eject(global),
    }
}

fn run_eject(global: &Global) -> ExitCode {
    let mut out = std::io::stdout().lock();
    let mut err = std::io::stderr().lock();
    let loaded = locate(global)
        .map_err(Failure::Config)
        .and_then(|config| Project::load_model(&config).map_err(Failure::Load));
    let file = match loaded {
        Ok(file) => file,
        Err(e) => return exit::code(exit::fail(&mut err, &e)),
    };
    match eject(&file.root, &file.model, &file.text) {
        Ok(ejected) => {
            for path in &ejected.written {
                let _ = writeln!(out, "wrote {path}");
            }
            let _ = writeln!(
                out,
                "The `{}` preset is now the project's own: `[checks.vale]` reads .vale.ini.",
                ejected.preset
            );
            exit::code(exit::OK)
        }
        Err(e) => exit::code(exit::fail(&mut err, &e)),
    }
}
