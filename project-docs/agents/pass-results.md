# The agents pass: results

Part of [Agents](README.md), [phase 11](phase-11-docs.md). What the full pass found, what was fixed, and what's still to run.

The pass has two halves. The tool surface (every command, the MCP server, `ascribe agents sync`'s files, and the hooks, run on scratch copies of `examples/quill` and `examples/monorepo`) was run in a cloud session on 2026-10-10, against `main` at 3dcc625. The runs with real agents need Claude Code, VS Code with Copilot, Copilot's cloud agent, and Codex or Cursor, which a cloud session doesn't have; they're listed, in order, in [hand-checks.md](hand-checks.md), and their rows below are filled in as they're run.

## The tool surface

Each was run in a scratch git repository outside this one, with `target/debug/ascribe`.

| What | How it was tried | Result |
|---|---|---|
| `check` on a file, `--format concise`, `--format prompt`, `--summary` | A page seeded with task 2's five problems (missing `title`, broken link, `type=danger`, unclosed `@note`, `{producta}`) | All five reported, one line each, with the fix in the message. The prompt names the file, lists them, and ends with `AGENTS.md` and the check. |
| `check --summary` on a few hundred warnings | 60 pages, six kinds of warning each (360) | Counts by code were right. **By file listed all 60 files**, uncapped: fixed (below). |
| `explain`, `model`, `outline`, `link`, `refs`, `render` | The seeded page and `docs/keys.md` | Each answered as documented. `link keys.md#rotate` exits `1` and offers `keys.md#rotate-keys`; `refs` on that heading finds the one link to it, which is what task 4 needs. `render` without `--build` on a project with three builds exits `2` and names them. |
| `agents sync`, every target, `--with-hook`, `--cloud`, `--agent`, `--check` | Quill at the repository's root | Every file written, a second run changed nothing, and `--check` passed. The cloud agent's MCP JSON was printed. |
| `agents sync` in the monorepo | Each of the three projects | **The handbook's rules applied to the security handbook's pages too**, which are a nested project with other rules: fixed (below). |
| `agents hook claude-code` and `copilot`, `--event edit` and `stop` | Hook input on standard input, with the seeded page | The edit hook reports the four errors, not the warning, and names the build. The stop hook blocks once, and lets the agent stop when `stop_hook_active` is set. |
| `ascribe mcp` | `initialize`, `tools/list`, `tools/call ascribe_check`, `prompts/list`, `prompts/get new-page`, `resources/list` over standard input | Nine tools, three prompts, three resources. `ascribe_check` returns the concise text by default, not JSON as the guide said: the guide is fixed. |
| The skill, read as an agent with only `npx` | Quill with `@ascribed/cli` as a dev dependency and nothing on the path | The skill and `AGENTS.md` say `ascribe`, which isn't on the path there, and said nothing about `npx`: fixed (below). |

## Fixed in this pass

- **`check --summary` lists at most 20 files** in text, then the command whose JSON lists them all (`ascribe check --summary --format json`), as plan decision 3 asks of every list. The JSON is unchanged. Test: `summary_lists_at_most_twenty_files` in `crates/ascribe-cli/tests/all/check.rs`.
- **A project's rules name the projects nested in its content root.** Claude Code's `paths` and Copilot's `applyTo` can't exclude a folder, so a page of the security handbook got the handbook's rules and its own. The handbook's rules, and its block in the root `AGENTS.md`, now say "The pages under `handbook/pages/security/` aren't: that folder is another project's, with its own `ascribe.toml` and rules." Test: the monorepo snapshots in `crates/ascribe-cli/tests/output/agents/monorepo/`.
- **The skill says to run `npx ascribe`** when `ascribe` isn't on the path.
- **The guide**: the rules files' names (`ascribe.md`, or `ascribe-<folder>.md` for a project in a subfolder), what the MCP tools return, and the hook's agent names were wrong or vague; the guide now has the loop, VS Code, other agents, agents that reach the network, and what's sent where.

## Needs design

- **Working rule by rule** ([#203](https://github.com/ascribed-dev/ascribe/issues/203)). Task 7 asks whether an agent works through hundreds of warnings rule by rule. `--summary` shows the rules, but nothing narrows `check` to one code, so the concise list cuts at 50 and its "next" command narrows by file, not by rule. A `--code` filter on `check` (and the MCP tool) would let an agent fix one rule across the project.
- **The stop hook passes with warnings** ([#204](https://github.com/ascribed-dev/ascribe/issues/204)). It holds an agent only on errors (decision 12 and phase 9), while this pass counts a run as passing at `--deny-warnings`. A project that wants warnings held needs a setting, or the hook needs to read `--deny-warnings` from somewhere. Left as it is until the runs below show whether agents leave warnings behind.

## Runs with agents

"Passes" means `ascribe check --deny-warnings` exits `0` for every build. Each cell: passes (yes/no), times the person stepped in, whether the agent ran the check itself, and what it got wrong that a tool could have told it.

| Task | Claude Code, nothing set up | Claude Code, everything | Copilot in VS Code, nothing set up | Copilot in VS Code, everything |
|---|---|---|---|---|
| 1. New guide page, linked from the index | not run: needs Claude Code | not run: needs Claude Code | not run: needs VS Code with Copilot | not run: needs VS Code with Copilot |
| 2. Five seeded problems | not run | not run | not run | not run |
| 2b. Literal product name (read by hand) | not run | not run | not run | not run |
| 3. A third platform's variant | not run | not run | not run | not run |
| 4. Rename a heading, keep links | not run | not run | not run | not run |
| 5. Security handbook page (monorepo) | not run | not run | not run | not run |
| 6. Three review comments, from **Prompt agent: all open** | not run | not run | not run | not run |
| 7. A few hundred warnings, from `--summary` | not run | not run | not run | not run |

An agent with only what `ascribe agents sync` writes (Codex, or Cursor):

| Task | Result |
|---|---|
| 1 | not run: needs Codex or Cursor |
| 2 | not run: needs Codex or Cursor |
