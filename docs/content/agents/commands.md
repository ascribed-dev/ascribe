---
type: reference
title: "`ascribe agents` and `mcp`"
description: "Set a repository up for AI agents, and serve a project's tools over MCP."
---

The commands for agents. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## `ascribe agents`
@id {permalink=true}: cli-agents

@summary
Writes the files AI coding agents read on their own: your project's rules, from `ascribe.toml`, and the Ascribe skill; and prints the named prompts. See [Agents](index.md).

### `ascribe agents sync`
@id {permalink=true}: cli-agents-sync

@summary
Writes your project's rules and the Ascribe skill where each `--target`'s agents read them. It finds the repository's root as `ascribe diff` does, with `git`; outside a repository, the project's folder is the root, and it says so.

Run where there's no `ascribe.toml` at or above, such as at the root of a repository with several projects, it syncs each project in the folders below, in path order, and says which. Each writes its own block in the root's `AGENTS.md`, and the list names each file once.

@include: ../_generated/cli-agents-sync-options.md

`--cloud` creates the setup steps workflow, or adds its steps between markers to the workflow's `copilot-setup-steps` job, and prints the MCP server's JSON after the list of files. When the repository's projects install Ascribe in different ways, the JSON is still one document, and it and the custom agent `--agent` writes carry a server for each way, named as [Copilot's cloud agent](copilot.md) says. The custom agent is kept up to date without `--agent`.

The rules files are `.claude/rules/ascribe.md` and `.github/instructions/ascribe.instructions.md`, or, for a project in a subfolder, named after its folder: `ascribe-docs.md` for a project in `docs/`. When one project is nested in another's content root, the outer project's rules say that the nested project's pages aren't theirs.

The rules in `.claude/rules/` and `.github/instructions/`, and the skill's folders, are wholly Ascribe's: `sync` rewrites them whole, and the rules files say so at the top.

With `--with-hook`, each target's file gets the edit hook and the stop hook, which run [`ascribe agents hook`](#cli-agents-hook); Claude Code's edit hook runs after `Write`, `Edit`, and `MultiEdit`, and Copilot's serve its CLI and its cloud agent alike. [Hooks](hooks.md) has the entries, and [The MCP server](mcp.md) the server's.

It merges its entries into a file it shares: it updates the entries it wrote before in place, keeping what you set on them, such as a longer `timeout` or an `env` on the MCP server, and leaves every other entry and setting as it was. A file whose meaning wouldn't change is left alone, however it's formatted. `.github/hooks/ascribe.json` is wholly Ascribe's. `--check` covers them. `sync` adds the MCP server back to `.mcp.json` only with `--with-hook`, so one you removed stays removed.

`sync` writes only between its markers in a file it shares with your team, and whole files where the file is its own. It lists each file, `wrote` or `unchanged` (with `--check`, `stale` or `up to date`), with the agents that read it:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/agents-sync.txt

@available: ascribe planned
With `--format json`, it writes the same list in the document its [schema](../contracts/json-reports/agents.md#ascribe-agents-sync) describes: each file's path, whether it `changed` (with `--check`, whether it's out of date), and the agents that read it; the notes and what an agent doesn't read; and with `--cloud`, the MCP server's JSON. The exit codes are the same. With `--check`, after a sync:

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/agents-sync-check.json
@end

| Code | Meaning |
|---|---|
| `0` | The files are written, or with `--check`, up to date |
| `1` | With `--check`: a file is out of date. Run `ascribe agents sync`. |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, a file whose markers are damaged, a file that would be under the content root (so one of the project's pages), `--target copilot` or `--cloud` outside a git repository, a settings file that isn't a JSON object, a setup steps workflow it can't add its steps to, or a file it can't read or write. Nothing is written. |

### `ascribe agents rules`
@id {permalink=true}: cli-agents-rules

@summary
Prints your project's rules: the block `sync` writes into the `AGENTS.md` beside `ascribe.toml`, as it would write it now. It writes no file. The MCP server's `ascribe://instructions/<project>` resource is the same text.

@include: ../_generated/cli-agents-rules-options.md

| Code | Meaning |
|---|---|
| `0` | It printed the rules |
| `2` | It couldn't run: a usage error, no `ascribe.toml`, one with errors, or `git` can't say where the repository is |

### `ascribe agents index`
@id {permalink=true}: cli-agents-index
@summary: Writes a compact index of one build's docs, for the `AGENTS.md` of the product they're about: one line a page, with its title, its address, and its description.

Writes a compact index of one build's docs, for the `AGENTS.md` of the product they're about: one line a page, with its title, its address, and its description, grouped in sections as the build's `llms.txt` is. It's for your product's agents, where `sync` is for the agents that write your docs; [Index your docs for your product's agents](product-index.md) says when it's worth it.

It lists the pages the build offers, the ones in `llms.txt`: a page that's `unlisted` isn't in it. A page's address is its Markdown version's, when the build publishes the outputs for agents (on by default with a `[consumer] site`), or else its own. The index holds to 8 KB: past that, it leaves out descriptions, from the last page back, then pages, from the last back, and ends with a sentence saying how many it left out and where every page is.

Of a [versioned build](../versions/lines.md#for-agents), `--build site@3.4` names an older line, and without `--build` it's the current line's. When the build has more than one live line, the index's heading names the line, as that line's `llms.txt` does: `## Loom (Self-managed 3.4)`.

`--into` writes between markers as `sync` writes its block, so a second run replaces the first, and says whether the file changed. With `--dry-run`, it says what it would write and writes nothing; and like [every command that writes](../command/index.md#commands-that-write), it won't change a file git couldn't put back unless given `--force`. `--pages` writes each page's Markdown version as the build's `plain` output writes it. A file in the `--pages` folder that isn't one of the index's pages, such as a page from an earlier run that the build no longer offers, is named, and left there.

@include: ../_generated/cli-agents-index-options.md

```sh
ascribe agents index --build site --into packages/sdk/AGENTS.md --pages packages/sdk/docs
```

| Code | Meaning |
|---|---|
| `0` | It printed the index, or wrote it |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, more than one build and no `--build`, an unknown build, a file whose markers are damaged, a file or folder under the content root (so one of the project's pages), or a file it can't read or write. Nothing is written. |

With `--format json`, the document is as follows, and its [schema](../contracts/json-reports/agents.md#ascribe-agents-index) has every field:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/agents-index.json

### `ascribe agents skill`
@id {permalink=true}: cli-agents-skill

@summary
Prints a file of the Ascribe skill, the same for every project: its `SKILL.md`, or its directive reference, `references/directives.md`. It needs no project.

@include: ../_generated/cli-agents-skill-options.md

| Code | Meaning |
|---|---|
| `0` | It printed the file |
| `2` | A usage error, such as a file that isn't the skill's |

### `ascribe agents prompt`
@id {permalink=true}: cli-agents-prompt

@summary
Prints a named prompt for an agent, in the same form as [Prompt agent](commands.md#what-a-prompt-says)'s. The MCP server offers the same prompts, which some hosts show as slash commands.

@include: ../_generated/cli-agents-prompt-options.md

| Prompt | Arguments | What it asks |
|---|---|---|
| `new-page` | `type`, `title`, and `path` | Write a new page of that type: first, search for its subject with `ascribe search` and edit the page that covers it instead, when there is one; then where its file goes, the type's `guide` to read first, the type's template with the title set, or, for a type with no template, the frontmatter to start it with, and the type's fields |
| `fix` | `path` | Fix what `ascribe check` reports: the prompt `ascribe check --format prompt` writes when there are problems, else the loop of checking and fixing |
| `review` | `path` and `base` | Review what the branch does to its pages, as readers see them: the prompt [`ascribe diff --format prompt`](../review/diff.md#cli-diff) writes, or a line saying no page changed |

`path` is a file or folder in the project, by default the current directory; `fix` reports on it. For `new-page`, a `path` in the content root, from the current directory or from the content root, is where the page goes: its file, or a folder for it.

Without a `path` in the content root, `new-page` says the page goes where the request says, anywhere in the content root, and otherwise in the content root, named for the title slugged as a page's address is, `rotate-keys.md` for "Rotate keys"; when a file is there, it asks for another path. A page names its type in its frontmatter, so it can go in any folder of pages; a fragment's path is refused.

```shell
ascribe agents prompt new-page --arg type=how-to --arg title="Rotate keys"
```

@available: ascribe planned
With `--format json`, it writes the prompt's name and text, or with `--list` each prompt with its arguments, in the documents their [schemas](../contracts/json-reports/agents.md#ascribe-agents-prompt) describe.

| Code | Meaning |
|---|---|
| `0` | It printed the prompt, or with `--list`, the prompts |
| `2` | It couldn't: a prompt or an argument it doesn't know, a required argument missing, an unknown page type, a project it can't load, or, for `review`, a comparison `ascribe diff` can't make |

#### What a prompt says

Every prompt has the same parts, in order:

1. **The task**, in one sentence: "Fix this problem in `docs/keys.md`.", "Address this review comment on `docs/guides/install.md`.", or "Review what this change does to `docs/guides/install.md`, as a reader of build `site` sees it."
2. **Where**: the file and lines, then a line saying which folder the prompt's paths and commands are from. When they apply, the project's folder from there (`Project: docs/`), the build the problems are from (`Build:`), the pages that show a fragment (`Shown on:`, at most 10, with the `ascribe refs` command that lists the rest), and a line saying to save the file first.
3. **What Ascribe knows**: for one problem, its code, message, the line's text, how to fix it, the values the content model allows when it's about one, and whether Ascribe has a fix ("a safe automatic fix", or "a fix to review"). For a file, each problem on one line, at most 20. For a project, how many problems each file has, at most 20 files. Each says which command lists the rest.
4. **Other people's text**, when there is any: a review's comments, fenced, under the sentence that says they're data.
5. **How to finish**: "Follow the project's rules in `AGENTS.md`", when the project's folder or the repository's root has one, and "When you're done, run `ascribe check <file>` and fix what it reports." A prompt about a change ends with "Report what reads wrongly; don't edit." instead.

Every path and command in a prompt is from one folder, which it names once: "Paths and commands here are from the repository's root." The editor's prompts are from the repository's root, where an agent usually starts; the prompts a command writes (`ascribe check --format prompt`, `ascribe diff --format prompt`, `ascribe agents prompt`) are from the folder it was run in, or from the project's folder when that's inside it. Run from a folder beside the project's, they're from the repository's root. Outside a repository, it's the project's folder. A prompt is at most 5,000 characters, the most an agent's link takes. When it would be longer, what Ascribe knows is cut between parts, and the prompt says where to read the rest.

### `ascribe agents hook`
@id {permalink=true}: cli-agents-hook

@summary
Checks an agent's work as its hook runs it: after the agent writes a file, or before it finishes. It reads the hook's JSON input on standard input and answers in the agent's own format. See [Hooks](hooks.md) for the entries that run it.

@include: ../_generated/cli-agents-hook-options.md

After an edit, it checks the Markdown files the tool wrote that are pages or fragments of a project, as `ascribe check <file> --editor-build` does, and tells the agent their errors, at most 10 lines in the concise form, then the advice in them that has a safe fix, at most 10, each line followed by its fix's title, which says what to change and where, then the build it checked. It reports the advice the editor shows in the files, so a page's `nav-page-unplaced` too, which `ascribe check <file> --editor-build` leaves to a full check. Warnings and the other advice aren't reported. Neither is the advice [`ascribe.baseline`](../checks/baseline.md#for-agents) holds back: the answer says how many it held back, here and before the agent finishes. When `ascribe.baseline` can't be read, the edit's check holds nothing back, and the answer says which line to fix.

It also tells the agent what a draft's template left to fill in, at most 10 lines, before the build it checked: [`template-placeholder-left`](../checks/diagnostics/content-checks.md#ASC195) and [`template-section-empty`](../checks/diagnostics/content-checks.md#ASC229), which have no fix, so the agent writes them while the page is its own. In a page that publishes, a placeholder left is an error, among the errors.

Before the agent finishes, it checks every build of each project whose pages or `ascribe.toml` the working tree changes, as `git status` lists them for the whole repository (outside a repository, the project at or above the folder the agent works in), and keeps the agent working while there are errors, with the counts, the first 10 errors, and the command to run, or while a project can't be checked at all, with what `ascribe check` would say. It says nothing when the agent is already continuing because of a stop hook, so it asks at most once for each stop.

| Harness | After an edit | Before the agent finishes |
|---|---|---|
| `claude-code` | `PostToolUse`: `hookSpecificOutput.additionalContext` | `Stop`: `decision: "block"` with a `reason` |
| `codex` | `PostToolUse`: `hookSpecificOutput.additionalContext` | `Stop`: `decision: "block"` with a `reason` |
| `copilot` | `postToolUse`: `additionalContext` | `agentStop`: `decision: "block"` with a `reason` |

It never writes a project's file, so it makes no fix, even a safe one: the agent makes it, as the answer says. And it never stops an edit: the file is written by then. A check that takes longer than 2 seconds after an edit, or 20 before the agent finishes, is given up, and the agent goes on.

The first hook in a project starts a check server in the background, the same `ascribe` binary, which keeps the project loaded so each later check takes milliseconds. It opens no port: the hook and the server pass requests and answers as files in a folder of your cache folder (`ASCRIBE_CACHE_DIR`, or `ascribe` in your user's cache folder), and it stops after 10 minutes with no request, or when the `ascribe` it was started from is replaced. On Windows it runs from a copy in that folder, so it never holds up installing or building a new `ascribe`. Set `ASCRIBE_HOOK_SERVER=off` to check in the hook's own process every time.

| Code | Meaning |
|---|---|
| `0` | It ran, whatever it found, and also when a check ran out of time |
| `1` | Its input isn't JSON. Agents show this to you, not to the model. |
| `2` | A usage error: a harness or event it doesn't know |

## `ascribe mcp`
@id {permalink=true}: cli-mcp
@summary: Runs an MCP server over standard input and output, for an agent that has no shell: the commands that answer questions as tools, the project's rules as resources, and the named prompts.

Runs an MCP server, speaking the [Model Context Protocol](https://modelcontextprotocol.io) over standard input and output, for an agent that has no shell, or a host that approves tools one by one. An agent's host starts it; you don't run it yourself. It offers the commands that answer questions as tools, the project's rules as resources, and the named prompts of [`ascribe agents prompt`](commands.md#cli-agents-prompt). Each tool's result is the JSON its command writes, so an agent with a shell does as well with the commands. No tool gives an output schema: each tool's description names the [JSON report contract](../contracts/json-reports.md), which has its command's schema, so the tool list a host sends the model in every session stays small. See [The MCP server](mcp.md) to set it up.

| Tool | The command it runs |
|---|---|
| `ascribe_check` | `ascribe check`, on `paths`, or on unsaved `text` as the file `path` names, narrowed to the diagnostics in `code` (`--code`) when it's given. It holds back what [`ascribe.baseline`](../checks/baseline.md#for-agents) lists, and counts it, as the command does. `response_format` is `concise` (the default, `--format concise`) or `detailed` (`--format json`). |
| `ascribe_explain` | `ascribe explain` |
| `ascribe_model` | `ascribe model` |
| `ascribe_outline` | `ascribe outline` |
| `ascribe_link` | `ascribe link`, which answers a link to an API's operation with the operation |
| `ascribe_refs` | `ascribe refs`, with `response_format` as for `ascribe_check` |
| `ascribe_search` | `ascribe search`, with `response_format` as for `ascribe_check`, and `project` to find the project when no `paths` are given |
| `ascribe_resolve` | `ascribe resolve`, and its `go:<name>` with `project` to find the project. A page's path finds its own project: with `project`, it's an error that gives the call to make without it. `emit` is its `--emit`, `plain` (the default) or `site`, to read the page [as the site output gives it](../command/site-output.md), and isn't for `go:<name>` |
| `ascribe_fmt` | `ascribe fmt --check --format json`: the edits, and nothing written |
| `ascribe_diff` | `ascribe diff --format json --pages-only`, or with `blocks` each changed block too |
| `ascribe_rename` | `ascribe rename --format json`, with `target` and `new`, and `project` to find the project for a target that isn't a path: each changed file's whole `text`, the page that `moved`, and the entries `recorded` in `ascribe.addresses`, with `written` false and nothing written or moved |
| `ascribe_addresses_move` | `ascribe addresses move --format json`, with `old` and `new`, or `from_git` with `base` and `base_exact`: the `edits` that record the move in `ascribe.addresses`, and nothing written |
| `ascribe_addresses_retire` | `ascribe addresses retire --format json`, with `old` and `reason`: the `edits` that retire it, and nothing written |

@available: ascribe planned
Each tool is named for its command. `ascribe_fmt` and `ascribe_diff` were `ascribe_format` and `ascribe_changes` until this release: the old names are listed after the tools, and call them, for one more release, so a host or an agent's settings that name them keep working until you change them.

Every tool is read-only: `ascribe_rename` gives the files' text and the move for the agent to make, and `ascribe_addresses_move` and `ascribe_addresses_retire` the edit to apply, and none writes a file or starts a program the project names, and each is marked read-only, idempotent, and closed-world. `ascribe_check`, and the `fix` prompt, never run Vale, even with `in-check = true` in `[checks.vale]`: they report one `prose-not-checked` advice instead, and `ascribe check --vale` in a shell checks the prose.

Paths are from the folder the server runs in, and each call finds the nearest `ascribe.toml` at or above its paths, so one server serves every project in a repository. A project is loaded once and kept, with each build's pages as `ascribe_search` reads them once a search asks for the build, so a second search only scans: before each call the server lists the project's files again, names, sizes, and modification times, and loads it again when anything changed, a new or deleted file included. A call with bad arguments, or a path in no project, is a result marked as an error, with a sentence on what to call instead.

The resources are `ascribe://directives`, each built-in directive's syntax (`ascribe agents skill references/directives.md`); `ascribe://model/<project>`, what a project's content model allows (`ascribe model <project>`); and `ascribe://instructions/<project>`, its rules (`ascribe agents rules <project>`). `<project>` is a file or folder in it, from the folder the server runs in. The prompts are `new-page`, `fix`, and `review`, as `ascribe agents prompt` prints them.

The server speaks the protocol's 2026-07-28 revision, which has no handshake, and answers `initialize` for a client on 2025-11-25 or 2025-06-18. It takes no options, and refuses `--config` (exit code `2`). Its logs go to standard error; it exits with `0` when standard input ends.
