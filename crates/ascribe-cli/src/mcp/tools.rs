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
use crate::commands::{check, diff, fmt, link, model, outline, refs, render};

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
            phrases, features, glossary terms, widgets, and builds. Example: {\"path\": \
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
                        "enum": ["types", "dimensions", "phrases", "features", "glossary",
                            "widgets", "builds"],
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
            to one of them. Example: {\"page\": \"docs/keys.md\"} returns `headings` such as \
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
            keys\", \"href\": \"keys.md#rotate-keys\"}`.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "The link's destination as the page writes it: \
                            `keys.md`, `keys.md#rotate-keys`, `#install`.",
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
        description: "Find where a page, a heading, a fragment, a phrase, or another content \
            model entry is used, before changing or removing it. Example: {\"target\": \
            \"phrase:product\", \"project\": \"docs\"} returns lines such as \
            `docs/install-agent.md:69:63: phrase` and `8 places use phrase:product.`",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "A page or fragment (`keys.md`), a heading \
                            (`keys.md#rotate-keys`), `phrase:<key>`, `feature:<key>`, \
                            `term:<id>`, `dimension:<name>`, `note:<type>`, or \
                            `widget:<name>`. A path is from the server's working directory or \
                            from the content root.",
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
        name: "ascribe_render",
        title: "Render a page",
        command: "ascribe render",
        description: "Read a page as one build's readers see it, as plain Markdown: \
            directives, phrases, includes, and availability resolved. Example: {\"page\": \
            \"docs/keys.md\", \"build\": \"cloud\"} returns `text` starting `# API keys`.",
        input: || {
            json!({
                "type": "object",
                "properties": {
                    "page": page(),
                    "build": {
                        "type": "string",
                        "description": "The build whose readers to show it as; needed when \
                            the project has more than one.",
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
        output: || schema(include_str!("../../../../schemas/render.schema.json")),
        call: call_render,
    },
    Spec {
        name: "ascribe_format",
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
        call: call_format,
    },
    Spec {
        name: "ascribe_changes",
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
        call: call_changes,
    },
];

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
        summary: false,
        format: if detailed {
            check::Format::Json
        } else {
            check::Format::Concise
        },
        deny_warnings: false,
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
    let args = refs::Args {
        target: a.string("target")?.unwrap_or_default(),
        limit: a.count("limit")?,
        project: a.path("project")?,
        format: if detailed { Format::Json } else { Format::Text },
    };
    let refs = refs::answer(&server.cache, &global(), &args).map_err(|s| s.message())?;
    if detailed {
        return json_result(&refs);
    }
    let mut out = Vec::new();
    refs::write_text(&mut out, &refs).map_err(|e| e.to_string())?;
    Ok(concise_result(out))
}

fn call_render(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
    let args = render::Args {
        page: a.path("page")?.unwrap_or_default(),
        build: a.string("build")?,
        frontmatter: a.boolean("frontmatter")?,
        format: Format::Json,
    };
    json_result(&render::answer(&server.cache, &global(), &args).map_err(|s| s.message())?)
}

fn call_format(_server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
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

fn call_changes(server: &Server, a: &Arguments<'_>) -> Result<ToolResult, String> {
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
