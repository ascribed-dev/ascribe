---
type: reference
title: Editor workspaces, settings, tasks, and commands
description: "How the VS Code extension handles a workspace with several projects, Project Settings, its tasks, and its settings and commands."
---

How the [VS Code extension](index.md) works in a workspace that holds more than one Ascribe project, the project's own settings, the tasks it runs, and every setting and command it adds.

## Workspaces with several projects
@id {permalink=true}: workspaces-with-several-projects

@summary
Every `ascribe.toml` in the workspace is a project, with its own language server. A repository that keeps its code and its documentation together can hold several, such as `site/ascribe.toml` and `handbook/ascribe.toml`, and you can work on all of them in one window.

Some folders hold an `ascribe.toml` that isn't a project to work on, and the editor leaves them out on its own:

- **What git ignores.** In a workspace folder that's a git repository, an `ascribe.toml` git ignores isn't a project, such as one a test run writes into a build folder. One git tracks is, whatever the ignore rules say. Without git, or outside a repository, the folders below are still left out.
- **Output, dependencies, and caches,** in every folder with one of these names, whether or not git ignores it: `.ascribe`, each project's outputs and caches; `node_modules`, `bower_components`, `vendor`, `.venv`, `venv`, and `__pypackages__`; and `target`, `dist`, `.output`, `.next`, `.nuxt`, `.svelte-kit`, `.astro`, `.turbo`, `.vercel`, `.netlify`, `.cache`, `coverage`, `.gradle`, `.vscode-test`, `.tox`, and `.nox`. Not `build`, `out`, or `bin`, which are also names of folders people write in: where they're output, git ignores them.
- **A source's copies.** The copies of a [source in another repository](../content-model/sources.md#20-sourcesname), `sources/<name>/` in a project's folder, can hold that repository's own `ascribe.toml`: it's code the project shows, not a project.
- **Folders that `files.exclude` hides.**

Anything else whose `ascribe.toml` isn't a project, such as test fixtures, is left out with `ascribe.projects.exclude`: a list of glob patterns, from each workspace folder, in your workspace's settings, such as `"ascribe.projects.exclude": ["tests/**"]`.

A project left out isn't listed in **Switch Project** and gets no language server, so its pages get no diagnostics and no preview. To bring one back, open its folder as a workspace folder of its own (**File > Add Folder to Workspace**): a workspace folder's own `ascribe.toml` is always a project, unless `files.exclude` or `ascribe.projects.exclude` names it. One that `ascribe.projects.exclude` leaves out comes back when you take its pattern out, and one git ignores comes back when git tracks its `ascribe.toml` (`git add -f`).

A file belongs to the nearest `ascribe.toml` above it, and only that project's server reports on it. Projects can be nested: a file in `handbook/internal/` belongs to `handbook/internal/ascribe.toml`, not to `handbook/ascribe.toml`. A project nested in another's content root isn't part of it: the outer project's server, `ascribe check`, and `ascribe build` skip the nested project's folder, so a link or an include from the outer project to one of its pages is reported as a missing file. A file in a project's folder but outside its content root isn't one of the project's sources, and gets no diagnostics.

In the Problems panel, the source of each problem names the project whose server found it, such as `ascribe · handbook`, when the workspace has more than one project.

Each project runs its own `ascribe`, found as [Which `ascribe` it runs](index.md#which-ascribe-it-runs) describes.

### When servers start

A project's server starts the first time you open one of its Markdown files or its `ascribe.toml`, or preview one of its pages. It also starts when something asks for the project: choosing it in **Switch Project** or the Project view, a command run on it, or **Check Every Build**.

A change on disk starts nothing: a branch switch, a pull, a test run that rewrites files, or an agent that writes files without opening them leaves a project that hasn't started as it was, with no problems in the Problems panel and "not checked yet" in **Switch Project**. Opening the window starts nothing either, and neither does adding an `ascribe.toml`.

Once a server is running, it follows changes on disk to its project's files. A server runs until you close the window or delete its `ascribe.toml` or its folder, and its problems leave the Problems panel with it. A server that doesn't stop when asked is killed, with any process it started, and none outlives the window. A project nothing opens costs nothing.

To start every project's server when the workspace opens, set `ascribe.startServers` to `"all"`.

An `ascribe.toml` you add or delete is picked up as you work. If the editor misses a new one, opening or switching to one of the new project's files picks it up. The extension serves at most 50 projects in a workspace: the first 50 by path. When there are more, the first project's output says so, and that **Switch Project** lists only those 50; `ascribe.projects.exclude` leaves out the folders that aren't projects, beside [those left out on their own](#workspaces-with-several-projects).

Each server uses about 6 MB of memory, plus about 70 KB per page of its project.

### Commands and output

Commands act on the project that owns the active file: the preview, its **Build** picker, **Switch Build**, review, and **Show Server Output**. The [status bar](sidebar.md#the-status-bar) names that project. The sidebar shows it too, and [Switch Project](sidebar.md#switch-project) lists them all and shows another in the sidebar until a file of another project is active. **Restart Language Server** restarts every project's server that has started. [Commands](settings.md#commands) has the details.

Each project's server logs to its own output channel, `Ascribe (<project>)`. The project's name is its folder relative to the workspace folder, such as `Ascribe (handbook/internal)`, or the workspace folder's name for a project at its root. In a workspace with one project, the channel is just `Ascribe`.

A channel's name is set when the channel is first needed, usually when its project's server first starts. So if a workspace with one project gains a second, the first project's channel keeps the name `Ascribe`, beside `Ascribe (<other project>)`, until you reload the window.

A server's first line names its version and the `ascribe.toml` it uses, as [the language server](language-server.md#the-project-a-server-serves) says.

## Project Settings

**Ascribe: Open Project Settings** opens a tab of the project's own settings: those in its `ascribe.toml`, which everyone working on the project shares, and which the file holds for them all to read, review, and commit. Your editor's settings, the `ascribe.*` ones below, are VS Code's, and the tab links to them.

The tab opens from the Command Palette, from the gear in the [Project view](sidebar.md#the-project-view)'s head, and from the gear in `ascribe.toml`'s title bar. The button is there only on the `ascribe.toml` of a project the extension found: not on one that `ascribe.projects.exclude` leaves out, such as a test fixture's (see [Workspaces with several projects](#workspaces-with-several-projects)). Each project has one tab, titled with the project's name: opening it again shows the tab already open. Closing and reopening `ascribe.toml` changes nothing, since the tab reads the file from the disk while it isn't open. When `ascribe.toml` is deleted or renamed, the project is gone, so the tab says so, and closes when you click it.

### Finding a setting

The search at the top looks in every setting's name, its key, and its whole description, so a word from any sentence of what a setting does finds it, and so does a check's name or what it reports. Each word you type must be there, in any case. **Set in ascribe.toml** keeps to what the file sets. On the left, the sections list the tables, the parts of the project, and the checks; click one to go to it. A section with nothing that matches is left out of the list. The tab keeps the search, the filter, and where you were while you look at another tab.

### What the tab shows

Each setting shows its name, its key (`[consumer] trailing-slash`), what it does, its default, and where the file sets it, as a link to the line (`ascribe.toml:12`), or that the file leaves it out. A bar beside a setting marks one the file sets, as VS Code's Settings editor marks what's changed. The tab shows every setting that's a switch, a choice, a number, or a short text, under a heading for its table (`[project]`, `[consumer]`, `[editor]`, and the rest), in the order of the [content model reference](../content-model/index.md). A table or a list is the file's to edit, and a part of the project with a name of its own, such as a content type or a build, is a card that opens the file at it.

### Changing a value

The tab changes a value the file sets: choose another, or type one and press Enter. The change is made in `ascribe.toml` as an edit to that one value, unsaved, so the file's comments, blank lines, and order stay as they were, and **Undo** in the file puts it back. Typing in `ascribe.toml` changes the tab too: it always shows the file as the editor has it, unsaved text included, and never an older version of it. A value the content model doesn't take is still set, and the language server's problem shows under the setting, in the words of the Problems panel.

A setting the file leaves out is shown with its default, and changing it adds it to its table, after the table's last setting, or adds the table at the end of the file when it has none. **Reset to default**, beside a setting the file sets, takes its line out of the file, and the comment directly above it with it: a comment that touches the line is about that setting. A comment with a blank line before the setting stays, since it may be about the lines around it too. Both are unsaved edits that **Undo** puts back.

### Content types, builds, and the rest

A content type, a dimension, a release target, a build, and a source are tables of their own, which the file holds as text for a reviewer to read. Each is a card under its section, **Content types**, **Dimensions**, **Releases**, **Builds**, or **Sources**, with its name, its table (`[types.guide]`), the keys the file sets in it, and its line. **Edit in ascribe.toml** opens the file at its table. `[frontmatter]`, the fields every content type shares, is a card of its own, first under **Content types**, with the fields it declares. Phrases, features, and glossary terms are added, renamed, and found in the [Content model view](sidebar.md#content-model), which **Open Content model** shows.

### Checks

Under **Checks**, the tab lists every check whose level `[checks]` can set ([Setting a check's level](../checks/diagnostics.md#setting-a-checks-level)), the ones the file names first, then the rest in the diagnostics reference's order. Each shows what it reports, its level, and its own level, which it has when the file sets none. `page-size`, `image-large`, `memo-long`, and `summary-long` show their `limit` too, with its default. A level and a limit are set and reset as any setting is, in the form the file writes the check: `page-size = "warning"` in `[checks]`, an inline table, or a `[checks.page-size]` table of its own.

## Tasks

The extension adds a task type, `ascribe`, for the jobs that write files: **Build** runs `ascribe build`, **Sync Agent Files** runs `ascribe agents sync`, and **Observe Examples** runs [`ascribe observe`](../examples/tested.md#2-run-the-test-and-commit-what-it-records). Each runs in the terminal, with its output in view as it would be in CI, and with no problem matcher: the Problems panel stays the editor's and [Check Every Build](problems.md#the-problems-panel)'s. The [Project view](sidebar.md#the-project-view)'s Actions run them, and so does **Tasks: Run Task**, where each project has its own, named after the project when the workspace has more than one.

Each runs the `ascribe` binary the project's language server runs: the `ascribe.path` setting, the project's own `node_modules/.bin/ascribe`, or the one bundled with the extension. Build and Sync Agent Files run in the project's folder, with `--config` naming its `ascribe.toml` when the project isn't the workspace folder. None is offered in Restricted Mode, where the extension runs nothing until you trust the folder.

**Observe Examples** runs in the folder of a [source with tested examples](../examples/tested.md), one task for each, named after the source when the project has more than one; `ascribe observe` runs only the `ascribe.examples.toml` in the folder it's run in, and takes no `--config`. Which sources have tested examples is what `ascribe model --section sources` says: those with an `ascribe.examples.toml`, and not a [source in another repository](../examples/docs-repository.md), whose config is never run. The config names commands to run, so before running them, `ascribe observe` lists them in the task's terminal and asks **Run them? [y/N]**. Type `y` to run them; anything else runs nothing. The Project view's **Observe Examples** asks which source first when there's more than one.

To bind one to a key, or run it from your own task, name it in `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    { "type": "ascribe", "task": "build", "project": "docs", "label": "Build the docs" }
  ]
}
```

| Property | What it says |
|---|---|
| `task` | `build`, `syncAgentFiles`, or `observeExamples` |
| `project` | The project's folder, relative to the workspace folder. Leave it out for the project whose `ascribe.toml` is in the workspace folder. |
| `source` | For `observeExamples`: the source to observe, by its name in `ascribe.toml`. Leave it out when the project has one source with tested examples. |

## Settings
@id {permalink=true}: editor-settings
@summary: The extension's settings: which `ascribe` binary it runs, when it starts each project's language server, and what it shows.

@include: ../_generated/editor-settings.md

## Commands

@include: ../_generated/editor-commands.md
