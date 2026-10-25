# Phase 3: Settings

Part of [Redirects](README.md). Needs phase 2.

## Goal

Changing `base-path`, `permalink-path`, or `agents` is one warning whose fix forwards every address it moved, permalinks included.

```console
$ ascribe check --base main
warning[address-lost]: base-path changed from "/docs/" to "/handbook/", so 554 addresses moved, 333 of them permalinks
  = fix: record the settings change: ascribe check --fix
$ ascribe check --fix && ascribe build
```

The build's `redirects.json` forwards every `/docs/` address, each page, Markdown version, `go/` permalink, and `llms.txt`, to its `/handbook/` address, and keeps doing so after the base moves past the change.

## Context

- README decisions; the proposal's Settled 8, 10, and 11, and its `[former]`.
- Phase 0's `[former]`: `base-path`, `permalink-path`, `agents`, and `builds`.
- Phase 1's comparison and build.
- `crates/ascribe-resolve/src/build/permalinks.rs`, and `crates/ascribe-emit/src/site/permalinks.rs` for each permalink's address; the plain emitter's `llms.txt` and its section files.

## Design

### What's compared

The comparison adds each `go/` permalink's address and each `llms.txt` file to phase 1's, in every build matched by name. Linked files stay out (Settled 10).

### The settings change

When the same sources publish at addresses a setting moved, the check says so once: `address-lost`'s settings message, naming the setting, its old and new values, and how many addresses moved, in place of a warning per source. Its fix writes the old value into `[former]`, and is safe: the setting's change is the decision. A renamed build is the same, written in `[former]`'s `builds`.

### The build

For each `[former]` value, every address each source would have had under it forwards to its address now, permalinks and `llms.txt` included, so a second change composes with the first: under `base-path = ["/docs/", "/guide/"]`, an address under either forwards straight to today's, flat.

### `agents` turned off

`agents = [true]` in `[former]`, which the settings fix writes when `[consumer] agents` goes from on to off, forwards each Markdown version to its page and `llms.txt` to the home page (Settled 11).

## Tasks

1. `go/` permalinks and `llms.txt` in the comparison, with tests.
2. The settings message and its fix, for each setting and a renamed build, with conformance cases.
3. `[former]` in the build, composed, with emit tests: one change, two, and each setting.
4. `agents` turned off, with an emit test.
5. SPEC's section for `ascribe.addresses` gains `[former]`, with its decision file.
6. The docs: the redirects page's settings changes; `changes/language/redirects-settings.md`.

## Out of scope

`trailing-slash` (phase 0: a host matches either form), and a build's own host, site, and base path (the later proposal).

## Acceptance criteria

- The goal's commands, in a test on `examples/astro-site`, with the base moved past the change and the redirects still written.
- A second change composes with the first, in an emit test.

## Verify

```sh
cargo test -p ascribe-check addresses
cargo test -p ascribe-emit redirects
```

## Commits

1. "Compare permalinks' and llms.txt's addresses with the base's"
2. "Record a settings change in ascribe.addresses, and forward every address it moved"
