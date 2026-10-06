//! The `ascribe` binary.
//!
//! The command structure is in [`cli`]: one module per subcommand under
//! `commands/` (`check`, `build`, `diff`, `fmt`, `lsp`), so a new subcommand is its
//! own module and a line in `cli.rs`.

// The binary is what reports: the one crate that prints (AGENTS.md).
#![allow(clippy::print_stdout, clippy::print_stderr)]

// `docs_page!`, for the help of subcommands defined in their own modules.
#[macro_use]
mod cli;
mod commands;
mod context;
#[cfg(test)]
mod docs;
mod exit;
mod report;

use std::process::ExitCode;

fn main() -> ExitCode {
    cli::run(std::env::args_os())
}
