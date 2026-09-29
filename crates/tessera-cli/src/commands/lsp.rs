//! `ascribe lsp`: the language server, speaking LSP over standard input and
//! output.

use std::process::ExitCode;

use tessera_lsp::Exit;

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
/// without shutting down, 2 when the server can't run.
pub fn run(_global: &Global, _args: Args) -> ExitCode {
    match tessera_lsp::run_stdio() {
        Ok(Exit::Clean) => exit::code(exit::OK),
        Ok(Exit::Abrupt) => exit::code(exit::PROBLEMS),
        Err(e) => {
            eprintln!("error: the language server stopped: {e}");
            exit::code(exit::FAILURE)
        }
    }
}
