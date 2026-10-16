# Proposal: the editor's views

A proposal, not a plan. It says what the extension's sidebar and its other surfaces would become, so the idea can be judged before anything is designed in detail. It comes from a survey of what other VS Code extensions did with their UI, and what they kept and took back: [VS Code extension UI patterns](../reports/VS%20Code%20extension%20UI%20patterns.md).

## The goal

The sidebar says what needs attention where the author is: this page's problems, its connections, its review, and what a branch changed. Today it lists what exists.

Everything else stays where VS Code users already look: problems in the Problems panel with their fixes, the build and the server in the status bar, writing in the actions bar, and the rendered page in the preview.

## What exists

- **Four tree views** in one container, `ascribe` (`packages/vscode/package.json`; `src/ui/sidebar.ts`, `sidebarViews.ts`, `projectsView.ts`):
  - **Projects:** every `ascribe.toml` the registry finds, up to 50, each with its server's state, its editor build, and its binary, and buttons to show the output and restart. It holds the two welcome messages, for Restricted Mode and for a workspace with no project, since it's the one view without `when: ascribe.active`.
  - **Used by:** what links to the active page and what includes it, narrowed to the heading at the cursor.
  - **Pages:** pages by content type, then fragments, then orphans.
  - **Content model:** phrases, features, glossary terms, dimensions, note types, widgets, and builds, with their uses. Clicking an entry opens its declaration, and that's all an entry does.
- **The three project views follow the active file's project,** and stay on the last one when the file is in none (`sidebarViews.ts`, `follow`).
- **The status bar item** shows the project, the build, the server's state, the build lens, and how much of the Problems panel the build covers. Its tooltip names the folder, `ascribe.toml`, and the binary, and its menu has Switch Build, Check Every Build, Show Output, and Restart (`src/ui/statusBar.ts`, `describe.ts`).
- **Review** lives in the preview: the marks, stepping through changes, the threads, and Submit. Outside it there's the status item, the comment controller on source lines, and **Changed Pages** (`ascribe.changedPages`), a quick pick that closes when a page is chosen. `ascribe.reviewOn` is set while review is on (`src/preview/review.ts`).
- **The actions bar** (`ascribe.actions`, `Cmd+K A`) lists what applies at the cursor, among them **Insert a phrase** and **Insert a widget** (`src/actions/registry.ts`).
- **The language server** has one CodeLens, on `@include`, and quick fixes one at a time; `ascribe check --fix` applies every `safe` fix from the command line. The client sends it Markdown files and Ascribe's TOML files only (`src/client.ts`).
- **Pages are the `markdown` language,** so VS Code's Markdown extension gives them their Outline, breadcrumbs, `Ctrl+T`, folding, and drop and paste of files and images.
- **Agents** get the MCP server, three tools, and Prompt agent (decision 57). `ascribe agents sync` writes the skill to `.agents/skills/ascribe/` and `.claude/skills/ascribe/`; the editor sets up no agent files (the surface audit's D4).

## The sidebar

Three views, and a fourth while review is on:

| View | Shown |
|---|---|
| **Review** | While review is on, first |
| **This page** | While a project is active |
| **Pages** | Always, with the welcome messages when there's no project |
| **Content model** | While a project is active |

They're native trees. Whether a view would be better as a webview is a question for later, below.

Pages being always shown keeps the Ascribe container in a workspace with no project, as Projects does today. That's deliberate: it's where the welcome messages say how to start one.

### No Projects view

A workspace with one project, the common case, has its project in the status bar already. For more than one:

- **Ascribe: Switch Project** lists each project with its folder and its state ("running, docs build", "not checked yet", "failed"), the current one first and marked. Show Output and Restart are buttons on each item. Choosing a project switches the sidebar to it and starts its server, since the views need one.
- **It's shown only while the workspace has two or more projects,** by a context key, `ascribe.multipleProjects`, that the registry keeps as projects are found, left out by `ascribe.projects.exclude`, or deleted. Then it's a button in the title bars of Pages and Content model, beside the project's name, in the Command Palette, and in the status bar item's menu.
- **The sidebar keeps following the open file.** Choosing a project is a jump, not a pin: the next file of another project moves the views to it.
- **With one project,** Show Output and Restart are in the status bar item's menu, as now.

Pages loses `when: ascribe.active` and takes the two welcome messages. The 50-project cap and `ascribe.projects.exclude` stay, and the cap's message names **Switch Project** instead of the view.

### This page

**Used by** becomes **This page**: what Ascribe knows about the active page that an author can use, in sections.

- **About:** the title, the type, the address, and the address of its Markdown version; the description, or "missing", with its length against the limit; the review date, due in so many days or overdue since when, from the field `review-overdue` reads; the page's availability, and the builds that publish it and leave it out; its listing (listed, unquoted, or unlisted) and the rule it came from, which answers why a page isn't in the sitemap or `llms.txt`; its permalinks, the page's and its headings', which are what a product links to and what a rename must keep; and the template it was started from, with the placeholders left.
- **Health:** the problems on the page by severity, which open the Problems panel filtered to it; its size against `page-size`'s limit in its largest build, before it's over; and its tested examples, each tested, stale, or not tested.
- **Connections:** linked from, narrowed to the heading at the cursor and saying which; included by; and what the page includes, its fragments and its snippets with their files.
- **Memos,** from the [memos plan](../memos/phase-6-editor.md).
- **Review,** while review is on: whether the branch changes the page, how many changes it has, and its open threads.

Most of it comes from one new request, `ascribe/page`, answered by the server as `ascribe/inventory` is. It takes the page and the heading at the cursor in one call, since Connections narrows to the heading, and it's asked as the cursor moves, as Used by's request is today. So the parts that don't depend on the cursor, which are most of it, are answered from what the server keeps for the page, and a move within the page costs what Used by's request costs now. The problems are VS Code's own diagnostics, and the review section is what the review already has.

### Pages

The [navigation proposal](navigation.md#in-the-editor) gives Pages its structure: sections in order, dragging to reorder, a page not yet placed shown where it will appear, fragments by folder, and the page in the active editor revealed. Beyond that:

- **A page's context menu:** Open Preview to the Side, which is the one inline button; Copy Link, the page's address from the content root; Rename or Move, which makes the edits renaming its file in the Explorer does; Find All References; and Reveal in Explorer. A section's menu has **New Page Here**, which runs New Page with the folder chosen.
- **Problem colors:** a page with errors is red and one with warnings yellow, as in the Explorer. A tree item with its file as `resourceUri` gets the Explorer's decorations from the same providers, problems among them.
- **A page the current build leaves out is dimmed,** as the build lens dims text. The inventory says which builds publish each page, as This page needs too.
- **Orphans are a filter,** in the title bar, for pages and fragments, not a group.

### Content model

- **An entry's context menu:** Find All References; Rename, for a phrase or a dimension value; and **Insert at the Cursor**, for a phrase or a widget. Insert runs the actions bar's operation with the entry already chosen, into the last Markdown editor of the project, at its cursor. The extension keeps track of that editor, which nothing does today: VS Code's active editor is none while the preview or another webview has focus. Where the cursor can't take one, it says what the bar's hint says, and changes nothing.
- **No dragging an entry into a page.** That's a different drag from the navigation proposal's, which moves pages within Pages to reorder them, and stays.
- **"Unused only"**, a toggle in the title bar.
- **Page types,** and the versions once [versions](versions.md) ships, since `ascribe model` reports both and the view doesn't.

### Review

While review is on, a **Review** view lists the pages the branch changes:

- **Grouped as Added, Changed, and Removed,** each page with its count of changes and of open threads. A page changed only through something else says what, such as `_fragments/requirements.md` or `ascribe.toml`, as Changed Pages does.
- **Choosing a page** opens it and its preview at its first change.
- **A badge** counts the unsent comments, so they're seen with the preview closed. It's a view badge, a stable API the sidebar hasn't used yet.
- **Its title bar** has Submit Review…, Stop Review, and **Open Source Changes**, which opens VS Code's multi-file diff (`vscode.changes`) of the source files the branch changed. Each page's item has a button for its own files' diff. The preview shows what a change reads as; this shows the Markdown that made it.
- **Problems in changed pages,** a section of the view: the problems on the pages the branch changes, and nothing else.

The changed pages are the server's, from the base `ascribe/review/setBase` set, so the view, its problems, and `ascribe diff` never disagree. **Changed Pages** stays, for the keyboard and the preview's button.

The diff's base side needs the base's text. It comes first from VS Code's Git extension, whose API turns a file and a revision into a URI (`toGitUri`): the extension is built in and on by default, the diff editor gets the file's language from the URI, and the revision is the one `setBase` named. Review doesn't use the Git extension today, since it runs `git` itself, so this adds a dependency on it. Where the Git extension is off, the server serves the base's files as documents of its own, read through `GitFs` as `ascribe diff` reads them.

## In the editor

- **"Shown in 3 pages",** a CodeLens in a code file, above a region or a file a `@snippet` shows. Choosing it lists the pages. It's for an engineer changing code the docs show, before the change reaches them. The server registers the lens for its sources' folders itself, by the protocol's dynamic registration, since the client doesn't send it code files. **The lens is placed from the saved file:** the server knows a region's lines from disk, as a build does, so while lines are added above a region the lens sits where the region was until the file is saved. Sending the server the open code files under the sources too would keep it in place, but would have the server follow every edit to every code file opened under a source, which for a source rooted at the repository is every file an engineer opens.
- **`source.fixAll.ascribe`:** every safe fix for the file's problems, the set `ascribe check --fix` applies to it. It's in the Source Action menu and at the top of the actions bar's fixes, as **Apply the safe fixes in this page**, and on save only for someone who sets `"editor.codeActionsOnSave": { "source.fixAll.ascribe": "explicit" }`. Every editor with a language client gets it.
- **Drop and paste stay VS Code's.** The plan checks that what the Markdown extension writes passes `ascribe check`: its relative paths, its links to files, and its images' alt text. If it does, the editor's page says how to set `markdown.copyFiles.destination`. If it doesn't, Ascribe adds a drop handler that asks for the alt text, as **Insert an image** does.

## For agents

The extension ships the skill in itself, through `chatSkills`, so VS Code's chat has it in a project that never ran `ascribe agents sync`:

- **Only where there's no synced copy chat can see:** `when: ascribe.active && !ascribe.agentsSynced`. The key is set when the synced skill is in a folder VS Code's chat loads skills from, so chat never has two copies and never has none. `sync` writes to `.agents/skills/ascribe/` and `.claude/skills/ascribe/`; the plan checks which folders chat reads, and sets the key only for a copy in one of them.
- **The skill, and not the rules.** `SKILL.md` is the same for every project; the rules are made from a project's content model, so they can't be a file in the extension. They stay with `sync` and the MCP server's `ascribe://instructions/{+project}`.
- **It writes nothing,** so the editor still sets up no agent files.

The skill is the extension's version. A project using another `ascribe` gets a skill that may be a version apart until it syncs its own. `chatSkills` takes a `path` to a `SKILL.md` and an optional `when`, which is all this needs.

## What it touches

| Where | What |
|---|---|
| `ascribe-lsp` | `ascribe/page`; each page's builds in `ascribe/inventory`; `source.fixAll.ascribe`; the lens in code files and its registration; the base's text for the source diff, if the server serves it |
| `ascribe-cli` | The new request's types in `src/shapes.rs`, for the extension's generated types |
| `packages/vscode` | The views, Switch Project and its context key, the context menus and filters, the Review view, `chatSkills`, and the tests that went through the Projects view's API, the multi-project ones on Windows among them |
| The docs | The editor's page (the sidebar, the status bar, the settings), review's how-to, the agents page, and `changes/` |
| `project-docs/surface.md` | The commands and views, regenerated |

No output, command's output, JSON report, or part of the language changes. The extension's commands do: Switch Project is new, and the surface map shows it.

## What this leaves out

What other extensions built and took back, or that their users complained of:

- **A dashboard in a webview.** A finding with a place in the source goes to the Problems panel.
- **A graph view.**
- **Webview trees and webview views,** until the question below is taken up.
- **A custom editor as the default for `.md`.** An editable preview makes each edit in the source, as the [brainstorm](../brainstorm.md) has it.
- **A count lens on headings.** This page answers it at the cursor.
- **Gutter icons, and inlay hints with no length limit.**
- **A preview that opens, reveals, or focuses itself.**
- **A second view container,** or one in the Secondary Side Bar beside Chat.
- **A language status item.** The status bar item already shows everything it would.
- **Document symbols from the server.** VS Code would show a second Outline beside the Markdown extension's, and nobody in another editor has asked for them.
- **A chat participant.** The MCP server and the tools cover it.
- **A notification without "Don't show again".**
- **An account or a network service** for any part of review or the reports, beyond GitHub for comments.
- **External links' status in This page.** Only the report opens `.ascribe/history/` (decision 125).
- **Word counts and reading times.** No check uses them.

## Later

- **Whether a view should be a webview,** This page first: a card's layout against a tree's theming, keyboard, and cost.

## Settled

Answered by the maintainer on 16 October 2026, while this proposal was drafted.

1. **The Projects view goes, and Switch Project appears only with two or more projects.** _Why:_ more than one project is supported, but it isn't the common case, and the status bar has the one project already.
2. **The sidebar follows the open file, after a switch too.** _Why:_ a pin would be state nothing on screen shows.
3. **Used by becomes This page, and widens to what Ascribe knows about the page.** _Why:_ it's the one view about where the author is, and memos need a home.
4. **This page is a native tree for now.** _Why:_ its rows are labels and descriptions; the webview question is kept for later.
5. **Insert at the Cursor is for phrases and widgets, and there's no dragging an entry from Content model into a page.** _Why:_ they're the two kinds inserted by name; a drag does what the menu does, for more work.
6. **The Review view has the problems in changed pages.** _Why:_ that's the whole of a "changed by this branch" filter, in the place a reviewer already is.
7. **No language status item, no count lens, and no drop handler of Ascribe's own.** _Why:_ each repeats something that's already shown or already works.
8. **The skill ships in the extension, where there's no synced copy.** _Why:_ it writes nothing, and a project that never ran `sync` gets it.
9. **No document symbols.** _Why:_ above.
