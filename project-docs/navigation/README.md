# Navigation

**This plan is finished:** its last phase merged in [#588](https://github.com/ascribed-dev/ascribe/pull/588). Its decisions in force are files in [decisions/](../decisions/README.md): decisions [#537a](../decisions/537a-navigation-file.md) to [#537d](../decisions/537d-unlisted-sidebar.md), [#539a](../decisions/539a-llms-follows-navigation.md) to [#539d](../decisions/539d-neighbors.md), [#578](../decisions/578-hook-safe-fixes.md), [#585](../decisions/585-folder-label.md), [#594](../decisions/594-plain-text-labels.md), and [#541a](../decisions/541a-navigation-problems.md) to [#541g](../decisions/541g-navigation-docs.md). This README is the record of what was intended; the code, SPEC §7.8, and those decisions say what is.

A project says what order its sections and pages come in, once, in `ascribe.nav`, and everything that needs a sidebar or a table of contents takes it from there: a site, `llms.txt`, an archive, `ascribe agents index`, and the editor, where pages are put in order by dragging them. The idea is in the [proposal](../proposals/navigation.md). This is the plan for building it.

## Who it's for

- **A writer** who wants a page earlier in its section, and today edits a TypeScript list in the site's code.
- **A site's maintainer,** whose `src/nav.ts` fails the build each time a writer adds a page, until someone adds it to the list.
- **An agent** adding a page, which can't know a site's list exists, and is told by nothing that the page is missing from it.
- **A reader of `llms.txt` or an archive,** who gets pages in path order, since nothing else is known.

## What exists today

The proposal's three prerequisites have landed:

- **A page declares its type,** `type: how-to` in its frontmatter, and `[types.<name>]` has no `files` or `default` (decision [#450](../decisions/450-page-type-key.md)). The source index reads it once, and every check, output, and editor feature reads it from there.
- **The starter set gives no type a folder.** `ascribe templates eject` writes no `files` (the same decision).
- **Ascribe's docs are grouped by feature** (decision [#452](../decisions/452-docs-by-feature.md)): `review/`, `editor/`, `checks/`, `writing/`, and the rest, each with every type of page about it, and `contracts/` as one section.

What reads an order, each in its own way:

- **Ascribe's site** lists every page by hand in `site/src/nav.ts`, a group for each folder of `docs/content/`, the folder's `index.md` first, then by type in `docs/ascribe.toml`'s order, then by title, with a page's parts after it; contracts keep an order of their own. `site/test/nav.test.ts` fails on a page left out, one that doesn't exist, or one named twice. That's the proposal's default order, kept by hand.
- **`@ascribed/astro`:** `sidebar(nav, entries, options)` (`packages/astro/src/content.ts`) joins a site's list to the collection, and throws on a page the list leaves out (`navProblems` and `sidebarGroups` in `src/nav.ts`). With a versioned build, it shows the line the page is on. `npx @ascribed/astro init` writes a site's first `src/nav.ts`, the root under **Start** and a group for each folder (`src/init.ts`). `examples/astro-site` has one.
- **`llms.txt`** (`crates/ascribe-resolve/src/llms.rs`) makes a section of each folder, named by its `index.md`'s title, with the root's pages first under `TOP_SECTION`, "Pages", and splits into `<folder>/llms.txt` when it's long. `ascribe agents index` (`crates/ascribe-query/src/agents_index.rs`) has the same sections.
- **The archive** (`crates/ascribe-emit/src/archive/mod.rs`, `Nav`) writes a sidebar and a contents page "in reading order: the folders in path order, the line's root first, and in each its `index` page, then the rest by path".
- **The site output's generated files** are under `_ascribe/` in each build's root (`docs/content/contracts/output-layout.md`): the Zod schema, `permalinks.json`, `pages.json`, and `headers.json`.
- **`ascribe.permalinks`** is a TOML file beside `ascribe.toml`, loaded by `crates/ascribe-model/src/permalinks.rs`, checked in `crates/ascribe-check/src/permalinks.rs`, and sent to the language server by the client (`packages/vscode/src/client.ts`). It's the pattern `ascribe.nav` follows.
- **`page-orphan`** reports a page nothing links to or includes, other than an `index.md`, and hedges: a menu may still reach it. The editor's Pages view lists such pages under Orphans, by content type otherwise (`packages/vscode/src/ui/sidebar.ts`).
- **Renaming a page** rewrites links to it, and the names of it in `ascribe.toml`, from `ascribe rename` and the editor (`crates/ascribe-lsp/src/rename.rs`).

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

From the proposal's Settled list, answered by the maintainer on 12 October 2026:

1. **The file is `ascribe.nav`:** TOML, beside `ascribe.toml`. One `filenames` entry gives it, `ascribe.permalinks`, and `ascribe.lock` VS Code's `toml` language id. VS Code has no TOML grammar of its own, so they're colored only where a TOML extension is installed, as `ascribe.toml` is today.
2. **A page that isn't listed is advice,** in a section that lists pages, and its fix, appending the page, is safe. A section with no `pages` reports nothing.
3. **Sections nest as folders do,** with no cap and nothing reported. The outputs carry the tree. `llms.txt` keeps one level, a section for each top folder, with nested sections folded into their parent in navigation order.
4. **A section's own page is its `index.md`, or the page beside the folder with the folder's name.** It's the section's `page`, never in `pages`, always first, and its title is the section's default label. The root's `index.md` is the home page.
5. **The root section's label is "Start"** unless `label` says otherwise, and `llms.txt` takes the same word in place of "Pages", for every project with a plain output: an output change accepted with the change, named file by file.
6. **A section is a folder, and a folder is a feature.** A page can't be in another section; to move it, move the file.
7. **A page beside a folder of its name is that section's own page.** A folder with both is an error.
8. **A section's default order is its own page, then by type in `ascribe.toml`'s order, then by title,** and a site is given each page's type.

From the proposal's design:

9. **The file says order and labels, nothing else:** `version = 1`, and `[[sections]]` with `folder` (a path from the content root ending in `/`, `""` for the root, each at most once), `pages` (file names in that folder, in order), and `label`. A nested section is its own entry. Where it sits in its parent is phase 1's decision 7.
10. **What the file doesn't say has a default:** a section not listed comes after the listed ones in path order; a page not listed comes after its section's listed pages, by title; a section with no `pages` has the default order.
11. **A page's own label is `nav-label`,** a reserved frontmatter key. A page left out is `listing: unlisted`; there's no second setting. This widens `unlisted`: the search and answers plan's [decision 15](https://github.com/ascribed-dev/ascribe/blob/5b22286fd2f370e796219e68a7c9d7cdb2ba8acc/project-docs/search-and-answers/README.md#decisions) made it about crawlers and agents, a page not offered, and this adds the sidebar, which readers see. Phase 1's decision 10 records the change.
12. **One navigation for a project,** and each build shows its share: a page the build doesn't publish isn't in its navigation, nor is a section with no published pages.
13. **Folders Ascribe leaves out stay out:** fragments, and names starting with `_`. A folder with no pages, at any depth, isn't a section.
14. **`page-orphan` stops hedging** about menus Ascribe can't see. What it reports once the navigation is known is phase 1's decision 9.
15. **The editor's Pages view shows the navigation,** and dragging there edits `ascribe.nav` as any edit is made. Dragging a page onto another section is a file move, asked first. Renaming or moving a page updates `ascribe.nav`.
16. **The starter stops writing `src/nav.ts`.** A site with one keeps working: `sidebar()` still takes a list.
17. **Left out:** a page in two sections, a section for a content type, an entry that isn't a page, an order for each build, weights in frontmatter, routes, a table of contents inside a page, generated pages, and a version switcher.

From [phase 1](phase-1-decisions.md), which keeps the options each was chosen from:

18. **The navigation is a file of its own, `_ascribe/nav.json`,** in each site output root and in the JSON output, beside `pages.json`: the root section, each section `{ folder, label, page?, pages, sections }`, and each page `{ source, label, type }`, joined to the records on `source`. A label is there both ways, `label` plain and `formatted.label` in the output's formatted form. Routes aren't repeated. A versioned build's lines each have their own.
19. **`sidebar(entries, options)` reads the project's navigation, and a site's own list is an option, `{ nav }`, which overrides it.** `neighbors(entry, entries, options)` gives `{ previous?, next? }`. `SidebarGroup` gains `sections`. A published API changes, so the changelog says **Behavior change**.
20. **`llms.txt` follows the navigation always,** the default order included: one accepted output change for every project with a plain output, named file by file with decision 5's.
21. **The diagnostics:** `nav-invalid` (error, `write`); `nav-page-unknown` and `nav-folder-unknown` (errors, `choose`, an unsafe fix to the closest name); `nav-duplicate` (error, `fix`, a safe fix removing the second); `section-page-both` (error, `choose`, on the folder's `index.md`, naming the page beside the folder); `nav-page-unplaced` and `nav-section-unplaced` (advice, `fix`, a safe fix that appends); and `nav-label-long` (advice, `write`). `nav-page-unplaced` is reported on the page, and its fix edits `ascribe.nav`.
22. **`ascribe model` gains a `navigation` key,** the same tree as `nav.json` with each section's `listed`, and its `schema_version` doesn't move.
23. **`nav-label` is a reserved frontmatter key,** one line, code spans allowed as in `title`, used wherever a page is an entry in a list; `llms.txt` keeps the title. A type can't declare it and a fragment can't use it.
24. **A nested section whose own page is beside its folder sits where that page sits** among the parent's pages, below the content root only since decision [#610](../decisions/610-top-sections.md), by default or by its place in the parent's `pages`, so naming `diagnostics.md` in `checks/`'s list places the subsection there. One with an `index.md` and no page beside it comes after the parent's pages. Reading order follows the same placement, depth first, and the home page has no previous. Decision 9's `pages` may therefore name the page beside a nested folder.
25. **The docs live in a feature folder, `docs/content/navigation/`:** the reference as its `index.md`, a how-to in phase 6, the contract at `contracts/navigation.md`, the file in `command/project-files.md`, and `sidebar()` and `neighbors()` in the Astro reference.
26. **`page-orphan` keeps reporting a page no prose links to or includes,** and its message says what reaches the page: only the sidebar, or nothing but its address for one that's `unlisted`. Decision 14 reads so.
27. **`unlisted` leaving a page out of the navigation amends search and answers' decision 15,** which made the setting about crawlers and agents; the decision file names it, and the content model reference's `listing` table changes in phase 2.

## The thread

**The use case:** `examples/astro-site` adds an `ascribe.nav` that puts its Reference section before Guides, deletes its `src/nav.ts`, and calls `sidebar()` with the collection alone. `ascribe build`, then `astro build`, and the sidebar lists Start, Reference, Guides. Remove `ascribe.nav`, build again, and the sidebar has the default order, Start, Guides, Reference.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The language | `SPEC.md` §7 | `ascribe.nav`'s grammar |
| The content model | `ascribe-model` | Loading `ascribe.nav`, as `ascribe.permalinks` is loaded, and its errors |
| Resolve | `ascribe-resolve` | The navigation: the default order, the file's order over it, and a build's share |
| Emit | `ascribe-emit` | The navigation in the site output, beside the page records |
| `@ascribed/astro` | `packages/astro` | `sidebar()` from the project's navigation, without a list |
| The docs | `docs/`, `examples/astro-site` | The file's reference and contract, and `sidebar()`'s |

**The seam it tests** is the navigation the binary writes, joined by `@ascribed/astro` to Astro's collection: two programs agreeing on which page is which, through the routes and source paths `pages.json` already shares, for a folder with a space and a capital in its name ("Guides/My Setup.md"). It's a published shape on both sides, so phase 1 decides it first. If the join can't be made from what the output carries, the output's shape changes before anything else reads it.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| Every problem but a file that can't be read as `ascribe.nav`; the safe fix; `page-orphan`; the editor's diagnostics and completion in the file | 3 |
| The JSON output, `llms.txt`, the archive, `ascribe agents index`, `ascribe model`, `nav-label`, previous and next, and a versioned build's lines | 4 |
| The Pages view, dragging, and rename | 5 |
| Ascribe's own site, and the starter | 6 |

After phase 2, a `pages` entry that names no page is ignored, and the reference says so until phase 3 makes it an error. A misspelled name doing nothing and saying nothing is the thread's one silent case, so phases 2 and 3 should go out in one release: phase 3 lands before the release after phase 2's, or the release notes say it.

## Phases

| Phase | Result |
|---|---|
| [1: Decisions](phase-1-decisions.md) | The maintainer settles the ten contracts the phases ship. |
| [2: The thread](phase-2-thread.md) | An Astro site's sidebar comes from the project: the default order with no file, and `ascribe.nav`'s with one. |
| [3: The file, checked](phase-3-checks.md) | A mistake in `ascribe.nav` is an error where it's written, an unplaced page is advice with a safe fix, and an orphan says what reaches it. |
| [4: Every reader](phase-4-readers.md) | `llms.txt`, the archive, the JSON output, `ascribe agents index`, and `ascribe model` follow the navigation, with each page's short label and its previous and next. |
| [5: The editor](phase-5-editor.md) | The Pages view is the navigation, a page is put in order by dragging it, and a rename keeps `ascribe.nav` right. |
| [6: Ascribe's own site](phase-6-site.md) | Ascribe's site and every new site have no list of their own, and the docs have a guide. |

Phase 1 comes first because the thread ships a file format, an output, and a change to a published package's API. Phase 2 is the thread, through the seam between the binary and the Astro integration. After it, the order is by risk: phase 3 settles what the file may say before more readers depend on it; phase 4 changes `llms.txt` for every project, the widest output change; phase 5 is the editor, which needs the checks for its drags to land cleanly. Phase 6 moves Ascribe's own site last, once everything it needs is there.

Each phase's result is something a person or an agent can run or open.

## Carved out, and waiting

- **A section for a content type,** and a page in two sections (decision 17).
- **An entry that isn't a page:** an external link, a heading with no folder, a divider.
- **Generated pages' place in a section,** until something generates pages.
- **A site-wide selector,** in the [brainstorm](../brainstorm.md).

## The other plans

- **[Versions](../versions/README.md).** There's one `ascribe.nav` in the source. Each line is a build of its own, so its navigation is the file's order over the pages that line publishes. `sidebar()` already shows the line a page is on, and keeps doing so from each line's `nav.json`. The versions plan's archive takes its sidebar, contents, and previous and next from the navigation in phase 4.
- **[The editor's views](../editor-ui/README.md).** Its phase 5 adds menus, colors, dimming, and filters to Pages, in whatever structure Pages has. This plan's phase 5 changes the structure. Whichever lands second keeps the other's work: the editor plan's actions apply to sections as they did to type groups, and its New Page Here is a section's.
- **[Glossary pages](../glossary-pages/README.md).** A glossary page is a page in its folder's section.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **Outputs change only where a recorded decision says.** Every phase runs `node scripts/compare/outputs.ts --base main`, and each difference is named with `--accept`, file by file. Phase 2's are the site outputs' new navigation file; phase 4's are `llms.txt` and the archive's.
- **The language and the contracts change only by a recorded decision.** Go through the matching list in `project-docs/checklists.md` first, and record the decision in `project-docs/decisions/` in the same pull request.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case.
- **Behavior changes come with a conformance case** in `tests/conformance/cases/`.
- **Project files are read through `ascribe_resolve::FileSystem`,** `ascribe.nav` among them. Libraries don't print, and don't panic on input.
- **Paths, case, and links differ by platform.** A phase that matches `pages` and `folder` against files runs the full matrix from its branch (Actions → CI → Run workflow).
- **The map of the surface** is regenerated when a phase changes it (`node scripts/surface/inventory.ts`), and its diff is read against the conventions.
- **User-visible changes** update their page under `docs/content/`, marked `@available: next` where they're new, and add an entry in `changes/` in the same phase. Each phase file names its pages. A tested example the docs show is observed again (`cargo run -p ascribe-cli -- observe`).
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- Anything in decision 17.
- A command that writes `ascribe.nav` from a site's existing `src/nav.ts`. A site moves by hand, as Ascribe's does in phase 6.
