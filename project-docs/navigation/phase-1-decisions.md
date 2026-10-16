# Phase 1: Decisions

Part of [Navigation](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Ten choices are made before anything ships, because each goes into a contract: the site and JSON outputs, `@ascribed/astro`'s API, `llms.txt`, the diagnostics, `ascribe model`'s JSON, the frontmatter, what the file can express, what a check means, what a setting another plan shipped means, and the docs' structure.

The thread can't make them as it goes. It may narrow what it handles. It may not ship a shape the plan expects to change.

## Context

- The [proposal](../proposals/navigation.md), whole, and the README's decisions 1 to 17.
- `docs/content/contracts/output-layout.md` §1.1 and §1.3: `_ascribe/` and the page records, `_ascribe/pages.json`.
- `packages/astro/src/content.ts` (`sidebar`, `pageRecord`) and `src/nav.ts` (`NavGroup`, `navProblems`, `sidebarGroups`): the published API a site calls today.
- `crates/ascribe-resolve/src/llms.rs`: the sections, `TOP_SECTION`, and the order within a section.
- `crates/ascribe-query/src/model.rs`: `ascribe model`'s JSON and its `schema_version`.
- `docs/content/checks/diagnostics.md`, "What to do next", for the kinds of next step.
- SPEC §2.1, the reserved frontmatter keys.
- `site/src/nav.ts`'s `checks/` group: `check.md`, then `diagnostics.md` and its four parts in `checks/diagnostics/`, then `vale.md` and `report.md`.
- The content model reference's `listing` table (`docs/content/content-model/index.md`), and the search and answers plan's [decision 15](https://github.com/ascribed-dev/ascribe/blob/5b22286fd2f370e796219e68a7c9d7cdb2ba8acc/project-docs/search-and-answers/README.md#decisions), which made `unlisted` about crawlers and agents.
- How each output carries a title both ways (`docs/content/content-model/index.md`, "Code in a field"): plain text, and the formatted form under `formatted`, HTML in the site output and a list of text and code pieces in the JSON output.

## The decisions

Each has a recommendation. The maintainer may take it, change it, or ask for more.

### 1. How the navigation reaches a site, and the JSON output

- **A. A file of its own, `_ascribe/nav.json`,** in each site output root, beside `pages.json`: the root section, each section `{ folder, label, page?, pages, sections }`, and each page `{ source, label, type }`, where `source` is the key `pages.json` already uses for the page. A section's and a page's label are there both ways, as a title is: `label` as plain text, and `formatted.label` in the output's formatted form, so a sidebar keeps a title's code spans. The JSON output gets the same file at the same path.
- **B. In `pages.json`,** each record gaining its section and its place.
- **C. Only in the JSON output,** for a site to read from there.

**Recommended: A.** It's a tree, and the records are a table; a site that wants the order reads one file, and one that wants a page's details joins on `source`, as `sidebar()` does now with the collection. Routes aren't repeated: a page's route is its record's. A versioned build's lines each have their own, in their own roots.

### 2. `sidebar()`'s signature, and previous and next

- **A. `sidebar(entries, options)`,** reading the project's navigation, with a site's own list as an option, `sidebar(entries, { nav })`, which overrides it. A site that calls `sidebar(nav, entries)` today moves its list into the options.
- **B. Both forms, told apart by the first argument.** A list of groups and a collection are both arrays, and an empty collection is either.
- **C. A new function, `projectSidebar(entries, options)`,** leaving `sidebar()` as it is.

With each, a new **`neighbors(entry, entries, options)`** gives a page's previous and next, `{ previous?, next? }`, each an entry and its address.

**Recommended: A.** One signature, no test of an argument's shape, and it reads as what it is: the site's list overriding the project's. Before 1.0, a site that passes a list changes one call, and the Astro reference and the changelog say how. `SidebarGroup` gains `sections` for nesting; a site that draws one level ignores it.

### 3. `llms.txt`'s order in a project with no `ascribe.nav`

The default order (decision 8) isn't `llms.txt`'s today: inside a section, `llms.txt` goes by path, and the default by type, then title.

- **A. `llms.txt` follows the navigation always,** the default order included. Every project with `[consumer] agents = true` sees its sections' pages reordered, once, with "Start" for "Pages" (decision 5), as one accepted output change.
- **B. `llms.txt` keeps path order until a project has `ascribe.nav`.**

**Recommended: A.** One order from one place is the proposal's point, and B is two orders a reader can tell apart. Decision 5 already changes every such `llms.txt`.

### 4. The diagnostics

| Name | Severity | Next | At |
|---|---|---|---|
| `nav-invalid` | Error | `write` | `ascribe.nav`: not TOML, a missing or unknown `version`, an unknown key, a value of the wrong kind |
| `nav-page-unknown` | Error | `choose` | A `pages` entry that isn't a page of that folder, or has a `/`; an unsafe fix to the closest name |
| `nav-folder-unknown` | Error | `choose` | A `folder` with no pages; an unsafe fix to the closest folder |
| `nav-duplicate` | Error | `fix` | A folder or a page named twice, at the second; a safe fix that removes it |
| `section-page-both` | Error | `choose` | A folder with an `index.md` and a page of its name beside it, at the second, on the page |
| `nav-page-unplaced` | Advice | `fix` | A page not listed, in a section that lists pages; a safe fix that appends it |
| `nav-section-unplaced` | Advice | `fix` | A section not listed, in a file that lists sections; a safe fix that appends it |
| `nav-label-long` | Advice | `write` | A `nav-label` longer than the title it shortens |

`nav-page-unplaced` is reported on the page, so an agent that wrote it sees it there, and its fix edits `ascribe.nav`. **Recommended:** as the table has it.

### 5. `ascribe model`'s JSON

- **A. A `navigation` key, the same tree as `nav.json`,** with each section's `listed` (in the file or not), added without moving `schema_version`.
- **B. A separate command, `ascribe nav`.**

**Recommended: A.** It's what the proposal says (`ascribe model` reports the sections and their order), and a key added moves no version.

### 6. `nav-label`

A reserved frontmatter key, a string of one line, with code spans allowed as `title`'s are. It's used wherever a page is an entry in a list: a sidebar, previous and next, and the editor's Pages view. `llms.txt` keeps the title, since it's a page's name for an agent, not a menu's. A type can't declare a field `nav-label` (`model-field-reserved`), and a fragment can't use it.

**Recommended:** as written.

### 7. Where a nested section sits, and reading order

Ascribe's site puts `checks/diagnostics.md` and its four parts, the nested section `checks/diagnostics/`, between `check.md` and `vale.md`. If a section's nested sections always come after its pages, no `ascribe.nav` can say that, and phase 6's sidebar can't be the same as today's.

- **A. A nested section whose own page is beside its folder sits where that page sits** among the parent's pages: by the default order, by its page's type and title, or by its place in the parent's `pages`. Naming `diagnostics.md` in `checks/`'s list places the subsection there; that's a page of `checks/`, and isn't in `checks/diagnostics/`'s list, so decision 4 of the README holds. A nested section with an `index.md` and no page beside it comes after the parent's pages, in the order of the parent's nested sections.
- **B. Nested sections after the pages, always,** and Ascribe's site changes order.

**Recommended: A.** It's what splitting a page into a folder means (decision #394): the parts stay where the page was. It changes phase 2's ordering and README decision 9's meaning a little, and nothing else.

**Reading order,** for previous and next, the archive, and `llms.txt`'s folding of nested sections, follows the same placement, depth first: a section's own page, then its entries in order, a nested section's own page followed by its pages and nested sections where it sits. The root's own page, the home page, has no previous.

### 8. Where the docs live

**Recommended:** a feature folder, `docs/content/navigation/`, with the reference for `ascribe.nav` and `nav-label` as its `index.md` and, in phase 6, a how-to; the contract as `docs/content/contracts/navigation.md`, beside `permalinks-list.md`. `docs/content/command/project-files.md` lists the file, and the Astro reference documents `sidebar()` and `neighbors()`.

### 9. What `page-orphan` reports

Today it reports a page nothing links to or includes, other than an `index.md`, and hedges that a menu may reach it. With a navigation, every published page is in its folder's section unless it's `unlisted`, so "in no navigation" means unlisted. A check on unlinked and unlisted pages is narrower than today's, and a page only the sidebar reaches is still worth knowing about.

- **A. `page-orphan` keeps what it reports,** a page nothing links to or includes, and its message says what reaches it: "only the sidebar" for a page in the navigation, and "nothing but its address" for one that's `unlisted`.
- **B. `page-orphan` reports only an unlinked, unlisted page,** and a page only the sidebar reaches is reported by nothing.
- **C. Two checks:** `page-orphan` for B's case, and a new advice, `page-unlinked`, for a page only the sidebar reaches.

**Recommended: A.** It keeps what's useful about the check, a page prose never points to, drops the hedge with a fact in its place, and adds no name.

### 10. `unlisted` in the sidebar

README decision 11 leaves a page out of the navigation by `listing: unlisted`. That widens a setting the search and answers plan shipped (its decision 15), which was about crawlers and agents, to something readers see.

**Recommended:** record it as an amendment of that decision, in a decision file that names it, and change the content model reference's `listing` table in phase 2, when the sidebar first follows it.

## Tasks

1. A decision file for each, in `project-docs/decisions/`, numbered by this plan's issue, as decided.
2. The README's decisions list filled in.

## Out of scope

Everything that builds.

## Acceptance criteria

- Each decision is recorded, with its _Why_, or sent back for another option.

## Commits

1. "Record navigation's contracts"
