---
type: reference
title: Command reference
description: "The ascribe commands: their options, outputs, and exit codes."
permalink: cli-reference
---

The `ascribe` command checks, builds, and formats an Ascribe project, and runs the language server the editor uses. Its `explain`, `model`, `outline`, `link`, `refs`, and `render` commands answer questions about a project, for writers and for agents, without changing anything, and `ascribe mcp` offers them to agents as the tools of an MCP server.

@include: ../_generated/cli-synopsis.md

Install it in a project with `npm install --save-dev @ascribed/cli` (see [Install](../install.md#npm)) and run it with `npx ascribe`, or from a script in `package.json`. Where there's no Node, install it on your path with Homebrew or the install script ([Install](../install.md#without-node)) and run it as `ascribe`.

## Commands

Each command's options, output, and exit codes are on the page for its kind of work:

- [`ascribe check`, `build`, and `fmt`](../checks/check.md): check a project, build its outputs, and format its Markdown.
- [`ascribe diff` and `drift`](../review/diff.md): compare a project with another revision, and find what drifted from its sources.
- [`ascribe report`](../checks/report-command.md): report on a project's health.
- [`ascribe explain`, `model`, `outline`, `link`, `refs`, `rename`, and `render`](questions.md): answer questions about a project, and rename what pages link to.
- [`ascribe agents` and `mcp`](../agents/commands.md): set a repository up for agents, and serve a project to agents.
- [`ascribe lsp`](../editor/index.md#cli-lsp): serve a project to an editor.
- [`ascribe versions`](../versions/release.md): list a project's release lines, say what differs between two, release one, archive one, and retire one.
- [`ascribe init`, `sources`, `observe`, `permalinks`, `templates`, and `vale`](../examples/sources.md): start a project, and look after its source repositories, its tested examples, permalinks, page templates, and Vale's configuration.

## Options every command accepts

@include: ../_generated/cli-global-options.md

Without `--config`, a command's project is the nearest `ascribe.toml` at or above the current directory, or at or above the path it's given.

@available: ascribe 0.3.0
When there's none, and the folder holds projects in its subfolders, as a repository with several does at its root, the error names their folders, so you can run the command in one, or pass `--config` with its folder: `the projects below it are in docs, handbook: run ascribe in one of them, or pass --config with its folder`. `ascribe agents sync` is the exception: there, it syncs each of them.
