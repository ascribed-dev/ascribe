//! The loop: read a line, answer it, write a line.

use std::io::{self, BufRead, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};

use serde_json::{Map, Value, json};

use crate::{Handler, LEGACY, MODERN, ServerInfo};

/// The `_meta` key that carries a request's protocol version.
const PROTOCOL_VERSION: &str = "io.modelcontextprotocol/protocolVersion";
/// The `_meta` key that carries the client's capabilities.
const CLIENT_CAPABILITIES: &str = "io.modelcontextprotocol/clientCapabilities";
/// The result `_meta` key that names the server.
const SERVER_INFO: &str = "io.modelcontextprotocol/serverInfo";

/// How long a client may keep a list that doesn't change while the server
/// runs: the tools, the prompts, and the resource templates.
const FIXED_TTL_MS: u64 = 3_600_000;

/// JSON-RPC's error codes, and MCP's.
mod code {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;
    /// A resource that isn't there, in the revisions before 2026-07-28.
    pub const LEGACY_RESOURCE_NOT_FOUND: i64 = -32002;
    pub const UNSUPPORTED_PROTOCOL_VERSION: i64 = -32022;
}

/// A JSON-RPC error.
struct RpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl RpcError {
    fn new(code: i64, message: impl Into<String>) -> RpcError {
        RpcError {
            code,
            message: message.into(),
            data: None,
        }
    }

    fn params(message: impl Into<String>) -> RpcError {
        RpcError::new(code::INVALID_PARAMS, message)
    }
}

/// Which revision a request is served as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Era {
    /// 2026-07-28: no handshake, and results carry `resultType`.
    Modern,
    /// The revision `initialize` agreed on.
    Legacy(&'static str),
}

/// What the server remembers between messages: only the revision an
/// `initialize` agreed on, for the clients that send one.
struct State<'a> {
    info: &'a ServerInfo,
    legacy: Option<&'static str>,
}

/// Serves `handler` until `input` ends: reads one JSON-RPC message per line
/// from `input`, and writes each response, on one line, to `output`. What's
/// worth knowing about a message that couldn't be served goes to `log`, one
/// line each.
///
/// A line that isn't JSON gets a parse error, a request it can't serve gets
/// a JSON-RPC error, and the loop goes on. Notifications, and responses from
/// the client, get no answer.
///
/// # Errors
///
/// Reading `input` or writing `output` failed.
pub fn serve(
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    handler: &mut dyn Handler,
    info: &ServerInfo,
    log: &mut dyn Write,
) -> io::Result<()> {
    let mut state = State { info, legacy: None };
    let mut line = Vec::new();
    loop {
        line.clear();
        if input.read_until(b'\n', &mut line)? == 0 {
            return Ok(());
        }
        let text = line.trim_ascii();
        if text.is_empty() {
            continue;
        }
        let Some(reply) = answer(text, &mut state, handler, log) else {
            continue;
        };
        // Compact JSON escapes every newline inside a string, so the reply
        // is one line.
        let mut bytes = serde_json::to_vec(&reply).map_err(io::Error::other)?;
        bytes.push(b'\n');
        output.write_all(&bytes)?;
        output.flush()?;
    }
}

/// The reply to one line, if it gets one.
fn answer(
    text: &[u8],
    state: &mut State<'_>,
    handler: &mut dyn Handler,
    log: &mut dyn Write,
) -> Option<Value> {
    let message: Value = match serde_json::from_slice(text) {
        Ok(message) => message,
        Err(e) => {
            let _ = writeln!(log, "ascribe-mcp: a line that isn't JSON: {e}");
            return Some(failure(
                Value::Null,
                RpcError::new(code::PARSE_ERROR, format!("not JSON: {e}")),
            ));
        }
    };
    let Value::Object(message) = message else {
        return Some(failure(
            Value::Null,
            RpcError::new(
                code::INVALID_REQUEST,
                "a message is one JSON object; batches aren't supported",
            ),
        ));
    };
    let Some(method) = message.get("method") else {
        // A response from the client: the server sends no requests, so
        // there's nothing it answers.
        return None;
    };
    let id = message.get("id")?.clone();
    if !(id.is_string() || id.is_number()) {
        return Some(failure(
            Value::Null,
            RpcError::new(
                code::INVALID_REQUEST,
                "a request's id is a string or a number",
            ),
        ));
    }
    let Some(method) = method.as_str() else {
        return Some(failure(
            id,
            RpcError::new(code::INVALID_REQUEST, "`method` is a string"),
        ));
    };
    let empty = Map::new();
    let params = match message.get("params") {
        None => &empty,
        Some(Value::Object(params)) => params,
        Some(_) => return Some(failure(id, RpcError::params("`params` is an object"))),
    };
    let served = catch_unwind(AssertUnwindSafe(|| {
        dispatch(method, params, state, handler)
    }))
    .unwrap_or_else(|_| {
        Err(RpcError::new(
            code::INTERNAL_ERROR,
            format!("`{method}` failed inside the server; this is a bug in Ascribe"),
        ))
    });
    Some(match served {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(e) => {
            let _ = writeln!(log, "ascribe-mcp: {method}: {}", e.message);
            failure(id, e)
        }
    })
}

fn failure(id: Value, error: RpcError) -> Value {
    let mut body = json!({ "code": error.code, "message": error.message });
    if let Some(data) = error.data {
        body["data"] = data;
    }
    json!({ "jsonrpc": "2.0", "id": id, "error": body })
}

/// Which revision serves a request: the one in its `_meta`, else the one
/// `initialize` agreed on.
fn era_of(method: &str, params: &Map<String, Value>, state: &State<'_>) -> Result<Era, RpcError> {
    let meta = params.get("_meta").and_then(Value::as_object);
    match meta.and_then(|m| m.get(PROTOCOL_VERSION)) {
        Some(requested) => {
            let requested = requested.as_str().unwrap_or_default();
            if !MODERN.contains(&requested) {
                return Err(RpcError {
                    code: code::UNSUPPORTED_PROTOCOL_VERSION,
                    message: "Unsupported protocol version".to_owned(),
                    data: Some(json!({ "supported": MODERN, "requested": requested })),
                });
            }
            if !meta.is_some_and(|m| m.get(CLIENT_CAPABILITIES).is_some_and(Value::is_object)) {
                return Err(RpcError::params(format!(
                    "`_meta` needs `{CLIENT_CAPABILITIES}`, an object"
                )));
            }
            Ok(Era::Modern)
        }
        // A probe: answered whatever it carries, so a client can learn what
        // the server speaks.
        None if method == "server/discover" => Ok(Era::Modern),
        None => match state.legacy {
            Some(version) => Ok(Era::Legacy(version)),
            None => Err(RpcError::params(format!(
                "every request carries `_meta` with `{PROTOCOL_VERSION}` ({}), or the client \
                 sends `initialize` first ({})",
                MODERN.join(", "),
                LEGACY.join(", ")
            ))),
        },
    }
}

fn dispatch(
    method: &str,
    params: &Map<String, Value>,
    state: &mut State<'_>,
    handler: &mut dyn Handler,
) -> Result<Value, RpcError> {
    if method == "initialize" {
        return Ok(initialize(params, state));
    }
    let era = era_of(method, params, state)?;
    let mut result = match method {
        "server/discover" => {
            let mut result = json!({
                "supportedVersions": MODERN,
                "capabilities": capabilities(),
                "instructions": state.info.instructions,
            });
            cacheable(&mut result, FIXED_TTL_MS, "public");
            result
        }
        "ping" if era != Era::Modern => json!({}),
        "tools/list" => {
            let mut result = json!({ "tools": handler.tools() });
            cacheable(&mut result, FIXED_TTL_MS, "public");
            result
        }
        "tools/call" => call_tool(params, handler)?,
        "resources/list" => {
            let mut result = json!({ "resources": handler.resources() });
            // What resources there are depends on the projects on disk.
            cacheable(&mut result, 0, "private");
            result
        }
        "resources/templates/list" => {
            let mut result = json!({ "resourceTemplates": handler.resource_templates() });
            cacheable(&mut result, FIXED_TTL_MS, "public");
            result
        }
        "resources/read" => read_resource(params, era, handler)?,
        "prompts/list" => {
            let mut result = json!({ "prompts": handler.prompts() });
            cacheable(&mut result, FIXED_TTL_MS, "public");
            result
        }
        "prompts/get" => get_prompt(params, handler)?,
        _ => {
            return Err(RpcError::new(
                code::METHOD_NOT_FOUND,
                format!("no method `{method}`"),
            ));
        }
    };
    if let Value::Object(fields) = &mut result {
        match era {
            Era::Modern => {
                fields.insert("resultType".to_owned(), json!("complete"));
                fields.insert(
                    "_meta".to_owned(),
                    json!({ SERVER_INFO: server_info(state.info) }),
                );
            }
            // The fields 2026-07-28 added mean nothing to an older client.
            Era::Legacy(_) => {
                fields.remove("ttlMs");
                fields.remove("cacheScope");
            }
        }
    }
    Ok(result)
}

/// The 2025-11-25 handshake: agrees on the version the client asks for when
/// the server speaks it, else on the newest legacy one, which the client may
/// refuse.
fn initialize(params: &Map<String, Value>, state: &mut State<'_>) -> Value {
    let asked = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let agreed = LEGACY
        .iter()
        .find(|v| **v == asked)
        .or(LEGACY.first())
        .copied()
        .unwrap_or_default();
    state.legacy = Some(agreed);
    json!({
        "protocolVersion": agreed,
        "capabilities": capabilities(),
        "serverInfo": server_info(state.info),
        "instructions": state.info.instructions,
    })
}

fn capabilities() -> Value {
    json!({ "tools": {}, "resources": {}, "prompts": {} })
}

fn server_info(info: &ServerInfo) -> Value {
    json!({ "name": info.name, "version": info.version })
}

/// Adds the 2026-07-28 caching fields to a list's result.
fn cacheable(result: &mut Value, ttl_ms: u64, scope: &str) {
    result["ttlMs"] = json!(ttl_ms);
    result["cacheScope"] = json!(scope);
}

fn string_param<'p>(params: &'p Map<String, Value>, name: &str) -> Result<&'p str, RpcError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| RpcError::params(format!("`{name}` is a string, and it's required")))
}

fn arguments(params: &Map<String, Value>) -> Result<Map<String, Value>, RpcError> {
    match params.get("arguments") {
        None | Some(Value::Null) => Ok(Map::new()),
        Some(Value::Object(arguments)) => Ok(arguments.clone()),
        Some(_) => Err(RpcError::params("`arguments` is an object")),
    }
}

fn call_tool(params: &Map<String, Value>, handler: &mut dyn Handler) -> Result<Value, RpcError> {
    let name = string_param(params, "name")?;
    let arguments = arguments(params)?;
    let result = handler
        .call_tool(name, &arguments)
        .ok_or_else(|| RpcError::params(format!("Unknown tool: {name}")))?;
    let mut out = json!({
        "content": [{ "type": "text", "text": result.text }],
        "isError": result.is_error,
    });
    if let Some(structured) = result.structured {
        out["structuredContent"] = structured;
    }
    Ok(out)
}

fn read_resource(
    params: &Map<String, Value>,
    era: Era,
    handler: &mut dyn Handler,
) -> Result<Value, RpcError> {
    let uri = string_param(params, "uri")?;
    match handler.read_resource(uri) {
        Ok(resource) => {
            let mut result = json!({
                "contents": [{ "uri": uri, "mimeType": resource.mime_type, "text": resource.text }],
            });
            cacheable(&mut result, 0, "private");
            Ok(result)
        }
        Err(message) => {
            let code = match era {
                Era::Modern => code::INVALID_PARAMS,
                Era::Legacy(_) => code::LEGACY_RESOURCE_NOT_FOUND,
            };
            Err(RpcError {
                code,
                message,
                data: Some(json!({ "uri": uri })),
            })
        }
    }
}

fn get_prompt(params: &Map<String, Value>, handler: &mut dyn Handler) -> Result<Value, RpcError> {
    let name = string_param(params, "name")?;
    let arguments = arguments(params)?;
    let prompt = handler
        .get_prompt(name, &arguments)
        .ok_or_else(|| RpcError::params(format!("Unknown prompt: {name}")))?
        .map_err(RpcError::params)?;
    Ok(json!({
        "description": prompt.description,
        "messages": [{ "role": "user", "content": { "type": "text", "text": prompt.text } }],
    }))
}
