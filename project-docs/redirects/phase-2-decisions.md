# Phase 2: Decisions

Part of [Redirects](README.md). Needs phase 1.

## Goal

Every lost address has a fix: forward it, retire it, or record git's renames. A retired address serves a page saying why, and agents are told how.

```console
$ git mv guides/setup.md start/setup.md && git rm reference/options.md
$ ascribe check --base main
warning[address-lost]: guides/setup.md was published at /docs/guides/setup and /docs/guides/setup.md, and nothing publishes them now
  = fix: git sees it renamed to start/setup.md: ascribe addresses move --from-git
warning[address-lost]: reference/options.md was published at …
  = fix: forward it to the page most like it, guides/configure.md: ascribe addresses move reference/options.md guides/configure.md
  = fix: retire it: ascribe addresses retire reference/options.md --reason "…"
$ ascribe addresses move --from-git
Recorded 1 move git sees: guides/setup.md → start/setup.md.
```

## Context

- README decisions; phase 0's diagnostics table, rows for phase 2; the proposal's "The file's other checks" and "Agents".
- Phase 1's comparison, file, and commands.
- `crates/ascribe-lsp/src/retire.rs` and `code_action.rs`: the editor's fix for `permalink-removed`, the pattern for `address-lost`'s.
- `crates/ascribe-lsp/src/rename.rs`, `Renaming::Id`, and `refactor.rs`: the id rename.
- `crates/ascribe-cli/src/mcp/tools.rs`, and `crates/ascribe-cli/src/agents/` for the rules `ascribe agents sync` writes and the skill.
- `crates/ascribe-diff/src/git.rs` for reading renames between two revisions.

## Design

### The fixes

`address-lost`'s fixes, in `ascribe check --fix` where they're safe and in the editor's code actions, each an edit to `ascribe.addresses`:

- **Forward to…:** for a page, a pick of pages, offering first the one most like it, by its headings and text against the base's page; for an `#id`, a pick of headings, the same id on another page first. Not safe: it's a choice.
- **Retire with a reason…:** the entry with the placeholder reason, which is reported until replaced, as `[permalinks.retired]`'s is. Not safe.
- **Record git's renames:** offered when git sees the source renamed; safe, since git's rename is the decision.

### `--from-git`

`ascribe addresses move --from-git` records every loss git sees as a rename between the base and the working tree, one entry per page; a rename git sees at below its similarity threshold isn't one.

### The id rename

`Renaming::Id`, the editor's F2 on an `@id` and `ascribe rename <file>#<id> <new>`, adds the `<file>#<id>` entry in the same edit, as a move does. A permalink's id still can't be renamed.

### The file's other checks

Each row of phase 0's table for phase 2, as the proposal's "The file's other checks" says: a value that isn't a page; a key whose source is back, with a fix that removes the entry, or whose address another source has taken; an entry the base's file doesn't have whose key the base never published, with a fix that removes it, and an entry already in the base's file never pruned; a key twice, naming both values; a chain, with a fix that flattens it, and retiring or forwarding a page entries lead to rewriting them in the same edit; an `#id` collision; the file deleted while the base has it; and many addresses forwarded to one index or the home page.

### The `gone` page

Each `gone` address in `redirects.json` gets a page: the integration adds one route per retired address, with the reason and a link to the site's home, `noindex`, and `_ascribe/host/_redirects` gains a rule serving it at the old address with a 404. The page's markup is the integration's; another site generator reads `gone` and writes its own.

### Agents

The rules `ascribe agents sync` writes and the skill say: move a page with `ascribe rename`; record a reorganization made with `git mv` with `ascribe addresses move --from-git`; forward or retire a page deleted on purpose. Written only for a project with `ascribe.addresses`. The MCP server's `ascribe_addresses_move` and `ascribe_addresses_retire` return the edit, as phase 0 says, and its `ascribe_check` over a project reports `address-lost`.

## Tasks

1. The fixes, in the check and the editor, with conformance cases and the editor's code action tests.
2. `--from-git`, with repository tests: a rename, a rename and an edit, a copy, a delete.
3. The id rename's entry, through F2 and `ascribe rename`.
4. The file's other checks, each a diagnostic with its conformance case.
5. The `gone` page and its rule, with the integration's tests and the end-to-end test asking for a retired address and getting the page with a 404.
6. The agents' rules, the skill, and the two tools, with their tests; the surface map regenerated.
7. The docs: the redirects page's fixes, `--from-git`, the file's checks, and the `gone` page; `docs/content/agents/instructions.md`'s rules for moving pages; `changes/language/redirects-decisions.md` and `changes/agents/redirects.md`.

## Out of scope

Everything for phases 3 and 4.

## Acceptance criteria

- The goal's commands, in a test.
- Every phase 2 diagnostic has a conformance case.
- `examples/astro-site`'s end-to-end test gets a retired address's page with a 404.
- An agent's session in a test moves a page with `git mv`, runs `ascribe_check`, applies `ascribe_addresses_move`'s edit, and passes.

## Verify

```sh
cargo test -p ascribe-check addresses
cargo test -p ascribe-cli addresses
cargo test -p ascribe-lsp addresses
pnpm --filter @ascribed/example-astro-site test:e2e
```

## Commits

1. "Fix a lost address: forward it, retire it, or record git's rename"
2. "Record a heading id's rename in ascribe.addresses"
3. "Check ascribe.addresses' entries"
4. "Serve a retired address's page with a 404"
5. "Tell agents how to move and retire pages"
