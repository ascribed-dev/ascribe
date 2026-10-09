//! `ascribe mcp`: the Model Context Protocol server, over standard input
//! and output. The protocol is `ascribe_mcp`'s; the tools, resources, and
//! prompts are `crate::mcp`'s.

use std::io;
use std::process::ExitCode;

use crate::cli::Global;
use crate::exit;
use crate::mcp::Server;

/// Arguments of `ascribe mcp`. There are none: each tool call names the
/// paths it's about, and the project is the nearest `ascribe.toml` at or
/// above them.
#[derive(Debug, clap::Args)]
pub struct Args {}

/// Runs the server until standard input ends. Standard output carries MCP
/// messages and nothing else; logs go to standard error. Exit codes: 0 when
/// standard input ends, 2 when the server can't run or is given `--config`.
pub fn run(global: &Global, _args: Args) -> ExitCode {
    // Every call names its own paths, so a `--config` would be ignored;
    // refuse it rather than look as if it were used.
    if global.config.is_some() {
        eprintln!(
            "error: `ascribe mcp` doesn't take --config: each tool finds the nearest ascribe.toml at or above the paths it's given"
        );
        return exit::code(exit::FAILURE);
    }
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(e) => {
            eprintln!("error: can't read the current directory: {e}");
            return exit::code(exit::FAILURE);
        }
    };
    let mut server = Server::new(cwd);
    let stdin = io::stdin();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let served = ascribe_mcp::serve(
        &mut stdin.lock(),
        &mut stdout.lock(),
        &mut server,
        &Server::info(),
        &mut stderr.lock(),
    );
    match served {
        Ok(()) => exit::code(exit::OK),
        Err(e) => {
            eprintln!("error: the MCP server stopped: {e}");
            exit::code(exit::FAILURE)
        }
    }
}
