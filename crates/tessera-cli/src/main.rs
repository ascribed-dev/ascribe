//! The `ascribe` binary.
//!
//! The command structure is in [`cli`]: one module per subcommand under
//! `commands/` (`check`, `build`, `diff`, `fmt`, `lsp`), so a new subcommand is its
//! own module and a line in `cli.rs`.

mod cli;
mod commands;
mod context;
mod exit;
mod report;

use std::process::ExitCode;

fn main() -> ExitCode {
    cli::run(std::env::args_os())
}
