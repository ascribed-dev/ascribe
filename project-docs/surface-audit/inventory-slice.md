# The inventory, one slice

Part of [The surface audit](README.md), [phase 1](phase-1-thread.md). Two tables were built by hand once from the generated option lists (`docs/content/_generated/cli-*-options.md`), the command reference (`docs/content/reference/cli.md`), the diagnostics registry, the plugin, and the MCP server's tool list, all at `main` on 10 October 2026. Phase 2 replaced the first with the generated [map of the surface](../surface.md); the second is still here, since it's quotations from one project and not something a script can read.

They lay things side by side so the seven questions can be asked of them. They don't answer the questions: what was noticed while building them is listed at the end, as candidates for findings, with the walks still to come.

## 1. Every command

The commands, their arguments, their options, and the words `--format` takes are in the generated map now: [The command line](../surface.md#the-command-line). It has the four `ascribe permalinks` commands, which merged after this table was made.

What the map can't read is what each command's exit codes mean, since only the reference's prose says. That column is kept here, as it was on 10 October 2026, because [findings 11, 12, and 20](findings.md) quote it:

| Command | Exit codes |
|---|---|
| `ascribe agents sync` | 0 the files are written, or with `--check` up to date; 1 with `--check`, a file is out of date; 2 couldn't run (nothing is written) |
| `ascribe agents rules` | not stated separately from `sync`'s |
| `ascribe agents skill` | not stated separately |
| `ascribe agents prompt` | not stated separately |
| `ascribe agents hook` | not stated separately |
| `ascribe build` | 0 built; 1 the checks found errors, nothing written; 2 couldn't run |
| `ascribe check` | 0 no errors; 1 errors, or warnings with `--deny-warnings`; 2 couldn't check |
| `ascribe diff` | 0 compared; 1 with `--exit-code`, something changed; 2 couldn't run |
| `ascribe drift` | 0 compared; 1 with `--exit-code`, an example broke or changed alone; 2 couldn't run |
| `ascribe explain`† | 0 explained or listed; 2 no such diagnostic, or a usage error |
| `ascribe fmt` | 0 formatted, or with `--check` nothing to format; 1 with `--check`, a file would change; 2 a problem |
| `ascribe link`† | 0 the target exists; 1 it doesn't; 2 couldn't run |
| `ascribe lsp` | 2 on `--config` |
| `ascribe mcp` | 0 when standard input ends; 2 on `--config` |
| `ascribe model` | 0 showed the model; 2 no `ascribe.toml`, or one with errors |
| `ascribe outline` | 0 showed the outline; 1 with `--build`, the build doesn't publish the page; 2 couldn't run |
| `ascribe refs` | 0 the target exists, used or not; 1 it doesn't; 2 couldn't run |
| `ascribe render` | 0 wrote the page; 1 the build doesn't publish it; 2 couldn't run |
| `ascribe report` | 0 ran, whatever it found; 1 with `--exit-code`, a finding at that level or above; 2 with `--exit-code`, a section asked for couldn't run, or a usage error |
| `ascribe sources fetch` | 0 the copies match the lock; 1 a file a snippet names couldn't be copied; 2 couldn't run |
| `ascribe sources update` | 0 ran, whether or not a pin moved; 2 couldn't |
| `ascribe sources status` | not stated separately |
| `ascribe vale eject` | 0 the preset is the project's own; 2 wrote nothing |

`ascribe permalinks verify` exits 0 when the list has every name, 1 when it doesn't, and 2 when it couldn't check; `sync` and `retire` exit 0 or 2. An option marked † in a quotation from the old table was in no release yet.

## 2. Everything that reports or fixes a problem

Each place a person or an agent meets a problem, what a problem is called there, what it shows for this phase's broken link (a link to `keys.md` after the file was renamed `api-keys.md`, in a copy of `examples/quill`), and what it offers next. The outputs quoted were taken from the release build of `main` on 10 October 2026 (`ascribe 0.2.0`), on that copy.

| Surface | Where | Called | What it shows for the broken link | What it offers next |
|---|---|---|---|---|
| The registry | `tests/conformance/diagnostics.toml`; the Diagnostics reference | a *diagnostic*, with a *severity* (`error`, `warning`, `advice`) and a *next step* (`fix`, `choose`, `write`, `outside`, `review`) | `ASC036` `link-target-missing`, error, next step `choose`, message `` `{path}` doesn't exist `` | `fix` text: "Fix the path, or create the file. Names must match exactly, including case, and the file must be inside the project." |
| `ascribe check` | terminal, text | `Error:`, `Warning:`, `Advice:` on each; the summary counts *errors*, *warnings*, *advice* | `[ASC036] Error: `keys.md` doesn't exist`, a source snippet with `link-target-missing` under the link, then `checked 4 files: 1 error, 0 warnings, 1 advice`. No `Help:` line (an `ASC041` in the reference's sample has one, from its fix's title) | the code to look up; exit 1 |
| `ascribe check --format concise` | terminal, for an agent | "one line per *problem*" (the docs); the summary counts errors, warnings, advice | `docs/install-agent.md:93: [ASC036] `keys.md` doesn't exist`, then the summary line | at most 50, then the command that narrows the check |
| `ascribe check --format json` | tools | `diagnostics[]`, each with `code`, `slug`, `severity`, `next`, `message`, `fixes`, `help`, `docs` | the entry above with `"next": "choose"`, `"fixes": []`, `help` = the registry's fix text, `docs` = the diagnostic's page | `fixes`, each `safe` or `unsafe`; here none |
| `ascribe check --format prompt` | an agent, via **Prompt agent** or by hand | "Fix the N *problems*" | `Fix the 2 problems `ascribe check` reports in this project.` (the error and the orphan advice), problems by file, `To list each problem: `ascribe check --format concise`` | `When you're done, run `ascribe check` and fix what it reports.` |
| `ascribe check --summary`† | terminal | counts by code and by file | `ASC036` once, `ASC154` once | |
| `ascribe build` | terminal, CI | "the checks found errors; nothing was written" | the same diagnostics as `check`, text or JSON | exit 1 |
| `ascribe explain <code>`† | terminal | *diagnostic*; `Message:`, `Fix:`, `Wrong`/`Right`, `Docs:` | `ASC036 link-target-missing: error, found in each file`, the three message variants, the fix text, a wrong page and a right page, `Docs: https://ascribed-dev.com/go/ASC036/` | the docs page |
| `ascribe link <target> --from <page>`† | terminal | "doesn't work"; `Closest:` | `keys.md#rotate-keys doesn't work from install-agent.md: `keys.md` doesn't exist.` and no `Closest:` list (the reference says it lists "pages with a similar path for a page that isn't"; `api-keys.md` wasn't offered) | exit 1 |
| `ascribe refs <target>` | terminal | "uses"; "the links to fix after renaming a heading" | `ascribe refs api-keys.md`: `0 places use api-keys.md.`, exit 0; `ascribe refs keys.md`: `keys.md doesn't exist.`, exit 1 | |
| `ascribe report problems` | terminal, CI, a scheduled job | *findings*, "what `ascribe check` finds, counted by check, by file, and by kind of next step" | the counts, and `--format prompt` for the section | `--exit-code[=LEVEL]` |
| CI | a workflow | the exit codes | `ascribe check --deny-warnings` exits 1; `ascribe report --exit-code`; `ascribe agents sync --check` | |
| The Problems panel | VS Code | *problems*; source `ascribe`; the editor's build only | `` `keys.md` doesn't exist `` with `ASC036`, on the link, as you type, open or not; each diagnostic's `data` carries `next`, `help`, and the fixes' `applicability` | the lightbulb; the code's docs link |
| The lightbulb (`Cmd+.`) | VS Code | *quick fixes*, from each fix's title, then **Prompt agent: fix this problem**, then the actions that rewrite what's at the cursor | no quick fix (`fixes` is empty); **Prompt agent: fix this problem**; **Change where the link goes** is an action on a link | a fix applied; a prompt copied or opened as `ascribe.agents.promptTarget` says |
| The actions bar (`Cmd+K A`)† | VS Code | "the *fixes for problems* at the cursor first, then the actions" | as the lightbulb, in the **Link** group: **Change where the link goes**, **Show the page's title as the link text** | |
| **Ascribe: Prompt Agent to Fix This File** / **…This Project**† | VS Code, the palette | *problems*; "shown when the file has problems" | `check --format prompt`'s text for the file or the project | copied or opened, never sent |
| The preview's header, in review | VS Code | "how many *errors*"; **Show problems** | the count, the Problems panel's | opens the Problems panel |
| `#ascribe_editor_problems`† | VS Code's agents | *problems* | `check`'s JSON for the editor's build, with `document_version`, `current`, `unsaved` | |
| `ascribe_check` | MCP | **Check pages**; "every *problem* `ascribe check` reports, with how to fix it" | the concise text by default; `response_format: detailed` gives the JSON | |
| `ascribe_explain` | MCP | **Explain a diagnostic** | `explain`'s JSON | |
| `ascribe_link` | MCP | **Check a link**; `exists`, `problem`, `closest` | `exists: false`, the problem, `closest` empty | |
| The MCP prompt `fix` | MCP; `/ascribe:check` in the plugin | "Check the project, fix what's reported, and check again" | `ascribe agents prompt fix`'s text | |
| The hook, after an edit | Claude Code, Codex, Copilot | *errors*, at most 10, errors only | `` `ascribe check docs/install-agent.md` reports 1 error: `` the concise line, then `Checked build `site` only. Run `ascribe explain <code>` for any you don't recognize.` | `ascribe explain` |
| The hook, before the agent stops | the same | *errors* and *warnings* | `` `ascribe check` reports 1 error and 0 warnings: `` the lines, `Fix these, then run `ascribe check`.`; holds the agent once. Claude Code shows the person "Stop hook error occurred" | `ascribe check` |
| The plugin's language server | Claude Code (`.lsp.json`, `"diagnostics": true`) | *diagnostics*, pushed after an edit, the editor's build | the diagnostic as the Problems panel has it, with the agent's next request | |
| The skill, "The loop" | `SKILL.md`, from the plugin or `agents sync` | *problem*; "fix every *error*, and the *warnings* your edit caused" | `ascribe check <file> --format concise`, then `ascribe check` | `ascribe explain <code>` |
| The rules in `AGENTS.md` | written by `ascribe agents sync` | "Programmatic check: after editing a page, run `ascribe check <file> --format concise` and fix every *error*" | the same loop; `ascribe link <target> --from <page>` under **Links** | `ascribe explain <code>` |

## Noticed while building them

Candidates for findings, each with its evidence in the tables. Phase 1's task 4 asks the seven questions properly, after the walks.

- **`choose` with nothing to choose from.** `ASC036`'s next step is `choose`, which the Diagnostics reference defines as "You pick among things Ascribe can list: a link's target", but on this project `check`'s JSON gives `"fixes": []`, the editor has no quick fix, and `ascribe link` lists no `Closest:` although `api-keys.md` exists and the reference says it lists "pages with a similar path". Question 6.
- **What counts as a problem differs by format.** `check --format prompt` says "Fix the 2 problems", counting the orphan advice; the text summary says `1 error, 0 warnings, 1 advice`; the hook reports errors only after an edit and errors and warnings at the stop; `--deny-warnings` makes warnings count. Question 1.
- **`check`'s text output shows `Help:` only for a diagnostic with a fix.** `ASC036` has `help` in the JSON and `Fix:` in `explain`, and its text output has no `Help:` line; `ASC041`'s has one, from its fix's title. Question 6.
- **Five sets of `--format` words.** `text|json` (nine commands); `text|concise|json|prompt` (`check`); `text|json|html|prompt` (`diff`); `text|json|summary` (`drift`, `sources update`); `text|summary|json|prompt` (`report`). `summary` means Markdown for CI in three commands, and `--summary` on `check` means counts. Question 2.
- **Exit code 1 means seven things.** Errors (`check`, `build`); a file out of date (`agents sync --check`, `fmt --check`); a target that doesn't exist (`link`, `refs`); a build that doesn't publish the page (`outline`, `render`); something changed (`diff --exit-code`); a finding at a level (`report --exit-code`); a file that couldn't be copied (`sources fetch`). `report --exit-code` adds a 2 for a section that couldn't run, which is `check`'s 2 for a different reason. Question 2.
- **Three ways to name the project.** `--config <PATH>` on every command; `[PATH]` positional on `model` and `agents rules`; `--project <PATH>` on `refs`. Question 2.
- **Five ways to name a page.** `PAGE` (`outline`, `render`); `[PAGE]` (`diff`); `--from <PAGE>` (`link`); `[PATHS]...` (`check`, `fmt`); `--path <PATH>` (`check --stdin`). Question 2.
- **Five words for one thing.** The registry, `explain`, the JSON, and the plugin's server say *diagnostic*; the docs for `concise`, the Problems panel, the prompts, the MCP tool, and the skill say *problem*; `report` says *finding*; Vale's are *alerts*; the summary line and the hooks count *errors*, *warnings*, *advice*. Question 1.
- **Nothing in the editor checks every build.** The Problems panel shows the editor's build only, and the editor guide says "before calling a project clean, run `ascribe check`"; `editor-commands.md` lists no command that runs it, so a person with no terminal can't. The editor walk will show whether that matters for this task. Questions 3 and 4.
