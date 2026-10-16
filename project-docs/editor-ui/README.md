# The editor's views

The sidebar says what needs attention where the author is: this page's problems and connections, its review, and what a branch changed. Everything else stays where VS Code users already look. The idea, and the research behind it, are in the [proposal](../proposals/editor-ui.md). This is the plan for building it.

## Who it's for

- **A writer,** who wants to know, without leaving the page, whether it's published, listed, linked, overdue, or over size, and to put a phrase in from the list of them.
- **An engineer documenting their own code,** who changes a file a page shows and doesn't know which pages will change.
- **A reviewer,** who works through a branch's changed pages and today reopens a quick pick for each one.
- **A maintainer of a repository with several projects,** who needs to switch between them, and a team with one project, which doesn't.
- **An agent in VS Code's chat,** which has Ascribe's skill only where someone ran `ascribe agents sync`.

## What exists today

- **Four tree views** in one container, `ascribe` (`packages/vscode/package.json`): **Projects** (`src/ui/projectsView.ts`), and **Used by**, **Pages**, and **Content model** (`src/ui/sidebar.ts`, `src/ui/sidebarViews.ts`). Projects is the one view without `when: ascribe.active`, so it holds the two welcome messages.
- **Used by asks twice for each cursor move:** `ascribe/context` for the heading at the cursor, then `textDocument/references` at it (`sidebarViews.ts`, `askUsedBy`).
- **Pages and Content model are drawn from `ascribe/inventory`** (`crates/ascribe-lsp/src/inventory.rs`), which answers from the server's snapshot, unsaved edits included. Its result type goes through `crates/ascribe-cli/src/shapes.rs` to the extension's generated `src/shapes.ts`.
- **The extension's test API** (`AscribeApi.ui` in `src/extension.ts`) has `projects`, the Projects view's, and `sidebar`, the other three's. The integration tests `monorepo.it.ts` and `quill.it.ts` and the unit test `sidebar.test.ts` go through them; the multi-project ones run on Windows too.
- **The status bar item** shows the project, the build, the server's state, the build lens, and the Problems panel's coverage, with the folder, `ascribe.toml`, and the binary in its tooltip, and a menu (`src/ui/statusBar.ts`, `describe.ts`).
- **Review** is the preview's, with the status item, a comment controller on source lines, and **Changed Pages** (`ascribe.changedPages`), a quick pick. `ascribe.reviewOn` is set while it's on (`src/preview/review.ts`). Review runs `git` itself (`src/preview/threads.ts`) and doesn't use VS Code's Git extension. The server reads the base for `ascribe diff` through `GitFs` (`crates/ascribe-diff/src/gitfs.rs`).
- **The actions bar** (`ascribe.actions`) has **Insert a phrase** and **Insert a widget** (`src/actions/registry.ts`).
- **The language server** has one CodeLens, on `@include`, and quick fixes one at a time (`crates/ascribe-lsp/src/code_action.rs`). `ascribe check --fix` applies every `safe` fix. The client sends the server Markdown files and Ascribe's TOML files only (`src/client.ts`).
- **Pages are the `markdown` language,** so VS Code's Markdown extension gives them an Outline, folding, and drop and paste of files and images.
- **`ascribe agents sync`** writes the skill to `.agents/skills/ascribe/` and `.claude/skills/ascribe/`. The extension writes no agent files.
- **The docs** describe the four views in `docs/content/editor/index.md` ("The Ascribe sidebar"), the walkthrough's sidebar step in `packages/vscode/media/walkthrough/sidebar.md`, and the extension's README in `packages/vscode/README.md`. Review's how-to is `docs/content/review/index.md`, several projects are in `docs/content/editor/settings.md`, and agents in VS Code in `docs/content/agents/index.md`.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

From the proposal's Settled list, answered by the maintainer on 16 October 2026:

1. **The Projects view goes.** **Ascribe: Switch Project** replaces it, shown only while the workspace has two or more projects.
2. **The sidebar follows the open file,** after a switch too. A switch is a jump, not a pin.
3. **Used by becomes This page,** and widens to what Ascribe knows about the page: About, Health, Connections, Memos, and Review.
4. **This page is a native tree.** Whether any view should be a webview is a later question, outside this plan.
5. **Content model's Insert at the Cursor is for phrases and widgets,** and nothing is dragged from Content model into a page. (Dragging pages within Pages, from the navigation proposal, is a different drag, and not this plan's.)
6. **The Review view has a section of the problems in changed pages.** It's the whole of a "changed by this branch" filter.
7. **No language status item, no count lens on headings, and no drop or paste handler of Ascribe's own.**
8. **The skill ships in the extension,** only where chat can't see a synced copy.
9. **No document symbols** from the server.

From the proposal's design and its review:

10. **The views, in order:** Review while review is on, This page, Pages, and Content model. Pages is always shown, so the container is there in a workspace with no project, with the welcome messages.
11. **This page is answered by one request, `ascribe/page`,** which takes the page and the heading at the cursor. The parts that don't depend on the cursor are answered from what the server keeps for the page, so a cursor move costs what Used by's two requests cost now. The problems are VS Code's diagnostics.
12. **The Review view's changed pages, and its problems, are the server's,** from the base `ascribe/review/setBase` set, so they never disagree with `ascribe diff`.
13. **The base side of Open Source Changes comes from the Git extension first** (its API's `toGitUri`), and from documents the server serves through `GitFs` where the Git extension is off.
14. **"Shown in N pages" is placed from the saved file.** The server registers the lens for its sources' folders by dynamic registration, and reads the regions from disk, as a build does. The client doesn't send it code files. **A code file in more than one project's sources gets a lens from each,** each counting its own project's pages and naming the project ("docs: shown in 3 pages"); a file one project shows gets one lens, unnamed. The pages are in different projects, built and published apart, so one count across them would add things a writer can't act on together.
15. **`source.fixAll.ascribe` is every safe fix for the file's problems,** the set `ascribe check --fix` applies to it, and runs on save only where someone sets it in `editor.codeActionsOnSave`.
16. **The skill's key, `ascribe.agentsSynced`, is set only for a copy in a folder VS Code's chat loads skills from:** `.github/skills/`, `.claude/skills/`, or `.agents/skills/` in a workspace folder, which takes in both of `sync`'s. The rules aren't shipped: they're made from a project's model.
17. **Insert at the Cursor writes into the project's last Markdown editor,** which the extension tracks, since VS Code's active editor is none while a webview has focus.

No decision phase comes first: nothing in this plan changes the language, an output, a command's JSON, or a published package's API. `ascribe/page`'s result is a type the extension generates from the server's, like `ascribe/inventory`'s, and can change with both.

## The thread

**The use case:** in `examples/quill`, with `docs/keys.md` open and the cursor under "Rotate keys", the **This page** view shows the page's title, type, address, and Markdown version's address, and says that `install-agent.md` links to "Rotate keys".

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The language server | `crates/ascribe-lsp` | `ascribe/page`, with the page's identity and its connections at the cursor, and a store of what doesn't depend on the cursor |
| The shapes | `crates/ascribe-cli/src/shapes.rs` | The result type, and the extension's generated types |
| The extension | `packages/vscode` | This page, replacing Used by, and its test API |
| The docs | `docs/`, the walkthrough, the README | The view described |

**The seam it tests** is one request per cursor move that answers more than Used by's does, from a store the server keeps current as the page changes. If the store can't be kept current without resolving the page on every keystroke, or a cursor move is slower than Used by's two requests, the design of decision 11 changes: This page splits its cursor part from the rest, as two requests, before any section is added.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| The Review view, and This page's Review section | 2 |
| The lens in code files, and the fix-all action | 3 |
| The rest of About, Health, and what the page includes; each page's builds in `ascribe/inventory`, which phase 5's dimming reads | 4 |
| Pages' and Content model's actions, filters, colors, and dimming | 5 |
| Removing Projects, and Switch Project | 6 |
| The skill in chat, and the drop and paste check | 7 |
| Memos in This page | The [memos plan](../memos/phase-6-editor.md) |

## Phases

| Phase | Result |
|---|---|
| [1: The thread](phase-1-thread.md) | **This page** replaces Used by: the page's title, type, and addresses, and what links to the heading at the cursor, from one request. |
| [2: Review](phase-2-review.md) | While review is on, a Review view lists the changed pages with their counts, their problems, and a badge for unsent comments, and opens the branch's source changes in VS Code's diff. |
| [3: In the editor](phase-3-in-the-editor.md) | A code file a page shows says which pages show it, and the safe fixes for a page are one action, or one save. |
| [4: This page, whole](phase-4-this-page.md) | This page says whether the page is published, listed, overdue, over size, or unfinished, and what it includes. |
| [5: Pages and Content model](phase-5-pages-and-model.md) | Pages and the content model's entries have menus and filters, pages show their problems and the build's reach, and a phrase or a widget goes in from the list. |
| [6: No Projects view](phase-6-projects.md) | The Projects view is gone; a workspace with several projects switches with **Switch Project**. |
| [7: Chat, drop, and paste](phase-7-chat.md) | VS Code's chat has Ascribe's skill in a project that never synced it, and the docs say how dropped images land. |

Phase 1 is the thread, through the riskiest seam. After it, the order is by risk. Phase 2 meets two seams nothing in the extension uses yet, the Git extension's API and the multi-file diff; phase 3 a server answering for code files. Phase 4 then fills the view the thread made, and phase 5 uses the data phase 4 adds, each page's builds. Phase 6 is low risk and high value, and waits only so its test move happens once the other views' APIs have settled. Phase 7 is the smallest.

Each phase's result is something a person can open or run. None builds a layer for a later phase to use.

## Carved out, and waiting

- **Whether any view should be a webview,** This page first. A question for after this plan, with the views in use.
- **External links' status in This page.** Only the report opens `.ascribe/history/` (decision 125).
- **Word counts and reading times.**
- **Document symbols for other editors,** until someone using one asks.

## The other plans

- **[Memos](../memos/README.md),** phase 6, lists memos in Used by. Built after this plan's phase 1, it fills This page's Memos section instead; built before, phase 1 keeps them when Used by becomes This page.
- **[Glossary pages](../glossary-pages/README.md)** moves terms onto pages. Content model's glossary entries then open their headings, and their Find All References and Rename are the glossary plan's.
- **[Navigation](../navigation/README.md)** gives Pages its structure, in its phase 5: sections in order, dragging to reorder, fragments by folder. This plan's phase 5 adds to whatever structure Pages has when it lands, by content type today.
- **[Versions](../versions/README.md).** Content model lists the versions once `ascribe model` reports them; phase 5 adds them if they're there.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **No output changes.** Every phase runs `node scripts/compare/outputs.ts --base main` and shows every project byte for byte the same.
- **A new command, action, or setting** goes through "A new editor command, action, or setting" in `project-docs/checklists.md`: its title, its category, and the generated tables. The docs' tables are rewritten by `ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts`; read the diff.
- **A new request's types** go through `crates/ascribe-cli/src/shapes.rs`, rewritten by `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`.
- **Project files are read through `ascribe_resolve::FileSystem`.** Libraries don't print, and don't panic on input.
- **The extension's views and commands have tests** through its test API, unit tests where the logic is the extension's, and integration tests, which run in CI only, where VS Code's behavior is.
- **The map of the surface** is regenerated when a phase changes it (`node scripts/surface/inventory.ts`), and its diff is read against the conventions.
- **User-visible changes** update their page under `docs/content/`, marked `@available: next` where they're new, and add an entry in `changes/editor/` (or `changes/language-server/`, `changes/review/`, `changes/agents/`) in the same phase. Each phase file names its pages. The walkthrough and the extension's README change where they describe what changed.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- The webview question in decision 4.
- A drop handler that asks for alt text, if phase 7's check finds VS Code's drop doesn't pass `ascribe check`.
