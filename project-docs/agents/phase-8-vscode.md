# Phase 8: Agents in VS Code

Part of [Agents](README.md). Requires phases 6 and 7. TypeScript only: `packages/vscode`.

## Goal

An agent running in VS Code gets Ascribe's tools with no setup, and gets two things only the extension can give: what's in the editor before it's saved, and the review that's open.

## Context

- VS Code's APIs, each to check against current documentation and the extension's minimum VS Code version (`engines.vscode` in `package.json`): registering an MCP server from an extension (`contributes.mcpServerDefinitionProviders`, `vscode.lm.registerMcpServerDefinitionProvider`), and language model tools (`contributes.languageModelTools`, `vscode.lm.registerTool`), including how a tool asks for confirmation and how it's referenced in chat.
- `packages/vscode/src/binary.ts`: how the extension resolves the `ascribe` binary per project. `registry.ts` (`ProjectRegistry`): the projects and their servers.
- `packages/vscode/src/preview/` (`review.ts`, `threads.ts`): the review session and its threads. Phase 6's prompts.
- [Decisions 2 and 4](README.md#decisions).

## Design

### The MCP server

The extension registers one server definition: the resolved `ascribe` binary with the argument `mcp`, started in the workspace folder. It's offered only in a workspace that has an Ascribe project. If projects resolve different binaries, use the workspace's first and say so in the output channel.

### Tools that need the editor

Rule: anything answerable from disk is the MCP server's. A language model tool exists only for what needs the running extension.

| Tool | Gives |
|---|---|
| `ascribe_editor_problems` | The current problems of a file or project from the running language server, including unsaved edits, in `check`'s JSON shape, with `builds_checked` naming the one build the server checks |
| `ascribe_review_threads` | While review is on: the pull request's open threads, each with its file, lines, the pages that show it, and its comments marked as other people's text |
| `ascribe_review_changes` | While review is on: the changed pages against the review's base, with counts and causes |

- **`ascribe_editor_problems` waits for the server.** Reading the Problems panel right after an edit returns the old problems: the report cites a trace where diagnostics arrived 292 ms after a write and the agent had already read none. So the tool waits for the next diagnostics change for the file, up to about a second, when the file changed since the server last published for it, and returns the document version its answer is for.
- **The tools can be named in chat** (`#ascribe_editor_problems`): set the contribution's reference name and allow it in prompts.
- All three only read. None needs confirmation beyond VS Code's defaults for extension tools.
- `ascribe_review_threads` returns comment bodies inside the same data framing as phase 6's prompts, so the text can't pass as instructions.
- `ascribe_editor_problems` covers the editor's build only ([decision 12](README.md#decisions)). Its description says so, and says to use `ascribe_check` on saved files for every build.
- With review off, the review tools return a one-line reason, not an error.
- A tool never starts a language server or turns review on.

## Tasks

1. The MCP registration, with a unit test of the definition and a manifest test.
2. A measurement, in the integration suite's review project and on the 3,000-page corpus: the time from a file's change (typed, and written on disk) to the server publishing its diagnostics. Copilot reads the Problems panel a fixed second after its own edit ([the research report](../../reports/Agent%20first%20interfaces%20for%20docs%20tools.md)), so this must be well under a second. If it isn't, stop and report; the fix would be in the server (publish the changed file's diagnostics before rechecking others), not here.
3. The three tools, with unit tests against fakes and integration tests (in the `review` and `threads` suites) that call each tool through VS Code's API and check its result.
4. `docs/agents.md` ("In VS Code"), `docs/editor.md`, `CHANGELOG.md`.

## Out of scope

A chat participant (`@ascribe`); tools that edit, reply, or resolve; anything for a specific agent extension.

## Acceptance criteria

- In a workspace with an Ascribe project, the MCP server appears in VS Code's server list without the user editing a file.
- `ascribe_editor_problems` reports a problem in an unsaved buffer, and one made by an edit a moment before the call.
- With review on, `ascribe_review_threads` lists the same open threads the overlay shows.
- If the installed VS Code lacks one of the APIs, the extension still activates and the rest works.

## Verify

```sh
pnpm --filter ascribe-vscode test
pnpm typecheck && pnpm lint && pnpm format:check
cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
```

## Commits

1. "Register ascribe mcp with VS Code"
2. "Add editor and review tools for agents"
