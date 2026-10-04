# Phase 2: Generated reference

Part of [Docs](README.md). Requires phase 1. Can run at the same time as phase 4. Rust tests, and fragments.

## Goal

The parts of the reference that repeat what the code already says are generated from it, as fragments that pages include, and a test fails when one is stale. This is the first of the three drift checks, and the one this repository already half has.

## Context

- `tests/conformance/tests/docs.rs`: generates the diagnostics page from `tests/conformance/diagnostics.toml`, and fails when the file differs; `ASCRIBE_BLESS=1` rewrites it. The pattern every generator here follows.
- `crates/tessera-cli/src/` (the `clap` definitions): each command's options and their help text. `docs/content/reference/cli.md`: where those options are also written by hand.
- `packages/vscode/package.json` (`contributes.configuration`, `contributes.commands`): the extension's settings and commands, also written by hand in `docs/content/guides/editor.md`.
- `Cargo.toml`, `packages/*/package.json`, `rust-toolchain.toml`: where phase 1's version phrases get their values.
- [Decision 8](README.md#decisions).

## Design

### Where generated text goes

`docs/content/_generated/`, one fragment per thing. Each starts with a comment saying what generates it and how to regenerate. Pages use `@include`; a page's own prose stays hand-written around it.

### What's generated

| Fragment | From | Included by |
|---|---|---|
| `diagnostics-table.md`, and one fragment per group of diagnostics | The registry | `reference/diagnostics.md`, which becomes a hand-written page with an introduction and includes |
| `cli-<command>-options.md` | `clap`'s definition of each command: option, value, default, help | `reference/cli.md`, under each command |
| `editor-settings.md`, `editor-commands.md` | The extension's manifest | `guides/editor.md` |

Each generator is a test beside the code it reads, in that code's language: Rust tests for the registry and the CLI, a vitest test for the manifest. Each writes Ascribe source in canonical form (`ascribe fmt` changes nothing in it).

### Phrases with a source

For each phrase in `docs/ascribe.toml` whose value is in another file (the version, the minimum VS Code version, the Node.js version, the Rust toolchain), a test compares the two and fails on a difference, naming both files. Don't generate `ascribe.toml`: it keeps its comments and its hand-written parts.

### What stays hand-written

Explanations, examples, and anything a generator would have to guess. If a command's help text isn't good enough to publish, fix the help text.

## Tasks

1. The diagnostics page as fragments and includes; the generator's test updated.
2. The CLI options fragments and their test; `reference/cli.md` using them, with its hand-written option lists removed.
3. The extension's settings and commands fragments and their test; `guides/editor.md` using them.
4. The phrase tests.
5. `CONTRIBUTING.md`: one paragraph on generated fragments and `ASCRIBE_BLESS`.

## Out of scope

Generating JSON schemas or examples (phase 7's snippets take examples from tested files); the site; offering generation to users as a feature (it's a recipe: tests that write fragments).

## Acceptance criteria

- Adding an option to a command, a setting to the extension, or a diagnostic to the registry fails a test until the fragment is regenerated.
- Bumping the version fails a test until the phrase matches.
- `ascribe check --deny-warnings` still passes, and the generated fragments are in canonical form.
- No option, setting, command, or diagnostic is described in two places.

## Verify

```sh
cargo test --workspace --locked
pnpm test
./target/debug/ascribe check --deny-warnings --config docs/ascribe.toml
```

## Commits

1. "Include the diagnostics reference as generated fragments"
2. "Generate each command's options for the reference"
3. "Generate the editor's settings and commands for the guide"
4. "Check the docs' version phrases against their sources"
