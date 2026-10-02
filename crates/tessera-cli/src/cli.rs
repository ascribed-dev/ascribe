//! The command line: options every subcommand shares, and the subcommands.
//!
//! To add a subcommand, write `commands/<name>.rs` with an `Args` type
//! (`clap::Args`) and a `run(&Global, Args) -> ExitCode`, declare it in
//! `commands/mod.rs`, and add one variant to [`Command`] and one arm to
//! [`Command::run`].

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::{commands, exit};

/// Ascribe: check, build, format, and serve documentation written as code.
#[derive(Debug, Parser)]
#[command(name = "ascribe", version, arg_required_else_help = true)]
pub struct Cli {
    #[command(flatten)]
    pub global: Global,

    #[command(subcommand)]
    pub command: Command,
}

/// Options every subcommand accepts, before or after its name.
#[derive(Debug, clap::Args)]
pub struct Global {
    /// The content model, `ascribe.toml`. By default, the nearest one in the
    /// current directory or a parent. Not for `lsp`, whose project comes from
    /// the editor.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// When to color output.
    #[arg(long, global = true, value_enum, default_value_t = Color::Auto, value_name = "WHEN")]
    pub color: Color,
}

/// When to color output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Color {
    /// When writing to a terminal, unless `NO_COLOR` is set.
    Auto,
    /// Always.
    Always,
    /// Never.
    Never,
}

/// The subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build the documentation set's outputs.
    Build(commands::build::Args),
    /// Check every source file for problems, without building anything.
    Check(commands::check::Args),
    /// Rewrite Ascribe constructs into canonical form.
    Fmt(commands::fmt::Args),
    /// Run the language server, speaking LSP over standard input and output.
    Lsp(commands::lsp::Args),
}

impl Command {
    fn run(self, global: &Global) -> ExitCode {
        match self {
            Command::Build(args) => commands::build::run(global, args),
            Command::Check(args) => commands::check::run(global, args),
            Command::Fmt(args) => commands::fmt::run(global, args),
            Command::Lsp(args) => commands::lsp::run(global, args),
        }
    }
}

/// Parses the arguments and runs the subcommand. A usage error prints its
/// message and exits with 2.
pub fn run(args: impl IntoIterator<Item = OsString>) -> ExitCode {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            // `--help` and `--version` are successes; a mistake is a usage
            // error, exit code 2 (clap prints the message to the right stream).
            let _ = e.print();
            return exit::code(if e.use_stderr() {
                exit::FAILURE
            } else {
                exit::OK
            });
        }
    };
    cli.command.run(&cli.global)
}
