# Phase 6: What a link carries

Part of [Permalinks](README.md). Requires phase 2; better after phase 5, so the page for a retired name exists to match. Mostly `@ascribed/astro` and `@ascribed/elements`.

## Goal

- **A link can say what its reader is using,** and the docs open on the right variant: a product on macOS sends its readers to the macOS tab.
- **A name nobody declared gets a helpful page,** not a bare error. An old copy of a product may ask for one.

## Context

- The [proposal](../permalinks.md), "What a link can carry", and Firefox's help links, which carry the version, the system, and the language, and let the site decide which to act on.
- `packages/elements/src/tabs.ts`: a reader's choice for a dimension is kept in `localStorage`, by dimension, and holds across pages. `packages/elements/CONTRACT.md`: what the tabs promise.
- Phase 2's forwarding page, which has no script, and the list, which a site can read.
- `site/src/pages/404.astro`: the docs site's own not-found page.
- [Versions](../versions.md) isn't built. A version on a link waits for it.

## Design

### Parameters

A permalink's address takes parameters named for the content model's dimensions:

```
/go/install-agent?platform=macos&pm=pnpm
```

- **A parameter that names a dimension, with one of its values,** sets the reader's choice for that dimension, as if they'd picked the tab, then forwards.
- **Anything else is ignored:** a name that isn't a dimension, a value the dimension doesn't have. A product sends what it knows and never has to ask what the docs accept.
- **The forwarding page gains a small script** for this, and still forwards without it: a reader with scripts off lands on the right heading with the tabs as they were.
- **The list names the dimensions and their values,** so the page needs nothing else to decide.

How the choice is set is the tabs' business. If the elements' contract has no way for a page to set a choice before the tabs load, adding one is a change to that contract, recorded as such.

### A name nobody declared

A static host serves one not-found page for everything. So this is help for a site's own page, not a new one:

- `@ascribed/astro` exports a small component a site puts on its 404 page. On an address under the permalinks' path, it says the link came from a product that expected a section here, that it isn't in these docs, and offers the docs' home and search. Anywhere else it shows nothing.
- The docs site uses it.

### Left for versions

A parameter naming a versioned target, such as `self-managed=3.4`, is ignored like any unknown one until versions exist. The proposal has it sending the reader to that line or its archive.

## Tasks

1. The dimensions and values in the build's list, as an addition to its format.
2. The script on the forwarding page, and whatever the elements need for a choice to be set from outside, with tests in `packages/elements/test/` in all three browsers and end-to-end in `examples/astro-site`: a parameter opens the right tab; an unknown one changes nothing; with scripts off, the page still forwards.
3. The component for a site's 404 page, used on the docs site, with a test.
4. The guide's section on linking from a product: what a link can carry, with an example of building one. `guides/astro.md`, the elements' contract if it changed, `CHANGELOG.md`.

## Out of scope

Versions on a link; choosing a variant from anything but the link (a browser's language, the reader's system); redirect rules for a particular host.

## Acceptance criteria

- Following `/go/<id>?<dimension>=<value>` on a built site opens the heading with that tab showing, and the choice holds on the next page, as a click would.
- A link with a parameter the docs don't know behaves exactly like one without it.
- An address under the permalinks' path that matches nothing shows the site's 404 page with the explanation on it.

## Verify

```sh
pnpm --filter @ascribed/elements test
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test:e2e
```

## Commits

1. "Open the right variant from a permalink's parameters"
2. "Explain a permalink that doesn't exist on a site's not-found page"
