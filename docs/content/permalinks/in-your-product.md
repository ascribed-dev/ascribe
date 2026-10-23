---
type: how-to
title: Show the docs in your product
description: Show a section of the docs inside your product, in a tooltip, a side panel, or beside an error, by the permalink the product already links to.
permalink: help-in-a-product
available: ascribe planned
---

A product that links to the docs by [permalink](index.md) can show the section the permalink names where its user is: the first paragraph in a tooltip, the whole section in a side panel, or the answer beside an error message, with a link to read more. The text is the docs' own, for the product's release line, so nothing is copied into the product's strings to drift from them.

Every build writes the sections to `_ascribe/help.json` in its site output, beside the list of permalinks. A product copies the file into its own build, looks a permalink up by name, and shows its title, summary, text, or address.

## What the file holds
@id: what-the-file-holds

One entry for each permalink the build publishes, and for each alias of one, keyed by name:

- **`title`:** the heading's text, or the page's title for a page's own permalink, as plain text.
- **`summary`:** the section's first paragraph, when the section opens with one; a page's own permalink has its description. Absent when the section opens with something else, such as a code block or a group of variants.
- **`text`:** the whole section, as the plain output writes it: Markdown, with phrases and includes resolved, each variant arm under a heading of its own, availability as an `Available:` line, and each image as its alt text, since a product can't count on loading an image from the docs' site.
- **`choices`:** for a section with a switcher, the section with each arm chosen, keyed by dimension and value, such as `choices.pm.pnpm` (see [Show your reader's arm](#choosing-an-arm)).
- **`address`:** the permalink's address, which a "Read more" link opens.
- **`source`:** the page the section is on.

A heading's section runs to the next heading at its level or above, or to the end of the note or variant arm the heading is in. A page's own permalink has the page's opening, before its first heading. A retired permalink has no entry. The [output-layout contract](../contracts/output-layout.md#17-the-help-file) has the format.

## Read it in the product
@id: reading-the-file

The product reads the file it bundled and looks the name up. This program, beside the example project `examples/quill`, prints a permalink's title, summary, and address, and exits with `1` for a name the file doesn't have:

@snippet {lang=ts, title=examples/quill/product/help.ts}: code:examples/quill/product/help.ts#help

After `ascribe build --config examples/quill`, the program reads the build's file:

```sh
node examples/quill/product/help.ts examples/quill/.ascribe/build/site/site/_ascribe/help.json prerequisites
```

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/help-product.txt

A product renders `text` as Markdown its own way: it's CommonMark with no HTML.

## Show your reader's arm
@id: choosing-an-arm

A section with a [switcher](../writing/variants.md) has every arm in `text`, each under a heading of its own, as the plain output writes it. A product that knows its reader's choice, such as the package manager they installed it with, shows the section with that arm alone from `choices`: keyed by the switcher's dimension, then by each value its arms name, each the section as a build that selects that value writes it. Other groups in the section stay as they were, and a section with switchers on two dimensions has a key for each. The summary is the section's, the same for every choice. A product that doesn't know shows `text`.

The program above, given `--choose pm=pnpm`, prints the `install-agent` section with its pnpm arm, and adds the choice to the address, which sets the reader's choice on the site:

```sh
node examples/quill/product/help.ts examples/quill/.ascribe/build/site/site/_ascribe/help.json install-agent --choose pm=pnpm
```

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/help-product-pnpm.txt

A section with no switcher, or one whose dimension the build already selects, has no `choices`, and the program prints what it prints without `--choose`.

## Bundle your own line's file
@id: bundling-the-file

Copy `help.json` from the build of the docs line your product's release documents, so the text matches what the product does. A line of a [versioned build](../versions/index.md) has its own file, in its own output, and the file says which line it is in `line`, which a product's build can check against its own version.

Set [`[consumer] site`](../content-model/output.md#consumer) in the docs' `ascribe.toml`. Without it, links in `text` are routes from the docs site's root, which a product can't open, and entries have no `address`.

## Check the names in the product's CI
@id: checking-the-names

A product that shows help by name links to the docs by name, so check its names as [any link to a permalink](index.md#checking-a-products-links) is checked: list them, and run `ascribe permalinks verify` against the docs' `ascribe.permalinks` in the product's CI. A name that's gone fails the product's pull request instead of showing no help to its users. The program above exits with `1` on a name its bundled file doesn't have, which a product's own tests can run too.
