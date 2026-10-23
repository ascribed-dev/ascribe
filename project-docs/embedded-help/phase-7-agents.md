# Phase 7: For agents

Part of [Embedded help](README.md). Needs phase 1; after phase 3, its summaries are the marked ones.

## Goal

`ascribe resolve go:weave-config` and `ascribe_resolve` show an entry.

```console
$ ascribe resolve go:prerequisites --build site --config examples/quill
Prerequisites
Before you install the agent, make sure you have:

- Node.js 20 or later.
- A docs repository that Quill can read.

Checklist of prerequisites

Read more: https://docs.quill.dev/go/prerequisites/
```

The line "Checklist of prerequisites" is the fragment's image, as its alt text (decision 5). `--format json` prints the entry as `help.json` writes it, with `version`, `build`, and `line`. A name the build doesn't publish is exit code `1`, with the names closest to it; one `ascribe permalinks verify` would call retired gives its reason.

## Context

- README decision 15.
- `crates/ascribe-cli/src/commands/resolve.rs`, its JSON and its schema; `crates/ascribe-cli/src/mcp/tools.rs`'s `ascribe_resolve`.
- `project-docs/checklists.md`: "A new command or option", and the MCP tool's arguments named as the command's.

## Design

- **`go:<name>`** in `resolve`'s page argument names a permalink (decision 15). The text output is the title, the summary when there is one, the text, and "Read more" with the address. The JSON is the entry, from `help.rs`'s function, so the command and the file can't disagree.
- **The tool:** `ascribe_resolve` takes the same `page`, `go:<name>`, and returns the same.
- **Agents are told:** the rules `ascribe agents sync` writes say that a product's message linking to the docs can read what the docs will show with `ascribe resolve go:<name>`.

## Tasks

1. `go:` in `resolve`, with CLI tests for an entry, its JSON, an unknown name, and a retired one; its output in `crates/ascribe-cli/tests/output/`, observed again.
2. The tool, with its tests.
3. The rules' sentence, and the agents' tests.
4. The docs: `resolve`'s reference and its JSON contract, the embedded help page; a decision file for `go:` and the JSON; `changes/command/resolve-permalink.md`; the surface map.

## Acceptance criteria

- The goal's command and its JSON, in tests.
- `ascribe_resolve` with `go:` in the MCP tests.

## Verify

```sh
cargo test -p ascribe-cli resolve
cargo run -p ascribe-cli -- resolve go:prerequisites --build site --config examples/quill
```

## Commits

1. "Resolve a permalink to the help a product shows"
