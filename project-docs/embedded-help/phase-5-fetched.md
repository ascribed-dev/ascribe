# Phase 5: Fetched for its line

Part of [Embedded help](README.md). Needs phase 1.

## Goal

`@ascribed/astro` serves each entry of each line's `help.json` at its own address, and a product fetches its line's, falling back to its bundled copy.

In a site built with `@ascribed/astro` from a versioned build with live lines 3.4 and 3.5, `GET /docs/go/connect.json` returns 3.5's entry for `connect` and `GET /docs/3.4/go/connect.json` returns 3.4's. A product on 3.4 fetches the second; when the docs no longer publish its line, the fetch fails and it shows its bundled copy. With `[consumer] headers = "netlify"`, each answers with `Access-Control-Allow-Origin: *`, so a product on another origin can fetch it.

## Context

- The proposal's phase 3, after its review: permalink addresses are published for the current line only, so fetching `go/<name>.json` alone would give a product on 3.4 the text of 3.5.
- `packages/astro/src/permalinks.ts` and `Permalink.astro`: how `permalinks.json` becomes one address per name, for the current line, and how a versioned build's lines are served under their numbers ([versions](../../docs/content/versions/lines.md)). README decision 1 put `help.json` beside `permalinks.json` for this pairing.
- `crates/ascribe-emit/src/site/headers.rs` and output-layout contract §1.4: how a rule is written for the host.

## Design

- **No new output.** The integration reads each line's `help.json`, as it reads `permalinks.json`, and serves each entry as JSON at `<permalink-path><name>.json` for the current line, and at the line's own permalink path under its number for an older line, `<base-path><line>/go/<name>.json`. Each answer is one entry with `version`, `build`, and `line`, as the file has them. A site on another generator has the one file, and the contract says how to serve it the same way.
- **Headers:** with `[consumer] headers`, `headers.json` gains a rule that gives each entry's address `Access-Control-Allow-Origin: *`. The text is already public, so any origin may read it.
- **The product's rule,** in the docs: fetch its own line's entry, check its `line`, and use the bundled copy offline, on any error, or when its line is gone.

## Tasks

1. The integration's routes for every live line, with the package's tests, and its reference.
2. The headers' rule, with emit tests and `site/test/headers.test.ts`'s pattern; output-layout §1.4's addition.
3. The embedded help page: fetching, the line, and falling back; the program in `examples/quill/product/` given a fetch, still tested against the bundled file.
4. A decision file for the addresses; `changes/astro/help-addresses.md`.

## Out of scope

A service that answers for an archived line: an archived line's site is wherever its `url` says, and its entries are there if that site serves them.

## Acceptance criteria

- The goal's two requests, against a built site in `packages/astro`'s tests.
- The header, in the headers' test.
- The build's outputs are unchanged but for `headers.json`'s new rule, named with `--accept`.

## Verify

```sh
pnpm --filter @ascribed/astro test
cargo test -p ascribe-emit headers
```

## Commits

1. "Serve each line's help beside its permalinks"
2. "Let a product on another origin fetch the help"
