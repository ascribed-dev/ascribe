---
type: reference
title: "`ascribe agents`, `mcp`, and `lsp`"
description: "Set a repository up for AI agents, serve a project's tools over MCP, and run the language server."
---

The commands for agents and editors. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## `ascribe agents`
@id {permalink=true}: cli-agents
@available: ascribe 0.3.0

Writes the files AI coding agents read on their own: your project's rules, from `ascribe.toml`, and the Ascribe skill; and prints the named prompts. See [Agents](index.md).

### `ascribe agents sync`
@id {permalink=true}: cli-agents-sync

Writes `AGENTS.md` beside `ascribe.toml`, a short block in the repository root's `AGENTS.md` when the project is in a subfolder, and the skill in the root's `.agents/skills/ascribe/`. It also keeps up to date each other target whose files exist already, and writes those `--target` names. It finds the repository's root as `ascribe diff` does, with `git`; outside a repository, the project's folder is the root, and it says so.

Run where there's no `ascribe.toml` at or above, such as at the root of a repository with several projects, it syncs each project in the folders below, in path order, and says which. Each writes its own block in the root's `AGENTS.md`, and the list names each file once.

With `--with-hook`, it also writes the [hooks](index.md#hooks) for `claude`, `codex`, and `copilot`, and for `claude` the MCP server, merging them into the agents' settings files at the root. Hooks it wrote before are kept up to date without it.

With `--cloud`, it also writes what [Copilot's cloud agent](index.md#github-copilot) needs: steps that install Ascribe in `.github/workflows/copilot-setup-steps.yml`, creating the workflow or adding them between markers to its `copilot-setup-steps` job. It then prints the MCP server's JSON for the repository's settings, after the list of files; with `--agent`, it writes the custom agent `.github/agents/ascribe-docs.md`, which carries the server, instead. When the repository's projects install Ascribe in different ways, the JSON is still one document, and it and the custom agent carry a server for each way, named as [the guide](index.md#github-copilot) says. `--cloud` implies `--target copilot`. The steps and the custom agent are kept up to date without it.

@include: ../_generated/cli-agents-sync-options.md

Each target writes its own files, beside `AGENTS.md` and the skill:

| Target | What it writes |
|---|---|
| `claude` | An import of `AGENTS.md` (`@AGENTS.md`) in `CLAUDE.md`, and the skill in `.claude/skills/ascribe/`, loaded for your pages |
| `claude-rules` | The rules in `.claude/rules/ascribe.md`, loaded only when Claude Code works on your pages |
| `copilot` | The rules in `.github/instructions/ascribe.instructions.md`, applied only to your pages; with `--cloud`, what [Copilot's cloud agent](index.md#github-copilot) needs |
| `codex` | Nothing of its own, since Codex reads `AGENTS.md` and the skill; it's there for [its hooks](index.md#hooks) |

For a project in a subfolder, the rules files are named after its folder: `ascribe-docs.md` for a project in `docs/`. When one project is nested in another's content root, the outer project's rules say that the nested project's pages aren't theirs.

The rules in `.claude/rules/` and `.github/instructions/`, and the skill's folders, are wholly Ascribe's: `sync` rewrites them whole, and the rules files say so at the top.

With `--with-hook`, each target's file gets these entries:

| Target | File | Entries |
|---|---|---|
| `claude` | `.claude/settings.json` and `.mcp.json` | The edit hook (after `Write`, `Edit`, and `MultiEdit`), the stop hook, and [the MCP server](index.md#the-mcp-server) |
| `codex` | `.codex/hooks.json` | The edit hook and the stop hook |
| `copilot` | `.github/hooks/ascribe.json` | The edit hook and the stop hook, for Copilot's CLI and its cloud agent |

It merges its entries into a file it shares: it updates the entries it wrote before in place, keeping what you set on them, such as a longer `timeout` or an `env` on the MCP server, and leaves every other entry and setting as it was. A file whose meaning wouldn't change is left alone, however it's formatted. `.github/hooks/ascribe.json` is wholly Ascribe's. `--check` covers them, and once a file has Ascribe's hooks, `sync` keeps them up to date without `--with-hook`; it adds the MCP server back to `.mcp.json` only with `--with-hook`, so one you removed stays removed.

`sync` writes only between its markers in a file it shares with your team, and whole files where the file is its own; it lists each file, `wrote` or `unchanged`, with the agents that read it, and when Claude Code reads none of the skill it wrote, says that `--target claude` writes the copy it does. With `--check`, it lists each file as `stale` or `up to date`, and writes nothing.

| Code | Meaning |
|---|---|
| `0` | The files are written, or with `--check`, up to date |
| `1` | With `--check`: a file is out of date. Run `ascribe agents sync`. |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, a file whose markers are damaged, a file that would be under the content root (so one of the project's pages), `--target copilot` or `--cloud` outside a git repository, a settings file that isn't a JSON object, a setup steps workflow it can't add its steps to, or a file it can't read or write. Nothing is written. |

### `ascribe agents rules`
@id {permalink=true}: cli-agents-rules

Prints your project's rules: the block `sync` writes into the `AGENTS.md` beside `ascribe.toml`, as it would write it now. It writes no file. The MCP server's `ascribe://instructions/<project>` resource is the same text.

@include: ../_generated/cli-agents-rules-options.md

| Code | Meaning |
|---|---|
| `0` | It printed the rules |
| `2` | It couldn't run: a usage error, no `ascribe.toml`, one with errors, or `git` can't say where the repository is |

### `ascribe agents index`
@id {permalink=true}: cli-agents-index

Writes a compact index of one build's docs, for the `AGENTS.md` of the product they're about: one line a page, with its title, its address, and its description, grouped in sections as the build's `llms.txt` is. It's for your product's agents, where `sync` is for the agents that write your docs; [Index your docs for your product's agents](index.md#index-your-docs-for-your-products-agents) says when it's worth it.

It lists the pages the build offers, the ones in `llms.txt`: a page that's `unlisted` isn't in it. A page's address is its Markdown version's, with `[consumer] agents = true`, or else its own. The index holds to 8 KB: past that, it leaves out descriptions, from the last page back, then pages, from the last back, and ends with a sentence saying how many it left out and where every page is.

@available: ascribe 0.3.2
Of a [versioned build](../versions/index.md#for-agents), `--build site@3.4` names an older line, and without `--build` it's the current line's. When the build has more than one live line, the index's heading names the line, as that line's `llms.txt` does: `## Loom (Self-managed 3.4)`.

It prints the index. With `--into`, it writes it between markers in that file instead, as `sync` writes its block, leaving the rest of the file as it is, so a second run replaces the first; it says whether the file changed. With `--pages`, it also writes each offered page's Markdown version, as the build's `plain` output writes it, into that folder, and the index's addresses are paths into it, from the folder of the `--into` file: the folder a package ships beside its `AGENTS.md`. A file in the folder that isn't one of the index's pages, such as a page from an earlier run that the build no longer offers, is named, and left there.

@include: ../_generated/cli-agents-index-options.md

```sh
ascribe agents index --build site --into packages/sdk/AGENTS.md --pages packages/sdk/docs
```

| Code | Meaning |
|---|---|
| `0` | It printed the index, or wrote it |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, more than one build and no `--build`, an unknown build, a file whose markers are damaged, a file or folder under the content root (so one of the project's pages), or a file it can't read or write. Nothing is written. |

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `build`, `limit` and `bytes` (the index's size, in bytes, and the most it holds), `offered` (how many pages the build offers), `listed`, `left_out`, and `without_description` (pages, as paths from the content root), `file` and `changed` (the `--into` file, and whether it changed), `folder` and `pages` (the `--pages` folder, and each page copied into it, with its `source` and its `path` in the folder), and `text`, the index. Its schema is in the [JSON report contract](../contracts/json-reports/questions.md#ascribe-agents-index).

### `ascribe agents skill`
@id {permalink=true}: cli-agents-skill

Prints a file of the Ascribe skill, the same for every project: its `SKILL.md`, or its directive reference, `references/directives.md`. It needs no project.

@include: ../_generated/cli-agents-skill-options.md

| Code | Meaning |
|---|---|
| `0` | It printed the file |
| `2` | A usage error, such as a file that isn't the skill's |

### `ascribe agents prompt`
@id {permalink=true}: cli-agents-prompt

Prints a named prompt for an agent, in the same form as [Prompt agent](commands.md#what-a-prompt-says)'s. The MCP server offers the same prompts, which some hosts show as slash commands.

@include: ../_generated/cli-agents-prompt-options.md

| Prompt | Arguments | What it asks |
|---|---|---|
| `new-page` | `type`, `title`, and `path` | Write a new page of that type: where its file goes, the type's `guide` to read first, the type's template with the title set, or, for a type with no template, the frontmatter to start it with, and the type's fields |
| `fix` | `path` | Fix what `ascribe check` reports: the prompt `ascribe check --format prompt` writes when there are problems, else the loop of checking and fixing |
| `review` | `path` and `base` | Review what the branch does to its pages, as readers see them: the prompt [`ascribe diff --format prompt`](../review/diff.md#cli-diff) writes, or a line saying no page changed |

`path` is a file or folder in the project, by default the current directory; `fix` reports on it. For `new-page`, a `path` in the content root, from the current directory or from the content root, is where the page goes: its file, or a folder for it. `--list` lists the prompts and their arguments.

Without a `path` in the content root, `new-page` says the page goes where the request says, anywhere in the content root, and otherwise in the content root, named for the title slugged as a page's address is, `rotate-keys.md` for "Rotate keys"; when a file is there, it asks for another path. A page names its type in its frontmatter, so it can go in any folder of pages; a fragment's path is refused.

```shell
ascribe agents prompt new-page --arg type=how-to --arg title="Rotate keys"
```

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

Checks an agent's work as its hook runs it: after the agent writes a file, or before it finishes. It reads the hook's JSON input on standard input and answers in the agent's own format. See [Hooks](index.md#hooks) for the entries that run it.

@include: ../_generated/cli-agents-hook-options.md

With `--event edit`, it checks the Markdown files the tool wrote that are pages or fragments of a project, as `ascribe check <file> --editor-build` does, and tells the agent their errors: at most 10 lines in the concise form, then the build it checked. Warnings and advice aren't reported, and advice is never counted. With `--event stop`, it checks every build of each project whose pages or `ascribe.toml` the working tree changes, as `git status` lists them for the whole repository (outside a repository, the project at or above the folder the agent works in), and keeps the agent working while there are errors, with the counts, the first 10 errors, and the command to run, or while a project can't be checked at all, with what `ascribe check` would say. It says nothing when the agent is already continuing because of a stop hook, so it asks at most once for each stop.

| Harness | After an edit | Before the agent finishes |
|---|---|---|
| `claude-code` | `PostToolUse`: `hookSpecificOutput.additionalContext` | `Stop`: `decision: "block"` with a `reason` |
| `codex` | `PostToolUse`: `hookSpecificOutput.additionalContext` | `Stop`: `decision: "block"` with a `reason` |
| `copilot` | `postToolUse`: `additionalContext` | `agentStop`: `decision: "block"` with a `reason` |

It never writes a project's file, and never stops an edit: the file is written by then. A check that takes longer than 2 seconds after an edit, or 20 before the agent finishes, is given up, and the agent goes on.

The first hook in a project starts a check server in the background, the same `ascribe` binary, which keeps the project loaded so each later check takes milliseconds. It opens no port: the hook and the server pass requests and answers as files in a folder of your cache folder (`ASCRIBE_CACHE_DIR`, or `ascribe` in your user's cache folder), and it stops after 10 minutes with no request, or when the `ascribe` it was started from is replaced. On Windows it runs from a copy in that folder, so it never holds up installing or building a new `ascribe`. Set `ASCRIBE_HOOK_SERVER=off` to check in the hook's own process every time.

| Code | Meaning |
|---|---|
| `0` | It ran, whatever it found, and also when a check ran out of time |
| `1` | Its input isn't JSON. Agents show this to you, not to the model. |
| `2` | A usage error: a harness or event it doesn't know |

## `ascribe mcp`
@id {permalink=true}: cli-mcp
@available: ascribe 0.3.0

Runs an MCP server, speaking the [Model Context Protocol](https://modelcontextprotocol.io) over standard input and output, for an agent that has no shell, or a host that approves tools one by one. An agent's host starts it; you don't run it yourself. It offers the commands that answer questions as tools, the project's rules as resources, and the named prompts of [`ascribe agents prompt`](commands.md#cli-agents-prompt). Each tool's result is the JSON its command writes, so an agent with a shell does as well with the commands. See [The MCP server](index.md#the-mcp-server) to set it up.

| Tool | The command it runs |
|---|---|
| `ascribe_check` | `ascribe check`, on `paths`, or on unsaved `text` as the file `path` names, narrowed to the diagnostics in `code` (`--code`) when it's given. `response_format` is `concise` (the default, `--format concise`) or `detailed` (`--format json`). |
| `ascribe_explain` | `ascribe explain` |
| `ascribe_model` | `ascribe model` |
| `ascribe_outline` | `ascribe outline` |
| `ascribe_link` | `ascribe link` |
| `ascribe_refs` | `ascribe refs`, with `response_format` as for `ascribe_check` |
| `ascribe_render` | `ascribe render` |
| `ascribe_format` | `ascribe fmt --check --format json`: the edits, and nothing written |
| `ascribe_changes` | `ascribe diff --format json --pages-only`, or with `blocks` each changed block too |

Every tool is read-only: none writes a file or starts a program the project names, and each is marked read-only, idempotent, and closed-world. `ascribe_check`, and the `fix` prompt, never run Vale, even with `in-check = true` in `[checks.vale]`: they report one `prose-not-checked` advice instead, and `ascribe check --vale` in a shell checks the prose. Paths are from the folder the server runs in, and each call finds the nearest `ascribe.toml` at or above its paths, so one server serves every project in a repository. A project is loaded once and kept: before each call the server lists the project's files again, names, sizes, and modification times, and loads it again when anything changed, a new or deleted file included. A call with bad arguments, or a path in no project, is a result marked as an error, with a sentence on what to call instead.

The resources are `ascribe://directives`, each built-in directive's syntax (`ascribe agents skill references/directives.md`); `ascribe://model/<project>`, what a project's content model allows (`ascribe model <project>`); and `ascribe://instructions/<project>`, its rules (`ascribe agents rules <project>`). `<project>` is a file or folder in it, from the folder the server runs in. The prompts are `new-page`, `fix`, and `review`, as `ascribe agents prompt` prints them.

The server speaks the protocol's 2026-07-28 revision, which has no handshake, and answers `initialize` for a client on 2025-11-25 or 2025-06-18. It takes no options, and refuses `--config` (exit code `2`). Its logs go to standard error; it exits with `0` when standard input ends.

## `ascribe lsp`
@id {permalink=true}: cli-lsp

Runs the language server, speaking the Language Server Protocol over standard input and output. An editor starts it; you don't run it yourself. It takes no options of its own, and refuses `--config` (exit code `2`): its project is the nearest `ascribe.toml` at or above the workspace folder the editor gives it, never one below, and `ascribe.toml`'s `[editor] build` says which build's page-level diagnostics to report. Its logs go to standard error, starting with the project it uses.

The VS Code extension runs it for you, one server for each project in the workspace. See [Editing](../editor/index.md). Any editor with an LSP client can run `ascribe lsp` too; for several projects, start one per project ([Other editors](../editor/index.md#other-editors)).
