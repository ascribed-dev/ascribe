//! The Model Context Protocol over standard input and output: what `ascribe
//! mcp` speaks.
//!
//! This crate knows the protocol and nothing about Ascribe. [`serve`] reads
//! one JSON-RPC message per line, answers each request, and writes one
//! message per line. The tools, resources, and prompts come from its caller,
//! a [`Handler`]: the binary, whose tools return what its commands write.
//!
//! It speaks two revisions of the protocol:
//!
//! - **2026-07-28** ([`MODERN`]), which has no handshake: every request
//!   carries its protocol version and the client's capabilities in `_meta`,
//!   and `server/discover` names the server, what it offers, and the
//!   versions it speaks. Every result carries `resultType`, and the lists
//!   carry `ttlMs` and `cacheScope`.
//! - **2025-11-25** and **2025-06-18** ([`LEGACY`]), for clients that open
//!   with `initialize`. After it, requests without `_meta` are served as the
//!   version `initialize` agreed on, and results leave out what that
//!   revision doesn't know.
//!
//! The server keeps no state between requests apart from that agreement:
//! a [`Handler`] may cache what it computes, but no answer depends on an
//! earlier request.

mod server;

use serde::Serialize;
use serde_json::{Map, Value};

pub use server::serve;

/// The revisions served without a handshake, newest first.
pub const MODERN: &[&str] = &["2026-07-28"];

/// The revisions served after `initialize`, newest first.
pub const LEGACY: &[&str] = &["2025-11-25", "2025-06-18"];

/// Who the server is, for `server/discover` and `initialize`.
#[derive(Clone, Debug)]
pub struct ServerInfo {
    /// Its name, such as `ascribe`.
    pub name: String,
    /// Its version.
    pub version: String,
    /// What a model should know to use it well, in a few sentences.
    pub instructions: String,
}

/// What the server offers. Each list is in a fixed order, the same on every
/// call.
pub trait Handler {
    /// The tools.
    fn tools(&self) -> Vec<Tool>;

    /// Calls the tool `name`. `None` when there's no such tool. Bad
    /// arguments are a result with [`ToolResult::is_error`], which the model
    /// reads, not a protocol error.
    fn call_tool(&mut self, name: &str, arguments: &Map<String, Value>) -> Option<ToolResult>;

    /// The resources there are now. By default, none.
    fn resources(&mut self) -> Vec<Resource> {
        Vec::new()
    }

    /// The resources' URI templates. By default, none.
    fn resource_templates(&self) -> Vec<ResourceTemplate> {
        Vec::new()
    }

    /// The text of the resource at `uri`.
    ///
    /// # Errors
    ///
    /// No resource has that URI, or it can't be read now: the message says
    /// which, and what to do instead.
    fn read_resource(&mut self, uri: &str) -> Result<ResourceText, String> {
        Err(format!("no resource {uri}; there are none"))
    }

    /// The named prompts. By default, none.
    fn prompts(&self) -> Vec<Prompt> {
        Vec::new()
    }

    /// The prompt `name` with these arguments. `None` when there's no such
    /// prompt.
    ///
    /// # Errors
    ///
    /// The arguments are wrong, or the prompt can't be built: the message
    /// says why.
    fn get_prompt(
        &mut self,
        name: &str,
        arguments: &Map<String, Value>,
    ) -> Option<Result<PromptText, String>> {
        let _ = (name, arguments);
        None
    }
}

/// A tool's definition.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Its name.
    pub name: String,
    /// A short name for people.
    pub title: String,
    /// When to use it, for the model choosing a tool.
    pub description: String,
    /// The JSON Schema of its arguments.
    pub input_schema: Value,
    /// The JSON Schema of its `structuredContent`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    /// What it does to the world.
    pub annotations: Annotations,
}

/// What a tool does to the world, as hints a client may show or act on.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    /// It changes nothing.
    pub read_only_hint: bool,
    /// It may destroy something (meaningful only when it isn't read-only).
    pub destructive_hint: bool,
    /// Calling it again with the same arguments does nothing more.
    pub idempotent_hint: bool,
    /// It reaches beyond the machine it runs on.
    pub open_world_hint: bool,
}

impl Annotations {
    /// A tool that only reads, on this machine.
    pub const READ_ONLY: Annotations = Annotations {
        read_only_hint: true,
        destructive_hint: false,
        idempotent_hint: true,
        open_world_hint: false,
    };
}

/// What a tool call returns.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolResult {
    /// The result as text, for clients that read only text.
    pub text: String,
    /// The result as data, matching the tool's output schema; `None` for an
    /// error.
    pub structured: Option<Value>,
    /// Whether the call failed: `text` says why, and what to call instead.
    pub is_error: bool,
}

impl ToolResult {
    /// A result: `structured`, with `text` beside it.
    pub fn ok(text: String, structured: Value) -> ToolResult {
        ToolResult {
            text,
            structured: Some(structured),
            is_error: false,
        }
    }

    /// A failed call, which `message` explains.
    pub fn error(message: impl Into<String>) -> ToolResult {
        ToolResult {
            text: message.into(),
            structured: None,
            is_error: true,
        }
    }
}

/// A resource there is now.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    /// Its address.
    pub uri: String,
    /// A short name.
    pub name: String,
    /// What it is.
    pub description: String,
    /// Its media type.
    pub mime_type: String,
}

/// A family of resources, by URI template (RFC 6570).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTemplate {
    /// The template.
    pub uri_template: String,
    /// A short name.
    pub name: String,
    /// What each resource is.
    pub description: String,
    /// Their media type.
    pub mime_type: String,
}

/// A resource's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceText {
    /// Its media type.
    pub mime_type: String,
    /// The text.
    pub text: String,
}

/// A named prompt.
#[derive(Clone, Debug, Serialize)]
pub struct Prompt {
    /// Its name.
    pub name: String,
    /// A short name for people.
    pub title: String,
    /// What it asks an agent to do.
    pub description: String,
    /// Its arguments.
    pub arguments: Vec<PromptArgument>,
}

/// An argument of a named prompt.
#[derive(Clone, Debug, Serialize)]
pub struct PromptArgument {
    /// Its name.
    pub name: String,
    /// What it is.
    pub description: String,
    /// Whether the prompt needs it.
    pub required: bool,
}

/// A prompt's text, which the client puts in the conversation as the user's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptText {
    /// What the prompt is, for people.
    pub description: String,
    /// The prompt.
    pub text: String,
}
