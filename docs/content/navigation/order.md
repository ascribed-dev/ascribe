---
type: how-to
title: Put your pages in order
description: Choose the order of a project's sections and pages and their labels, in ascribe.nav or by dragging in the editor, leave a page out, and move a site's own list of pages over.
available: next
---

A site's sidebar, previous and next, `llms.txt`, and an archive all take their order from the project's [navigation](index.md). A project has one before you write anything, the default order. This page is for when you want another: a page earlier in its section, a section first, or a shorter label.

## A section is a folder

Each folder of the content root with a page in it is a section, and a page is in its folder's section. The content root is a section too, labeled **Start**, with the home page, its `index.md`, first. A folder's own page is its `index.md`, or the page beside the folder with its name, such as `guides.md` for `guides/`, and the section is labeled with that page's title.

To put a page in another section, move the file, with [`ascribe rename`](../command/questions.md#cli-rename) or in the editor; `ascribe.nav` only orders a section's pages. Links to the page and `ascribe.nav`'s names of it change with it.

## The order you get with no file

With no `ascribe.nav`, sections come in path order, and in each section its own page is first, then its pages by content type, in the order `ascribe.toml` declares the types, then by title, as a reader sees it, without a code span's backticks and with phrases' values. A project whose types go from reference to how-to has its references first in each section.

`ascribe model --format json` shows the project's navigation, under `navigation`.

## Write `ascribe.nav`

Add `ascribe.nav` beside `ascribe.toml`. List the sections in the order you want them, and for a section whose pages you want in another order, its `pages`:

```toml
version = 1

[[sections]]
folder = ""

[[sections]]
folder = "Reference/"

[[sections]]
folder = "Guides/"
pages = ["My Setup.md", "Upgrade.md"]
```

`folder` is the section's folder from the content root, ending in `/`, and `""` is the content root. `pages` names file names in that folder, without the section's own `index.md`. Whatever the file leaves out keeps the default order: a section it doesn't list comes after the ones it does, and a page a section's `pages` doesn't name comes after the ones it names, by title. A nested folder is a section of its own, with its own `[[sections]]` table. A top section's tables order it among the top sections, after Start's pages, even when its own page sits beside its folder, such as `guides.md` beside `guides/`. Below the top level, a section whose own page sits beside its folder, such as `checks/diagnostics.md` beside `checks/diagnostics/`, goes where that page goes among its parent's pages instead: to move it, name the page in the parent's `pages`; its own table doesn't move it.

Run `ascribe check`. A name that isn't a page, or a folder that isn't a section, is an error at the value. A page added to a section that lists its pages is advice, `nav-page-unplaced`, on the page; `ascribe check --fix` adds it to `pages` where it shows now, and you move it from there. Every key and problem is in [the reference](index.md#the-file).

## Or drag them in the editor

In VS Code, the Ascribe sidebar's **Pages** view shows the navigation. Drag a page or a section to where you want it, and the view writes `ascribe.nav` for you, keeping its comments; the file opens with the change unsaved. Dragging a page onto another section moves the file, after asking. See [the editor](../editor/index.md#this-page-pages-and-content-model).

## Give a section or a page a short label

A section's label is its own page's title, or else one made from its folder's name. Give it another with `label`:

```toml
[[sections]]
folder = "Guides/"
label = "How-to guides"
```

A page's label in a list is its title, unless its frontmatter gives a shorter one, `nav-label`:

```yaml
---
type: reference
title: Create and rotate API keys
nav-label: Keys
---
```

The page keeps its title everywhere else: its heading, search, and `llms.txt`.

## Leave a page out

A page that's [`listing: unlisted`](../content-model/index.md#listing-a-page) is published at its address and left out of the navigation, so out of the sidebar, previous and next, and `llms.txt`. A page a build doesn't publish isn't in that build's navigation either.

## Move a site's own list over
@id: own-list

A site that [`npx @ascribed/astro init`](../astro/index.md#set-up-a-site) started before has its own list of pages, `src/nav.ts`, which the layout passes as `sidebar(entries, { nav, entry })`. It keeps working. To have the project's order instead:

@steps
1. Write `ascribe.nav` with the list's order: a `[[sections]]` table for each group, in order, with its folder, a `label` where the group's label isn't its own page's title, and `pages` where the group's order isn't the default.
2. Take `nav` out of the layout's `sidebar()` and `neighbors()` calls: `sidebar(entries, { entry })` and `neighbors(entry, entries)`.
3. Delete `src/nav.ts`.
4. Build the site, and compare its sidebar with the one it had.

A group of the list that isn't a folder, or a page in a group other than its folder's, has no place in `ascribe.nav`: move the files into the folders you want as sections first.

## See also

- [Navigation](index.md), the file's keys, the default order, and its problems
- [The sidebar](../astro/reference.md#sidebar), `sidebar()` and `neighbors()` in `@ascribed/astro`
- [The navigation contract](../contracts/navigation.md), `_ascribe/nav.json` in the site output
