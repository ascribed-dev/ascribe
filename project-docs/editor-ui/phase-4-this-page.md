# Phase 4: This page, whole

Part of [The editor's views](README.md). Needs phase 1.

## Goal

This page says whether the page is published, listed, overdue, over size, or unfinished, and what it includes.

In VS Code, open a page of Ascribe's docs. **About** has, beside its title and addresses: its description's length against the limit; "Review overdue since 3 October" or "due in 12 days"; its availability; "Published by site, agents; left out by cloud"; "Listed (the default)" or "Unlisted, by `[listing.folders]`"; its permalinks, the page's and each heading's; and, for a page started from a template, "how-to template, 2 placeholders left". **Health** has "1 warning, 2 advice", which opens the Problems panel filtered to the page; "Size: 31,200 of 50,000 characters, in site"; and each tested example, tested, stale, or not tested. **Connections** adds what the page includes: its fragments, and its snippets with their files.

## Context

- README decisions 3 and 11.
- Phase 1's `ascribe/page`, and what it keeps for the page.
- What each row is drawn from, which the server computes already:
  - the description field and the limit `description-too-long` checks;
  - the field with the review role and today's date, as `review-overdue` reads them (`ASCRIBE_TODAY` in tests);
  - the page's availability, and its builds, as `check` resolves the page for each;
  - its listing (`Listing` and `ListingLevel` in `crates/ascribe-model/src/model.rs`), which `pages.json` writes as `listing` and `listing_from`;
  - its permalinks, as `ascribe permalinks list` lists them;
  - the template it started from and `template_places` (`crates/ascribe-check/src/checks/template.rs`);
  - the Markdown size `page-size` measures per build;
  - the tested-example state the editor's `example-stale` and `example-unobserved` come from;
  - the page's includes and snippets.
- `crates/ascribe-lsp/src/inventory.rs`: each page's entry, which gains its builds here for phase 5.
- The docs: the editor page's This page section, and the reference pages each row points to.

## Design

### The rows

About and Health are rows of a label and a description. A row with somewhere to go opens it:

| Row | Opens |
|---|---|
| Description | The field in the frontmatter |
| Review | The field |
| Builds | The build picker, with the build lens |
| Listing | The rule: the page's own field, or `ascribe.toml` at the folder's or the default's |
| A permalink | Its `@id`, or the page's `permalink` |
| Template | The next placeholder (**Go to Next Placeholder**) |
| Problems | The Problems panel, filtered to the page |
| Size | Nothing; its tooltip says which build is largest, and `page-size`'s limit |
| A tested example | Its `@snippet` line |

A row with nothing to say isn't shown: no review field, no template, no permalinks. "Missing" is shown only for a description, since `page-description-missing` reports it.

**Size** is the largest of the page's builds, and says which. It's shown always, not only over the limit, so a page is seen growing toward it.

### The request

`ascribe/page`'s `about` gains `description`, `review`, `availability`, `builds`, `listing`, `permalinks`, and `template`; it gains `health` (`size`, `examples`) and `includes`. All of it is cursor-free, kept as phase 1 keeps `about`, and made again when the page, its includes, `ascribe.toml`, or a source's observations change.

The Problems row is VS Code's diagnostics for the file, counted in the extension; the server adds nothing for it.

**One answer for each fact.** Where a row says something an output also writes, the server takes it from the code that writes the output, not a second reading of the model: the listing and where it comes from are what `pages.json`'s `listing` and `listing_from` would be for the page, the size is `page-size`'s measure, and the builds are the builds' own. A test of the server compares each with the output for the same page, so the view and the output can't disagree.

### The inventory

Each page in `ascribe/inventory` gains `builds`, the builds that publish it, for phase 5's dimming. It's the same set About's Builds row shows, from the same place in the server.

## Tasks

1. The rows' data in `ascribe/page`, with tests of the server for each row, and for a row that isn't shown.
2. Each page's builds in `ascribe/inventory`.
3. The types in `shapes.rs`, blessed.
4. The rows in the view, what each opens, and its tooltip, with unit tests.
5. An integration test on `examples/quill` and on `examples/tested`.
6. The docs: the editor page's This page section, each row and what it opens; links to it from the reference for review dates (`content-model/index.md#54-field-roles`), listing, permalinks, `page-size`, and templates; `changes/editor/this-page-whole.md`; and the surface map.

## Out of scope

- External links' status, word counts, and reading times (README, "Carved out").
- Changing anything from a row but by opening the place to change it.

## Acceptance criteria

- The goal's rows, for a page of Ascribe's docs, in a test of the server.
- With `ASCRIBE_TODAY` past a page's review date, About says overdue, and `ascribe check` reports `review-overdue` for the same page.
- Size agrees with `page-size`'s measure of the same build, and Listing with `pages.json`'s `listing` and `listing_from` for the same page, in tests that build the project.
- A cursor move still asks one request, and phase 1's measurement still holds.

## Verify

```sh
cargo test -p ascribe-lsp --locked
ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes && git diff --stat
pnpm --filter ascribe-vscode test
```

## Commits

1. "Say whether a page is published, listed, and due, in This page"
2. "Show a page's health and what it includes"
