# The agents pass: results

Part of [Agents](README.md), [phase 11](phase-11-docs.md). What the full pass found, what was fixed, and what's still to run.

The pass has two halves. The tool surface (every command, the MCP server, `ascribe agents sync`'s files, and the hooks, run on scratch copies of `examples/quill` and `examples/monorepo`) was run in a cloud session on 2026-10-10 (UTC), against `main` at 3dcc625. The runs with real agents, listed in order in [hand-checks.md](hand-checks.md), came after it, also on 2026-10-10 (UTC), on Kyle's machine with Claude Code, the only agent he has access to. The checks and runs that need Codex, Cursor, VS Code with Copilot, or Copilot's cloud agent aren't run, for that reason.

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

## Hand checks

From [hand-checks.md](hand-checks.md), sections 1 to 4. Kyle ran 1.1, 1.2, and 1.4 in Claude Code's terminal and its VS Code extension; the rest were run with `claude -p` and its stream of hook events, not in the terminal's interface.

| Check | Result |
|---|---|
| 1.1 The plugin installs and `/ascribe:check` appears | Yes: in the terminal, `/plugin marketplace add` and `/plugin install` worked and the commands appeared. Loaded with `--plugin-dir`, `/ascribe:check`, `/ascribe:new-page`, `/ascribe:review`, the skill, and the MCP server all appeared too. |
| 1.2 A broken link is reported once, after the edit | Yes with the project's hooks: one `ASC036` line after the edit, naming the build. **With the plugin, nothing reached the model after the edit** in a `claude -p` run; only the stop hook told it. See [The plugin's diagnostics after an edit](#the-plugins-diagnostics-after-an-edit). In the terminal, with the plugin, Claude did get the error right after its edit (Kyle). |
| 1.3 Sent back once, then allowed to stop | Yes, with the project's hooks and with the plugin. Claude Code shows the person "Stop hook error occurred" for the block. |
| 1.4 The same in the VS Code extension | Yes (Kyle). The plugin can't be installed from the extension: `/plugin` answers "/plugin isn't available in this environment", so it's installed from the terminal. The guide and the plugin's README say so. |
| 1.5 `ascribe` off the path | Yes: both hooks fail with `ascribe: command not found` (exit 127), shown to the person and not to the model. The model learned it from the skill and its own `which`, and said the check wasn't run. |
| 1.6 Context one error takes | Three lines, about 190 characters, after the edit; about 170 at the stop. |
| 2.7 Codex | not run: no access to Codex |
| 3.8 to 3.12 VS Code with Copilot | not run: no access to Copilot |
| 4.13 to 4.15 Copilot's cloud agent | not run: no access to Copilot |

### The plugin's diagnostics after an edit

In the headless run, the plugin's language server told the model nothing after its edit. The server isn't the cause:

- Started cold the way Claude Code starts it (`initialize` with `rootUri`, `rootPath`, and one workspace folder, all the working directory, then `didOpen` with the edited text), `ascribe lsp` publishes the broken link's diagnostic 23 ms after it starts, on quill.
- Claude Code starts a plugin's server only at the first edit of a file it handles, and hands what the server publishes to the model with its *next* request, not as the edit's result (read in the client code of Claude Code 2.1.42, the version in the cloud session that read it; its documentation doesn't say, and 2.1.289, the version on Kyle's machine that made the runs, may differ). An edit that's the last thing an agent does before it stops has no next request but the stop hook's, which is what was seen.

In the terminal, Kyle saw Claude get the error right after its edit. So it isn't a gap in the plugin, but the language server can't be counted on to report the last edit: the stop hook is what catches it, which is why the plugin has one.

## Runs with agents

"Passes" means `ascribe check --deny-warnings` exits `0` for every build. Each cell: passes (yes/no), times the person stepped in, whether the agent ran the check itself, and what it got wrong that a tool could have told it.

The Claude Code columns were run on Kyle's machine on 2026-10-10 (UTC) with Claude Code 2.1.289, without a person at the keyboard: `claude -p` with the task's sentence as the whole prompt, edits accepted, and Bash allowed, each in a fresh copy, with `ascribe` built on the path from the phase 11 branch ([#202](https://github.com/ascribed-dev/ascribe/pull/202)), and Kyle's own Claude Code settings and plugins rather than a clean profile. Task 6 needs VS Code's review and isn't run that way. In the runs with everything set up, the edit hook never had an error to report and the stop hook never held the agent, and no run called an MCP tool: each used the command line. The seeded pages of task 7 were alike but for a number, so whether an agent works rule by rule on pages that differ is still open.

| Task | Claude Code, nothing set up | Claude Code, everything | Copilot in VS Code, nothing set up | Copilot in VS Code, everything |
|---|---|---|---|---|
| 1. New guide page, linked from the index | passes; 0; ran the check, after finding `ascribe` on the path and reading `--help`; also listed the page in the README's file tree | passes; 0; ran the check; nothing | not run: no access to Copilot | not run: no access to Copilot |
| 2. Five seeded problems | passes; 0; ran the check; nothing (removed the sentence with the broken link) | passes; 0; ran the check; nothing (removed the same sentence) | not run | not run |
| 2b. Literal product name (read by hand) | left: "Quill" and "Quill Cloud" stayed in the prose, and it wrote "Quill" in the new title | replaced in the prose with `{product}` and `{cloud}`; left in `description`, as the other pages have it | not run | not run |
| 3. A third platform's variant | passes; 0; ran the check and a build; nothing | passes; 0; ran the check; nothing | not run | not run |
| 4. Rename a heading, keep links | passes; 0; ran the check and `link`; nothing (kept the `@id`) | passes; 0; ran the check and `link`; nothing (kept the `@id`) | not run | not run |
| 5. Security handbook page (monorepo) | passes in all three projects; 0; ran the check in the security project and the handbook; nothing | passes in all three projects; 0; ran the check with `--config handbook/pages/security`; nothing | not run | not run |
| 6. Three review comments, from **Prompt agent: all open** | not run: needs VS Code's review, which a headless run doesn't have | not run: as before | not run: no access to Copilot | not run: no access to Copilot |
| 7. A few hundred warnings, from `--summary` | passes (360 to 0); 0; ran the check; nothing. Read `explain` for each of the six codes, then fixed all six in every file with one script | passes (360 to 0); 0; ran the check; nothing. The same: `explain` for each code, one file fixed and rendered, then one script for the rest | not run | not run |

An agent with only what `ascribe agents sync` writes (Codex, or Cursor):

| Task | Result |
|---|---|
| 1 | not run: no access to Codex or Cursor |
| 2 | not run: no access to Codex or Cursor |
