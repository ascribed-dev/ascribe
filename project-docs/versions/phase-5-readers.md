# Phase 5: What a reader and a crawler are told

Part of [Versions](README.md). Needs phases 3 and 4.

## Goal

An older line says what it is and points at the current one, to a person, a search engine, and an agent. A page in `site@3.4` has a banner that names its version and links to the same page in the current line. Its canonical link points there. The sitemap, `llms.txt`, and the page's Markdown agree with both.

The best-documented reader problem in the research is a search engine sending people to an old version. After phase 2 an older line's pages each claim to be canonical, so this phase is what makes a window safe to publish.

## Context

- The [proposal](../proposals/versions.md): "What every line knows about the others".
- Phase 1's decision 9.
- The [search and answers plan](https://github.com/ascribed-dev/ascribe/blob/5b22286fd2f370e796219e68a7c9d7cdb2ba8acc/project-docs/search-and-answers/README.md): decisions 5, 15, 17, 19, 20, and 21. A record's keys are added and none change.
- `docs/content/contracts/output-layout.md`: `_ascribe/pages.json`.
- `packages/astro/src/Head.astro`, `records.ts`, and `root-files.ts`: the head, the sitemap, and `robots.txt`.
- `crates/ascribe-resolve/src/llms.rs` and `crates/ascribe-emit/src/plain/`: `llms.txt` and the Markdown version's first lines.
- `ascribe agents index`, in the command reference.

## Design

### Canonical

A page's `canonical` in its record is, in order:

1. Its own address, in the current line.
2. In an older line, the address in the current line of the same page, or of the page that replaced it.
3. Its own address, when the current line has neither.

It's absent when `[consumer] site` isn't set, as now. `Head.astro` writes what the record says and needs no change beyond reading it.

### The rule for an older line

`[listing] older-lines`, as phase 1's decision 9 has it: `listed` by default, and a project may set `unquoted` or `unlisted`. It's a key at the table's top, beside `default`, `folders`, and `types`, and in the order of rules it sits between the site's `default` and a folder or a type. It can only restrict, a page's record says its listing came from it, and a page in an older line that sets a looser value is `listing-loosened`.

It applies to every live line but the current one, of every versioned build.

### The sitemap, and `robots.txt`

The site's one sitemap lists a page when it's listed and its canonical address is its own. So an older line's page is in it only when the current line has no such page. Nothing is added to `robots.txt`: an older line is kept out of an index by its pages, not by a rule over its folder, which would also stop a crawler reading the canonical link.

### The banner

`@ascribed/astro/VersionBanner.astro`, given a page's entry. On the current line it writes nothing. On an older line:

> Documentation for Self-managed 3.4. Support ends on 31 January 2027. [See this page for 3.5.](/install/)

- The date comes from `ends`, and the sentence is left out without one.
- The link goes to the page's canonical address when that isn't its own, and to the current line's home otherwise, with words that say so.
- The words are properties of the component, each with a default, so a site can translate them.

`Versions.astro` gains the lines that don't publish the page, shown and not linked, with a word saying so, and the archived lines that have an address.

A site that isn't Astro has everything these read in `versions.json` and the page records, and the guide says which keys.

### For agents

- **The integration publishes each line's plain output** at that line's base path, which phase 2 held back to the current line.
- **`llms.txt`** is one for each line, in the line's output, with its heading naming the line. The current line's is at the base path, as now. An older line's lists that line's pages and opens with a line pointing at the current one.
- **A page's Markdown** opens, after the pointer to `llms.txt`, with a line saying which version it documents and its status, on every page of a versioned build: "Self-managed 3.4. The current version is 3.5: <address>". On the current line it says the version alone.
- **`ascribe agents index`** takes a line's name with `--build`, and indexes the current line without one.
- An older line that isn't listed is left out of all three, as any page that isn't listed is.

The Web Documentation Delivery Spec's checker runs against an older line's output in the end-to-end tests, as it does against the current one.

## Tasks

1. `canonical` across lines, in the page records, with the contract.
2. `[listing] older-lines`, with its rules and the record's source.
3. The sitemap's rule.
4. `VersionBanner.astro`, and the rest of `Versions.astro`.
5. `llms.txt` for each line, the Markdown version's line, and each line's plain output published.
6. `ascribe agents index` for a line.
7. `examples/astro-site`: the banner, and end-to-end tests for the head, the sitemap, and the outputs for agents in both lines.
8. The reference, the contracts, the Astro guide and the versions guide, entries in `changes/`, and the decisions recorded.

## Out of scope

- A banner on an archive. Phase 8 writes its own, in words that don't age.
- A response header for an older line's Markdown. The search and answers plan's host phase writes headers from the records, and gets these for nothing.

## Acceptance criteria

- A page in `site@3.4` of the example has a canonical link to its address in the current line, and the page that was replaced has one to its replacement.
- The sitemap lists each page of the current line once and no page of 3.4 that the current line has.
- With `older-lines = "unlisted"`, every page of 3.4 has `noindex`, none is in the sitemap or an `llms.txt`, and a page there that sets `listing: listed` is an error.
- The banner names the line, gives the date when there is one, and links to the same page.
- A page's Markdown in 3.4 says which version it is and where the current one is.
- A build that isn't versioned is byte for byte the same.

## Verify

```sh
cargo test --workspace --locked
pnpm test
pnpm --filter @ascribed/example-astro-site test:e2e
node scripts/compare/outputs.ts --base main --accept 'quill: *'
```

## Commits

1. "Point an older line's canonical links at the current one"
2. "Let a project keep its older lines out of an index"
3. "Say which version a page is, and where the current one is"
4. "Give each line its outputs for agents"
