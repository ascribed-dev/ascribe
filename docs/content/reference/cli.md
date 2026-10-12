---
title: Command reference
description: "The ascribe commands: their options, outputs, and exit codes."
permalink: cli-reference
---

The `ascribe` command checks, builds, and formats an Ascribe project, and runs the language server the editor uses. Its `explain`, `model`, `outline`, `link`, `refs`, and `render` commands answer questions about a project, for writers and for agents, without changing anything, and `ascribe mcp` offers them to agents as the tools of an MCP server.

@include: ../_generated/cli-synopsis.md

Install it in a project with `npm install --save-dev @ascribed/cli` (see [Getting started](../getting-started.md#install-the-command)) and run it with `npx ascribe`, or from a script in `package.json`.

## Commands

Each command's options, output, and exit codes are on the page for its kind of work:

- [`ascribe check`, `build`, and `fmt`](cli/check.md): check a project, build its outputs, and format its Markdown.
- [`ascribe diff`, `drift`, and `report`](cli/diff.md): compare a project with another revision, find what drifted from its sources, and report on its health.
- [`ascribe explain`, `model`, `outline`, `link`, `refs`, `rename`, and `render`](cli/questions.md): answer questions about a project, and rename what pages link to.
- [`ascribe agents`, `mcp`, and `lsp`](cli/agents.md): set a repository up for agents, and serve a project to agents and editors.
- [`ascribe sources`, `permalinks`, and `vale`](cli/sources.md): look after source repositories, permalinks, and Vale's configuration.

## Options every command accepts

@include: ../_generated/cli-global-options.md

Without `--config`, a command's project is the nearest `ascribe.toml` at or above the current directory, or at or above the path it's given.

@available: next
When there's none, and the folder holds projects in its subfolders, as a repository with several does at its root, the error names their folders, so you can run the command in one, or pass `--config` with its folder: `the projects below it are in docs, handbook: run ascribe in one of them, or pass --config with its folder`. `ascribe agents sync` is the exception: there, it syncs each of them.
