//! `ascribe mcp`'s tools. The protocol is `ascribe_mcp`'s; what the server
//! offers is here, beside the commands, because every tool returns what its
//! command writes.
//!
//! Every tool takes paths from the server's working directory, finds the
//! nearest `ascribe.toml` at or above them, and keeps the project loaded
//! for the next call ([`cache`]), so one server serves every project in a
//! repository. No tool writes a file.

mod cache;
mod tools;

use std::path::PathBuf;

use ascribe_mcp::{Handler, ServerInfo, Tool, ToolResult};
use serde_json::{Map, Value};

pub use cache::Cache;
pub use tools::TOOLS;

/// What the server says about itself to the model.
pub const INSTRUCTIONS_TEXT: &str = "Ascribe checks documentation written as code: Markdown \
    pages with `@` directives in a folder with an `ascribe.toml`. After you edit a page, call \
    `ascribe_check` with its path and fix what it reports; before you write frontmatter, a \
    directive, or a phrase, read `ascribe_model`. Paths are from the server's working \
    directory, and no tool writes a file. With a shell, the `ascribe` commands of the same \
    names give the same answers.";

/// The server's side of the protocol.
pub struct Server {
    cache: Cache,
    /// The working directory, which every path is read from.
    cwd: PathBuf,
}

impl Server {
    /// A server working in `cwd`.
    pub fn new(cwd: PathBuf) -> Server {
        Server {
            cache: Cache::default(),
            cwd,
        }
    }

    /// Who the server is.
    pub fn info() -> ServerInfo {
        ServerInfo {
            name: "ascribe".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            instructions: INSTRUCTIONS_TEXT.to_owned(),
        }
    }
}

impl Handler for Server {
    fn tools(&self) -> Vec<Tool> {
        TOOLS.iter().map(tools::Spec::tool).collect()
    }

    fn call_tool(&mut self, name: &str, arguments: &Map<String, Value>) -> Option<ToolResult> {
        let spec = TOOLS.iter().find(|t| t.name == name)?;
        Some(spec.call(self, arguments))
    }
}
