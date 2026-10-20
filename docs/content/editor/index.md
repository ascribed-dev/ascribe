---
type: reference
title: Editing
description: The VS Code extension, workspaces with several projects, other editors, and the language server, `ascribe lsp`.
permalink: editor-guide
---

The Ascribe extension for VS Code turns the editor into an Ascribe authoring environment. Its features come from the language server, [`ascribe lsp`](#cli-lsp), which runs the same checks as `ascribe check`: what the editor reports is what CI reports.

Every setting and command the extension adds, and how it works in a workspace with several projects, are on [their own page](settings.md).

## Install

Install **Ascribe** (`Ascribe.ascribe-vscode`) from the Visual Studio Marketplace: search for "Ascribe" in the Extensions view, or run:

```sh
code --install-extension Ascribe.ascribe-vscode
```

The extension activates in a workspace that has an `ascribe.toml` anywhere in it, and a workspace can hold several projects (see [Workspaces with several projects](settings.md#workspaces-with-several-projects)). It needs VS Code {vscode} or later, and a trusted workspace, since it runs the project's `ascribe` binary. It's available for macOS on Apple silicon, Linux (x64 and arm64), and Windows (x64); Intel Macs aren't supported.

### Which `ascribe` it runs

The extension includes an `ascribe` binary for your platform, so it works immediately. For each project, it looks for a binary in this order:

1. The `ascribe.path` setting, if you set it. It applies to every project. A path that doesn't work is an error; the extension doesn't fall back.
2. The project's own `node_modules/.bin/ascribe`, installed with `npm install --save-dev @ascribed/cli`. It looks in the folder that holds the project's `ascribe.toml`, then in each parent up to the workspace folder, so a project in a monorepo finds the `node_modules` at the repository root.
3. The binary included in the extension.

A project that installs `@ascribed/cli` gets exactly the version it pins, in the editor as in CI, and two projects in one workspace can pin different versions. When a version is older than the extension expects, the extension warns you to update it. **Ascribe: Show Server Output** shows which binary a project uses.

### The walkthrough
@available: ascribe 0.3.0

After you install the extension, VS Code opens its walkthrough, **Get Started with Ascribe**, which goes through what a writer does first: open a project, preview a page, use the actions bar, look at the Ascribe sidebar, see what a build leaves out, and check the project in CI. To see it again, open **Help → Welcome**, or run **Welcome: Open Walkthrough…** and choose it. Its buttons act on the page you last had open.

## What it does

Each of the extension's features is on the page for what you're doing with it:

- [Problems and fixes](problems.md): the problems the editor shows as you type, their quick fixes, and applying the safe ones.
- [Writing in the editor](writing.md): completion, hover, navigation, the lens on code files a page shows, renaming, formatting, and highlighting.
- [Actions, new pages, and memos](actions.md): the actions that write Ascribe for you, starting a page from its template, and memo files.
- [The preview and builds](preview.md): the page preview, the site preview, the status bar, and the build lens.
- [The Ascribe sidebar](sidebar.md): This page, Pages, and Content model, and switching the project they show.
- [Review in the preview](review.md): what changed against a base, the Review view, and a pull request's comments.
- [Agents in the editor](agents.md): prompting your agent to fix a problem, and the tools and skill an agent in VS Code gets.
- [Workspaces, settings, and commands](settings.md): several projects in one workspace, and every setting and command.

## Other editors

Any editor with a Language Server Protocol client can run [`ascribe lsp`](#cli-lsp) from a project installed with `@ascribed/cli`. The preview is a VS Code feature.

A server serves one project, the one [`ascribe lsp`](#cli-lsp) finds, so a server started at the root of a repository whose projects are all in subfolders has no project. For a workspace with several projects, start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml`, and give each file to the server of the nearest `ascribe.toml` above it. In Neovim, for example, `root_markers = { "ascribe.toml" }` in the server's `vim.lsp.config` does both.

The lens in a code file a page shows and the action that applies a page's safe fixes come from the server, so any editor that asks gets them. The server asks for lenses in its sources' files by registering them for those files (`client/registerCapability` for `textDocument/codeLens`), when the editor supports registering lenses; one that doesn't gets the lenses on `@include` in the pages it sends. The lens's command, `ascribe.showReferences`, is the editor's to run, with a file, a position in it, and the places, as VS Code's `editor.action.showReferences` takes them. The quick fix that writes a term's definition carries `ascribe.selectRange`, with a file and the range to select in it after the edit; an editor that doesn't run it still gets the edit. The action's kind is `source.fixAll.ascribe`.

When it starts, the server sends a `window/logMessage` at Info with its version and the project it uses: `ascribe-lsp <version>: using the project at <path>`, or, when it finds none, `ascribe-lsp <version>: no ascribe.toml at or above <folder>`. A server with no project reports nothing. If an `ascribe.toml` is then created in the workspace folder itself, the server loads it, when the editor supports watching files for the server.

When one project is nested in another's content root, the outer project's server leaves the nested project's files alone, open or not, so each file gets diagnostics from one server only. A nested project's `ascribe.toml` created or deleted while the server runs changes which files it checks, when the editor supports watching files for the server.

## `ascribe lsp`
@id {permalink=true}: cli-lsp

Runs the language server, speaking the Language Server Protocol over standard input and output. An editor starts it; you don't run it yourself. It takes no options of its own, and refuses `--config` (exit code `2`): its project is the nearest `ascribe.toml` at or above the workspace folder the editor gives it, never one below, and `ascribe.toml`'s `[editor] build` says which build's page-level diagnostics to report. Its logs go to standard error, starting with the project it uses.

The VS Code extension runs it for you, one server for each project in the workspace. Any other editor with an LSP client can run it too; for several projects, start one per project ([Other editors](#other-editors)). The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## Other formatters

A Markdown formatter that reflows paragraphs, such as Prettier with `proseWrap: always`, doesn't know that title and directive lines start new blocks, and would join them with the text below. Exclude Ascribe sources from it, for example with `**/*.md` in `.prettierignore`, and use `ascribe fmt`. In VS Code, the extension offers to be the Markdown formatter for your pages; see [Formatting](writing.md#formatting).
