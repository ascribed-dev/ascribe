# Phase 2: The thread

Part of [Permalinks](README.md). Requires phase 1. Rust, `@ascribed/astro`, and the docs. This is the plan's steel thread: the thinnest path through every layer, shipped and kept.

## Goal

A heading marked as a permalink has an address on the site, and Ascribe's own binary links to one:

```sh
$ ascribe check --help | tail -1
Documentation: https://ascribed-dev.com/go/cli-check
```

Following that address opens the section about `ascribe check`, wherever the page is.

One heading, one build, one command's help. Everything it takes is built to the standard of the rest, and nothing in it is thrown away.

## Context

- [The thread](README.md#the-thread): the layers, and the seam this phase exists to test.
- Phase 1's decisions: what a name may look like, that it's unique within what a build publishes, and the three names (`permalink`, `[consumer] permalink-path`, `_ascribe/permalinks.json`).
- `crates/ascribe-core/src/schema.rs`: `Builtin::Id`'s schema, which declares no attributes, and `Builtin::Include`'s beside it, which declares a boolean one.
- `SPEC.md` §4.1 (`@id`) and §5.5 (a heading's source id and page id). A heading in a fragment is published on every page that includes it.
- `crates/ascribe-resolve`: the source index's headings, and how a build resolves a page's ids and routes. `crates/ascribe-check/src/page/collect.rs`: `id-duplicate`, the nearest existing check.
- `crates/ascribe-emit/src/site/mod.rs`: `SCHEMA_PATH` and `FILES_DIR`, what the site emitter already writes under `_ascribe/`, and how the manifest lists them. `docs/content/contracts/output-layout.md`: the contract.
- `packages/astro/src/index.ts`: the hooks the integration uses. It adds no route today. `packages/astro/src/project.ts`: how it reads `[consumer]`, and the check that Astro's `base` equals the base path. `packages/astro/src/files.ts`: serving something under the base path in `astro dev` and copying it in `astro build`.
- `examples/astro-site/test/e2e/`: the end-to-end suite the Astro job runs against a real build.
- `crates/ascribe-cli/src/cli.rs`: `docs_page!`, and the test that holds `docs_site!` to `docs/ascribe.toml`.
- `project-docs/checklists.md`: the lists for a change to the language, to an output, and to the content model. This phase is all three.

## Design

### The language

`@id` takes one attribute, `permalink`, a boolean that's false when absent:

```markdown
## `ascribe check`
@id {permalink=true}: cli-check
```

The id is the heading's anchor, as before, and now also its permalink. `ascribe fmt` writes the attribute in canonical form like any other. Nothing else about `@id` changes.

### What a build knows

For each build, the permalinks it publishes: each name, the page it's on, and the anchor. A marked heading in a fragment is published on every page that includes the fragment, so it's found on each.

One diagnostic, an error, at page level: `permalink-duplicate`, when a build publishes one name twice. It names both places. A fragment with a marked heading, included by two pages, is the case most people will meet first, and the message says so when that's the cause.

### What a build writes

`_ascribe/permalinks.json`, in the site output, listed in the manifest:

```json
{
  "version": 1,
  "path": "/go/",
  "permalinks": {
    "cli-check": { "route": "/reference/cli/", "fragment": "cli-check", "source": "reference/cli.md" }
  }
}
```

- An entry is an object, so later phases add keys to it (a page has no `fragment`; a retired name has a reason) without changing these.
- `path` is where the addresses are published, from `[consumer] permalink-path`. It's a path from the site's root, and is `go/` under the base path when unset.
- The file is written when the build has no permalinks too, with an empty table, so a consumer doesn't have to tell "none" from "an older Ascribe".

### The address

`@ascribed/astro` adds a route at `<path>[id]` and builds one page for each entry, in `astro build` and `astro dev`. The page:

- sends the reader to the route and anchor at once, without a script;
- has a link to the same place in its body, for a reader whose browser doesn't follow;
- is marked `noindex`, with a canonical link to where it leads.

The path must be under Astro's `base`, since that's all an Astro site serves. The integration says so when it isn't.

**This is the seam.** If a route added by the integration can't do all of this (in both commands, with a base path, with the anchor kept, on a static host), stop and report before going on. The alternative is the emitter writing the forwarding pages as files itself, which is a different contract.

### Ascribe's use of it

- `docs/content/reference/cli.md`: the heading for `ascribe check` is marked, with the id `cli-check`. Links to it inside the docs follow the new anchor; they're checked.
- `docs/ascribe.toml` sets `permalink-path = "/go/"`, so the address doesn't move when the base path does.
- `ascribe check`'s help link is built from the site and an id, by a macro beside `docs_page!`. The other commands keep `docs_page!` until phase 3.
- A test builds `docs/` and checks that the id is in the list.

## What this phase doesn't handle, said where a user will look

The directive reference says, while it's true: a page can't have a permalink yet, and nothing reports a permalink that's removed.

## Tasks

1. The attribute on `@id`, in the schema and `SPEC.md`, with conformance cases: marked, unmarked, `permalink=false`, and a value that isn't a boolean.
2. `[consumer] permalink-path`, in the model, its reference, and its contract, with validation (it starts with `/`).
3. Each build's permalinks in the resolver, and `permalink-duplicate`, with conformance cases: two headings on two pages; one marked heading in a fragment two pages include; the same name in two pages that no build publishes together, which isn't an error.
4. `_ascribe/permalinks.json` from the site emitter, in the manifest and the output-layout contract.
5. The route in `@ascribed/astro`, with unit tests, and end-to-end tests in `examples/astro-site`: the address forwards to the heading in a built site and under `astro dev`; with a base path; and the integration refuses a path outside `base`.
6. The docs' own heading, `permalink-path`, the help link, and the test that ties the id to the list.
7. `docs/content/reference/directives.md` (`@id`), `content-model.md` (`[consumer]`), `guides/astro.md` (the route, and what a site with another generator does with the file), `CHANGELOG.md`, and the decision in `project-docs/decisions.md`.

## Out of scope

Pages; the binary's other links; the committed list; retiring; aliases; anything a link carries; a page for a name nobody declared. Each is a later phase.

## Acceptance criteria

- `ascribe check --help` ends with `https://ascribed-dev.com/go/cli-check`.
- In `examples/astro-site`, built and served, a marked heading's address opens its page at the heading. The end-to-end test does exactly this.
- A project with no permalinks builds the same site output as before, plus the list with an empty table. The outputs comparison shows that file and nothing else.
- The night after this merges, the address in the first criterion works on the live site. Until then it doesn't: the site gets the integration from the canary. That's accepted (README decision 9), and the pull request says when it was checked.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test:e2e
cargo run -p ascribe-cli -- check --help | tail -1
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Let an @id be a permalink"
2. "Report a permalink a build publishes twice"
3. "Write each build's permalinks beside its site output"
4. "Give each permalink an address in an Astro site"
5. "Link ascribe check's help to a permalink"
