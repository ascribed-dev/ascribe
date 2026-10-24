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
- **`summary`:** the section's [summary](#writing-summaries), when it has one; a page's own permalink has its description. Absent when there's none.
- **`text`:** the whole section, as the plain output writes it: Markdown, with phrases and includes resolved, each variant arm under a heading of its own, availability as an `Available:` line, and each image as its alt text, since a product can't count on loading an image from the docs' site.
- **`address`:** the permalink's address, which a "Read more" link opens.
- **`source`:** the page the section is on.

A heading's section runs to the next heading at its level or above, or to the end of the note or variant arm the heading is in. A page's own permalink has the page's opening, before its first heading. A retired permalink has no entry. The [output-layout contract](../contracts/output-layout.md#17-the-help-file) has the format.

## Write the summaries
@id: writing-summaries
@available: ascribe planned

A tooltip or a line beside an error has room for a sentence or two, so give each section a product shows a summary with [`@summary`](../writing/directives.md#summary). Write the summary's own text directly under the heading, which the page doesn't show:

```markdown
## Prerequisites
@id {permalink=true}: prerequisites
@summary: The agent needs Node.js and a docs repository that {product} can read.

Before you install the agent, make sure you have:
```

Or, when the section's first sentence reads well alone, make it a paragraph of its own and mark it, so it's written once and shown in both places:

```markdown
## Rotate a key
@id {permalink=true}: rotate-key

@summary
Replace a key before it expires, without stopping the agent.

Generate a new key in the dashboard, then …
```

Nothing is guessed: a section without `@summary` has no `summary` in the file, whatever it opens with, and a product shows its title and "Read more" alone.

`ascribe check` says where one is missing, and where one is too long, as advice:

- [`summary-missing`](../checks/diagnostics/content-checks.md#ASC252): a heading's permalink with no `@summary`. A page's own permalink has its description, which [`page-description-missing`](../checks/diagnostics/content-checks.md#ASC150) asks for.
- [`summary-long`](../checks/diagnostics/content-checks.md#ASC253): a summary of 300 characters or more, counted as a description is. `limit` changes the number.

To require a summary for every permalink, raise `summary-missing` to a warning in [`[checks]`](../content-model/checks.md):

```toml
[checks]
summary-missing = "warning"

[checks.summary-long]
limit = 200
```

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

@available: ascribe planned
To see an entry without building the site, run [`ascribe resolve go:<name>`](../command/questions.md#cli-resolve), with the build: it prints what the file has for the name, from the code that writes the file, and `--format json` prints the entry as the file writes it. An agent writing your product's message can read what it will show the same way, with the `ascribe_resolve` tool or the command, which the rules [`ascribe agents sync`](../agents/index.md) writes tell it about.

```sh
ascribe resolve go:prerequisites --build site --config examples/quill
```

## Bundle your own line's file
@id: bundling-the-file

Copy `help.json` from the build of the docs line your product's release documents, so the text matches what the product does. A line of a [versioned build](../versions/index.md) has its own file, in its own output, and the file says which line it is in `line`, which a product's build can check against its own version.

Set [`[consumer] site`](../content-model/output.md#consumer) in the docs' `ascribe.toml`. Without it, links in `text` are routes from the docs site's root, which a product can't open, and entries have no `address`.

## Check the names in the product's CI
@id: checking-the-names

A product that shows help by name links to the docs by name, so check its names as [any link to a permalink](index.md#checking-a-products-links) is checked: list them, and run `ascribe permalinks verify` against the docs' `ascribe.permalinks` in the product's CI. A name that's gone fails the product's pull request instead of showing no help to its users. The program above exits with `1` on a name its bundled file doesn't have, which a product's own tests can run too.
