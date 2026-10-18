---
type: how-to
title: Permalinks
description: Giving a section an address your product can link to that keeps working when the docs move, checking those links in the product's CI, and moving, aliasing, or retiring one.
available: ascribe 0.3.0
---

A heading marked as a [permalink](../writing/directives.md#permalinks), `@id {permalink=true}: weave-config`, gets an address of its own, `go/weave-config` under the base path, which sends the reader to the heading wherever it is now. A page with `permalink: options` in its frontmatter gets `go/options`, which leads to the top of the page. Link to that address from outside the docs, such as from your product's interface or its error messages, and the link keeps working when the page moves.

This guide covers publishing the addresses, linking to them from a product, checking those links in the product's CI, and what to do when a section moves, gets a second name, or goes away.

@available: next
In VS Code, the sidebar's [This page](../editor/sidebar.md#this-page-pages-and-content-model) lists the permalinks a page declares, its own and its headings', and each opens its line.

## Publish the addresses

Each build lists its permalinks in the site output, `_ascribe/permalinks.json` ([output-layout contract §1.2](../contracts/output-layout.md#12-the-permalinks-list)). A site generator publishes a page at each address, which:

- forwards at once with a `<meta http-equiv="refresh">`, without a script, keeping a heading's anchor;
- with a script, first takes the reader's choices from the address's parameters ([Linking from a product](index.md#linking-from-a-product));
- links to the same place, for a browser that doesn't follow;
- is marked `noindex`, with a canonical link to where it leads, so search engines list the page and not the address.

So it works on any static host.

### With Astro

[`@ascribed/astro`](../astro/index.md) adds the route itself, from the list, in `astro build` and in `astro dev`, where a permalink you add or remove is served on the next rebuild, with no restart. The addresses end the way the site's routes do (`trailing-slash`), and aren't pages of the collection, `llms.txt`, or the Markdown for agents.

[`permalink-path`](../content-model/output.md#consumer) moves them, as a path from the site's root, such as `"/go/"` when the docs move under `/docs/` but the addresses your product prints shouldn't. It must be under Astro's `base`, since that's all an Astro site serves; the integration fails the build when it isn't.

### With another generator

Read `_ascribe/permalinks.json` from the build's site output and publish a page like these at `path` and each name, forwarding to `route`, and `#` and `fragment` for a heading. For the parameters, its script calls `@ascribed/elements`' `choose` for each one that names a dimension in the list's `dimensions` with one of its values, then forwards. A host's redirect rules can do the same as the page without its script, but a redirect drops nothing only if the host keeps the anchor.

## Linking from a product

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

- **A parameter that names a dimension, with one of its values,** sets the reader's choice for that dimension, as if they'd picked the tab: the section opens with the `pnpm` tab showing, and the choice holds on the pages they read next.
- **Anything else is ignored:** a name that isn't a dimension, or a value the dimension doesn't have. Send what the product knows, without checking what the docs accept; a link with only unknown parameters works as one with none.
- **The choice needs a script and the browser's storage.** With scripts off, the address forwards to the section with the tabs as they were; with storage blocked, the choice doesn't reach the next page.
- **A version isn't a parameter yet.** `self-managed=3.4` is ignored like any unknown one.

## Checking a product's links

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

`docs-links.txt` lists the names the product links to, one on each line. [`ascribe permalinks verify`](../examples/sources.md#cli-permalinks-verify) exits with 1, naming each name the list doesn't have, and each the docs retired, with the reason they gave, so a link to a section the docs removed, or never had, fails the product's pull request rather than reaching its users. An alias passes, as the permalink it leads to does.

In the docs, `ascribe check` reports a list that's behind the content (`permalinks-list-behind`): a new permalink, or one that moved. Run `ascribe permalinks sync`, or take the editor's quick fix, and commit the list with the change; `ascribe permalinks sync --check` fails in CI when it wasn't. Moving a permalink to another page changes one line of the list and removes nothing. Deleting one is a warning (`permalink-removed`), which `sync` doesn't clear: restore the name, or retire it ([Moving, aliasing, and retiring](index.md#moving-aliasing-and-retiring)).

## Moving, aliasing, and retiring

A permalink is a promise to whatever links to it, so what to do when its section changes depends on what happened to the section:

- **It moved,** to another page or under another heading: move the `@id` with it, and run `ascribe permalinks sync`. The address leads to the new place, and nothing else changes.
- **It has a second name,** such as one an older release of the product prints: add an alias in [`[permalinks.aliases]`](../content-model/checks.md#20-permalinks), `config = "weave-config"`. The alias's address leads where the permalink's does, and `verify` accepts both names.
- **It's gone,** merged into another section or removed with the feature it described: retire it, with a reason a reader can understand.

```sh
ascribe permalinks retire rotate-keys --reason "Merged into the security guide" --to guides/security.md
ascribe permalinks retire legacy-sync --reason "The sync agent was removed in 4.0"
```

[`ascribe permalinks retire`](../examples/sources.md#cli-permalinks-retire) writes each to `[permalinks.retired]` in `ascribe.toml`, keeping the file's comments, and records it in `ascribe.permalinks`, so `ascribe check` no longer warns. With `--to`, its address leads there; without it, the address is a page that says what was there was removed from the docs, gives the reason, and links to the docs' home page. A product's `verify` then fails on the name, with the reason, so the product changes its link. In the editor, the quick fix on `permalink-removed` writes the entry with a placeholder reason, which is an error until you replace it. To retire every permalink on a page you're about to delete, run it with `--page` before deleting the page.

Retire rather than delete the line from the list: a deleted line makes the address a 404 with nothing to say why, and a product's `verify` says only that the name is missing.

## A permalink that isn't here

A product released before a section was renamed, or after one was added, may link to a name these docs don't have. The host answers with the site's 404 page, as for any missing address. With Astro, put `@ascribed/astro`'s `MissingPermalink` on it:

```astro
---
// src/pages/404.astro
import MissingPermalink from "@ascribed/astro/MissingPermalink.astro";
---

<h1>Page not found</h1>
<MissingPermalink />
```

On an address under the permalinks' path, it says the link came from a product that expected a section of the docs there, names it, and offers the docs' home page and search. Anywhere else, and without a script, it shows nothing. `home` sets where it sends the reader, the docs' base path by default; children replace its last paragraph, such as to point at your search:

```astro
<MissingPermalink>
  <p>Search the docs for what you need, or go back to the <a href="/docs">home page</a>.</p>
</MissingPermalink>
```

## See also

- [Permalinks](../writing/directives.md#permalinks), in the directive reference: how a heading or a page gets one, and the names it may have.
- [`[permalinks]`](../content-model/checks.md#20-permalinks): the moved, aliased, and retired names in `ascribe.toml`.
- [`ascribe permalinks`](../examples/sources.md#cli-permalinks-verify): verifying a product's list, and retiring a name.
- [The permalinks list](../contracts/permalinks-list.md): the file a product's CI checks against.
