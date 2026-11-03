---
type: reference
title: The language server
description: "`ascribe lsp`, for an editor other than VS Code: setups for Neovim, Helix, and Zed, the project a server serves, what it asks of the editor, and the requests of its own, with their parameters."
---

Every feature of the [VS Code extension](index.md) that isn't drawn on the screen comes from the language server, `ascribe lsp`, which runs the same checks as `ascribe check`. Any editor with a Language Server Protocol client can run it from a project installed with `@ascribed/cli`, and gets the problems, completion, hover, navigation, renaming, formatting, and fixes the extension has. The preview, the sidebar, and review are drawn by the extension, from the requests of the server's own below.

## `ascribe lsp`
@id {permalink=true}: cli-lsp
@summary: Runs the language server, speaking the Language Server Protocol over standard input and output. An editor starts it; you don't run it yourself.

Runs the language server, speaking the Language Server Protocol over standard input and output. An editor starts it; you don't run it yourself. It takes no options of its own, and refuses `--config` (exit code `2`): its project is the nearest `ascribe.toml` at or above the workspace folder the editor gives it, never one below, and `ascribe.toml`'s `[editor] build` says which build's page-level diagnostics to report. Its logs go to standard error, starting with the project it uses. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## Set up your editor

An editor needs three things to run the server: the command, `npx ascribe lsp`; the files to send it, Markdown pages and `ascribe.toml`; and the project's root, the folder that holds `ascribe.toml`. `npx` runs the `@ascribed/cli` a project installs ([In a project, with npm](../install.md#npm)), looking for it from the folder the editor starts the server in and then in each parent. When the editor starts it somewhere else, or you put `ascribe` on your path instead ([On your path, without Node](../install.md#without-node)), write the binary's path, or `ascribe`, in place of `npx` and `ascribe`.

The server reads no settings of the editor's: it ignores `initializationOptions`, never asks for `workspace/configuration`, and doesn't follow `workspace/didChangeConfiguration`. What it checks and which build it reports come from `ascribe.toml`, its `[editor] build` above all, so every editor gets the same problems. The VS Code extension's [settings](settings.md) are the extension's own, and none of them reaches the server.

### Neovim

Neovim 0.11 and later configure a server with `vim.lsp.config`. In your `init.lua`:

```lua
vim.lsp.config("ascribe", {
  cmd = { "npx", "ascribe", "lsp" },
  filetypes = { "markdown", "toml" },
  root_markers = { "ascribe.toml" },
})
vim.lsp.enable("ascribe")
```

`root_markers` starts one server for each project, at the nearest `ascribe.toml` above the file, and a file outside every project gets none.

### Helix

Helix configures servers in `languages.toml`, in its configuration folder or in a project's `.helix/` folder:

```toml
[language-server.ascribe]
command = "npx"
args = ["ascribe", "lsp"]

[[language]]
name = "markdown"
roots = ["ascribe.toml"]
language-servers = ["marksman", "ascribe"]

[[language]]
name = "toml"
roots = ["ascribe.toml"]
language-servers = ["taplo", "ascribe"]
```

A language's `language-servers` replaces the list Helix has for it, so name the other servers you use beside `ascribe`, as these keep `marksman` and `taplo`. `roots` makes the nearest `ascribe.toml` above the file the server's root. `hx --health markdown` says whether Helix finds the command.

### Zed

Zed runs a language server only from an extension: its settings can change how an extension's server starts, but can't add a server. Ascribe has no Zed extension, so there's no setup to copy for Zed yet.

## The project a server serves

A server serves one project, the one `ascribe lsp` finds, so a server started at the root of a repository whose projects are all in subfolders has no project. For a workspace with several projects, start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml`, and give each file to the server of the nearest `ascribe.toml` above it. Marking a project's root by its `ascribe.toml` does both, as each [setup below](#set-up-your-editor) does. The VS Code extension does the same, one server for each project in the workspace: see [Workspaces with several projects](settings.md#workspaces-with-several-projects).

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

Beside the standard requests, the server answers requests of its own, which the VS Code extension draws its views from. None is advertised as a capability: a client that wants one sends it. Each answers from the text the editor has, unsaved edits included, and each result's shape is a JSON Schema in the repository's [`schemas/`]({repo}/tree/main/schemas/) folder, generated from the code that answers it. [Their parameters](#lsp-requests) are below.

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

## The requests' parameters
@id {permalink=true}: lsp-requests

Each request names a document by its URI, as `textDocument`, and the server answers for that document's project. Positions and ranges are in the position encoding the editor and the server agreed on at `initialize`. A request whose parameters don't parse gets the error `InvalidParams`; every other problem is in the result, as its schema says.

### `ascribe/preview`

```json
{
  "textDocument": { "uri": "file:///…/docs/install.md" },
  "build": "cloud",
  "review": true
}
```

`build` names the build to render the page for, and without it the editor's build, `[editor] build`. With `review` true and a base set, the result says what changed on the page against the base too.

### `ascribe/review/setBase`

```json
{ "base": "main" }
```

`base` is a branch, a tag, or a commit, compared from its merge base with `HEAD`, as [`ascribe diff`](../review/diff.md#cli-diff) compares. Without `base`, the default branch: the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master`. `null` stops comparing and frees the base. Nothing runs `git` until a base is set.

### `ascribe/review/changes`

```json
{ "build": "cloud" }
```

`build` is optional, the editor's build without it. The pages are compared with the base that `ascribe/review/setBase` set.

### `ascribe/review/baseFile`

```json
{ "textDocument": { "uri": "file:///…/docs/install.md" } }
```

The document is `ascribe.toml` or a source file in the content root.

### `ascribe/context`

```json
{
  "textDocument": { "uri": "file:///…/docs/install.md" },
  "range": { "start": { "line": 20, "character": 5 }, "end": { "line": 20, "character": 5 } }
}
```

An empty `range` is the cursor; any other is a selection.

### `ascribe/targets`

```json
{
  "textDocument": { "uri": "file:///…/docs/guide/install.md" },
  "kinds": ["pages", "headings", "fragments", "images", "snippets", "phrases",
            "notes", "dimensions", "widgets", "features", "builds", "occurrences", "types"],
  "range": { "start": { "line": 8, "character": 4 }, "end": { "line": 8, "character": 15 } },
  "folder": "file:///…/docs/guide"
}
```

`kinds` names the lists to answer, and the result has those and no others. `range`, the cursor or a selection, is optional, and only `occurrences` reads it: the other places in the project's prose with the words it selects. `folder`, a folder in the content root, is optional too, and only `types` reads it: the content types a page started there can be. The document can be any of the project's files; from `ascribe.toml`, paths are written from the content root.

### `ascribe/edit`

```json
{
  "textDocument": { "uri": "file:///…/docs/install.md" },
  "range": { "start": { "line": 20, "character": 0 }, "end": { "line": 20, "character": 0 } },
  "action": "wrapNote",
  "args": { "type": "tip" },
  "version": 12
}
```

`range` is the cursor or selection the action was chosen at, `action` one of the actions below, and `args` what it needs, `{}` when left out. `version`, optional, is the document version the editor saw: when the document has changed since, the action is refused. An insertable line is one `ascribe/context` answers `insertable` for, a blank line between blocks. `attributes` is an object of key to value: a string, a number, a boolean, or a list for a set. Names, types, keys, pages, and the rest are checked against the project, and `ascribe/targets` lists the valid choices. `?` marks an optional argument.

| Action | Applies to | Its arguments | What it writes |
|---|---|---|---|
| `wrapNote` | One paragraph, or whole blocks | `type?` (default `note`) | `@note` before one paragraph, or a container (`@note:` … `@end`) around several blocks; `{type=…}` unless the type is `note` |
| `setNoteType` | A note | `type` | The note's `type` attribute, or none for `note` |
| `unwrapNote` | A note | | The note's content, without the directive |
| `noteToDetails` | A note | `title` | `.Title` and `@details`, keeping the content and the form |
| `wrapDetails` | One block, or whole blocks | `title` | `.Title` and `@details` (or `@details:` … `@end`) |
| `unwrapDetails` | A details block | | Its content, without the title and the directive |
| `makeSteps` | An ordered list | | `@steps` above it |
| `removeSteps` | Steps | | The ordered list, without `@steps` |
| `addHeadingId` | A heading without `@id` | `id?` (default its slug) | `@id: …` after the heading |
| `insertNote` | An insertable line | `type?`, `text?` | A note; without `text`, a placeholder |
| `insertSteps` | An insertable line | `count?` (1 to 50, default 3) | `@steps` and a numbered list of placeholders |
| `insertVariantGroup` | An insertable line | `dimension`, `values` | One `@variant {dimension=value}:` arm per value, each with a placeholder, and `@end` |
| `addVariantArm` | A group of one dimension's arms | `value` | A new arm, in the dimension's declared order |
| `removeVariantArm` | An arm of a group with others | | The group without it |
| `insertDetails` | An insertable line | `title` | `.Title`, `@details:`, a placeholder, `@end` |
| `insertInclude` | An insertable line | `path` (with `#id` for a section) | `@include: …` |
| `insertSnippet` | An insertable line | `address`, `lang?`, `title?` | `@snippet: …`, with the attributes given |
| `insertImage` | An insertable line | `path`, `alt`, `attributes?` | An image of `path`, with `alt` as its text and the attributes after it |
| `insertWidget` | An insertable line | `name`, `primary?`, `attributes?` | The widget, attributes in declared order; the container form with a placeholder when it takes one, and a `.Title` placeholder when its title is required |
| `markAvailable` | A heading (its section), a block, or a table's body row | `spec` (a spec or a feature key) | `@available: …` under the heading or before the block; for a row, `{available=…}` at the end of its first cell |
| `setPageVariant` | Anywhere in a page | `dimension`, `value` | That dimension's value under `variant:` in the frontmatter, keeping the rest |
| `setPageAvailable` | Anywhere in a page | `spec` | `available:` in the frontmatter |
| `setPageLabel` | Any of the project's pages, open or not | `label?` (none or empty to take it out) | `nav-label:` in the frontmatter, a comment after it kept; nothing when it's already so |
| `linkSelection` | Prose selected in one paragraph or heading | `destination` | A link to `destination`, with the selection as its text |
| `insertLink` | A cursor in prose | `destination` | A link to `destination` with empty text, so the target's title fills it |
| `insertPhrase` | A cursor in prose | `key` | `{key}` |
| `setLinkTarget` | A link | `destination` | The link's destination |
| `useTargetTitle` | A link to a page | | Empty link text, so the target's title fills it |
| `setImageWidth` | An image | `width` | The image's `width` attribute, when the model declares one |
| `setImageAlt` | An image | `alt` | The image's alt text |
| `makePhrase` | Plain text selected in prose, on one line | `key`, `everywhere?` (default `false`) | `key = "<the text>"` in `[phrases]`, and `{key}` in place of the selection; with `everywhere`, in place of each of its `occurrences` too |
| `addGlossaryTerm` | Anywhere in a page | `term`, `definition`, `aliases?` (a list) | A section at the end of the first page `[glossary] pages` lists: the term as its heading, `@term` with the aliases, and the definition as its paragraph |
| `promoteFeature` | Anywhere in a page | `key`, `spec` | The feature's `available` in `[features.<key>]` |
| `newPage` | Any of the project's files: `textDocument` names the project, and `range` is unused | `type`, `title`, `folder?` (a folder in the content root, as a `file:` URI) | A new page, made by `ascribe_templates::start`: the type's template with its `title` set, or its frontmatter when it has none, with `type:` first in its frontmatter, in `folder` or the content root, named for the title. Never over a file. `select` is the first thing the template left to fill in |
| `moveSection` | Any of the project's files, as `newPage` | `folder` (a section's folder from the content root, ending in `/`), `before?` (a sibling section's folder; none for the end) | `ascribe.nav`'s `[[sections]]` with the section before `before` among its siblings: the siblings it didn't list are listed in the order they had, so nothing else moves |
| `movePage` | Any of the project's files, as `newPage` | `page` (a content path), `before?` (a page of the same section; none for the end) | The section's `pages`, listing its pages as they stood with `page` before `before`; its table is added, and its siblings', when the file doesn't list it |
| `placePage` | Any of the project's files, as `newPage` | `page` | `page` at the end of its section's `pages`, which leaves it out |
| `setSectionLabel` | Any of the project's files, as `newPage` | `folder`, `label?` (none or empty for the default) | The section's `label` |
| `setSetting` | Any of the project's files, as `newPage` | `table` (a dotted path, `consumer`; a check's is `checks.<check>`, with the key `level` or `limit`), `key`, `value` (a boolean, a whole number, or a string, as the setting's kind takes, or `null` to reset it to its default), `shown` (the `version` the `ascribe/settings` answer it was shown from had: a number, or `null` for the disk) | The value where `ascribe.toml` writes it, as it is now, and nothing else; a value the file leaves out added after its table's last entry, or in a new table at the end; or, for `null`, the setting's line taken out with the comment lines touching it from above. A check is set in the form the file writes it (a level alone, an inline table, or a `[checks.<check>]` table). Refused when `shown` isn't the version `ascribe.toml` has now, a buffer's for the disk's and the disk's for a buffer's included. Not refused for a problem the model then has, which `ascribe/settings` shows under the setting |
| `nextPlaceholder` | A page of a type with a template | | Nothing: `select` is the next thing the template left to fill in after `range`, or the first when none is after it |
| `addMemo` | Anywhere in a page | | A new memo file in `memos/`, its `documentChanges` creating it and giving it its text: `about` is the innermost section with an id at the cursor, as `page.md#id`, or the page above the first heading; `quote` is the selection, its whitespace runs as one space, or the first line of the block at the cursor, left out on a heading or when the memo wouldn't find it. Named from the quote's first words, or the heading's id, never over a file. `select` is the empty line for the memo's text. Refused when `memos/` is in the content root |
| `addPersonalMemo` | Anywhere in a page | | As `addMemo`, in `.ascribe/memos/` |

### `ascribe/inventory`

```json
{ "textDocument": { "uri": "file:///…/ascribe.toml" } }
```

The document can be any of the project's files.

### `ascribe/settings`

```json
{ "textDocument": { "uri": "file:///…/ascribe.toml" } }
```

The document can be any of the project's files.

### `ascribe/page`

```json
{
  "textDocument": { "uri": "file:///…/docs/keys.md" },
  "position": { "line": 14, "character": 0 }
}
```

`position` picks the heading whose links and includes the result lists.

### `ascribe/projectEdit`

```json
{ "textDocument": { "uri": "file:///…/ascribe.toml" }, "kind": "fixAll" }
```

The document can be any of the project's files. `kind` is `fixAll`, every safe fix of the problems last published for each file, or `format`, every file formatted as `ascribe fmt` formats it.

### `ascribe/buildView`

```json
{ "textDocument": { "uri": "file:///…/docs/guides/rollouts.md" }, "build": "self-hosted" }
```

`build` is optional, the editor's build without it.

### `ascribe/search`

```json
{ "textDocument": { "uri": "file:///…/ascribe.toml" }, "query": "\"sign in to quill cloud\"", "build": "site" }
```

The document can be any of the project's files. `query` is a query as [`ascribe search`](../command/questions.md#cli-search) takes it, and `build` is optional: without it, every build is searched.

### `ascribe/agentPrompt`

```json
{
  "kind": "problem",
  "textDocument": { "uri": "file:///…/docs/guides/install.md" },
  "diagnostic": { "range": { "start": { "line": 3, "character": 0 }, "end": { "line": 3, "character": 12 } }, "code": "ASC036", "message": "…" },
  "unsaved": ["file:///…/docs/guides/install.md"],
  "program": "/…/bin/linux-x64/ascribe"
}
```

`kind` is what the prompt is about:

| Kind | About | It also takes |
|---|---|---|
| `problem` | One problem | `textDocument`, and `diagnostic`, as the server published it |
| `file` | A file's problems | `textDocument` |
| `project` | The project's problems | `textDocument`, optional, to name the project |
| `pageChanges` | What changed on a page, while review is on | `textDocument`, and `build`, optional, the editor's build without it |
| `fragmentReach` | The pages that changed through a fragment, while review is on | `fragment`, a content path, and `build`, as `pageChanges` |

Every kind takes `unsaved`, the URIs of the documents with unsaved changes, so a prompt about one says to save it first, and `program`, optional, what the prompt's commands start with when `ascribe` isn't the binary the editor runs. Without `program`, they start with `ascribe`.
