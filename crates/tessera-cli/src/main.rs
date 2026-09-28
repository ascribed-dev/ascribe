//! The `tessera` binary.
//!
//! The command structure is in [`cli`]: one module per subcommand under
//! `commands/`, so a phase adds a subcommand with its own module and a line
//! in `cli.rs`. Phase 10 builds `check`; phases 15, 18, and 23 add `lsp`,
//! `build`, and `fmt`.

mod cli;
mod commands;
mod context;
mod exit;
mod report;

use std::process::ExitCode;

fn main() -> ExitCode {
    cli::run(std::env::args_os())
}
