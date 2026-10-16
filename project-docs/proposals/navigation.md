# Proposal: navigation

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail.

## The goal

A project says what order its sections and pages come in, once, and everything that needs a sidebar or a table of contents gets it from there: the site, `llms.txt`, an archive, and the editor.

Putting pages in order is done by dragging them, in the editor.

## What exists

Ascribe's content model has no order and no grouping. Five things work around that, each in its own way:

- **Ascribe's own site** lists every page by hand in `site/src/nav.ts`: eight groups, one for each content type and two more (**Start** for the home page, **Contracts** for the reference pages kept apart), and a test that fails when a published page is missing, when the list names a page that doesn't exist, or when it names one twice (`site/test/nav.test.ts`). Since the docs took the page types, a group crosses folders: **Tutorials** holds `getting-started.md` from the root and `guides/variants.md`, and **Reference** holds `guides/editor.md`.
- **`npx @ascribed/astro init`** writes a site's first `src/nav.ts` from the project's pages, "the root's under **Start** and a group for each folder", once. From then on the site keeps it. `sidebar()`, in `@ascribed/astro/content`, joins that list to the collection. The comment at the top of `packages/astro/src/nav.ts` says the list is the site's because "Ascribe's content model has no page order or grouping yet".
- **`llms.txt`** makes a section of each folder, named by the title of its `index.md` when it has one, with the pages outside every folder first, under "Pages". It has no order to follow but paths.
- **The [versions plan](../versions/README.md)'s archive** has to write a sidebar and a contents page for itself, "from the folders, the pages' titles, and their headings, in path order", and says it will use navigation when the model has it.
- **The editor's Pages view** lists pages by content type, and says of its orphans: "Ascribe doesn't know your site's navigation, so an orphan may still be reachable from a menu: it's a hint, not an error."

Four of the five treat a folder as a section, and `[listing.folders]` sets rules by folder too. The fifth, Ascribe's own site, grouped by folder until the docs took the page types, and now groups by type across folders: the one sidebar in the repository that the rule below can't write as it stands.

That's settled, on the side of the rule (below, 6): a section is a folder, a folder is a feature, and every type of page about the feature is in it. Grouping by type was an oversight. So three things outside this proposal are out of line with it, and each is changed apart from it:

- **A page can't say its own type.** A type applies to a page by where its file is (`files`, or the default type), so a how-to and a reference page in one folder can only be told apart by listing files one by one, as `docs/ascribe.toml` does. Each page will declare its type in its frontmatter.
- **The starter set gives each type a folder.** `ascribe templates eject` writes `files = ["how-to/**"]`, which makes a new project's folders its types, and its sidebar the grouping this proposal rejects.
- **Ascribe's own folders are types under another name.** `guides/`, `reference/`, and `contracts/` split one feature four ways: review's how-to is in `guides/`, its troubleshooting in `guides/review/`, its command in `reference/cli/diff.md`, and its JSON in `contracts/json-reports/diff.md`. The docs are to be regrouped by feature.

The surface audit found the docs' own structure wanting (its finding 28) and left this proposal owed.

## The design

### A section is a folder

A project's sections are its content root's folders, and a page belongs to the folder it's in. A page can't appear in another section: to move it, move the file.

A folder is for a feature or a subject, not for a kind of page. A reader looking into review wants its tutorial, its how-to, its reference, and its troubleshooting in one place, and doesn't care which of those each page is. Grouping by content type ships the docs' own filing system to people who never asked about it. So a section mixes types, and a type is something a page in it has, not somewhere it lives.

That one rule is what keeps the rest small. Navigation never says where a page goes. It says only what order things come in, and what a section is called.

- The pages at the content root are a section too, the first.
- A folder inside a folder is a section inside a section.
- A folder with no pages of its own and no folders with pages isn't a section.
- Folders Ascribe already leaves out stay out: fragments, and names that start with `_`.

### With nothing written, there's still an order

A project that has never thought about navigation gets one: sections in path order, the root's pages first, and within a section its own page first, then the rest by type, in the order `ascribe.toml` declares its types, and by title within a type.

Ordering by type inside a section is what's left of grouping by type, and it's the useful part: a section reads tutorial, how-tos, concepts, reference, troubleshooting, when that's the order the types are declared in, without any of them being sent to another part of the site. A project with no types gets title order.

A section's label is the title of its own page, or the folder's name, with its first letter capitalized, when it has none. A page's label is its title.

**A section's own page** is one of two files, and a section has at most one:

- `index.md` in the folder; or
- a page beside the folder with the folder's name, as `reference/cli.md` is beside `reference/cli/`. That's how a long page is split into a folder of its parts, and Ascribe's docs do it six times.

A folder with both is an error at the second, since two pages would claim one section.

That's enough for a reference section forever, and enough for any project on its first day.

### A file for the order

When the order matters, it's written in a file beside `ascribe.toml`, `ascribe.nav`:

```toml
version = 1

[[sections]]
folder = ""
pages = ["getting-started.md"]

[[sections]]
folder = "guides/"
pages = ["astro.md", "editor.md", "variants.md", "review.md"]

[[sections]]
folder = "reference/"
```

- **`[[sections]]` is an array of tables,** so the order of sections is TOML's own guarantee and not something Ascribe's parser adds. Sections are shown in the order they're written.
- **`folder`** is a path from the content root, ending in `/`, with `""` for the root. Each folder is named at most once.
- **`pages`** is file names in that folder, in order. A name with a `/` in it, or one that isn't a page of that folder, is an error. So the file can't put a page outside its section.
- **`label`** is the section's name, when the default isn't right.
- **A nested section** is its own entry, `folder = "guides/advanced/"`, and is shown inside its parent at the place its entry comes among the parent's nested sections.

It's a file of its own and not a table in `ascribe.toml` for three reasons. It grows with the site, and the content model shouldn't. It's edited by a different person, often by dragging. And a file whose every change is a reorder reads cleanly in a pull request by itself.

A person may edit it by hand. It isn't generated, and the editor is the easy way to change it, not the only one.

### What the file doesn't have to say

- **A section that isn't listed** comes after the listed ones, in path order.
- **A page that isn't listed** comes after its section's listed pages, by title.
- **A section listed with no `pages`** is in its place among the sections, with its pages in the default order. `reference/` above is one.

So a new page is in the sidebar the moment it's written, and nothing fails because someone forgot a list. `ascribe check` says so, as advice, with a fix that adds the page to its section's list. A project that wants every page placed by hand raises that advice to an error, as any advice can be.

### What's about one page stays on the page

- **A short label,** for a title too long for a sidebar: `nav-label: ascribe.toml` in frontmatter, a reserved key. It's about the page, it travels with the page when the file moves, and it isn't order.
- **A page left out on purpose** is one that's `listing: unlisted`. It's built and has its address, and nothing Ascribe writes offers it, which already covers the sitemap and `llms.txt`. Navigation is one more list it's left off, and there's no second setting.

## What reads it

One order, and every reader takes it from the same place:

| Reader | Gets |
|---|---|
| A site | The sections, in order, each with its label, its own page when it has one, and its pages, each with its label, its route, and its type. In the JSON output, and in a file in the site output beside the page records, which `@ascribed/astro` reads |
| `@ascribed/astro` | `sidebar()` from the project's navigation, with no list of the site's own, and previous and next for a page |
| `llms.txt` | Its sections and their order |
| An archive | Its sidebar, its contents page, and its previous and next |
| The editor | The Pages view, in navigation order |
| `ascribe agents index` | The same sections as `llms.txt`, as it has today |

A page's type is there for a sidebar to show, not to group by: a site can put "How-to" or "Reference" beside an entry, or leave it out.

For each build, a page the build doesn't publish isn't in its navigation, and a section with no published pages isn't either. There's one navigation for a project, and a build shows what it has of it.

The starter stops writing `src/nav.ts`. A site that has one keeps working: it's the site's own list, and `sidebar()` still takes one. Ascribe's own site drops its list and its test once its docs are regrouped by feature, and gets `ascribe.nav` for the order of its sections.

## Checking

| Problem | Reported as |
|---|---|
| `pages` names a file that isn't a page of that folder, or has a `/` in it | An error, in `ascribe.nav`, with the closest page's name as a fix |
| `folder` isn't a folder with pages | An error |
| A folder with an `index.md` and a page of its name beside it | An error at the second |
| A folder named twice, or a page named twice | An error at the second |
| A page that isn't listed, in a section that lists pages | Advice, with a fix that adds it at the end of the list |
| A section that isn't listed, in a file that lists sections | Advice, with the same kind of fix |
| A `nav-label` longer than the title it shortens | Advice |

The editor reports each in `ascribe.nav` as it's typed, as it does in `ascribe.toml`.

One existing check gets better. `page-orphan` reports a page nothing links to, and has to hedge because Ascribe didn't know the navigation. A page in the navigation is reachable from a menu, so "orphan" can mean what it says: linked from nowhere and unlisted.

## In the editor

The Pages view shows pages as the navigation has them: sections, in order, each with its pages. It stops grouping them by content type, the grouping settled 6 calls an oversight. With no `ascribe.nav` that's the default order, so every project gets it. Fragments aren't in the navigation, and stay a group of their own, by folder.

- **Dragging a page within its section** reorders it. The drop is one edit to `ascribe.nav`, made as any edit is, so undo, the dirty marker, and git behave as for typing. The first drag in a section that lists no pages writes the section's list as it stands, then moves the one.
- **Dragging a section** reorders the sections.
- **Dragging a page onto another section** is a file move, which changes the page's address. It asks first, and then does what renaming a page does today.
- **A page that isn't listed** is shown where it will appear, marked, with an action to place it.
- **Renaming a section or a page's label** edits `label` or `nav-label`.

With no `ascribe.nav`, the view shows the default order, and the first drag creates the file.

The view is a native tree, and the drags are VS Code's `TreeDragAndDropController`, which is stable. Dendron, the knowledge-base extension, built its tree in a webview to get drag and drop and went back to a native one in 0.93. The view reveals the page in the active editor when the editor changes, as Foam's and Terraform's views follow the open file: VS Code's API for a tree to track its active item is still proposed, so it's `reveal`.

Renaming or moving a page, from the editor or with `ascribe rename`, updates `ascribe.nav` as it updates links.

## For agents

`ascribe model` reports the sections and their order. An agent adding a page is told by `ascribe check`'s advice that it isn't placed, with the fix. Nothing new is needed for an agent to put a page in order: it's an edit to a small TOML file, checked on save.

## What it touches

| Where | What |
|---|---|
| `SPEC.md` | §7, a new file beside `ascribe.toml`; §2.1, `nav-label` as a reserved frontmatter key; §9.4, the navigation in the outputs |
| `ascribe-model` | Loading and validating `ascribe.nav`, as it loads `ascribe.permalinks` |
| `ascribe-resolve` | The default order, the order with a file, and each build's share of it |
| `ascribe-check` | The problems above, and `page-orphan` |
| `ascribe-emit` | The navigation in the site and JSON outputs; `llms.txt`'s sections |
| `ascribe-lsp`, the extension | Diagnostics and completion in `ascribe.nav`, the Pages view, drags, and the rename |
| `ascribe-cli` | `ascribe rename` updating the file; `ascribe model` |
| `@ascribed/astro` | `sidebar()` from the project, previous and next, and the starter |
| The docs | A contract for the file, the reference, a guide, and Ascribe's own site moved over |

`llms.txt` changes for a project that adds the file: its sections follow the navigation. For a project with no file, the default order should be chosen to match what `llms.txt` does today, so that adding this changes nothing until a project asks.

## What this leaves out

- **A page in two sections,** or in one that isn't its folder.
- **A section for a content type.** A project that wants one makes a folder for it, and nothing stops that. Ascribe doesn't offer it, and its own starter set won't.
- **An entry that isn't a page:** a link to another site, a heading with no folder behind it, a divider.
- **A different order for each build.** A build shows fewer pages, in the same order.
- **Weights in frontmatter.** Order is a relation between pages, and no one file shows it. A drag would rewrite several.
- **Routes.** The file orders pages. Where a page is published is still its path and `[consumer]`.
- **A table of contents inside a page.** That's the page's headings, which every output has.
- **Generated pages,** such as an API reference built from a spec. They'd need a place in a section, and that waits for something to generate them.
- **A version switcher, or any selector.** The versions plan has its own, and a selector for a whole site is in the [brainstorm](../brainstorm.md).

## Settled

The five questions this proposal left open, answered by the maintainer on 12 October 2026, and three more from the same day, after the docs' sidebar was found grouped by type.

1. **The file is `ascribe.nav`.** It follows `ascribe.permalinks` and `ascribe.lock`: TOML, beside `ascribe.toml`, named for what it holds. Highlighting is the extension's to give, not the name's: it contributes none of the three to the TOML language today, and one `filenames` entry fixes all three at once. An editor without the extension shows them plain, as it does the other two, and that's accepted.
2. **A page that isn't listed is advice, in a section that lists pages, and the fix is safe.** Appending the page at the end of the list, in title order among the unlisted, gives exactly the order the default already shows, so applying the fix changes nothing a reader sees: `ascribe check --fix` and the agents' hook settle it, and nobody has to drag. A section with no `pages` reports nothing, so a project that never orders a section never hears about it. Without the advice, a hand-ordered section grows a tail nobody placed and nothing says so.
3. **Sections nest as folders do, with no cap and nothing reported.** The outputs carry the tree, `sidebar()` hands it to a site whole, and how deep a sidebar draws it is the site's design, which Ascribe doesn't hold an opinion on. `llms.txt` keeps its one level, a section for each top folder as today, with nested sections folded into their parent in navigation order, since an index is a list.
4. **A section's own page is its `index.md`, or the page beside the folder with the folder's name** (amended by 7). It's carried as the section's `page`, never in `pages`, always first, and its title is the section's default label. Whether a sidebar links the section's heading to it or lists it first is the site's drawing, and `sidebar()` gives a site what it needs for either. The root section's `index.md` is the home page, by the same rule. Naming it in `pages` is the error any name that isn't a page of the folder gets.
5. **The root section's label is "Start" unless `label` says otherwise.** Every other section takes its `index.md`'s title, and the root can't: a home page's title names the whole site, not a group. "Start" is what the starter and Ascribe's site use. `llms.txt` takes the same word, so its "Pages" heading becomes "Start" for every project with a plain output: an output change accepted with the change and named file by file, since one label from one place is the point.
6. **A section is a folder, and a folder is a feature.** Every type of page about a feature is in its folder. Grouping the docs' sidebar by content type was an oversight: it's like shipping an org chart, the docs' internal filing shown to readers who don't care about it, and it sends someone to another part of the site for related content. What follows from it is outside this proposal: each page declares its type in its frontmatter, the starter set stops giving each type a folder, and Ascribe's own docs are regrouped by feature.
7. **A page beside a folder of its name is that section's own page,** as `index.md` in the folder is. The docs split their long pages that way (decision #394), and a nested section made by a split would otherwise have no page and no label. A folder with both is an error.
8. **A section's default order is by type, then title, and a site is given each page's type.** Types come in the order `ascribe.toml` declares them. It keeps what was useful about the grouping, a reader finding the tutorial before the reference, inside a section and not across the site.
