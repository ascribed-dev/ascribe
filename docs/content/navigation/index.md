---
type: reference
title: Navigation
description: The order a project's sections and pages come in, from ascribe.nav or the default, and their labels, which every sidebar is drawn from.
available: ascribe 0.4.0
---

A project's **navigation** is its sections and pages in order, each with its label. Every sidebar and table of contents is drawn from it: an Astro site's `sidebar()` and `neighbors()` read it from the build (see [the Astro reference](../astro/reference.md#sidebar)), and so do [`llms.txt`](../content-model/output.md#consumer), `ascribe agents index`, an [archive](../versions/archive.md) of a release line, the JSON output, and `ascribe model`. With no file, a project has the default order; with `ascribe.nav` beside `ascribe.toml`, the file's order is laid over it, for the sections and pages it names. The rules are SPEC §7.8, and what the site output carries is the [navigation contract](../contracts/navigation.md).

To choose the order and the labels, see [Put your pages in order](order.md).

## Sections and pages

| Term | What it is |
|---|---|
| Section | The content root, or a folder under it with a page at any depth. Fragments aren't pages, so a folder whose name starts with `_` isn't a section. A page is in its folder's section, and only there: to move it to another, move the file. |
| A section's own page | Its `index.md`, or else the page beside the folder with the folder's name, such as `checks/diagnostics.md` for `checks/diagnostics/`. A page beside its folder is also one of the parent section's pages, which decides [where the section sits](#a-nested-section-s-place). The content root's own page is its `index.md`, the home page. |
| Label | A section's `label` in the file, else its own page's title, else its folder's name with `-` and `_` as spaces and its first letter capitalized. The content root's is **Start** unless the file gives it another. A page's label is its [`nav-label`](#a-page-s-label), else its title. |
| Reading order | A section's own page, then its pages and nested sections in order, each nested section [where it sits](#a-nested-section-s-place); from the content root, depth first: the root's own page and its other pages first, then each top section from its own page, as the sidebar's groups are. It's the order of previous and next, an archive, and a section's pages in `llms.txt`. The home page has no previous. |

## The default order

With no file, and for whatever the file leaves out:

- Sections come in path order, the content root first.
- In a section, its own page is first, then its pages by their content type, in the order `ascribe.toml` declares the types, then by title as a reader sees it, without a code span's backticks and with its phrases' values: a title written `` `zeta.yaml` options `` sorts under z, and `{product} overview` under the product's name.
- A nested section takes [its place](#a-nested-section-s-place), among its parent's pages or after them.

## A nested section's place
@id: a-nested-section-s-place

Below the content root, a nested section whose own page is beside its folder, such as `checks/diagnostics/` with `checks/diagnostics.md`, sits where that page sits among its parent's pages, in the default order and in the file's: its own `[[sections]]` table doesn't move it, so to move it, move the page in the parent's `pages`. Any other nested section comes after its parent's pages.

A top section comes after Start's pages, and is ordered among the top sections, by its `[[sections]]` table or in path order, even when its own page is beside its folder, such as `guides.md` beside `guides/`: never by where its page would sit among Start's.

## A page's label
@id: a-page-s-label

A page's label is its title unless its frontmatter gives it a shorter one, `nav-label`, for wherever it's an entry in a list: the sidebar, and the links to the previous and next pages.

```yaml
---
type: reference
title: Create and rotate API keys
nav-label: Keys
---
```

`nav-label` is a string on one line, read as the page's `title` is: with code spans in it where `title` sets `inline = "code"`, and phrases where `title` takes them. It's reserved, so a content type can't declare it and a fragment can't use it. Everywhere else the page keeps its title: its heading, search, and `llms.txt`, which names each page as an agent should look for it. One longer than the title it shortens is advice, `nav-label-long`. In the editor, **Rename Label…** in a page's menu in the **Pages** view sets it ([the editor](../editor/sidebar.md#this-page-pages-and-content-model)).

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
| `[[sections]]` | One table for each section the file lists, in the order they come. A section it doesn't list comes after the listed ones among its siblings, in path order. A nested section whose own page is beside its folder [sits where the page does](#a-nested-section-s-place), whatever its table says. | |
| `folder` | The section's folder, from the content root, ending in `/`, or `""` for the content root. | `folder = "Guides/"` |
| `pages` | Optional. File names in that folder, in the order they come; the section's other pages come after them, by title. It may name the page beside a nested folder, which [places that section](#a-nested-section-s-place). A section without `pages` has the default order. | `pages = ["My Setup.md"]` |
| `label` | Optional. The section's label, on one line. | `label = "How-to guides"` |

A file that isn't TOML, has no `version` or another one, has a key other than these, or a value of the wrong kind is `nav-invalid`, and the project has the default order until it reads.

The editor is the easy way to change it: drag sections and pages in its **Pages** view, which shows the navigation and writes the file, keeping its comments ([the editor](../editor/sidebar.md#this-page-pages-and-content-model)). Moving a page or a folder, with `ascribe rename` or in the editor, changes the file to match.

## Problems
@id: problems

Once the file reads, `ascribe check` and the editor hold it to the project. Each problem in the file is reported at the value it's about, and what it names wrongly is passed over until it's fixed:

| Problem | Reported | What fixes it |
|---|---|---|
| `nav-page-unknown`, error | At a `pages` entry that isn't a page in the section's folder: a misspelling, another case, a path with `/`, or the section's own `index.md` | Write the file name as it is on disk. The fix offered changes it to the closest page's name, so check it's the one you meant. |
| `nav-folder-unknown`, error | At a `folder` that isn't a section: no page is in it or under it, its case differs, or its name starts with `_` | Write a section's folder as it is on disk. The fix offered changes it to the closest section's. |
| `nav-duplicate`, error | At a folder listed a second time, or a page named a second time in one `pages` | Only the first counts. The fix removes the second table or entry, so keep what you need from it in the first. |
| `section-page-both`, error | On a folder's `index.md` when the page beside the folder has its name too, with or without the file | Keep one of them as the section's own page. Until then, the `index.md` is. |
| `nav-page-unplaced`, advice | On a page its section's `pages` doesn't name, unless it's `unlisted`, which no sidebar shows. A section without `pages` reports nothing. | The fix adds the section's unnamed pages to `pages`, in the order they come in now, so nothing moves. Then move them where you want them. |
| `nav-section-unplaced`, advice | On the first page of a section the file doesn't list when it lists one of the section's siblings. A folder none of whose sections are listed reports nothing, nor does a section [its own page places](#a-nested-section-s-place), which a table wouldn't move. | The fix adds a `[[sections]]` table for each, in the order they come in now. A file that writes `sections = [...]` inline gets no fix: add the sections to its list. |
| `nav-label-long`, advice | At a page's `nav-label` that's longer than its title, each counted as a reader sees it | Shorten it, or remove it so the title is the label. |

The fixes keep the file's comments and layout. The two pieces of advice are what tells a writer that a page or a folder they've just added isn't where the file says the others go, and an agent too, since the [agents' hook](../agents/hooks.md) passes on advice that has a safe fix, with what its fix changes: `ascribe check --fix` places every one of them at once, where the default already shows it, and the editor offers the same fix on the page. A project that wants every page placed can raise them to errors in [`[checks]`](../content-model/checks.md).

## What a build shows

The navigation is the project's, and each build shows its share: the pages it publishes and offers, and the sections left with one. A page that's [`listing: unlisted`](../content-model/index.md#listing-a-page) is published at its address and left out of the navigation, so out of the sidebar too. A build with no pages has an empty navigation.

Each build's site output has its share as `_ascribe/nav.json`, beside the page records, and so does its JSON output, with each formatted label in its own form. In a [versioned build](../versions/index.md), each line has its own: a page a newer line adds isn't in an older line's navigation.

## Who reads it

| Reader | What it takes |
|---|---|
| A site's sidebar | `sidebar()` in `@ascribed/astro`: the root's pages under its label, then a group for each top section, with nested sections inside. |
| Previous and next | `neighbors()` in `@ascribed/astro`: the pages before and after one in reading order. |
| [`llms.txt`](../content-model/output.md#consumer) and `ascribe agents index` | A section for the root's pages, under its label, then one for each top section, with its nested sections' pages in reading order. A page keeps its title there. |
| An [archive](../versions/archive.md) | Its sidebar and contents page, nested sections in their place, and previous and next in reading order. |
| `ascribe model --format json` | The project's navigation under `navigation`, every page that isn't `unlisted`, with each section's `listed`: whether `ascribe.nav` lists it. |
