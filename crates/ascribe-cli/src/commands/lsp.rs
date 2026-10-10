//! `ascribe lsp`: the language server, speaking LSP over standard input and
//! output.

use std::process::ExitCode;

use ascribe_lsp::Exit;

use crate::cli::Global;
use crate::exit;

/// Arguments of `ascribe lsp`. There are none: the editor's workspace folders
/// say where the project is, and the content model's `[editor]` table says which
/// build to check.
#[derive(Debug, clap::Args)]
pub struct Args {}

/// Runs the server until the client ends the session. Standard output carries
/// LSP messages and nothing else; logs go to standard error. Exit codes follow
/// the protocol: 0 after `shutdown` then `exit`, 1 when the client exits
/// without shutting down, 2 when the server can't run or is given `--config`.
pub fn run(global: &Global, _args: Args) -> ExitCode {
    // The project comes from the editor's workspace folders, so a `--config`
    // would be ignored; refuse it rather than look as if it were used.
    if global.config.is_some() {
        eprintln!(
            "error: `ascribe lsp` doesn't take --config: its project is the nearest ascribe.toml at or above the editor's workspace folder"
        );
        return exit::code(exit::FAILURE);
    }
    let options = ascribe_lsp::Options {
        today: Some(std::sync::Arc::new(crate::clock::today)),
        ..ascribe_lsp::Options::default()
    };
    match ascribe_lsp::run_stdio(options) {
        Ok(Exit::Clean) => exit::code(exit::OK),
        Ok(Exit::Abrupt) => exit::code(exit::PROBLEMS),
        Err(e) => {
            eprintln!("error: the language server stopped: {e}");
            exit::code(exit::FAILURE)
        }
    }
}
