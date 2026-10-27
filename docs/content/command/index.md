---
type: reference
title: Command reference
description: "The `ascribe` commands: their options, outputs, and exit codes."
permalink: cli-reference
---

The `ascribe` command checks, builds, and formats an Ascribe project, and runs the language server the editor uses. Its `explain`, `model`, `outline`, `link`, `refs`, `resolve`, and `search` commands answer questions about a project, for writers and for agents, without changing anything, and `ascribe mcp` offers them to agents as the tools of an MCP server.

Install it in a project with `npm install --save-dev @ascribed/cli` (see [Install](../install.md#npm)) and run it with `npx ascribe`, or from a script in `package.json`. Where there's no Node, install it on your path with Homebrew or the install script ([Install](../install.md#without-node)) and run it as `ascribe`.

## Commands

Each command links to its section, with its options, output, and exit codes, on the page for its kind of work.

| Command | What it does |
|---|---|
| [`ascribe addresses`](../publishing/redirects.md#cli-addresses) | Keeps `ascribe.addresses`, the record of where the addresses you published went: makes it, records a move or a retired address, and lists the redirects each build writes. |
| [`ascribe agents`](../agents/commands.md#cli-agents) | Sets a repository up for agents: writes the project's rules and the skill where agents read them, indexes the docs for a product's agents, and runs an agent's hook. |
| [`ascribe baseline`](../checks/baseline.md#cli-baseline) | Puts a project's existing advice off for now, so `ascribe check` reports only what's new. |
| [`ascribe build`](../checks/check.md#cli-build) | Checks the project, then writes each build's outputs. |
| [`ascribe check`](../checks/check.md#cli-check) | Checks every source file for problems, without building anything. |
| [`ascribe diff`](../review/diff.md#cli-diff) | Shows what changed between a git revision and the working tree, as readers will see it. |
| [`ascribe drift`](../review/diff.md#cli-drift) | Lists the pages whose code examples changed, and whether the words around them changed too. |
| [`ascribe explain`](questions.md#cli-explain) | Says what a diagnostic means, and how to fix it. |
| [`ascribe fmt`](../checks/check.md#cli-fmt) | Rewrites Ascribe constructs into canonical form. |
| [`ascribe init`](../install.md#cli-init) | Starts a project: writes its first `ascribe.toml`, and makes the folder its pages go in. |
| [`ascribe link`](questions.md#cli-link) | Says whether a link works, written on a given page, and the destination to write there. |
| [`ascribe lsp`](../editor/language-server.md#cli-lsp) | Runs the language server, for an editor. |
| [`ascribe mcp`](../agents/commands.md#cli-mcp) | Runs an MCP server, for an agent that has no shell, or a host that approves tools one by one. |
| [`ascribe model`](questions.md#cli-model) | Shows the content model as `ascribe` reads it, with the defaults filled in. |
| [`ascribe new`](../templates/index.md#cli-new) | Starts a page from its type's template, with its type and title set. |
| [`ascribe observe`](../examples/sources.md#cli-observe) | Runs the test of a folder of tested examples, and records what it passed with. |
| [`ascribe outline`](questions.md#cli-outline) | Shows a page's title and type, and the headings a link to it can name. |
| [`ascribe permalinks`](../permalinks/command.md#cli-permalinks) | Keeps the list of permalinks, checks a product's names against it, and retires a permalink. |
| [`ascribe preview`](../checks/check.md#cli-preview) | Renders a build's pages as a site, serves it on this computer, and renders it again as you edit. |
| [`ascribe refs`](questions.md#cli-refs) | Lists the places that use a page, a fragment, a heading, an image, or an entry of the content model. |
| [`ascribe rename`](questions.md#cli-rename) | Renames a phrase, a dimension's value, a heading's id, or a page, and changes everything that uses it. |
| [`ascribe report`](../checks/report-command.md#cli-report) | Reports what state a project is in, section by section. |
| [`ascribe resolve`](questions.md#cli-resolve) | Resolves a page for one build and writes it as a reader of that build sees it. [A page in the site output](site-output.md) says what `--emit site` writes. |
| [`ascribe search`](questions.md#cli-search) | Finds where the pages say something, as their readers read it, and names the line to edit. |
| [`ascribe sources`](../examples/sources.md#cli-sources) | Copies code from other repositories, pins it in `ascribe.lock`, and says what changed when the pin moves. |
| [`ascribe templates`](../templates/command.md#cli-templates) | Writes the page templates into a project, and says what's changed in them. |
| [`ascribe vale`](../checks/vale.md#cli-vale) | Writes Vale's preset out as the project's own configuration. |
| [`ascribe versions`](../versions/release.md#cli-versions) | Lists a project's release lines, says what differs between two, and releases, archives, or retires one. |

### Every command's synopsis

@include: ../_generated/cli-synopsis.md

### Output formats
@available: ascribe planned

`--format` takes the same words on every command that has it. Every command that takes `text` takes `concise` too: `ascribe check` lists one line a problem, and a command with no one-line form writes its text.

### Commands that write
@available: ascribe planned

A command that changes a file you commit, such as `ascribe baseline write`, `ascribe permalinks sync`, or `ascribe addresses move`, takes `--dry-run`, which says what it would change and changes nothing. It also refuses to change a file git couldn't put back as it was: one with changes not committed, one git doesn't track, or one in no repository. The error names the files; commit or stash them, or pass `--force` to write them anyway. A file that isn't there yet is only added, so it's never refused.

```console
$ ascribe addresses move guides/old.md guides/new.md
error: this would change files git can't put back as they are: ascribe.addresses; commit or stash them, or pass --force
```

## Options every command accepts

@include: ../_generated/cli-global-options.md

Without `--config`, a command's project is the nearest `ascribe.toml` at or above the current directory, or at or above the path it's given.

When there's none, and the folder holds projects in its subfolders, as a repository with several does at its root, the error names their folders, so you can run the command in one, or pass `--config` with its folder: `the projects below it are in docs, handbook: run ascribe in one of them, or pass --config with its folder`. `ascribe agents sync` is the exception: there, it syncs each of them.
