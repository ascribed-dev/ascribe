//! The protocol, driven as a client drives it: lines in, lines out, with a
//! handler that has one tool, one resource, and one prompt.

#![allow(clippy::expect_used)]

use std::io::{self, BufReader, Read};

use ascribe_mcp::{
    Annotations, Handler, Prompt, PromptArgument, PromptText, Resource, ResourceTemplate,
    ResourceText, ServerInfo, Tool, ToolResult, serve,
};
use serde_json::{Map, Value, json};

struct Echo;

impl Handler for Echo {
    fn tools(&self) -> Vec<Tool> {
        ["echo", "other"]
            .into_iter()
            .map(|name| Tool {
                name: name.to_owned(),
                title: "Echo".to_owned(),
                description: "Says `text` back.".to_owned(),
                input_schema: json!({
                    "type": "object",
                    "properties": { "text": { "type": "string" } },
                    "required": ["text"],
                }),
                output_schema: Some(json!({
                    "type": "object",
                    "properties": { "text": { "type": "string" } },
                })),
                annotations: Annotations::READ_ONLY,
            })
            .collect()
    }

    fn call_tool(&mut self, name: &str, arguments: &Map<String, Value>) -> Option<ToolResult> {
        if name != "echo" {
            return None;
        }
        Some(match arguments.get("text").and_then(Value::as_str) {
            Some(text) => ToolResult::ok(text.to_owned(), json!({ "text": text })),
            None => ToolResult::error("`text` is required: call echo with {\"text\": \"…\"}"),
        })
    }

    fn resources(&mut self) -> Vec<Resource> {
        vec![Resource {
            uri: "test://one".to_owned(),
            name: "one".to_owned(),
            description: "The one resource.".to_owned(),
            mime_type: "text/markdown".to_owned(),
        }]
    }

    fn resource_templates(&self) -> Vec<ResourceTemplate> {
        Vec::new()
    }

    fn read_resource(&mut self, uri: &str) -> Result<ResourceText, String> {
        if uri == "test://one" {
            Ok(ResourceText {
                mime_type: "text/markdown".to_owned(),
                text: "# One\n".to_owned(),
            })
        } else {
            Err(format!("no resource {uri}"))
        }
    }

    fn prompts(&self) -> Vec<Prompt> {
        vec![Prompt {
            name: "greet".to_owned(),
            title: "Greet".to_owned(),
            description: "Greets someone.".to_owned(),
            arguments: vec![PromptArgument {
                name: "who".to_owned(),
                description: "Who.".to_owned(),
                required: true,
            }],
        }]
    }

    fn get_prompt(
        &mut self,
        name: &str,
        arguments: &Map<String, Value>,
    ) -> Option<Result<PromptText, String>> {
        (name == "greet").then(|| match arguments.get("who").and_then(Value::as_str) {
            Some(who) => Ok(PromptText {
                description: "A greeting.".to_owned(),
                text: format!("Greet {who}."),
            }),
            None => Err("`who` is required".to_owned()),
        })
    }
}

fn info() -> ServerInfo {
    ServerInfo {
        name: "test".to_owned(),
        version: "1.0.0".to_owned(),
        instructions: "Use echo.".to_owned(),
    }
}

/// A reader that hands out at most `chunk` bytes per read, so a message
/// arrives split across reads.
struct Trickle {
    bytes: Vec<u8>,
    at: usize,
    chunk: usize,
}

impl Read for Trickle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.chunk.min(buf.len()).min(self.bytes.len() - self.at);
        buf[..n].copy_from_slice(&self.bytes[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

/// Sends `input` and returns each line written back, parsed, and the log.
fn exchange_split(input: &str, chunk: usize) -> (Vec<Value>, String) {
    let mut reader = BufReader::with_capacity(
        4,
        Trickle {
            bytes: input.as_bytes().to_vec(),
            at: 0,
            chunk,
        },
    );
    let mut out = Vec::new();
    let mut log = Vec::new();
    serve(&mut reader, &mut out, &mut Echo, &info(), &mut log).expect("served");
    let out = String::from_utf8(out).expect("UTF-8");
    assert!(out.is_empty() || out.ends_with('\n'), "{out:?}");
    let replies = out
        .lines()
        .map(|line| serde_json::from_str(line).expect("a JSON line"))
        .collect();
    (replies, String::from_utf8(log).expect("UTF-8"))
}

fn exchange(input: &str) -> Vec<Value> {
    exchange_split(input, usize::MAX).0
}

/// A 2026-07-28 request.
fn modern(id: u64, method: &str, mut params: Value) -> String {
    params["_meta"] = json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": {},
        "io.modelcontextprotocol/clientInfo": { "name": "client", "version": "1" },
    });
    format!(
        "{}\n",
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
    )
}

/// A request with no `_meta`, as a 2025-11-25 client sends.
fn legacy(id: u64, method: &str, params: Value) -> String {
    format!(
        "{}\n",
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
    )
}

fn initialize(version: &str) -> String {
    legacy(
        0,
        "initialize",
        json!({
            "protocolVersion": version,
            "capabilities": {},
            "clientInfo": { "name": "client", "version": "1" },
        }),
    ) + "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n"
}

#[test]
fn a_modern_client_discovers_the_server() {
    let replies = exchange(&modern(1, "server/discover", json!({})));
    let result = &replies[0]["result"];
    assert_eq!(replies[0]["id"], 1);
    assert_eq!(result["resultType"], "complete");
    assert_eq!(result["supportedVersions"], json!(["2026-07-28"]));
    assert_eq!(
        result["capabilities"],
        json!({ "tools": {}, "resources": {}, "prompts": {} })
    );
    assert_eq!(result["instructions"], "Use echo.");
    assert_eq!(
        result["_meta"]["io.modelcontextprotocol/serverInfo"],
        json!({ "name": "test", "version": "1.0.0" })
    );
    assert!(result["ttlMs"].is_u64());
    assert_eq!(result["cacheScope"], "public");
}

#[test]
fn a_modern_client_lists_in_the_same_order_twice() {
    let input = modern(1, "tools/list", json!({})) + &modern(2, "tools/list", json!({}));
    let replies = exchange(&input);
    assert_eq!(replies[0]["result"]["tools"], replies[1]["result"]["tools"]);
    let tools = &replies[0]["result"];
    assert_eq!(tools["resultType"], "complete");
    assert_eq!(tools["cacheScope"], "public");
    assert!(tools["ttlMs"].as_u64().is_some_and(|ttl| ttl > 0));
    let names: Vec<&str> = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["echo", "other"]);
    assert_eq!(
        tools["tools"][0]["annotations"],
        json!({
            "readOnlyHint": true,
            "destructiveHint": false,
            "idempotentHint": true,
            "openWorldHint": false,
        })
    );
    assert!(tools["tools"][0]["outputSchema"].is_object());
    assert!(tools["tools"][0]["inputSchema"].is_object());

    let replies =
        exchange(&(modern(1, "prompts/list", json!({})) + &modern(2, "resources/list", json!({}))));
    assert_eq!(replies[0]["result"]["prompts"][0]["name"], "greet");
    assert_eq!(replies[1]["result"]["resources"][0]["uri"], "test://one");
    assert_eq!(replies[1]["result"]["cacheScope"], "private");
}

#[test]
fn a_modern_client_calls_a_tool() {
    let replies = exchange(&modern(
        7,
        "tools/call",
        json!({ "name": "echo", "arguments": { "text": "hi\nthere" } }),
    ));
    assert_eq!(
        replies[0],
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "result": {
                "content": [{ "type": "text", "text": "hi\nthere" }],
                "structuredContent": { "text": "hi\nthere" },
                "isError": false,
                "resultType": "complete",
                "_meta": {
                    "io.modelcontextprotocol/serverInfo": { "name": "test", "version": "1.0.0" },
                },
            },
        })
    );
}

#[test]
fn an_unknown_tool_is_a_protocol_error() {
    let replies = exchange(&modern(
        1,
        "tools/call",
        json!({ "name": "other", "arguments": {} }),
    ));
    assert_eq!(replies[0]["error"]["code"], -32602);
    assert_eq!(replies[0]["error"]["message"], "Unknown tool: other");
}

#[test]
fn bad_arguments_are_a_tool_result() {
    let replies = exchange(&modern(1, "tools/call", json!({ "name": "echo" })));
    let result = &replies[0]["result"];
    assert_eq!(result["isError"], true);
    assert!(result.get("structuredContent").is_none());
    assert!(
        result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("call echo")
    );
}

#[test]
fn a_malformed_line_gets_a_parse_error_and_the_server_goes_on() {
    let input = "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\"\n".to_owned()
        + "[1, 2]\n"
        + &modern(2, "tools/list", json!({}));
    let (replies, log) = exchange_split(&input, usize::MAX);
    assert_eq!(replies[0]["error"]["code"], -32700);
    assert_eq!(replies[0]["id"], Value::Null);
    assert_eq!(replies[1]["error"]["code"], -32600);
    assert_eq!(replies[2]["id"], 2);
    assert!(replies[2]["result"]["tools"].is_array());
    assert!(log.starts_with("ascribe-mcp: "), "{log}");
}

#[test]
fn a_message_split_across_reads_is_one_message() {
    let input = modern(
        1,
        "tools/call",
        json!({ "name": "echo", "arguments": { "text": "é" } }),
    ) + "\r\n"
        + &modern(2, "server/discover", json!({}));
    let (replies, _) = exchange_split(&input, 3);
    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0]["result"]["structuredContent"]["text"], "é");
    assert_eq!(replies[1]["id"], 2);
}

#[test]
fn a_request_without_a_version_is_refused() {
    let replies = exchange(&legacy(1, "tools/list", json!({})));
    assert_eq!(replies[0]["error"]["code"], -32602);
    assert!(
        replies[0]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("initialize")
    );

    let mut request: Value = serde_json::from_str(&modern(2, "tools/list", json!({}))).unwrap();
    request["params"]["_meta"]
        .as_object_mut()
        .unwrap()
        .remove("io.modelcontextprotocol/clientCapabilities");
    let replies = exchange(&format!("{request}\n"));
    assert_eq!(replies[0]["error"]["code"], -32602);
}

#[test]
fn an_unsupported_version_names_the_supported_ones() {
    let mut request: Value = serde_json::from_str(&modern(1, "tools/list", json!({}))).unwrap();
    request["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"] = json!("1900-01-01");
    let replies = exchange(&format!("{request}\n"));
    assert_eq!(
        replies[0]["error"],
        json!({
            "code": -32022,
            "message": "Unsupported protocol version",
            "data": { "supported": ["2026-07-28"], "requested": "1900-01-01" },
        })
    );
}

#[test]
fn notifications_and_responses_get_no_answer() {
    let input = "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/cancelled\",\"params\":{\"requestId\":1}}\n\
                 {\"jsonrpc\":\"2.0\",\"id\":5,\"result\":{}}\n\n";
    assert!(exchange(input).is_empty());
}

#[test]
fn a_legacy_client_shakes_hands_and_calls_a_tool() {
    let input = initialize("2025-11-25")
        + &legacy(1, "tools/list", json!({}))
        + &legacy(2, "tools/list", json!({}))
        + &legacy(
            3,
            "tools/call",
            json!({ "name": "echo", "arguments": { "text": "hi" } }),
        )
        + &legacy(4, "ping", json!({}));
    let replies = exchange(&input);
    assert_eq!(replies.len(), 5, "{replies:?}");
    assert_eq!(
        replies[0]["result"],
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": { "tools": {}, "resources": {}, "prompts": {} },
            "serverInfo": { "name": "test", "version": "1.0.0" },
            "instructions": "Use echo.",
        })
    );
    let list = &replies[1]["result"];
    assert_eq!(list, &replies[2]["result"]);
    assert!(list.get("resultType").is_none());
    assert!(list.get("ttlMs").is_none());
    assert!(list.get("cacheScope").is_none());
    assert_eq!(list["tools"][0]["name"], "echo");
    assert_eq!(
        replies[3]["result"],
        json!({
            "content": [{ "type": "text", "text": "hi" }],
            "structuredContent": { "text": "hi" },
            "isError": false,
        })
    );
    assert_eq!(replies[4]["result"], json!({}));
}

#[test]
fn a_legacy_client_gets_its_errors() {
    let input = initialize("2025-11-25")
        + &legacy(1, "tools/call", json!({ "name": "nope" }))
        + &legacy(2, "tools/call", json!({ "name": "echo", "arguments": {} }))
        + &legacy(3, "resources/read", json!({ "uri": "test://two" }))
        + "not json\n";
    let replies = exchange(&input);
    assert_eq!(replies[1]["error"]["code"], -32602);
    assert_eq!(replies[2]["result"]["isError"], true);
    assert_eq!(replies[3]["error"]["code"], -32002);
    assert_eq!(replies[4]["error"]["code"], -32700);
}

#[test]
fn an_unknown_legacy_version_gets_the_newest() {
    let replies = exchange(&initialize("2024-11-05"));
    assert_eq!(replies[0]["result"]["protocolVersion"], "2025-11-25");
    let replies = exchange(&initialize("2025-06-18"));
    assert_eq!(replies[0]["result"]["protocolVersion"], "2025-06-18");
}

#[test]
fn resources_and_prompts_are_read() {
    let input = modern(1, "resources/read", json!({ "uri": "test://one" }))
        + &modern(2, "resources/read", json!({ "uri": "test://two" }))
        + &modern(
            3,
            "prompts/get",
            json!({ "name": "greet", "arguments": { "who": "Ana" } }),
        )
        + &modern(4, "prompts/get", json!({ "name": "greet" }))
        + &modern(5, "prompts/get", json!({ "name": "nope" }))
        + &modern(6, "nope/nope", json!({}))
        + &modern(7, "ping", json!({}));
    let replies = exchange(&input);
    assert_eq!(
        replies[0]["result"]["contents"],
        json!([{ "uri": "test://one", "mimeType": "text/markdown", "text": "# One\n" }])
    );
    assert_eq!(replies[1]["error"]["code"], -32602);
    assert_eq!(
        replies[2]["result"]["messages"],
        json!([{ "role": "user", "content": { "type": "text", "text": "Greet Ana." } }])
    );
    assert_eq!(replies[3]["error"]["code"], -32602);
    assert_eq!(replies[4]["error"]["code"], -32602);
    assert_eq!(replies[5]["error"]["code"], -32601);
    assert_eq!(replies[6]["error"]["code"], -32601);
}
