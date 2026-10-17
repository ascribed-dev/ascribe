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
- A nested section whose own page is beside its folder sits where that page sits among its parent's pages. One with an `index.md` comes after the parent's pages, in path order.

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

A file that isn't TOML, has no `version` or another one, has a key other than these, or a value of the wrong kind is `nav-invalid`, and the project has the default order until it reads. A `folder` with no pages, or a `pages` entry that isn't a page of its folder, matches nothing and is passed over for now: nothing reports it, so check a name you've written against the file names.

## What a build shows

The navigation is the project's, and each build shows its share: the pages it publishes and offers, and the sections left with one. A page that's [`listing: unlisted`](../content-model/index.md#listing-a-page) is published at its address and left out of the navigation, so out of the sidebar too. A build with no pages has an empty navigation.

Each build's site output has its share as `_ascribe/nav.json`, beside the page records.
