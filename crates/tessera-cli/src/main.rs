//! The `tessera` binary.
//!
//! This is a placeholder until phase 10 builds the command structure and the
//! `check` subcommand. Phases 15, 18, and 23 add `lsp`, `build`, and `fmt`.
//! For now it only answers `--version` and `--help`.

mod fmt;

use std::process::ExitCode;

const USAGE: &str = "\
Usage: tessera [--version | --help]
       tessera fmt [--check] [paths]

Only `fmt` is implemented so far.";

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        // Interim wiring (phase 23): phase 10's command structure replaces it.
        Some("fmt") => fmt::run_from_env(),
        Some("--version" | "-V") => {
            println!("tessera {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
