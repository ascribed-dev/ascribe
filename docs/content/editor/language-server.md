---
type: reference
title: The language server
description: "`ascribe lsp`, for an editor other than VS Code: the project a server serves, what it asks of the editor, and the requests of its own."
---

Every feature of the [VS Code extension](index.md) that isn't drawn on the screen comes from the language server, `ascribe lsp`, which runs the same checks as `ascribe check`. Any editor with a Language Server Protocol client can run it from a project installed with `@ascribed/cli`, and gets the problems, completion, hover, navigation, renaming, formatting, and fixes the extension has. The preview, the sidebar, and review are drawn by the extension, from the requests of the server's own below.

## `ascribe lsp`
@id {permalink=true}: cli-lsp
@summary: Runs the language server, speaking the Language Server Protocol over standard input and output. An editor starts it; you don't run it yourself.

Runs the language server, speaking the Language Server Protocol over standard input and output. An editor starts it; you don't run it yourself. It takes no options of its own, and refuses `--config` (exit code `2`): its project is the nearest `ascribe.toml` at or above the workspace folder the editor gives it, never one below, and `ascribe.toml`'s `[editor] build` says which build's page-level diagnostics to report. Its logs go to standard error, starting with the project it uses. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## The project a server serves

A server serves one project, the one `ascribe lsp` finds, so a server started at the root of a repository whose projects are all in subfolders has no project. For a workspace with several projects, start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml`, and give each file to the server of the nearest `ascribe.toml` above it. In Neovim, for example, `root_markers = { "ascribe.toml" }` in the server's `vim.lsp.config` does both. The VS Code extension does the same, one server for each project in the workspace: see [Workspaces with several projects](settings.md#workspaces-with-several-projects).

When it starts, the server sends a `window/logMessage` at Info with its version and the project it uses: `ascribe-lsp <version>: using the project at <path>`, or, when it finds none, `ascribe-lsp <version>: no ascribe.toml at or above <folder>`. A server with no project reports nothing. If an `ascribe.toml` is then created in the workspace folder itself, the server loads it, when the editor supports watching files for the server.

When one project is nested in another's content root, the outer project's server leaves the nested project's files alone, open or not, so each file gets diagnostics from one server only. A nested project's `ascribe.toml` created or deleted while the server runs changes which files it checks, when the editor supports watching files for the server.

## What it asks of the editor

The lens in a code file a page shows and the action that applies a page's safe fixes come from the server, so any editor that asks gets them. The server asks for lenses in its sources' files by registering them for those files (`client/registerCapability` for `textDocument/codeLens`), when the editor supports registering lenses; one that doesn't gets the lenses on `@include` in the pages it sends.

The lens on an `@include`, and on one memo, runs `ascribe.openFile`, the server's own command, which needs nothing of the editor: the server answers it by asking the editor to show the file (`window/showDocument`). Three commands are the editor's to run:

| Command | Carried by | Its arguments |
|---|---|---|
| `ascribe.showReferences` | The lens in a code file a page shows | A file, a position in it, and the places, as VS Code's `editor.action.showReferences` takes them |
| `ascribe.showMemos` | A lens that counts several memos | The memos, to pick one from |
| `ascribe.selectRange` | The quick fix that writes a term's definition | A file, and the range to select in it after the edit; an editor that doesn't run it still gets the edit |

The action that applies a page's safe fixes has the kind `source.fixAll.ascribe`, which `editor.codeActionsOnSave` can name ([Apply the safe fixes](problems.md#apply-the-safe-fixes)).

## Ascribe's own requests

Beside the standard requests, the server answers requests of its own, which the VS Code extension draws its views from. None is advertised as a capability: a client that wants one sends it. Each answers from the text the editor has, unsaved edits included, and each result's shape is a JSON Schema in the repository's [`schemas/`]({repo}/tree/main/schemas/) folder, generated from the code that answers it. The [language server's README]({repo}/tree/main/crates/ascribe-lsp#readme) gives each request's parameters.

| Request | What it answers | Its schema |
|---|---|---|
| `ascribe/preview` | A page rendered as the published site renders it, for a build, with source anchors on its blocks: the [page preview](preview.md) | `lsp-preview.schema.json` |
| `ascribe/review/setBase` | Sets the git revision to compare the project with, or clears it | `lsp-set-base.schema.json` |
| `ascribe/review/changes` | The pages of a build that changed against the base: the [Review view](review.md#the-review-view) | `lsp-changes.schema.json` |
| `ascribe/review/baseFile` | A file's text at the base, for a diff of the source | `lsp-base-file.schema.json` |
| `ascribe/context` | What's at a position or a selection of a page, so a client knows which actions apply there | `lsp-context.schema.json` |
| `ascribe/targets` | What an action can point at or use in the project: its pages, headings, fragments, images, snippets, phrases, and the rest | `lsp-targets.schema.json` |
| `ascribe/edit` | The edit one [action](actions.md) makes to a page | `lsp-edit.schema.json` |
| `ascribe/inventory` | What the project has and how much each part is used: the [Pages](sidebar.md#pages) and [Content model](sidebar.md#content-model) views | `lsp-inventory.schema.json` |
| `ascribe/settings` | Every setting of `ascribe.toml` with its value, its default, and its line: [Project Settings](settings.md#project-settings) | `lsp-settings.schema.json` |
| `ascribe/page` | What's known about a page, and what links to and includes the heading at a position: the [This page](sidebar.md#this-page) view | `lsp-page.schema.json` |
| `ascribe/projectEdit` | One edit to every page of the project, its safe fixes or its formatting: [Edits to the whole project](sidebar.md#edits-to-the-whole-project) | `lsp-project-edit.schema.json` |
| `ascribe/buildView` | What a build leaves out of a page: the [build lens](preview.md#the-build-lens) | `lsp-build-view.schema.json` |
| `ascribe/search` | Where the pages say something, as readers read them: [Search](writing.md#search) | `lsp-search.schema.json` |
| `ascribe/agentPrompt` | A prompt for an agent about one problem, a file's, or the project's: [Prompt your agent](../agents/prompts.md) | `lsp-agent-prompt.schema.json` |

The server also sends one notification of its own, `ascribe/preview/changed`, with `{ "pages": [uri, …] }`, when files changed on disk change how pages read: a page or a fragment it includes, or a file a `@snippet` in either shows. A client with a preview of one asks for it again; one that doesn't preview can ignore it.

### Lost addresses

In a project that [keeps its addresses](../publishing/redirects.md), the server sends a second notification, `ascribe/addresses`, with what comparing the addresses with the base's found: `ascribe.addresses`' URI, the base, the count of pages lost, and the problems at the file, as `ascribe check` reports them (`lsp-addresses.schema.json`). It compares after the project loads, a save, a change on disk, or an edit to `ascribe.addresses`, never as a page is typed. A client shows the problems as the project's, not as a page's diagnostics. `ascribe/page`'s `about.alsoAnswersAt` lists the old addresses that lead to the page.
