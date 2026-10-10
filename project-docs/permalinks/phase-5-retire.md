# Phase 5: Retiring and aliases

Part of [Permalinks](README.md). Requires phase 4. Rust, `@ascribed/astro`, and the language server.

## Goal

- **A permalink can be ended on purpose.** Its address goes on leading somewhere, or says the content was removed and why.
- **One place can have several names,** so each thing that links there can be repointed alone.

Both are entries in `ascribe.toml` that give a name with no heading of its own somewhere to lead.

## Context

- README decision 6: deliberate and recorded, never blocked.
- Phase 4's file, `permalink-removed`, and commands. Phase 2's list in the site output, whose entries are objects so they can gain keys, and the integration's route.
- `docs/content/reference/content-model.md` and `docs/content/contracts/content-model.md`: how a section of `ascribe.toml` is specified.
- `crates/ascribe-lsp`: the content-model actions (making a phrase, adding a glossary term) already edit `ascribe.toml` in place with `toml_edit`, keeping its comments. Retiring writes to the same file the same way.
- The [content checks plan](../content-checks/README.md)'s acknowledgements: a required reason, and the same wording for why.
- `docs/content/_generated/diagnostics-retired.md`: Ascribe's own retired names, waiting for this.

## Design

### In `ascribe.toml`

```toml
[permalinks.retired]
rotate-keys = { to = "guides/security.md", reason = "Merged into the security guide" }
legacy-sync = { reason = "The sync agent was removed in 4.0" }

[permalinks.aliases]
agent-settings = "settings"
cli-settings = "settings"
```

- **`reason` is required** and not empty.
- **`to`** is a page, or a page and a heading's id, written as a link is. It's checked as one. It may be a permalink's own target, but not a retired name.
- **An alias** names a permalink that exists. Its target may be another alias only if that ends at a real one; a loop is an error.
- **A name is one thing:** on a page or heading, an alias, or retired. Two at once is an error that says which two.

### What changes downstream

- **`check`:** a retired name no longer raises `permalink-removed`. That's the proper way to quiet it.
- **The committed file** lists aliases and retired names too, each marked as what it is, so `verify` can tell a product "this name was retired", with the reason, which is a different message from "this name never existed". Both exit 1.
- **The build's list** gains entries for them: an alias has its target's route, and a retired name has its reason and, with `to`, a route.
- **The address:** an alias forwards like any permalink. A retired name with `to` forwards. One without shows a short page: what was here was removed, the reason as written, and a link to the docs' home. The reason is the author's text, shown as text.

### Retiring many

```sh
ascribe permalinks retire rotate-keys --reason "Merged into the security guide" --to guides/security.md
ascribe permalinks retire --page guides/legacy.md --reason "The sync agent was removed in 4.0"
ascribe permalinks retire --under guides/legacy/ --reason "…"
```

It writes the entries into `ascribe.toml`, in place, and says what it wrote. It retires only names that are already gone from the content, or that the named page or folder holds, and refuses one that's still declared elsewhere.

### In the editor

`permalink-removed` offers "Retire this permalink…", which writes the entry and leaves the cursor in the reason. It's a fix that needs a person's sentence, so it's marked as one to review, not one to apply unasked.

### Ascribe's own

The retired diagnostics' codes are retired here, each with its reason from the registry, forwarding to the page that lists them.

## Tasks

1. The two tables: the model, validation, the reference, and the contract, with conformance cases for each rule above.
2. `check`, the committed file, and `verify` for retired names and aliases.
3. The build's list and the integration: forwarding for both, and the removed page, with end-to-end tests in `examples/astro-site`.
4. `ascribe permalinks retire`, with tests that `ascribe.toml`'s comments and order survive.
5. The editor's fix, with a scenario test.
6. Retire Ascribe's retired diagnostics.
7. The guide: when to move, when to alias, when to retire. `CHANGELOG.md`, and the decision for the content model's new tables.

## Out of scope

Bringing a retired name back (delete its entry and declare it again; nothing needs building); reserving a name before its page exists, unless phase 1 decided otherwise.

## Acceptance criteria

- Removing a marked heading and retiring it leaves `ascribe check --deny-warnings` passing, and its address leads where `to` says, or shows the reason.
- `ascribe permalinks verify` exits 1 for a retired name and says it was retired, with the reason.
- Retiring every permalink on a page takes one command and one reason.
- An alias repointed to another permalink changes where its address leads and nothing else.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/example-astro-site test:e2e
target/debug/ascribe check --deny-warnings --config docs
```

## Commits

1. "Retire a permalink, with a reason"
2. "Let one place have several permalinks"
3. "Add ascribe permalinks retire"
4. "Offer to retire a removed permalink in the editor"
