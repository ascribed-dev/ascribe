//! The tools: each wraps one command, calls the same function, and returns
//! the JSON the command writes. The table is in the order `tools/list`
//! gives it.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use ascribe_mcp::{Annotations, Tool, ToolResult};
use serde::Serialize;
use serde_json::{Map, Value, json};

use super::Server;
use crate::answer::{self, Format};
use crate::cli::Global;
use crate::commands::{
    addresses, check, diff, fmt, link, model, outline, refs, rename, resolve, search,
};

/// A tool.
pub struct Spec {
    /// Its name.
    pub name: &'static str,
    /// A short name for people.
    pub title: &'static str,
    /// The command that gives the same thing, as `ascribe <command>`.
    pub command: &'static str,
    /// When to use it, and an example call with its result.
    pub description: &'static str,
    /// The JSON Schema of its arguments.
    pub input: fn() -> Value,
    /// The JSON Schema of its result: the command's JSON.
    pub output: fn() -> Value,
    /// Calls it.
    call: fn(&Server, &Arguments<'_>) -> Result<ToolResult, String>,
}

/// Every tool, in order.
pub const TOOLS: &[Spec] = &[
    Spec {
        name: "ascribe_check",
        title: "Check pages",
        command: "ascribe check",
        description: "Check pages for problems after you edit them, and before you finish: \
            every problem `ascribe check` reports, with how to fix it. Pass `text` with `path` \
            to check unsaved text as that file. Example: {\"paths\": [\"docs/quickstart.md\"]} \
            returns `docs/quickstart.md:5: [ASC037] `keys.md` has no heading with the id \
            `rotat`` and a summary line; `\"response_format\": \"detailed\"` returns the JSON \
            report, with each diagnostic's fixes.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Files and folders to report on, from the server's \
                            working directory; a project's folder reports on all of it. The \
                            whole project is still checked. By default, the project at or \
                            above the working directory.",
                    },
                    "text": {
                        "type": "string",
                        "description": "Unsaved text to check as the file `path` names. The \
                            file on disk isn't read or changed.",
                    },
                    "path": {
                        "type": "string",
                        "description": "With `text`: the file it's checked as.",
                    },
                    "build": builds(),
                    "editor_build": {
                        "type": "boolean",
                        "description": "Check only the editor's build, which is quicker; a \
                            full check covers every build.",
                    },
                    "code": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Only the problems with these diagnostic codes \
                            (`ASC037`) or names (`link-id-missing`), to fix one rule across \
                            the project. The whole project is still checked. By default, every \
                            code.",
                    },
                    "response_format": response_format(),
                },
                "additionalProperties": false,
            })
        },
        output: || {
            with_concise(schema(include_str!(
                "../../../../schemas/check.schema.json"
            )))
        },
        call: call_check,
    },
    Spec {
        name: "ascribe_explain",
        title: "Explain a diagnostic",
        command: "ascribe explain",
        description: "Explain a diagnostic code from a check: what it means, how to fix it, and \
            a wrong-and-right example. Example: {\"code\": \"ASC037\"} returns its `slug` \
            (`link-id-missing`), `message`, `fix`, and `example`.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "The diagnostic's code (`ASC037`) or name \
                            (`link-id-missing`).",
                    },
                },
                "required": ["code"],
                "additionalProperties": false,
            })
        },
        output: || schema(include_str!("../../../../schemas/explain.schema.json")),
        call: call_explain,
    },
    Spec {
        name: "ascribe_model",
        title: "Show the content model",
        command: "ascribe model",
        description: "Read what a project's content model allows before writing frontmatter, \
            a directive's attributes, or a phrase: page types and their fields, dimensions, \
            phrases, features, glossary terms, widgets, release lines with their status, builds, \
            and sources. Example: {\"path\": \
            \"docs\", \"section\": \"phrases\"} returns `phrases` such as `{\"key\": \
            \"product\", \"value\": \"Quill\"}`.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "A file or folder in the project, to find its \
                            ascribe.toml from. By default, the server's working directory.",
                    },
                    "section": {
                        "type": "string",
                        "enum": model_sections(),
                        "description": "Only this section.",
                    },
                },
                "additionalProperties": false,
            })
        },
        output: || schema(include_str!("../../../../schemas/model.schema.json")),
        call: call_model,
    },
    Spec {
        name: "ascribe_outline",
        title: "Outline a page",
        command: "ascribe outline",
        description: "List a page's title, type, and headings with their ids, to write a link \
            to one of them, and the memo files about the page and each heading, to read before \
            changing it. Example: {\"page\": \"docs/keys.md\"} returns `headings` such as \
            `{\"text\": \"Rotate keys\", \"id\": \"rotate-keys\", \"line\": 11}`.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "page": page(),
                    "build": {
                        "type": "string",
                        "description": "Only the headings this build publishes.",
                    },
                },
                "required": ["page"],
                "additionalProperties": false,
            })
        },
        output: || schema(include_str!("../../../../schemas/outline.schema.json")),
        call: call_outline,
    },
    Spec {
        name: "ascribe_link",
        title: "Check a link",
        command: "ascribe link",
        description: "Ask whether a link target exists as seen from a page, and what to write \
            for it. Example: {\"target\": \"keys.md#rotate\", \"from\": \"docs/quickstart.md\"} \
            returns `exists: false`, the `problem`, and `closest` with `{\"title\": \"Rotate \
            keys\", \"href\": \"keys.md#rotate-keys\"}`. A link to an operation of a \
            declared API, `../api/openapi.yaml#getUser`, returns `kind: operation` and the \
            `operation`: its method, path, summary, deprecation, and address.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "The link's destination as the page writes it: \
                            `keys.md`, `keys.md#rotate-keys`, `#install`, \
                            `../api/openapi.yaml#getUser`.",
                    },
                    "from": {
                        "type": "string",
                        "description": "The page the link is on, from the server's working \
                            directory or from the content root.",
                    },
                },
                "required": ["target", "from"],
                "additionalProperties": false,
            })
        },
        output: || schema(include_str!("../../../../schemas/link.schema.json")),
        call: call_link,
    },
    Spec {
        name: "ascribe_refs",
        title: "Find uses",
        command: "ascribe refs",
        description: "Find where a page, a heading, a fragment, an image, a phrase, or \
            another content model entry is used, before changing or removing it. Example: {\"target\": \
            \"phrase:product\", \"project\": \"docs\"} returns lines such as \
            `docs/install-agent.md:69:63: phrase` and `8 places use phrase:product.`",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "A page or fragment (`keys.md`), a heading \
                            (`keys.md#rotate-keys`), an image (`images/keys.png`), \
                            `phrase:<key>`, `feature:<key>`, `term:<id>`, `dimension:<name>`, \
                            `note:<type>`, `widget:<name>`, or a line of a release target, \
                            `release:<target>@<version>`. A path is from the server's \
                            working directory or from the content root.",
                    },
                    "project": {
                        "type": "string",
                        "description": "A file or folder in the project, for a target that \
                            isn't a path. By default, the server's working directory.",
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "List at most this many places. By default, 50 \
                            concise and 500 detailed.",
                    },
                    "response_format": response_format(),
                },
                "required": ["target"],
                "additionalProperties": false,
            })
        },
        output: || with_concise(schema(include_str!("../../../../schemas/refs.schema.json"))),
        call: call_refs,
    },
    Spec {
        name: "ascribe_search",
        title: "Search pages",
        command: "ascribe search",
        description: "Find where the pages say something, as their readers read it, with the \
            line to edit for each place, before writing a page about a subject: phrases, \
            fragments, and snippets resolved, which a text search of the source can't. Words \
            must all be in one block; a phrase in double quotes is matched as written. When \
            nothing matches, try other words for the subject. Example: {\"query\": \"\\\"sign \
            in to quill cloud\\\"\", \"project\": \"docs\"} returns lines such as \
            `docs/install-agent.md:62: Sign in to Quill Cloud and copy an API key…` and \
            `Connect to Quill #connect · deployment=cloud arm · through phrase cloud`.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Words, which must all be in one block in any order, \
                            or a phrase in double quotes, matched as written. Case is ignored \
                            unless `case`, and a word matches inside a longer one unless \
                            `word`.",
                    },
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Search only the pages in these folders, or these \
                            pages, from the server's working directory. By default, every \
                            page.",
                    },
                    "build": builds(),
                    "type": {
                        "type": "string",
                        "description": "Search only the pages of this content type.",
                    },
                    "case": {
                        "type": "boolean",
                        "description": "Match case as written.",
                    },
                    "word": {
                        "type": "boolean",
                        "description": "Match a word or a phrase only where it isn't inside \
                            a longer word.",
                    },
                    "summary": {
                        "type": "boolean",
                        "description": "List the pages and how many places each has, most \
                            first, instead of the places.",
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "List at most this many places, or with `summary`, \
                            pages. By default, 50 places concise and 500 detailed, or 20 \
                            pages.",
                    },
                    "project": {
                        "type": "string",
                        "description": "A file or folder in the project, when no `paths` \
                            are given. By default, the server's working directory.",
                    },
                    "response_format": response_format(),
                },
                "required": ["query"],
                "additionalProperties": false,
            })
        },
        output: || {
            with_concise(schema(include_str!(
                "../../../../schemas/search.schema.json"
            )))
        },
        call: call_search,
    },
    Spec {
        name: "ascribe_resolve",
        title: "Resolve a page",
        command: "ascribe resolve",
        description: "Read a page as one build's readers see it, by default as plain Markdown: \
            directives, phrases, includes, and availability resolved. Example: {\"page\": \
            \"docs/keys.md\", \"build\": \"cloud\"} returns `text` starting `# API keys`. \
            With `emit` of `site`, read it as the site output gives it to a site, with \
            Ascribe's elements, to see what a reader of the site gets: {\"page\": \
            \"docs/keys.md\", \"emit\": \"site\"} shows what a switcher or a badge becomes. \
            With `go:` and a permalink's name, read what a product that links to the docs by \
            that permalink shows: {\"page\": \"go:rotate-keys\"} returns its `title`, \
            `summary`, `text`, and `address`.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "page": {
                        "type": "string",
                        "description": "The page: a path from the server's working directory, \
                            or from the content root. Or `go:` and a permalink's name, for the \
                            help a product shows for it.",
                    },
                    "project": {
                        "type": "string",
                        "description": "With `go:<name>`, a file or folder in the project. By \
                            default, the server's working directory.",
                    },
                    "build": {
                        "type": "string",
                        "description": "The build whose readers to show it as; needed when \
                            the project has more than one.",
                    },
                    "emit": {
                        "type": "string",
                        "enum": ["plain", "site"],
                        "description": "Which output to write the page as: `plain` (the \
                            default), or `site`, as the site output writes it. Not for \
                            `go:<name>`, whose help has one form.",
                    },
                    "frontmatter": {
                        "type": "boolean",
                        "description": "Put the page's frontmatter first.",
                    },
                },
                "required": ["page"],
                "additionalProperties": false,
            })
        },
        output: || {
            either(
                schema(include_str!("../../../../schemas/resolve.schema.json")),
                schema(include_str!(
                    "../../../../schemas/resolve-permalink.schema.json"
                )),
            )
        },
        call: call_resolve,
    },
    Spec {
        name: "ascribe_fmt",
        title: "Format pages",
        command: "ascribe fmt",
        description: "Get the edits that put Ascribe's syntax in pages into canonical form; it \
            writes nothing, so apply the edits yourself. Example: {\"paths\": \
            [\"docs/keys.md\"]} returns `files` such as `{\"file\": \"docs/keys.md\", \
            \"edits\": [{\"range\": …, \"new_text\": \"{type=tip}\"}]}`, and none when \
            the pages are formatted.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Files and folders to format, from the server's \
                            working directory; by default, the content root of the project \
                            at or above it.",
                    },
                },
                "additionalProperties": false,
            })
        },
        output: || schema(include_str!("../../../../schemas/fmt.schema.json")),
        call: call_fmt,
    },
    Spec {
        name: "ascribe_diff",
        title: "List changed pages",
        command: "ascribe diff",
        description: "List the pages that changed against a git base, as readers see them, \
            including pages changed only through a fragment or a phrase. Example: \
            {\"project\": \"docs\", \"base\": \"main\"} returns `builds` with `pages` such as \
            `{\"path\": \"keys.md\", \"status\": \"changed\", \"counts\": {\"changed\": 2, …}}`; \
            `\"blocks\": true` adds each changed block.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "A file or folder in the project. By default, the \
                            server's working directory.",
                    },
                    "base": {
                        "type": "string",
                        "description": "The git revision to compare with. By default, the \
                            repository's default branch, from its merge base with HEAD.",
                    },
                    "base_exact": {
                        "type": "boolean",
                        "description": "Compare with the revision itself, not the merge base.",
                    },
                    "build": builds(),
                    "blocks": {
                        "type": "boolean",
                        "description": "List each page's changed blocks too.",
                    },
                },
                "additionalProperties": false,
            })
        },
        output: || schema(include_str!("../../../../schemas/diff.schema.json")),
        call: call_diff,
    },
    Spec {
        name: "ascribe_rename",
        title: "Rename",
        command: "ascribe rename",
        description: "Get the changes that rename a phrase, a dimension's value, a heading's \
            id, or a page, which moves, everywhere it's used: the links a moved page has \
            and the ones to it, and the move recorded in `ascribe.addresses`. It writes \
            nothing, so write each file's text, move the file, and delete it where it was \
            yourself. Example: {\"target\": \"docs/keys.md\", \"new\": \
            \"docs/security/keys.md\"} returns `files` such as `{\"file\": \
            \"docs/quickstart.md\", \"text\": …, \"edits\": 1}`, `moved` with `from` and `to`, \
            and `recorded` such as `{\"key\": \"keys.md\", \"value\": \"security/keys.md\"}`.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "What to rename: `phrase:<key>`, `value:<value>` (a \
                            dimension's value), a heading's id (`keys.md#rotate-keys`), or a \
                            page or fragment (`keys.md`), which moves. A path is from the \
                            server's working directory or from the content root.",
                    },
                    "new": {
                        "type": "string",
                        "description": "The new name: a phrase's key, a value, or an id. For a \
                            page or fragment, where it moves to, read as `target` is.",
                    },
                },
                "required": ["target", "new"],
                "additionalProperties": false,
            })
        },
        output: || schema(include_str!("../../../../schemas/rename.schema.json")),
        call: call_rename,
    },
    Spec {
        name: "ascribe_addresses_move",
        title: "Record a move",
        command: "ascribe addresses move",
        description: "Get the edit to `ascribe.addresses` that records where a page, or a \
            heading's id, moved, after you moved it without `ascribe_rename`, as with `git \
            mv`, so the build forwards its old addresses; `ascribe_check` reports a page \
            moved with nothing recorded as `address-lost`. It writes nothing, so apply the \
            edits yourself. Example: {\"from_git\": true} returns `moves` such as \
            `{\"key\": \"guides/setup.md\", \"value\": \"start/setup.md\"}`, every move git \
            sees, and `edits`; {\"old\": \"guides/keys.md\", \"new\": \
            \"security/keys.md\"} records one.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "old": {
                        "type": "string",
                        "description": "Where it was: a page's path, from the server's \
                            working directory or from the content root; a file's path, `#`, \
                            and a heading's id; or an address starting with `/`.",
                    },
                    "new": {
                        "type": "string",
                        "description": "Where it is now: a page's path, and optionally `#` \
                            and a heading's id.",
                    },
                    "from_git": {
                        "type": "boolean",
                        "description": "Instead of `old` and `new`, every move git sees \
                            between the base and the working tree whose old path is gone.",
                    },
                    "base": {
                        "type": "string",
                        "description": "With `from_git`: the git revision the moves are seen \
                            from. By default, the repository's default branch, from its merge \
                            base with HEAD.",
                    },
                    "base_exact": {
                        "type": "boolean",
                        "description": "With `from_git`: from the revision itself, not the \
                            merge base.",
                    },
                    "project": {
                        "type": "string",
                        "description": "A file or folder in the project. By default, the \
                            server's working directory.",
                    },
                },
                "additionalProperties": false,
            })
        },
        output: || {
            either(
                schema(include_str!(
                    "../../../../schemas/addresses-record.schema.json"
                )),
                schema(include_str!(
                    "../../../../schemas/addresses-from-git.schema.json"
                )),
            )
        },
        call: call_addresses_move,
    },
    Spec {
        name: "ascribe_addresses_retire",
        title: "Record a retired page",
        command: "ascribe addresses retire",
        description: "Get the edit to `ascribe.addresses` that records that a page, or a \
            heading's id, was removed on purpose, and why, so `ascribe_check` stops \
            reporting it as `address-lost`. Use it only for content that's gone with no \
            page to forward a reader to; otherwise record a move. It writes nothing, so \
            apply the edits yourself. Example: {\"old\": \"reference/legacy-sync.md\", \
            \"reason\": \"The sync agent was removed in 4.0\"} returns `key`, `value`, and \
            `edits`.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "old": {
                        "type": "string",
                        "description": "What was removed: a page's path, from the server's \
                            working directory or from the content root; a file's path, `#`, \
                            and a heading's id; or an address starting with `/`.",
                    },
                    "reason": {
                        "type": "string",
                        "description": "Why it was removed: a sentence for whoever reads \
                            the file later.",
                    },
                    "project": {
                        "type": "string",
                        "description": "A file or folder in the project. By default, the \
                            server's working directory.",
                    },
                },
                "required": ["old", "reason"],
                "additionalProperties": false,
            })
        },
        output: || {
            schema(include_str!(
                "../../../../schemas/addresses-record.schema.json"
            ))
        },
        call: call_addresses_retire,
    },
];

/// The tools' old names, each with the name of the tool it calls, kept for
/// a release after they changed to the commands' own words (decision #962):
/// `tools/list` gives each after the tools, as the tool it calls with a
/// sentence saying so first.
pub const OLD_NAMES: &[(&str, &str)] = &[
    ("ascribe_format", "ascribe_fmt"),
    ("ascribe_changes", "ascribe_diff"),
];

/// The tool `name` names: its own name, or an old one.
pub fn find(name: &str) -> Option<&'static Spec> {
    let name = OLD_NAMES
        .iter()
        .find(|(old, _)| *old == name)
        .map_or(name, |(_, new)| new);
    TOOLS.iter().find(|t| t.name == name)
}

/// Every tool's definition, in the order `tools/list` gives them: the
/// tools, then their old names.
pub fn listed() -> Vec<Tool> {
    let old = OLD_NAMES.iter().filter_map(|(old, new)| {
        let mut tool = find(new)?.tool();
        tool.name = (*old).to_owned();
        tool.description = format!(
            "The old name of `{new}`, kept for a release: call `{new}` instead. {}",
            tool.description
        );
        Some(tool)
    });
    TOOLS.iter().map(Spec::tool).chain(old).collect()
}

impl Spec {
    /// The definition `tools/list` gives.
    pub fn tool(&self) -> Tool {
        Tool {
            name: self.name.to_owned(),
            title: self.title.to_owned(),
            description: format!("{} The command: `{}`.", self.description, self.command),
            input_schema: (self.input)(),
            output_schema: Some((self.output)()),
            annotations: Annotations::READ_ONLY,
        }
    }

    /// Calls it with `arguments`.
    pub fn call(&self, server: &Server, arguments: &Map<String, Value>) -> ToolResult {
        let input = (self.input)();
        let arguments = match Arguments::new(self.name, arguments, &input) {
            Ok(arguments) => arguments,
            Err(message) => return ToolResult::error(message),
        };
        (self.call)(server, &arguments).unwrap_or_else(|message| {
            ToolResult::error(format!(
                "{message}\nPaths are read from the server's working directory, {}.",
                server.cwd.display()
            ))
        })
    }
}

/// `response_format`, for the tools whose command has a text form for
/// agents.
fn response_format() -> Value {
    json!({
        "type": "string",
        "enum": ["concise", "detailed"],
        "description": "`concise` (the default): the command's text, one line each. \
            `detailed`: the command's full JSON.",
    })
}

/// `build`, for the tools that check or compare builds.
fn builds() -> Value {
    json!({
        "type": "array",
        "items": { "type": "string" },
        "description": "Only these builds. By default, every build.",
    })
}

/// `page`.
fn page() -> Value {
    json!({
        "type": "string",
        "description": "The page or fragment: a path from the server's working directory, \
            or from the content root.",
    })
}

/// A command's JSON Schema, embedded from `schemas/`.
fn schema(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
}

/// `schema`, or the concise form `{"concise": "<text>"}`: the schema of a
/// tool that takes `response_format`. The definitions move to the root,
/// where the references in `schema` find them.
fn with_concise(mut schema: Value) -> Value {
    let Some(fields) = schema.as_object_mut() else {
        return schema;
    };
    let defs = fields.remove("$defs").unwrap_or_else(|| json!({}));
    let dialect = fields.remove("$schema");
    let mut root = json!({
        "type": "object",
        "anyOf": [
            schema,
            {
                "type": "object",
                "description": "With `response_format: concise`: the command's text.",
                "properties": {
                    "concise": {
                        "type": "string",
                        "description": "What the command writes without `--format json`.",
                    },
                },
                "required": ["concise"],
                "additionalProperties": false,
            },
        ],
        "$defs": defs,
    });
    if let Some(dialect) = dialect {
        root["$schema"] = dialect;
    }
    root
}

/// One schema or the other: the schema of a tool whose result is one of two
/// commands' JSON. The definitions move to the root, where the references
/// in each find them.
fn either(first: Value, second: Value) -> Value {
    let mut defs = Map::new();
    let mut dialect = None;
    let mut parts = Vec::new();
    for mut schema in [first, second] {
        if let Some(fields) = schema.as_object_mut() {
            if let Some(Value::Object(own)) = fields.remove("$defs") {
                defs.extend(own);
            }
            dialect = fields.remove("$schema").or(dialect);
        }
        parts.push(schema);
    }
    let mut root = json!({
        "type": "object",
        "anyOf": parts,
        "$defs": defs,
    });
    if let Some(dialect) = dialect {
        root["$schema"] = dialect;
    }
    root
}

/// A call's arguments, checked against the tool's input schema's property
/// names.
pub struct Arguments<'a> {
    tool: &'static str,
    map: &'a Map<String, Value>,
}

impl<'a> Arguments<'a> {
    fn new(
        tool: &'static str,
        map: &'a Map<String, Value>,
        input: &Value,
    ) -> Result<Arguments<'a>, String> {
        let properties = input["properties"].as_object();
        let known: Vec<&str> = properties
            .map(|p| p.keys().map(String::as_str).collect())
            .unwrap_or_default();
        if let Some(unknown) = map.keys().find(|k| !known.contains(&k.as_str())) {
            return Err(format!(
                "{tool} has no argument `{unknown}`; its arguments are {}",
                known
                    .iter()
                    .map(|k| format!("`{k}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let required = input["required"].as_array().cloned().unwrap_or_default();
        if let Some(missing) = required
            .iter()
            .filter_map(Value::as_str)
            .find(|r| map.get(*r).is_none_or(Value::is_null))
        {
            return Err(format!("{tool} needs `{missing}`"));
        }
        Ok(Arguments { tool, map })
    }

    fn wrong(&self, name: &str, what: &str) -> String {
        format!("{}'s `{name}` is {what}", self.tool)
    }

    fn string(&self, name: &str) -> Result<Option<String>, String> {
        match self.map.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(self.wrong(name, "a string")),
        }
    }

    fn path(&self, name: &str) -> Result<Option<PathBuf>, String> {
        Ok(self.string(name)?.map(PathBuf::from))
    }

    fn strings(&self, name: &str) -> Result<Vec<String>, String> {
        match self.map.get(name) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(Value::Array(items)) => items
                .iter()
                .map(|v| v.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| self.wrong(name, "a list of strings")),
            Some(_) => Err(self.wrong(name, "a list of strings")),
        }
    }

    fn boolean(&self, name: &str) -> Result<bool, String> {
        match self.map.get(name) {
            None | Some(Value::Null) => Ok(false),
            Some(Value::Bool(b)) => Ok(*b),
            Some(_) => Err(self.wrong(name, "true or false")),
        }
    }

    fn count(&self, name: &str) -> Result<Option<usize>, String> {
        match self.map.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => v
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .map(Some)
                .ok_or_else(|| self.wrong(name, "a whole number, 0 or more")),
        }
    }

    /// `response_format`: whether the full JSON is asked for.
    fn detailed(&self) -> Result<bool, String> {
        match self.string("response_format")?.as_deref() {
            None | Some("concise") => Ok(false),
            Some("detailed") => Ok(true),
            Some(_) => Err(self.wrong("response_format", "`concise` or `detailed`")),
        }
    }
}

/// What the commands share when the server runs them: no `--config`, and no
/// color.
fn global() -> Global {
    Global {
        config: None,
        color: crate::cli::Color::Never,
    }
}

/// `global()` with `--config` naming the project at or above `path`, for a
/// command that finds its project from the current directory.
fn global_at(path: Option<&Path>) -> Result<Global, String> {
    let mut global = global();
    if path.is_some() {
        let config = answer::locate(&global, path).map_err(|e| e.to_string())?;
        global.config = Some(config);
    }
    Ok(global)
}

/// A result that's the command's JSON: as structured content, and as the
/// text the command writes.
fn json_result(answer: &impl Serialize) -> Result<ToolResult, String> {
    let mut text = Vec::new();
    answer::write_json(&mut text, answer).map_err(|e| e.to_string())?;
    let structured = serde_json::to_value(answer).map_err(|e| e.to_string())?;
    Ok(ToolResult::ok(
        String::from_utf8_lossy(&text).into_owned(),
        structured,
    ))
}

/// A result that's the command's text, in the concise form.
fn concise_result(text: Vec<u8>) -> ToolResult {
    let text = String::from_utf8_lossy(&text).into_owned();
    let structured = json!({ "concise": text });
    ToolResult::ok(text, structured)
}

fn call_check(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let text = a.string("text")?;
    let path = a.path("path")?;
    let paths: Vec<PathBuf> = a.strings("paths")?.into_iter().map(PathBuf::from).collect();
    if text.is_some() != path.is_some() {
        return Err("`text` and `path` go together: the text, and the file it's checked as".into());
    }
    if text.is_some() && !paths.is_empty() {
        return Err("give `paths`, or `text` with `path`, not both".into());
    }
    let detailed = a.detailed()?;
    let args = check::Args {
        paths,
        stdin: text.is_some(),
        path,
        build: a.strings("build")?,
        editor_build: a.boolean("editor_build")?,
        code: a.strings("code")?,
        summary: false,
        format: if detailed {
            check::Format::Json
        } else {
            check::Format::Concise
        },
        limit: None,
        deny_warnings: false,
        base: None,
        base_exact: false,
        // Over the whole project, the addresses are compared with the
        // default branch, as `ascribe check` compares them.
        addresses: check::Addresses::Compared,
        fix: false,
        vale: false,
        // What `ascribe.baseline` holds back, the project's backlog, isn't
        // the agent's to fix now: the summary counts it (#790).
        no_baseline: false,
        vale_timeout: None,
        // The tool says it only reads: it starts no program the project
        // names (decision 76).
        vale_runs: check::ValeRuns::Never,
    };
    let global = global();
    let mut stdin = Cursor::new(text.unwrap_or_default());
    let outcome = match check::run_check(&server.cache, &global, &args, &mut stdin) {
        Ok(outcome) => outcome,
        Err(stopped) => {
            // What the command writes when it stops, both streams, as one
            // message.
            let mut out = Vec::new();
            let mut err = Vec::new();
            check::report_stopped(stopped, &args, false, &mut out, &mut err);
            out.extend(err);
            return Err(String::from_utf8_lossy(&out).trim_end().to_owned());
        }
    };
    let mut out = Vec::new();
    check::write(&outcome, &global, &args, false, &mut out).map_err(|e| e.to_string())?;
    if detailed {
        let structured: Value = serde_json::from_slice(&out).map_err(|e| e.to_string())?;
        Ok(ToolResult::ok(
            String::from_utf8_lossy(&out).into_owned(),
            structured,
        ))
    } else {
        Ok(concise_result(out))
    }
}

fn call_explain(_server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let code = a.string("code")?.unwrap_or_default();
    let explanation = ascribe_query::explain(&code).map_err(|e| {
        format!("{e}\n`ascribe explain --list` lists every diagnostic's code and name")
    })?;
    json_result(&explanation)
}

/// The names `ascribe model --section` takes.
fn model_sections() -> Vec<String> {
    <model::Section as clap::ValueEnum>::value_variants()
        .iter()
        .filter_map(clap::ValueEnum::to_possible_value)
        .map(|value| value.get_name().to_owned())
        .collect()
}

fn call_model(_server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let section = match a.string("section")?.as_deref() {
        None => None,
        Some(name) => Some(
            <model::Section as clap::ValueEnum>::from_str(name, false)
                .map_err(|_| a.wrong("section", "the name of a section"))?,
        ),
    };
    let args = model::Args {
        path: a.path("path")?,
        section,
        format: Format::Json,
    };
    json_result(&model::answer(&global(), &args).map_err(|s| s.message())?)
}

fn call_outline(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let args = outline::Args {
        page: a.path("page")?.unwrap_or_default(),
        build: a.string("build")?,
        format: Format::Json,
    };
    json_result(&outline::answer(&server.cache, &global(), &args).map_err(|s| s.message())?)
}

fn call_link(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let args = link::Args {
        target: a.string("target")?.unwrap_or_default(),
        from: a.path("from")?.unwrap_or_default(),
        format: Format::Json,
    };
    json_result(&link::answer(&server.cache, &global(), &args).map_err(|s| s.message())?)
}

fn call_refs(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let detailed = a.detailed()?;
    let global = global_at(a.path("project")?.as_deref())?;
    let args = refs::Args {
        target: a.string("target")?.unwrap_or_default(),
        limit: a.count("limit")?,
        format: if detailed { Format::Json } else { Format::Text },
    };
    let refs = refs::answer(&server.cache, &global, &args).map_err(|s| s.message())?;
    if detailed {
        return json_result(&refs);
    }
    let mut out = Vec::new();
    refs::write_text(&mut out, &refs).map_err(|e| e.to_string())?;
    Ok(concise_result(out))
}

fn call_search(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let detailed = a.detailed()?;
    let paths: Vec<PathBuf> = a.strings("paths")?.into_iter().map(PathBuf::from).collect();
    let project = a.path("project")?;
    if project.is_some() && !paths.is_empty() {
        return Err("give `paths` or `project`, not both: `paths` find the project".into());
    }
    let global = global_at(project.as_deref())?;
    let args = search::Args {
        query: a.string("query")?.unwrap_or_default(),
        paths,
        page_type: a.string("type")?,
        case: a.boolean("case")?,
        word: a.boolean("word")?,
        summary: a.boolean("summary")?,
        builds: a.strings("build")?,
        limit: a.count("limit")?,
        format: if detailed { Format::Json } else { Format::Text },
    };
    let searched = search::answer(&server.cache, &global, &args).map_err(|s| s.message())?;
    if detailed {
        return json_result(&searched);
    }
    let mut out = Vec::new();
    if args.summary {
        search::write_summary(&mut out, &searched)
    } else {
        search::write_text(&mut out, &searched)
    }
    .map_err(|e| e.to_string())?;
    Ok(concise_result(out))
}

fn call_resolve(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let args = resolve::Args {
        page: a.path("page")?.unwrap_or_default(),
        build: a.string("build")?,
        emit: match a.string("emit")?.as_deref() {
            None => None,
            Some("plain") => Some(resolve::Emit::Plain),
            Some("site") => Some(resolve::Emit::Site),
            Some(_) => return Err(a.wrong("emit", "`plain` or `site`")),
        },
        frontmatter: a.boolean("frontmatter")?,
        format: Format::Json,
    };
    let project = a.path("project")?;
    if project.is_some() && resolve::permalink_name(&args.page).is_none() {
        return Err("`project` goes with a `page` of `go:<name>`: a page finds its project".into());
    }
    let global = global_at(project.as_deref())?;
    let answer = resolve::answer(&server.cache, &global, &args).map_err(|s| s.message())?;
    match &answer {
        resolve::Answer::Page(resolved) => json_result(resolved),
        resolve::Answer::Help {
            help: ascribe_query::PermalinkHelp::Found(help),
            ..
        } => json_result(help),
        resolve::Answer::Help { .. } => Err(answer.no_help().unwrap_or_default()),
    }
}

fn call_fmt(_server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let paths: Vec<PathBuf> = a.strings("paths")?.into_iter().map(PathBuf::from).collect();
    let global = global_at(paths.first().map(PathBuf::as_path))?;
    let args = fmt::Args {
        check: true,
        paths,
        format: Format::Json,
    };
    let (_, report) = fmt::answer(&global, &args).map_err(|e| e.to_string())?;
    json_result(&report)
}

fn call_diff(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let global = global_at(a.path("project")?.as_deref())?;
    let args = diff::Args {
        page: None,
        base: a.string("base")?,
        base_exact: a.boolean("base_exact")?,
        build: a.strings("build")?,
        format: diff::Format::Json,
        exit_code: false,
        pages_only: !a.boolean("blocks")?,
    };
    let (_, diff) = diff::answer(&server.cache, &global, &args)?;
    json_result(&diff.report)
}

fn call_rename(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let args = rename::Args {
        target: a.string("target")?.unwrap_or_default(),
        new: a.string("new")?.unwrap_or_default(),
        format: Format::Json,
    };
    let answer = rename::answer(&server.cache, &global(), &args, false).map_err(|s| s.message)?;
    json_result(&answer.json)
}

fn call_addresses_move(_server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let global = global_at(a.path("project")?.as_deref())?;
    let project = crate::context::load_project(&global).map_err(|f| f.to_string())?;
    let from_git = a.boolean("from_git")?;
    let (old, new) = (a.string("old")?, a.string("new")?);
    if from_git == (old.is_some() || new.is_some()) || old.is_some() != new.is_some() {
        return Err("give `old` and `new`, or `from_git`".into());
    }
    let args = addresses::MoveArgs {
        old,
        new,
        from_git,
        base: a.string("base")?,
        base_exact: a.boolean("base_exact")?,
        format: Format::Json,
    };
    match addresses::answer_move(&project, &args, false).map_err(|e| e.to_string())? {
        addresses::Moved::One(json) => json_result(&json),
        addresses::Moved::FromGit(json) => json_result(&json),
    }
}

fn call_addresses_retire(_server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let global = global_at(a.path("project")?.as_deref())?;
    let project = crate::context::load_project(&global).map_err(|f| f.to_string())?;
    let args = addresses::RetireArgs {
        old: a.string("old")?.unwrap_or_default(),
        reason: a.string("reason")?.unwrap_or_default(),
        format: Format::Json,
    };
    let json = addresses::answer_retire(&project, &args, false).map_err(|e| e.to_string())?;
    json_result(&json)
}
