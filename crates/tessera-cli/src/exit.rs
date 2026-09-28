//! Exit codes, shared by every subcommand.

use std::process::ExitCode;

/// Everything passed: no errors (and, with `--deny-warnings`, no warnings).
pub const OK: u8 = 0;
/// The documentation set has errors (or warnings under `--deny-warnings`).
pub const PROBLEMS: u8 = 1;
/// The command couldn't do its work: a usage error, no `tessera.toml`, a
/// content model that doesn't load, or a file that can't be read.
pub const FAILURE: u8 = 2;

/// An exit code as the process's.
pub fn code(code: u8) -> ExitCode {
    ExitCode::from(code)
}
