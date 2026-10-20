---
type: how-to
title: Permalinks
description: Link to the docs from your product by permalink, check those links in the product's CI, and keep them working when a section moves, gets a second name, or goes away.
available: ascribe 0.3.0
---

A heading marked as a [permalink](../writing/directives.md#permalinks), `@id {permalink=true}: weave-config`, gets an address of its own, `go/weave-config` under the base path, which sends the reader to the heading wherever it is now. A page with `permalink: options` in its frontmatter gets `go/options`, which leads to the top of the page. Link to that address from outside the docs, such as from your product's interface or its error messages, and the link keeps working when the page moves.

This guide keeps a product's links to the docs working: linking to permalinks from the product, checking those links in the product's CI, and what to do in the docs when a section moves, gets a second name, or goes away. The site publishes each address: with Astro, [`@ascribed/astro` does](../astro/reference.md#permalinks).

@available: ascribe 0.4.0
In VS Code, the sidebar's [This page](../editor/sidebar.md#this-page-pages-and-content-model) lists the permalinks a page declares, its own and its headings', and each opens its line.

## Link from your product
@id: linking-from-a-product

A product links to a permalink's address: the site, the permalinks' path, and the name, ending the way the site's routes do. It can add what it knows about its reader, as parameters named for the content model's [dimensions](../content-model/content.md#dimensions):

```ts
// The docs' permalinks, as `[consumer]` publishes them.
const permalinks = "https://docs.example.com/docs/go/";

function docsLink(name: string, reader: Record<string, string> = {}): string {
  const url = new URL(name, permalinks);
  for (const [dimension, value] of Object.entries(reader)) url.searchParams.set(dimension, value);
  return url.href;
}

// https://docs.example.com/docs/go/weave-config?pm=pnpm
docsLink("weave-config", { pm: "pnpm" });
```

A parameter that names a dimension, with one of its values, sets the reader's choice, as if they'd picked the tab: this link opens the section with the `pnpm` tab showing. Anything else is ignored, so send what the product knows, without checking what the docs accept ([the parameters](../astro/reference.md#permalinks)).

## Check the links in the product's CI
@id: checking-a-products-links

A product that links to the docs can check, in its own CI, that every name it links to is still a permalink there, without building or even having the docs. The docs commit their list of permalinks, `ascribe.permalinks`, which `ascribe permalinks sync` writes beside `ascribe.toml` ([The permalinks list](../contracts/permalinks-list.md)). Then `ascribe check` in the docs warns when a permalink is removed, and the product checks its names against a copy of the list:

```yaml
# .github/workflows/docs-links.yml, in the product's repository
name: Docs links
on: [pull_request]
jobs:
  docs-links:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # The docs' list, from their repository; or keep a copy in this one.
      - run: curl -fsSL -o docs.permalinks https://raw.githubusercontent.com/example/docs/main/ascribe.permalinks
      - run: npx @ascribed/cli permalinks verify --against docs.permalinks --from docs-links.txt
```

`docs-links.txt` lists the names the product links to, one on each line. [`ascribe permalinks verify`](command.md#cli-permalinks-verify) exits with 1, naming each name the list doesn't have, and each the docs retired, with the reason they gave, so a link to a section the docs removed, or never had, fails the product's pull request rather than reaching its users. An alias passes, as the permalink it leads to does.

In the docs, `ascribe check` reports a list that's behind the content (`permalinks-list-behind`): a new permalink, or one that moved. Run `ascribe permalinks sync`, or take the editor's quick fix, and commit the list with the change; `ascribe permalinks sync --check` fails in CI when it wasn't. Moving a permalink to another page changes one line of the list and removes nothing. Deleting one is a warning (`permalink-removed`), which `sync` doesn't clear: restore the name, or retire it ([when a section moves or goes away](index.md#moving-aliasing-and-retiring)).

## When a section moves or goes away
@id: moving-aliasing-and-retiring

A permalink is a promise to whatever links to it, so what to do when its section changes depends on what happened to the section:

- **It moved,** to another page or under another heading: move the `@id` with it, and run `ascribe permalinks sync`. The address leads to the new place, and nothing else changes.
- **It has a second name,** such as one an older release of the product prints: add an alias in [`[permalinks.aliases]`](../content-model/checks.md#20-permalinks), `config = "weave-config"`. The alias's address leads where the permalink's does, and `verify` accepts both names.
- **It's gone,** merged into another section or removed with the feature it described: retire it, with a reason a reader can understand.

```sh
ascribe permalinks retire rotate-keys --reason "Merged into the security guide" --to guides/security.md
ascribe permalinks retire legacy-sync --reason "The sync agent was removed in 4.0"
```

[`ascribe permalinks retire`](command.md#cli-permalinks-retire) writes each to `[permalinks.retired]` in `ascribe.toml`, keeping the file's comments, and records it in `ascribe.permalinks`, so `ascribe check` no longer warns. With `--to`, its address leads there; without it, the address is a page that says the section was removed, with the reason. A product's `verify` then fails on the name, with the reason, so the product changes its link. In the editor, the quick fix on `permalink-removed` writes the entry with a placeholder reason, which is an error until you replace it. To retire every permalink on a page you're about to delete, run it with `--page` before deleting the page.

Retire rather than delete the line from the list: a deleted line makes the address a 404 with nothing to say why, and a product's `verify` says only that the name is missing.

## See also

- [Permalinks](../writing/directives.md#permalinks), in the directive reference: how a heading or a page gets one, and the names it may have.
- [`[permalinks]`](../content-model/checks.md#20-permalinks): the moved, aliased, and retired names in `ascribe.toml`.
- [`ascribe permalinks`](command.md): `sync`, `list`, `verify`, and `retire`, their options, and their output.
- [The permalinks list](../contracts/permalinks-list.md): the file a product's CI checks against.
- [Permalinks](../astro/reference.md#permalinks), in the `@ascribed/astro` reference: the page at each address, its parameters, and [`MissingPermalink`](../astro/reference.md#a-permalink-that-isnt-here) for the site's 404 page.
