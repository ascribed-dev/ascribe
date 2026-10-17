---
type: reference
title: Navigation
description: The order a project's sections and pages come in, from ascribe.nav or the default, and their labels, which every sidebar is drawn from.
available: next
---

A project's **navigation** is its sections and pages in order, each with its label. Every sidebar is drawn from it: an Astro site's `sidebar()` reads it from the build (see [the Astro reference](../astro/reference.md#sidebar)). With no file, a project has the default order; with `ascribe.nav` beside `ascribe.toml`, the file's order is laid over it, for the sections and pages it names. The rules are SPEC §7.8, and what the site output carries is the [navigation contract](../contracts/navigation.md).

## Sections and pages

| Term | What it is |
|---|---|
| Section | The content root, or a folder under it with a page at any depth. Fragments aren't pages, so a folder whose name starts with `_` isn't a section. A page is in its folder's section, and only there: to move it to another, move the file. |
| A section's own page | Its `index.md`, or else the page beside the folder with the folder's name, such as `checks/diagnostics.md` for `checks/diagnostics/`. A page beside its folder is also one of the parent section's pages, and the nested section sits right after it. The content root's own page is its `index.md`, the home page. |
| Label | A section's `label` in the file, else its own page's title, else its folder's name with its first letter capitalized. The content root's is **Start** unless the file gives it another. A page's label is its title. |

## The default order

With no file, and for whatever the file leaves out:

- Sections come in path order, the content root first.
- In a section, its own page is first, then its pages by their content type, in the order `ascribe.toml` declares the types, then by title.
- A nested section whose own page is beside its folder sits where that page sits among its parent's pages. Any other comes after the parent's pages, in path order.

## The file
@id: the-file

`ascribe.nav` is TOML, beside `ascribe.toml`:

```toml
version = 1

[[sections]]
folder = ""

[[sections]]
folder = "Reference/"

[[sections]]
folder = "Guides/"
label = "How-to guides"
pages = ["My Setup.md", "Upgrade.md"]
```

| Key | Description | Example |
|---|---|---|
| `version` | The file's format: `1`. | `version = 1` |
| `[[sections]]` | One table for each section the file lists, in the order they come. A section it doesn't list comes after the listed ones among its siblings, in path order. | |
| `folder` | The section's folder, from the content root, ending in `/`, or `""` for the content root. | `folder = "Guides/"` |
| `pages` | Optional. File names in that folder, in the order they come; the section's other pages come after them, by title. It may name the page beside a nested folder, which places that section where the page is. A section without `pages` has the default order. | `pages = ["My Setup.md"]` |
| `label` | Optional. The section's label, on one line. | `label = "How-to guides"` |

A file that isn't TOML, has no `version` or another one, has a key other than these, or a value of the wrong kind is `nav-invalid`, and the project has the default order until it reads.

## Problems
@id: problems

Once the file reads, `ascribe check` and the editor hold it to the project. Each problem in the file is reported at the value it's about, and what it names wrongly is passed over until it's fixed:

| Problem | Reported | What fixes it |
|---|---|---|
| `nav-page-unknown`, error | At a `pages` entry that isn't a page in the section's folder: a misspelling, another case, a path with `/`, or the section's own `index.md` | Write the file name as it is on disk. The fix offered changes it to the closest page's name, so check it's the one you meant. |
| `nav-folder-unknown`, error | At a `folder` that isn't a section: no page is in it or under it, its case differs, or its name starts with `_` | Write a section's folder as it is on disk. The fix offered changes it to the closest section's. |
| `nav-duplicate`, error | At a folder listed a second time, or a page named a second time in one `pages` | Only the first counts. The fix removes the second table or entry, so keep what you need from it in the first. |
| `section-page-both`, error | On a folder's `index.md` when the page beside the folder has its name too, with or without the file | Keep one of them as the section's own page. Until then, the `index.md` is. |
| `nav-page-unplaced`, advice | On a page its section's `pages` doesn't name. A section without `pages` reports nothing. | The fix adds the section's unnamed pages to `pages`, in the order they come in now, so nothing moves. Then move them where you want them. |
| `nav-section-unplaced`, advice | On the first page of a section the file doesn't list when it lists one of the section's siblings. A folder none of whose sections are listed reports nothing. | The fix adds a `[[sections]]` table for each, in the order they come in now. A file that writes `sections = [...]` inline gets no fix: add the sections to its list. |

The fixes keep the file's comments and layout. The two pieces of advice are what tells a writer that a page or a folder they've just added isn't where the file says the others go, and an agent too when it runs `ascribe check`, since the agents' hook reports only errors: `ascribe check --fix` places every one of them at once, where the default already shows it, and the editor offers the same fix on the page. A project that wants every page placed can raise them to errors in [`[checks]`](../content-model/checks.md).

## What a build shows

The navigation is the project's, and each build shows its share: the pages it publishes and offers, and the sections left with one. A page that's [`listing: unlisted`](../content-model/index.md#listing-a-page) is published at its address and left out of the navigation, so out of the sidebar too. A build with no pages has an empty navigation.

Each build's site output has its share as `_ascribe/nav.json`, beside the page records.
