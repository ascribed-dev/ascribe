---
type: how-to
title: Move a site's own list to the navigation
description: Replace an Astro site's own list of pages, the src/nav.ts that init wrote in 0.3, with the project's navigation in ascribe.nav.
---

A site that [`npx @ascribed/astro init`](../astro/index.md#set-up-a-site) started before 0.4 has its own list of pages, `src/nav.ts`, which the layout passes as `sidebar(entries, { nav, entry })`. It keeps working: the sidebar shows the list's groups in its order, and the build fails when the list leaves out a page, names a file that isn't one, or names a page twice. This guide moves the site over to the project's [navigation](index.md), so the site keeps no list of its own.

## Before you start

- Each group of the list is a folder, with its pages in it. A group that isn't a folder, or a page in a group other than its folder's, has no place in `ascribe.nav`: move the files into the folders you want as sections first, with [`ascribe rename`](../command/questions.md#cli-rename).

## Move the list over
@id: own-list

@steps
1. Write `ascribe.nav` with the list's order: a `[[sections]]` table for each group, in order, with its folder, a `label` where the group's label isn't its own page's title, and `pages` where the group's order isn't the default ([Put your pages in order](order.md)).
2. Take `nav` out of the layout's `sidebar()` and `neighbors()` calls: `sidebar(entries, { entry })` and `neighbors(entry, entries)`.
3. Delete `src/nav.ts`.
4. Build the site, and compare its sidebar with the one it had.

## See also

- [The sidebar](../astro/reference.md#sidebar), `sidebar()` and its `nav` option in `@ascribed/astro`
- [Navigation](index.md), the file's keys and the default order
