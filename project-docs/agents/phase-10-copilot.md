# Phase 10: GitHub Copilot

Part of [Agents](README.md). Requires phases 4, 7, and 9. Rust (targets in `ascribe agents sync`) and docs.

## Goal

Copilot gets the loop in both its forms. In VS Code, phases 4 and 8 already give it the instructions, the skill, and the tools. This phase gives the cloud agent on GitHub, which has no editor, what it needs: Ascribe installed, the skill, the hooks, and the MCP server.

## Context

- [The research report](../../reports/Agent%20first%20interfaces%20for%20docs%20tools.md), "Seven places the plan's assumptions no longer match the documentation". What it records, as of October 2026, each to confirm against the live documentation before building:
  - **Prompt files are deprecated for the cloud agent** ("aren't loaded by Agent Host"). They still work in VS Code's picker. So nothing for the cloud agent can rest on them.
  - **Skills:** `.github/skills/` is read by VS Code, Copilot's CLI, and the cloud agent.
  - **Hooks:** the cloud agent loads `.github/hooks/*.json`, and honors only command handlers from there.
  - **The environment:** `.github/workflows/copilot-setup-steps.yml`, whose job must be named `copilot-setup-steps`, with a short list of honored keys.
  - **MCP for the cloud agent** is configured in the repository's settings on GitHub, as JSON with a `type`, a required `tools` allowlist, and secrets named with a `COPILOT_MCP_` prefix. It doesn't read `.vscode/mcp.json`. A custom agent file (`.github/agents/NAME.md`) can carry `mcp-servers`, which is the one way to bundle the server in the repository.
  - **MCP prompts show as slash commands in VS Code** (`/<server>.<prompt>`).
- Phase 4's targets and skill; phase 7's server and named prompts; phase 9's hook and its `copilot` target.
- `packages/cli`: `@ascribed/cli`, how Ascribe is installed with npm.

## Design

### No prompt files

The brainstorm suggested prompt files (`.github/prompts/*.prompt.md`). This phase writes none: the cloud agent doesn't load them, and in VS Code the three named prompts already appear as slash commands through the MCP server phase 8 registers. One fewer set of generated files to keep current. If the by-hand check shows the slash commands don't appear, stop and report.

### What `sync` writes for Copilot

`ascribe agents sync --target copilot` writes, beyond phase 4's instruction file:

- **The skill** at `.github/skills/ascribe/SKILL.md`: the same text as `.agents/skills/ascribe/`, written here only if the current documentation still says the cloud agent doesn't read `.agents/skills/`.
- With `--with-hook`, **the hooks** (phase 9): `.github/hooks/ascribe.json`, with the edit and stop events.
- With `--cloud`, **the environment:** `.github/workflows/copilot-setup-steps.yml` (or a step added to the existing one, between markers) that installs Node and `@ascribed/cli` at the project's pinned version, so `ascribe` is on the path.
- With `--cloud`, **the MCP server**, in one of two forms, since a file can't change a repository's settings:
  - printed: the JSON for the repository's settings, with `ascribe mcp` as a local server and its tools listed, and where to paste it;
  - or, with `--agent`, written: `.github/agents/ascribe-docs.md`, a custom agent for documentation work that carries the server in its `mcp-servers`, for repositories whose maintainers can't change the settings.

Everything written is covered by `--check`.

### The examples

As in phase 4, nothing is written into this repository's own `.github/`. The files written for `examples/quill` are added to phase 4's snapshot test, and the checks by hand use a scratch repository.

## Tasks

1. The Copilot target's skill, hooks, setup steps, MCP snippet, and custom agent, with tests: no workflow, an existing workflow with other steps, markers, and each option alone.
2. The examples' snapshots.
3. A check by hand, in a scratch repository holding a copy of `examples/quill`: in VS Code, see that Copilot lists the skill and the three slash commands, and that an edit which breaks a link is reported back to it; on GitHub, assign the cloud agent an issue ("add a page about X") and see whether it has `ascribe`, runs the check, and is held by the stop hook when errors remain. Say in the pull request what you ran and saw. If the cloud agent isn't available to you, say so and leave this to phase 11.
4. `docs/agents.md` ("GitHub Copilot"), `CHANGELOG.md`.

## Out of scope

Prompt files; a chat participant; a Copilot extension; anything for GitHub.com's chat; starting a cloud agent task from Ascribe.

## Acceptance criteria

- The setup steps are valid for the cloud agent as currently documented, and `ascribe --version` works in them.
- The cloud agent's session has the skill and the hooks without anyone editing a file by hand.
- `--check` fails when a generated file is stale.
- Nothing is written into this repository's `.github/`.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
pnpm format:check
```

## Commits

1. "Write Copilot's skill and hooks from agents sync"
2. "Write the setup for Copilot's cloud agent"
