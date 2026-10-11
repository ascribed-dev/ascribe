# Phase 3: Levels

Part of [Search and answers](README.md). Requires phase 2. Rust, with a small part in `@ascribed/astro`, and the first work in the language server and the extension: completion, hover, and a quick fix.

## Goal

A project says how its pages are treated once, for many pages, and can rely on it:

```toml
[listing]
default = "listed"

[listing.folders]
"internal/" = "unlisted"

[listing.types]
changelog = "unquoted"
```

No page under `internal/` can list itself. One that tries is an error that names the rule it's under. And a page that's `unquoted` is listed with its text not offered for quoting.

The table's name and keys above are a sketch. Phase 1 decided the key and its values; this phase decides how the levels are written, by a recorded decision, before it writes them.

## Context

- README decision 5, and phase 1's decision 1: the three values are an order, from `listed` through `unquoted` to `unlisted`, and narrower may only move toward `unlisted`.
- How the content model already says something for many pages: `[types.<name>]` with its `files`, and `[checks]`. Use what's there before inventing a way to name a set of pages.
- `crates/ascribe-model`: where a project-wide rule is loaded and validated, and how a rule that can't hold is reported (`model-` diagnostics).
- The record, as phase 2 built it, and the three outputs that read it.
- `crates/ascribe-lsp/`: completion and hover for a reserved frontmatter key, as `available` and `variant` have them, and how a diagnostic carries a fix. `project-docs/checklists.md`, "A change to the language": the list of what the server and the extension owe a new key.
- The [versions proposal](../proposals/versions.md): a version's rule will sit between the site and a type. Leave the place, and build nothing for it.

## Design

### Where the setting can be made

Four levels, widest first: the site, a version (not built here), a type or a folder, the page. Each page's listing is the strictest that applies to it.

- **A rule for a folder** applies to every page under it, by path from the content root.
- **A rule for a type** applies to every page of the type.
- **A page in a folder with a rule, of a type with a rule,** gets the stricter of the two. They don't have an order between them, and don't need one.

### Restrict, never loosen

- A page under a rule may set a value as strict or stricter. A looser one is an error at the page, not advice: the project contradicts itself. The message names the rule and where it's written.
- A page that repeats its rule's value is fine.
- The site's default may be stricter than `listed`, for a site that isn't public yet. Then nothing in it can be listed, which is the guarantee working, and the message says so plainly, since it will surprise someone.

### The record says why

Each record gains where its listing came from: the page, or the rule's place in `ascribe.toml`. It's what phase 8's table shows, and what an error quotes.

### `unquoted`

A page that's `unquoted` gets `<meta name="robots" content="nosnippet">` from `Head.astro`, and stays in the sitemap and `llms.txt`. The reference says what that asks and of whom: it limits what a search result, or an AI answer inside a search product, may quote. It asks nothing of a crawler gathering training data.

### In the editor

The key completes its three values, and hovering it says which rule the page is under. The error has a quick fix where there's one: remove the page's own setting.

## Tasks

1. The decision on how levels are written, recorded.
2. The rules in the model, with validation: a folder that doesn't exist, a type that doesn't exist, a value that isn't one of three.
3. The strictest-wins resolution, and the error, with conformance cases: each level alone; a folder and a type together; a page that tightens; a page that loosens; a site default of `unlisted`.
4. Where it came from, in the record and its schema.
5. `nosnippet` in `Head.astro`, with its test.
6. Completion, hover, and the quick fix.
7. The content model reference, the Astro guide, `CHANGELOG.md`, the map of the surface, and the decision.

## Out of scope

A version's rule. `data-nosnippet` on part of a page, and `max-snippet`. Rules by anything but a folder or a type.

## Acceptance criteria

- With `"internal/" = "unlisted"`, every page under it is `noindex`, out of the sitemap and `llms.txt`, with no change to any page.
- A page under it with `listing: listed` fails `ascribe check` with an error that quotes the rule's line in `ascribe.toml`.
- The test that reads the record against the outputs passes with rules at every level.
- A project with no rules builds what it built after phase 2: `node scripts/compare/outputs.ts --base main` shows the record's new field and nothing else.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Let a project set how a folder's or a type's pages are listed"
2. "Refuse a page that loosens the rule it's under"
3. "Ask search engines not to quote a page"
