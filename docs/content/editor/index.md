---
type: reference
title: Editing
description: "Installing the VS Code extension, which `ascribe` it runs, and a page for each part of the editor."
permalink: editor-guide
---

The Ascribe extension for VS Code turns the editor into an Ascribe authoring environment. Its features come from the [language server](language-server.md), `ascribe lsp`, which runs the same checks as `ascribe check`: what the editor reports is what CI reports.

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

After you install the extension, VS Code opens its walkthrough, **Get Started with Ascribe**, which goes through what a writer does first: open a project, preview a page, use the actions bar, look at the Ascribe sidebar, see what a build leaves out, and check the project in CI. To see it again, open **Help → Welcome**, or run **Welcome: Open Walkthrough…** and choose it. Its buttons act on the page you last had open.

## What it does

Each of the extension's features is on the page for what you're doing with it:

- [Problems and fixes](problems.md): the problems the editor shows as you type, checking every build, their quick fixes, and applying the safe ones.
- [Writing in the editor](writing.md): completion, hover, navigation, the lens on code files a page shows, renaming, formatting, and highlighting.
- [Actions, new pages, and memos](actions.md): the actions that write Ascribe for you, starting a page from its template, and memo files.
- [The preview and builds](preview.md): the page preview, the site preview, the build you're looking at, and the build lens.
- [The sidebar and the status bar](sidebar.md): the Project, This page, Pages, and Content model views, the status bar item, and switching the project they show.
- [Review in the preview](review.md): what changed against a base, the Review view, and a pull request's comments.
- [Agents in the editor](agents.md): prompting your agent to fix a problem, and the tools and skill an agent in VS Code gets.
- [Workspaces, settings, tasks, and commands](settings.md): several projects in one workspace, Project Settings, the tasks, and every setting and command.
- [The language server](language-server.md): `ascribe lsp`, for an editor other than VS Code.
