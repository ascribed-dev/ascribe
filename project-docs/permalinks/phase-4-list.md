# Phase 4: The list

Part of [Permalinks](README.md). Requires phase 3. Rust, with a quick fix in the language server.

## Goal

- **A permalink that goes away is noticed.** `ascribe check` reports it, and a reviewer sees it in the pull request.
- **A product can check its own links.** `ascribe permalinks verify`, run in the product's CI, fails on a name the docs don't have.

Both come from one file, committed with the docs. It's the last new mechanism in the plan: a second file that follows the content, after `ascribe.lock`.

## Context

- README decisions 6 and 7.
- `crates/ascribe-model/src/lock.rs`: how `ascribe.lock` is read, versioned, and reported on (`lock-invalid`). `crates/ascribe-sources`: where it's written. `ascribe check`'s diagnostics for a lock that's behind (`source-copy-changed` and its neighbors): the nearest model for "this file no longer matches".
- `crates/ascribe-cli/src/commands/sources.rs`: a command with subcommands, `--format json`, and a JSON shape in `crates/ascribe-cli/src/shapes.rs`.
- `crates/ascribe-lsp/src/code_action.rs`: quick fixes from a diagnostic's fixes.
- Phase 2's per-build permalinks. The list is for the project, so it's their union over every build.

## Design

### The file

`ascribe.permalinks`, beside `ascribe.toml`:

```toml
# Written by Ascribe. Don't edit it: run `ascribe permalinks sync`.
version = 1

[permalinks]
ASC036 = "reference/diagnostics.md#ASC036"
cli-check = "reference/cli.md#cli-check"
cli-reference = "reference/cli.md"
```

- One line for each name, in order, so two changes that add different permalinks merge without conflict.
- A value is where the name is declared, in the source: a file, and an id for a heading. A heading in a fragment names the fragment. It isn't a published address, which differs by build and is in each build's own list.
- It has a format version, as the lock does.

### What `check` reports

| Diagnostic | Level | When | Fix |
|---|---|---|---|
| `permalinks-list-behind` | warning | A permalink in the content isn't in the file, or is declared somewhere other than the file says | `ascribe permalinks sync` |
| `permalink-removed` | warning | A name in the file isn't in the content | Restore it, or (from phase 5) retire it |
| `permalinks-list-invalid` | error | The file doesn't parse, or has a version this `ascribe` doesn't know | |

- **Moving isn't removing.** A heading that moves to another page, or a page that's renamed, makes the file behind, not a permalink removed.
- **`permalink-removed` says what can't be seen:** "`rotate-keys` was a permalink. Links to it from outside these docs, such as from a product, aren't found by this check."
- Both warnings fail `ascribe check --deny-warnings`. That's intended: it's what stops a removal reaching `main` unnoticed. A project without the file isn't warned about anything, so the file is the opt-in, and `sync` creates it.

### The commands

```sh
ascribe permalinks sync                 # bring the file up to date
ascribe permalinks list                 # every permalink, and where it's declared
ascribe permalinks verify cli-check ASC036
ascribe permalinks verify --from ids.txt
```

- `sync` adds and moves entries. It never drops a name that's gone from the content: that stays, and stays reported, until it's restored or retired. So running `sync` can't be how a removal is hidden.
- `verify` needs only the file, not the content, so a product's repository can check against a copy of it. It exits 1 naming each id that isn't there. `--format json` for tools.
- `list` reads the content, and says which names the file doesn't have yet.

### In the editor

`permalinks-list-behind` carries a fix that runs the sync, so the file is one click from current. Nothing writes the file without being asked.

### Ascribe's own

The docs commit their list. Phase 3's test, which checked the binary's names against a fresh build, checks them with `ascribe permalinks verify` against the committed file instead: the same thing a product would run.

## Tasks

1. The file: its format, reading it in `ascribe-model`, and the three diagnostics with conformance cases (a name added; a heading moved to another page; a name removed; a file from a newer version).
2. `ascribe permalinks sync`, `list`, and `verify`, with their JSON blessed, and a test that `sync` keeps a removed name.
3. The fix on `permalinks-list-behind`, with a scenario test in the language server.
4. The docs' own `ascribe.permalinks`, and the test moved to `verify`.
5. `docs/content/reference/cli.md` (the commands, each with its own permalink), a guide section on linking to the docs from a product, with the CI step, `CHANGELOG.md`, and the contract for the file under `docs/content/contracts/`.

## Out of scope

Retiring (phase 5): until then, the only ways to quiet `permalink-removed` are to restore the name or to edit the file by hand, and the guide says to wait.

## Acceptance criteria

- Deleting a marked heading from a project with the file makes `ascribe check` warn, naming the permalink and what the check can't see.
- Moving that heading to another page and running `sync` changes one line of the file and warns about nothing.
- `ascribe permalinks verify` passes for every name Ascribe's binary uses, and exits 1 for one that doesn't exist, with only the file present.
- A project with no `ascribe.permalinks` behaves exactly as it did after phase 3.

## Verify

```sh
cargo test --workspace --locked
target/debug/ascribe permalinks verify --config docs cli-check ASC036
ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes && git diff --stat schemas
```

## Commits

1. "Keep a project's permalinks in a committed list"
2. "Report a permalink that was removed"
3. "Add ascribe permalinks: sync, list, and verify"
4. "Commit the docs' permalinks, and verify the binary's names against them"
