# Phase 1: The thread

Part of [Redirects](README.md). Needs phase 0's decision, and [#903](https://github.com/ascribed-dev/ascribe/issues/903).

## Goal

A project with `ascribe.addresses` records its moves, CI stops a page deleted with no decision, and Netlify forwards the old address.

In a repository holding `examples/astro-site`, with `[consumer] host = "netlify"`:

```console
$ ascribe addresses init
Wrote ascribe.addresses. Moves are recorded from now on.
$ git commit -am "Keep the addresses we publish" && git switch -c restructure
$ ascribe rename guides/my-setup.md guides/setup.md
Renamed guides/my-setup.md to guides/setup.md, and changed 3 links.
Recorded the move in ascribe.addresses.
$ git commit -am "Rename the setup guide"
$ ascribe check --base main --deny-warnings
No problems.
$ git rm guides/teams/share-a-loom.md && ascribe check --base main --deny-warnings
warning[address-lost]: guides/teams/share-a-loom.md was published at /docs/guides/teams/share-a-loom and /docs/guides/teams/share-a-loom.md, and nothing publishes them now
  = fix: forward it: ascribe addresses move guides/teams/share-a-loom.md <page>
  = fix: retire it: ascribe addresses retire guides/teams/share-a-loom.md --reason "…"
```

`ascribe build` writes `_ascribe/redirects.json` with the pair `/docs/guides/my-setup` → `/docs/guides/setup`, and the Markdown version's, and `_ascribe/host/_redirects` with both as 301s. `astro build` places `_redirects` at the site's root. The exact output is phase 0's.

## Context

- README decisions 1 to 11, and phase 0's decision file.
- The proposal's "What the check compares", "Moving a page records the move", and "What a build writes".
- `crates/ascribe-model/src/permalinks.rs` and `crates/ascribe-check/src/permalinks.rs`: the list read, written, and compared, and how `PERMALINKS_FILE_ID` gives its diagnostics a file.
- The spike's listing, in the README's "What exists today": `for_build`, `resolve_build`, `AstroRouter::from_consumer`, `pages::records`, `headings()`.
- `crates/ascribe-diff/src/gitfs.rs`, `Repository::discover`, `Repository::base`, `Revision::read`; `crates/ascribe-cli/src/commands/diff.rs` for `--base` and `--base-exact`.
- `crates/ascribe-lsp/src/refactor.rs`, `will_rename`; `crates/ascribe-lsp/src/rename.rs`, `Renaming::File`.
- `crates/ascribe-emit/src/site/headers.rs`, and `packages/astro/src/headers.ts` for how the integration merges every project's rules and treats a site's own file.
- `docs/content/contracts/output-layout.md` §1.4 for the shape of a section about a generated file, and §3 for the manifest.

## Design

### The addresses

`ascribe-resolve` lists a resolved build's addresses: each page's route, its Markdown version's, and each explicit id's `route#id`, each with its source (the page, or the file the `@id` is written in). It's what the page records already compute, so it costs one pass. Two sources at one address in a build is a bug the spike didn't see; the listing returns both, and the check reports neither.

### The file

`ascribe-model` reads and writes `ascribe.addresses` as phase 0 says, with `addresses-file-invalid` at what can't be read. A key's address is found by the build, not stored: a source path's addresses are what the base published for it.

### The comparison

`ascribe-check` gains a function taking the working tree's project, `ascribe.addresses`, and the base's addresses, by build name. An address the base has, the working tree doesn't, and no entry accounts for (`[moved]` and `[retired]` here; `[former]` in phase 3), nor an availability mark that took it out of this build while another build publishes it, is lost. It groups what's lost by source: `address-lost` for a page and its Markdown version, `anchor-lost` for an id. A build only in the base is `addresses-build-gone`.

A `[moved]` value names a source, so the working tree's addresses for it are where the redirect leads. A value whose source no build publishes leaves its key lost here; phase 2 makes it an error.

### The base

`ascribe check` reads the base when `ascribe.addresses` exists, through `ascribe-diff`, as `ascribe diff` does, with `--base` and `--base-exact`. It's read on a thread beside the rest of the check, as `diff` does, and listed with the library, with no output written. When it can't be read, the rest of the check runs and `addresses-base-unreadable` says why: no repository, a revision that isn't there (a shallow clone's merge base, a force-push's `before`), or no project at the base, which says nothing was compared and isn't a warning.

The CI how-to (`docs/content/checks/ci.md`) gains the base for a pull request, `--base origin/$GITHUB_BASE_REF`, and for a push, `--base ${{ github.event.before }} --base-exact`, with `fetch-depth: 0`, as its `diff` and `drift` sections say.

### The move

`refactor::will_rename` adds the entry to `ascribe.addresses` in the same edit, for a page or a fragment that moves, when the file exists. A move back to an entry's key removes the entry. The editor's Explorer and Pages moves go through the same function, so they record too; phase 4 tests what's the editor's own. A file that can't be read or edited doesn't stop the move's other edits.

`ascribe addresses init`, `move`, `retire`, and `list` in `crates/ascribe-cli/src/commands/addresses.rs`, each writing through the model's writer.

### The build

`crates/ascribe-emit/src/site/redirects.rs` writes `redirects.json` from the sources and `ascribe.addresses` alone: for each `[moved]` entry, each address the key's source published, as the working tree's routing makes it, to its value's address; the Markdown version's beside it; flat; one pair per address; a pair whose `from` and `to` agree once case is ignored left out. `gone` from `[retired]`. The base isn't read: a source's old address is made from the key's path by today's routing, which is why a settings change needs phase 3's `[former]`.

With `[consumer] host = "netlify"`, the site emitter writes `_ascribe/host/_redirects` from it and `_ascribe/host/_headers` from `headers.json`, and `[consumer] headers` becomes the error phase 0 gives it, in the content model and its contract (§15).

### Astro

`@ascribed/astro` reads each project's `_ascribe/host/` files and writes the site root's `_redirects` and `_headers`, merged; it checks each pair's `from` against every project's routes, an error when another project publishes it. A site's own `public/_redirects` is merged after Ascribe's rules. `headers.ts`'s writing of `_headers` from the JSON goes: the compiler's file replaces it, with the same rules for a site's own `_headers`.

### The thread's tests

- **The seam:** in a repository a test makes, the move is committed, the base moves past it, and `redirects.json` still has the pair and the check still passes.
- **The bases:** a shallow clone with no merge base, a `--base-exact` revision that isn't there, and a base with no project, each giving its answer above.
- **quill:** the comparison across `examples/quill`'s builds and lines, with no host: a release's `site@3.5` loses nothing, matched by name.
- **Netlify:** `examples/astro-site`'s end-to-end test builds with Astro's base at `/`, and finds `_redirects` at the root with the pair, in the published slash form.

## Tasks

1. The addresses, in `ascribe-resolve`, with tests: a page, its Markdown version, an explicit id, a fragment's id once, every build of quill.
2. `ascribe.addresses` in `ascribe-model`: parse, write, and `addresses-file-invalid`, with tests.
3. The comparison and `address-lost`, `anchor-lost`, `addresses-build-gone`, in `ascribe-check`, with the base given as data; conformance cases for each, and `addresses-file-invalid`.
4. The base in `ascribe check`, `--base`, `--base-exact`, and `addresses-base-unreadable`, with the repository tests.
5. The move's entry in `refactor::will_rename`, with tests through `ascribe rename` and through `willRenameFiles`.
6. `ascribe addresses init`, `move`, `retire`, and `list`, their JSON and schemas, and their output tests.
7. `redirects.json`, the host files, `[consumer] host`, and `headers` as an error, with emit tests; `node scripts/compare/outputs.ts --base main` shows only the new files, the manifests, and `_headers` moving, each named with `--accept`.
8. `@ascribed/astro`'s placing and merging, with its tests, and the end-to-end test.
9. The contracts: SPEC's section for `ascribe.addresses` beside §7.5, the output layout's section for `redirects.json` and `_ascribe/host/`, the content-model contract's `[consumer] host`, a decision file for each, #457b amended, and the search and answers plan's decisions 9 and 35 marked superseded.
10. The docs: a page, `docs/content/publishing/redirects.md` ("Keep the addresses you've published"), `@available: ascribe planned`: turning it on, what a move records, what the check reports and from which base, what the build writes, Netlify, and that a lost `#id` is advice for now; the CI how-to's base; `docs/content/publishing/site.md`'s headers section for `host`; `ascribe.nav` places the page; `changes/language/redirects.md`.

## Out of scope

Everything in the README's "Left out" table. A lost address in phase 1 is answered by the two commands the warning names.

## Acceptance criteria

- The goal's commands, in a test.
- The seam's test and the three bases' tests pass.
- quill's builds and lines compare with nothing lost.
- `examples/astro-site`'s end-to-end test finds the pair in the site's `_redirects`.
- `node scripts/compare/outputs.ts --base main` changes only what task 7 names.

## Verify

```sh
cargo test -p ascribe-resolve addresses
cargo test -p ascribe-check addresses
cargo test -p ascribe-cli addresses
cargo test -p ascribe-emit redirects
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test:e2e
node scripts/compare/outputs.ts --base main
```

## Commits

1. "List each build's addresses"
2. "Read and write ascribe.addresses"
3. "Report an address the base published that nothing accounts for"
4. "Record a move in ascribe.addresses"
5. "Write redirects.json and Netlify's files, from [consumer] host"
6. "Place and merge the host's files in @ascribed/astro"
7. "Keep the addresses you've published: the page, the contracts, and the decisions"
