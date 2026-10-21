---
type: how-to
title: Put your pages in order
description: Choose the order of a project's sections and pages and their labels, in ascribe.nav or by dragging in the editor.
available: ascribe 0.4.0
---

A site's sidebar, previous and next, `llms.txt`, and an archive all take their order from the project's [navigation](index.md). A project has one before you write anything, [the default order](index.md#the-default-order): in each section, its own page first, then its pages by content type and title. This guide is for when you want another: a page earlier in its section, a section first, or a shorter label.

A section is a folder, and a page is in its folder's section. To put a page in another section, move the file, with [`ascribe rename`](../command/questions.md#cli-rename) or in the editor; `ascribe.nav` only orders a section's pages. Links to the page and `ascribe.nav`'s names of it change with it.

## Write `ascribe.nav`

@steps
1. Run `ascribe model --format json`, and read the project's navigation as it is now, under `navigation`.
2. Add `ascribe.nav` beside `ascribe.toml`, and list the sections in the order you want them, each a `[[sections]]` table with its `folder`, from the content root, ending in `/`; `""` is the content root. A nested folder is a section of its own, with its own table.
3. For a section whose pages you want in another order, list its file names in `pages`, without the section's own `index.md`:

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

   Whatever the file leaves out keeps the default order, after what it names. A nested section whose own page is beside its folder [moves with that page](index.md#a-nested-section-s-place), in its parent's `pages`.

4. To give a section a label other than its own page's title, add `label = "How-to guides"` to its table. To give a page a shorter label in the sidebar and in previous and next, give it a [`nav-label`](index.md#a-page-s-label) in its frontmatter.
5. Run `ascribe check`. A name that isn't a page, or a folder that isn't a section, is an error at the value, and a page that a section's `pages` doesn't name is advice, `nav-page-unplaced`: `ascribe check --fix` adds it where it shows now, and you move it from there. Every key and problem is in [the reference](index.md#the-file).

## Or drag them in the editor

In VS Code, the Ascribe sidebar's [**Pages** view](../editor/sidebar.md#pages) shows the navigation. Drag a page or a section to where you want it, and the view writes `ascribe.nav` for you as typing does, keeping its comments: the file opens with the change unsaved, and **Undo** takes it back, the file too when the drag made it.

- **Drag a section** onto another beside it, and it's put before it when you drag it up and after it when you drag it down. The sections beside it are listed in the file in the order they had, so nothing else moves.
- A section whose own page sits beside its folder, such as `checks/diagnostics/` with `checks/diagnostics.md`, is drawn where that page sits among its parent's pages, and dragging it moves that page in the parent's `pages`. At the top level there's no such placing: `guides/` with `guides.md` is drawn after Start, with the page first in it, and is dragged among the top sections like any other.
- **Drag a page** onto another of its section the same way, or onto its section to put it first. The section's `pages` lists its pages as they stood, with the one you moved where you put it.
- **Drag a page onto another section**, or a page of one, and it asks first, since the page's address changes. On **Move**, its file moves to that section's folder, and the links to it and `ascribe.nav` follow, as they do when you move a file in the Explorer.
- A page `ascribe.nav` leaves out of a section that lists its pages is at the end of its section, marked **not placed**; **Place Here** in its menu adds it to the end of the section's `pages`. A page that's `unlisted` is in no sidebar, so it's marked **unlisted** instead, and needs no place.
- **Rename Label…** in a section's menu sets its `label` in `ascribe.nav`, and in a page's menu its `nav-label`, in its frontmatter, unsaved as a drag's change is. It offers the label as it's written; leave it empty for the default, which for a page is its title.

## Leave a page out

A page that's [`listing: unlisted`](../content-model/index.md#listing-a-page) is published at its address and left out of the navigation, so out of the sidebar, previous and next, and `llms.txt`. A page a build doesn't publish isn't in that build's navigation either.

## See also

- [Navigation](index.md), the file's keys, the default order, and its problems
- [Move a site's own list to the navigation](site-list.md), for a site that `init` started in 0.3
- [The sidebar](../astro/reference.md#sidebar), `sidebar()` and `neighbors()` in `@ascribed/astro`
- [The navigation contract](../contracts/navigation.md), `_ascribe/nav.json` in the site output
