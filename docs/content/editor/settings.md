---
type: reference
title: Editor workspaces, settings, and commands
description: "How the VS Code extension handles a workspace with several projects, and its settings and commands."
---

How the [VS Code extension](index.md) works in a workspace that holds more than one Ascribe project, and every setting and command it adds.

## Workspaces with several projects
@id {permalink=true}: workspaces-with-several-projects

Every `ascribe.toml` in the workspace is a project, with its own language server. A repository that keeps its code and its documentation together can hold several, such as `site/ascribe.toml` and `handbook/ascribe.toml`, and you can work on all of them in one window. Folders named `node_modules` aren't searched.

@available: ascribe 0.3.2
A folder whose `ascribe.toml` isn't a project, such as test fixtures, is left out with `ascribe.projects.exclude`: a list of glob patterns, from each workspace folder, in your workspace's settings, such as `"ascribe.projects.exclude": ["tests/**"]`. A project there isn't listed in **Switch Project** and gets no language server, so its pages get no diagnostics and no preview. Folders that `files.exclude` hides are left out too.

A file belongs to the nearest `ascribe.toml` above it, and only that project's server reports on it. Projects can be nested: a file in `handbook/internal/` belongs to `handbook/internal/ascribe.toml`, not to `handbook/ascribe.toml`. A project nested in another's content root isn't part of it: the outer project's server, `ascribe check`, and `ascribe build` skip the nested project's folder, so a link or an include from the outer project to one of its pages is reported as a missing file. A file in a project's folder but outside its content root isn't one of the project's sources, and gets no diagnostics.

Each project runs its own `ascribe`, found as [Which `ascribe` it runs](index.md#which-ascribe-it-runs) describes.

### When servers start

A project's server starts the first time you open one of its Markdown files or its `ascribe.toml`, or preview one of its pages. It also starts when one of its files changes on disk while the window is open, so an agent or a script that writes the project's files without opening them still gets their problems in the Problems panel. A file counts if it's the project's `ascribe.toml`, is under its content root (not in a project nested there), or is in the folder of one of its [sources](../content-model/output.md#18-sourcesname), and isn't in its output directory, `node_modules`, or `.git`. Changes are gathered for up to a second, so a `git checkout` that touches hundreds of files starts each project's server once. Opening the window starts nothing, and neither does adding an `ascribe.toml`. A server runs until you close the window or delete its `ascribe.toml`. A project nothing opens or changes costs nothing.

To start every project's server when the workspace opens, set `ascribe.startServers` to `"all"`.

An `ascribe.toml` you add or delete is picked up as you work. If the editor misses a new one, opening or switching to one of the new project's files picks it up. The extension serves at most 50 projects in a workspace: the first 50 by path. When there are more, the first project's output says so, and that **Switch Project** lists only those 50; `ascribe.projects.exclude` leaves out the folders that aren't projects.

Each server uses about 6 MB of memory, plus about 70 KB per page of its project.

### The Problems panel

The Problems panel lists diagnostics only for the projects whose server is running. A project whose files you haven't opened or changed shows nothing there, even if it has problems. Open one of its files, or set `ascribe.startServers` to `"all"`, to see them.

A running server reports the problems in each file, and the problems of its pages as the editor's build (`[editor] build`) has them. A problem only another build has, such as a link to a page that build leaves out, isn't in the Problems panel until you check every build. While a project has several builds and the panel holds the editor build's problems alone, the [status bar](preview.md#the-status-bar) says so: **docs · site · problems: this build only**.

@available: ascribe 0.3.0
**Ascribe: Check Every Build** (also **Check every build** in the status bar's menu) runs `ascribe check` over every build of the active file's project, as it reads the files on disk, and adds the problems the server doesn't show to the Problems panel: those of other builds, each naming the builds it's in (`only in build cloud`), and those of the checks the editor doesn't run, such as `image-unused`. It says how many problems it found in all, and how many it added. When a file changes, the problems it added to that file go, since they may have moved, and the status bar says the panel covers the editor's build alone again, until you run it again. A change to any file of the project counts, not only its pages: `ascribe.nav`, `ascribe.permalinks`, and its memos and templates too. The [Project view](sidebar.md#the-project-view) counts the problems it found, and says when the count is out of date. In a workspace with several projects and no file of one active, it asks which project to check.

An agent that reads the Problems panel sees the same: before calling a project clean, check every build, or run `ascribe check`, which checks every build.

`ascribe check` checks a project whether or not the editor has it open. Run it in the project's folder, or from anywhere with `--config` naming the project's folder or its `ascribe.toml`:

```sh
npx ascribe check --config handbook
```

### Commands and output

Commands act on the project that owns the active file: the preview, its **Build** picker, **Switch Build**, review, and **Show Server Output**. The [status bar](preview.md#the-status-bar) names that project. The sidebar shows it too, and [Switch Project](sidebar.md#switch-project) lists them all and shows another in the sidebar until a file of another project is active. **Restart Language Server** restarts every project's server that has started. [Commands](settings.md#commands) has the details.

Each project's server logs to its own output channel, `Ascribe (<project>)`. The project's name is its folder relative to the workspace folder, such as `Ascribe (handbook/internal)`, or the workspace folder's name for a project at its root. In a workspace with one project, the channel is just `Ascribe`.

A channel's name is set when the channel is first needed, usually when its project's server first starts. So if a workspace with one project gains a second, the first project's channel keeps the name `Ascribe`, beside `Ascribe (<other project>)`, until you reload the window.

When it starts, a server logs its version and the `ascribe.toml` it uses: `ascribe-lsp <version>: using the project at <path>`.

## Settings
@id {permalink=true}: editor-settings

@include: ../_generated/editor-settings.md

## Commands

@include: ../_generated/editor-commands.md
