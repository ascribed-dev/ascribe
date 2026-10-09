# ascribe-mcp

The [Model Context Protocol](https://modelcontextprotocol.io) over standard input and output, for `ascribe mcp`: one JSON-RPC message per line in, one per line out. It knows the protocol and nothing about Ascribe. Its caller supplies the tools, resources, and prompts through the `Handler` trait, and the binary's `crates/ascribe-cli/src/mcp/` is that caller, since each tool returns what its command writes.

It's synchronous, with no async runtime, HTTP, or MCP library (decision 1 in `project-docs/decisions.md`): `serve` reads a line, answers it, and reads the next, until its input ends.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/lib.rs` | `Handler`, `Tool`, `ToolResult`, `Annotations`, `Resource`, `ResourceTemplate`, `Prompt`, `ServerInfo`, `MODERN`, `LEGACY` | What a server offers, as the caller gives it, and the revisions it speaks. |
| `src/server.rs` | `serve` | Reads messages, answers each, and writes what went wrong to the log it's given. |

## Revisions

It speaks 2026-07-28 (`MODERN`): no handshake, each request carries its revision and the client's capabilities in `_meta`, `server/discover` says what the server supports, and every result has `resultType`. A request without the revision, or with one it doesn't speak, is an error that lists the revisions it does (`-32022`).

A client that sends `initialize` gets the older handshake, for 2025-11-25 and 2025-06-18 (`LEGACY`): the version it asked for when it's one of them, else the newest. Those results leave out what 2026-07-28 added (`resultType`, `ttlMs`, `cacheScope`), `ping` is answered, and a missing resource is `-32002`.

## Rules

- **Bad arguments are the handler's to report** as a tool result with `is_error`, which the model reads. A protocol error is for what the model can't fix: an unknown tool, resource, or prompt, a malformed message, or a revision it doesn't speak.
- **A line is a message.** A message never has a newline inside it, and a malformed line is answered with a parse error and the server goes on. A message split across reads is put back together.
- **A panic in a handler doesn't stop the server.** It's answered as an internal error, and logged.
- **Nothing is printed.** The log is a writer the caller gives, standard error in the binary.

`tests/protocol.rs` drives it as a client does, over both revisions. The binary's `crates/ascribe-cli/tests/mcp.rs` drives `ascribe mcp` itself.
