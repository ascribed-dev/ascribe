# Phase 9: Hooks and the plugin

Part of [Agents](README.md). Requires phases 4 and 7. Rust, and a plugin's files.

## Goal

An agent's work on an Ascribe project is checked without the agent having to remember: its edits as it makes them, and the whole project before it finishes. One hook command serves the harnesses that share Claude Code's hook format, and one plugin serves the harnesses that share its plugin format.

## Context

- [The research report](../../reports/Agent%20first%20interfaces%20for%20docs%20tools.md), "Skills reach nine harnesses from one directory; hooks reach three from one script" and "The language server is a free channel into Claude Code". What it records, as of October 2026, each to confirm against the live documentation before building:
  - **Hooks.** Codex and GitHub Copilot adopted Claude Code's hook JSON. After a tool runs (`PostToolUse`, Copilot's `postToolUse`), a hook informs the model with `additionalContext`, or with `decision: "block"` and a `reason`; for Claude Code, exit code `2` with the message on standard error also reaches the model, and plain standard output on exit `0` does not. The input's field names differ by harness (Copilot's are camelCase). The edit has already happened by then: nothing here stops an edit.
  - **Stop hooks.** Claude Code's `Stop` (and `SubagentStop`), Codex's `Stop`, and Copilot's `agentStop` can return `decision: "block"` with a `reason`, after which the agent continues and can act on it. Each passes a flag (`stop_hook_active`) saying the agent is already continuing because of a stop hook, so a hook can avoid a loop.
  - **Language servers in Claude Code.** A plugin can declare a language server (`.lsp.json`, or `lspServers` in the manifest: a `command`, `extensionToLanguage`, and `diagnostics`). Claude Code starts it the first time it edits a matching file, and pushes new diagnostics into context after each edit. It doesn't start plugin language servers in cloud sessions.
  - **Plugins.** Copilot's CLI reads `.claude-plugin/` manifests; VS Code and Cursor read the Agent Plugins manifest (a root `plugin.json`). One folder can carry both.
- Phase 1's `--editor-build`, `--format concise`, and timings; phase 4's skill; phase 7's server and `ascribe agents prompt`.
- [Decisions 9 to 12](README.md#decisions): one file of each kind for many harnesses; the hook's protocol lives in `ascribe agents`; packaged files are generated and checked; a quick check says what it skipped.
- `packages/cli`: `@ascribed/cli`'s `ascribe` is a Node launcher that runs the platform's binary. `scripts/release/` and `RELEASING.md`: every published thing is versioned in lock step.

## Design

### The hook

```
ascribe agents hook <HARNESS> [--event edit|stop]
```

`<HARNESS>` is `claude-code`, `codex`, or `copilot`. It decides how the input is read and how the answer is written; what's checked is the same.

**`--event edit`** (the default), for the hook after a file is written:

- Reads the hook's JSON from standard input and finds the file the tool wrote.
- Says nothing (exit `0`, no output) when the file isn't a source file of an Ascribe project, or has no errors.
- Otherwise checks that file as `ascribe check <file> --editor-build` does, and returns its **errors** to the model through the harness's channel for informing it after a tool: `check`'s concise lines (`file:line: [code] message`), at most 10, then "Checked build `<name>` only. Run `ascribe explain <code>` for any you don't recognize." Warnings aren't returned: they'd interrupt every edit.
- Gives up silently (exit `0`) after a time limit, so a slow project can't stall the agent. Take the limit from phase 1's timings; if a single-file check with `--editor-build` is over half a second on the corpora, say so in the pull request before going on.

**`--event stop`**, for the hook when the agent is about to finish:

- When the input says the agent is already continuing because of a stop hook, it exits `0` at once. It never asks twice in a row.
- When no Ascribe source file changed in the working tree (ask `git status`; outside a repository, skip this test), it exits `0`.
- Otherwise it runs the full `ascribe check`, every build. With errors, it blocks the stop with a reason: the counts, the first 10 errors in concise lines, and "Fix these, then run `ascribe check`." With none, it exits `0`. Warnings don't block.
- Its time limit is longer (the whole-project check takes up to 2 s on the corpora), and on reaching it, it lets the agent stop.

The hook never writes a file. For each harness, the pull request names the output channel used for each event and links the documentation that defines it. If a harness's current documentation gives no way to inform the model after a tool, leave that harness's edit event out and say so.

### Finding `ascribe`

Hook and server entries run a plain `ascribe`, found on the path. There's no wrapper script, so nothing depends on a shell.

- The docs say how to put it on the path: `npm install -g @ascribed/cli`, or the release's binary.
- A project that pins Ascribe in `node_modules` gets entries naming the pinned binary's path from `--with-hook`, below.
- Measure the Node launcher's start-up against the native binary's for one hook run, and put both in the pull request. If the launcher adds more than the check itself takes, the docs recommend the binary.
- In the check by hand, see what each harness shows when `ascribe` isn't on the path. If it's an error on every edit, stop and report: the entries then need a way to stay quiet.

### The plugin

In `plugins/ascribe/`, with both manifests: `.claude-plugin/plugin.json` and a marketplace manifest for Claude Code and Copilot's CLI, and a root Agent Plugins `plugin.json` for VS Code and Cursor. It bundles:

- **The language server** (`.lsp.json`): `ascribe lsp` for `.md` files, with diagnostics pushed after edits. This is the per-edit check in Claude Code's terminal sessions, with no second process per edit. The server reports the editor's build, as the edit hook does.
- **The stop hook:** `ascribe agents hook claude-code --event stop`.
- **No edit hook.** With the language server pushing diagnostics, an edit hook would report every problem twice. Sessions where the plugin's language server doesn't run (cloud sessions) use the project's own hook entries, below.
- **The MCP server:** `ascribe mcp`.
- **The skill:** phase 4's `SKILL.md`.
- **Commands:** `/ascribe:check`, `/ascribe:new-page`, and `/ascribe:review`, whose bodies are the named prompts `fix`, `new-page`, and `review`.

**The plugin's text files are generated, and a test keeps them so.** The skill is `ascribe agents sync`'s skill, and each command's body is `ascribe agents prompt <name>`'s output. A test compares every file with the binary's output and fails on a difference; `ASCRIBE_BLESS=1` rewrites the files, as the generated docs do.

The plugin's README says how to turn the pushed diagnostics off (`diagnostics: false`) for a project where they crowd the context, and that the stop hook still checks.

### Without the plugin

`ascribe agents sync --with-hook` writes hook entries into the project, as merges that leave other entries alone, for each target named or already present:

| Target | File | Entries |
|---|---|---|
| `claude` | `.claude/settings.json`, `.mcp.json` | The edit hook, the stop hook, the MCP server |
| `codex` | `.codex/hooks.json` | The edit hook, the stop hook |
| `copilot` | `.github/hooks/ascribe.json` | The edit hook, the stop hook (phase 10 covers the rest of Copilot) |

They name `ascribe` as the project runs it (the path into `node_modules/.bin` when the project pins `@ascribed/cli`, a plain `ascribe` otherwise). `--check` covers them. The docs say to use the plugin or the project's entries for Claude Code, not both.

## Tasks

1. `ascribe agents hook`, with tests that feed each harness's recorded inputs for both events: a page with an error, a clean page, a page with only warnings, a file outside any project, a non-Markdown file, a deleted file, malformed input, the time limit, and for `stop`: nothing changed, errors, no errors, and already continuing. Each test checks the exit code and both output streams.
2. The plugin's files, a test that both manifests parse and name files that exist, the generated-files test, and its place in the release scripts.
3. `--with-hook` for the three targets, with tests for merging into existing settings.
4. A check by hand in Claude Code, in its terminal and in its VS Code extension: install the plugin from the local marketplace, break a link, and see Claude get the problem once (not twice) and fix it; finish a turn with an error left and see the stop hook hold it; then the same with `ascribe` off the path. Note how much context the pushed diagnostics take on a page with many problems. Then the project's entries in Codex, if you have it. Say in the pull request what you ran and saw.
5. `docs/agents.md` ("Hooks", "The plugin"), `plugins/ascribe/README.md`, `CHANGELOG.md`.

## Out of scope

Publishing to a public marketplace (a release decision); hook variants for Cursor and Gemini CLI, whose formats differ (add one when someone asks); applying safe fixes from the hook; anything that calls a model.

## Acceptance criteria

- In Claude Code with the plugin, an edit that breaks a link reaches Claude once, without being asked for.
- An agent can't finish a turn with errors in the project without being told, and is told at most once per stop.
- An edit to a file outside an Ascribe project costs no measurable time and prints nothing.
- The hook is one command with no script around it, and works on Windows.
- The plugin's skill and command files can't drift from the binary's output without a test failing.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
pnpm format:check && pnpm lint && pnpm test
```

## Commits

1. "Add the agent hook for edits"
2. "Check the project before an agent finishes"
3. "Add the Ascribe plugin"
4. "Write hook and MCP settings from agents sync"
