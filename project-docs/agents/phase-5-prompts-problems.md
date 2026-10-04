# Phase 5: Agent prompts for problems

Part of [Agents](README.md). Requires phase 1. Can run at the same time as phase 4. Rust and TypeScript.

## Goal

A person looking at a problem can hand it to their agent in one step. **Prompt agent** on a diagnostic, a file, or a project builds a prompt with what Ascribe knows, and copies it or opens the editor's chat with it filled in. The same prompts come from the command line, for people whose agent lives in a terminal.

This phase also builds the delivery (copy, or open in chat) that phase 6 reuses.

## Context

- [The agent prompt format](README.md#the-agent-prompt-format) and [decisions 5, 6, 7, and 8](README.md#decisions).
- Phase 1's `help`, `docs`, and `fixes` on each diagnostic.
- `crates/tessera-lsp/src/code_action.rs`: where quick fixes are offered. `crates/tessera-lsp/src/server.rs` (`execute_command`): how the server runs a command.
- `packages/vscode/src/`: how commands are registered, and `package.json`'s menus.
- VS Code, to check against current documentation: the command that opens chat with a query that isn't submitted; `vscode.env.clipboard`; `vscode.env.openExternal` for another extension's URI handler.
- No agent reads LSP code actions (the report found none that does), so the code action below is for the person at the editor. Agents get fixes from `check`'s JSON.

## Design

### The prompts

Built in the binary, in one module, tested as snapshots:

| Prompt | Task line | Carries |
|---|---|---|
| One problem | "Fix this problem in `<file>`." | Code, message, the line's text, `help`, the allowed values when the problem is about one (from the model), and whether a quick fix exists ("Ascribe has a safe automatic fix: `<title>`", or "…a fix to review…" for an unsafe one, from phase 1's `applicability`) |
| A file's problems | "Fix the N problems `ascribe check` reports in `<file>`." | Each problem in one line (`line: [code] message`), at most 20, then the command for the rest |
| A project's problems | "Fix the problems `ascribe check` reports in this project." | Counts by file, at most 20 files, and the command |

A problem that has an automatic fix and nothing else to decide doesn't need an agent: the single-problem prompt is offered only after the quick fix, never instead of it.

### Where they come from

- **`ascribe check --format prompt`** writes the file or project prompt (by whether paths were given) to standard output, and nothing when there are no problems. Exit codes are `check`'s.
- **`ascribe/agentPrompt`**, a server request: `{ kind: "problem" | "file" | "project", textDocument?, diagnostic?, unsaved?: string[] }` to `{ prompt: string } | null`. It answers from the current snapshot, so it includes unsaved edits, and it covers what the server checks: the editor's build. The prompt says which build, and that `ascribe check` covers the rest.

The finishing line about `AGENTS.md` is written when the project's folder, or the repository's root, has one. The binary looks; phase 4 doesn't have to be merged for that.

### Unsaved text

The editor's prompt can name lines that the file on disk doesn't have yet, and an agent in a terminal reads the disk. Ascribe doesn't save files for the user. So the extension passes the URIs of the documents with unsaved changes (`unsaved`), and a prompt about one of them carries the format's unsaved line: "The file has unsaved changes; save it before you start." The notice shown on delivery says the same.

### In VS Code

- A code action on every Ascribe diagnostic, after its quick fixes: **Prompt agent: fix this problem**. It's a command (`ascribe.promptAgent`), not an edit.
- Palette commands: **Ascribe: Prompt Agent to Fix This File** and **Ascribe: Prompt Agent to Fix This Project**, shown only when there are problems.
- **Delivery.** A setting, `ascribe.agents.promptTarget`. Every target fills the prompt in and none sends it ([decision 5](README.md#decisions)):
  - `clipboard` (the default) copies the prompt and shows "Prompt copied. Paste it into your agent." in the status bar.
  - `chat` opens VS Code's chat with the prompt as the query, not submitted.
  - `claude-code` opens Claude Code's VS Code extension with the prompt, through its URI handler.
  - `cursor` opens Cursor's chat with the prompt, through its `cursor://` link. Never through Cursor's web link, which would send the prompt to a website.
  - A target that isn't available (the extension isn't installed, the command is missing) falls back to copying and says so.
  - The first time, a notification offers the targets that are available, once, and never again.

  [The research report](../../reports/Agent%20first%20interfaces%20for%20docs%20tools.md) records the handlers and their limits as of October 2026 (`workbench.action.chat.open` with a partial query; `vscode://anthropic.claude-code/open?prompt=`; `cursor://anysphere.cursor-deeplink/prompt?text=`; Claude Code's links take 5,000 characters, which is where the format's limit comes from). Confirm each before building, and leave out a target whose handler can submit without the user.
- One function does delivery (`deliverPrompt(prompt)`), which phase 6 calls.

## Tasks

1. The prompt builders and their snapshot tests: each kind, a problem with allowed values, a problem in a fragment (with `Shown on:`), a project in a subfolder (`Project:`), with and without `AGENTS.md`, an unsaved file, the caps, and the 5,000-character limit.
2. `--format prompt` and `ascribe/agentPrompt`, with CLI and scenario tests.
3. The extension's command, code action, palette commands, setting, and delivery, with unit tests, and an integration test that the code action is offered and the clipboard holds the prompt.
4. `docs/agents.md` ("Prompt your agent"), `docs/editor.md`, `docs/cli.md`, `CHANGELOG.md`.

## Out of scope

Review prompts (phase 6); sending a prompt without the user seeing it; choosing or detecting which agent the user has.

## Acceptance criteria

- The prompt for a broken link names the file and line, quotes the message, gives the fix advice, and ends with the two finishing lines.
- For a saved file and a problem in the editor's build, the command line and the editor give the same prompt.
- A prompt about a file with unsaved changes says so.
- With `chat`, `claude-code`, or `cursor`, the prompt appears in the agent's input and isn't sent.
- No prompt is built or offered when there are no problems.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter ascribe-vscode test
pnpm typecheck && pnpm lint && pnpm format:check
cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
```

## Commits

1. "Build agent prompts for problems"
2. "Write check's problems as an agent prompt"
3. "Offer Prompt agent on problems in the editor"
