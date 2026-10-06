# Phase 7: `ascribe mcp`

Part of [Agents](README.md). Requires phases 1, 2, 4, and 5. Can run at the same time as phase 6. Rust only.

## Goal

`ascribe mcp` is an MCP server over standard input and output that offers the earlier phases' commands as typed tools, the project's rules as resources, and a few prompts. It's for agents that have no shell, and for hosts that turn a server's prompts into slash commands and approve tools one by one. It's a small surface beside the command line, not a replacement for it: every tool's definition costs an agent context whether it's used or not, and an agent with a shell does as well with the commands and the skill.

## Context

- The Model Context Protocol specification. [The research report](../../reports/Agent%20first%20interfaces%20for%20docs%20tools.md) ("MCP 2026-07-28 changed the handshake") records the revision to build against, **2026-07-28**, the first with breaking changes. Confirm each of these against its changelog, and say in the pull request which revision you built against:
  - `initialize`, `notifications/initialized`, and `ping` are gone. Each request carries the protocol version and the client's capabilities in `_meta`.
  - `server/discover` is required: it names the server, its capabilities, and the versions it speaks, and clients use it as the first probe on standard input.
  - Every result carries `resultType: "complete"`. Results of `tools/list`, `prompts/list`, `resources/list`, and `resources/read` carry `ttlMs` and `cacheScope`. `tools/list` returns tools in a fixed order.
  - Clients on the 2025-11-25 revision still send `initialize`; a server has to answer both.
  - Tools carry annotations (`readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`), can declare an `outputSchema` and return `structuredContent`, and report bad input as a tool result with `isError: true`, not as a protocol error.
- **The transport is not the language server's.** MCP's standard-input transport is newline-delimited JSON: one JSON-RPC message per line, with no embedded newlines and no `Content-Length` header. The `lsp-server` crate's transport can't be reused. What carries over from `crates/ascribe-lsp/src/server.rs` is the shape: a synchronous loop that reads a message, dispatches by method, and writes a reply.
- The official Rust SDK (`rmcp`) needs tokio, by the report's reading. [Decision 13](README.md#decisions) says the binary stays synchronous, so write the small part needed by hand: `server/discover`, the old `initialize` handshake, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, and `prompts/get`.
- [Decisions 2, 3, and 4](README.md#decisions). Decision 2 binds this phase: every tool, resource, and prompt has a command that gives the same thing.
- Phase 4's instruction block; phase 5's prompt module.

## Design

### The server

- A new crate, `ascribe-mcp`, holding the protocol and the tool table; `ascribe mcp` runs it.
- **Multi-project, with no session state.** Every tool takes a `path` and finds the nearest `ascribe.toml` at or above it, so one server serves a monorepo, and no call depends on an earlier one.
- **It caches loaded projects.** A loaded project is kept in memory so a second call is fast. Before each call, the server lists the project's files again (paths, sizes, and modification times, including `ascribe.toml`) and compares the listing with the one it loaded from; any difference, including a new or a deleted file, reloads the project. Listing is cheap beside loading. Measure a first and a second call on the 3,000-page corpus and put the numbers in the pull request. If the listing alone is slow there, stop and report.
- **Read-only.** No tool writes a file. Tools that produce changes return edits (`{file, range, new_text}`, the shape `check`'s `fixes` already use).
- **Logging** goes to standard error, never standard output.

### Tools

Each tool's result is the JSON its command writes. Names are prefixed so they're clear beside other servers' tools.

| Tool | Command it wraps | Notes |
|---|---|---|
| `ascribe_check` | `check` | `paths`, or `text` with `path` for unsaved content; `editor_build` |
| `ascribe_explain` | `explain` | |
| `ascribe_model` | `model` | optional `section` |
| `ascribe_outline` | `outline` | |
| `ascribe_link` | `link` | "does `keys.md#rotate-keys` exist from `guides/install.md`, and how do I write the link?" |
| `ascribe_refs` | `refs` | |
| `ascribe_render` | `render` | |
| `ascribe_format` | `fmt` | returns the edits; writes nothing. If `fmt` can't report edits without writing, add `ascribe fmt --format json --dry-run` first, so the tool still wraps a command. |
| `ascribe_changes` | `diff` | changed pages against a base, without the per-block detail unless asked |

For every tool:

- **The description** says in one sentence when to use it, and gives one example call with its result, since that's what an agent reads to choose. Descriptions are fixed text: nothing from a project's files is ever put into a tool's or a resource's description.
- **Annotations:** read-only, idempotent, and closed-world (`readOnlyHint: true`, `idempotentHint: true`, `openWorldHint: false`).
- **The result** is declared (`outputSchema`), returned as `structuredContent`, and also as a text block for clients that read only text. Text from a project's files comes back in fields named for what it is (`excerpt`, `source_path`), not loose in prose.
- **Bad arguments** (a path in no project, an unknown build) are tool results with `isError: true` and a sentence that says what to call instead.

`ascribe_check` and `ascribe_refs` take `response_format`: `concise` (the default: phase 1's concise lines) or `detailed` (the full JSON).

### Resources and prompts

- **Resources:** `ascribe://directives` (the cheat sheet), `ascribe://model/<project>` (phase 2's model summary), and `ascribe://instructions/<project>` (phase 4's block).
- Many hosts never show resources to the model, so nothing depends on them: the same text is in `ascribe_model` and the skill.
- **Prompts:** three named prompts, each in [the agent prompt format](README.md#the-agent-prompt-format):
  - `new-page` (arguments: type and title): the frontmatter the type requires and where the file goes;
  - `fix`: check the project, fix what's reported, check again (phase 5's project prompt when there are problems);
  - `review`: read the changed pages as readers see them and report what reads wrongly (phase 6's every-page prompt, if phase 6 is merged; otherwise its text here and phase 6 replaces it).

  In VS Code and Gemini CLI, a server's prompts appear as slash commands, so these three reach users there with no other file.

- **`ascribe agents prompt <NAME> [--arg key=value]...`** prints a named prompt, and `ascribe agents prompt --list` lists them. This is the command behind the MCP prompts (decision 2), and the one source for phases 9 and 10, whose command and prompt files are checked against its output.

## Tasks

1. The protocol, with tests that drive the server over pipes as a 2026-07-28 client and as a 2025-11-25 client: discovery or the handshake, listing (in the same order twice), a call, an unknown tool, bad arguments, a malformed line, and a message split across reads.
2. The tools, each tested against a temporary project, including two projects in one repository and a path in neither.
3. The cache, with tests: a file changed, a file added, a file deleted, and `ascribe.toml` changed between two calls each give the new answer.
4. Resources, the named prompts, and `ascribe agents prompt`.
5. A test that every tool's result equals its command's JSON for the same input.
6. `docs/agents.md` ("The MCP server": the configuration snippet for a generic client), `docs/cli.md`, `CHANGELOG.md`.

## Out of scope

An HTTP transport; tools that write; review threads (they need GitHub, which the binary doesn't reach); registering the server with any harness (phases 8 to 10).

## Acceptance criteria

- A generic MCP client can list and call every tool, on either revision.
- Every tool is annotated read-only and declares its result.
- No tool changes a file on disk; a test checks the project's files are unchanged after calling each.
- Nothing is available only through MCP: each tool, resource, and prompt names the command that gives the same thing, and a test checks the table.
- The binary gains no async runtime and no HTTP library (`cargo tree` in the pull request).

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

## Commits

1. "Speak the Model Context Protocol over standard input and output"
2. "Offer Ascribe's commands as MCP tools"
3. "Offer the project's rules as MCP resources and prompts"
