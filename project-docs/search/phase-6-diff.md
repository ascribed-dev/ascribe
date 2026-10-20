# Phase 6: `diff` on the reading

Part of [Search](README.md). Needs phase 2, and runs after phase 5. A clean-up: labeled `optimization`, and no output changes.

## Goal

`ascribe diff` draws its block text from the reading, and its output is byte for byte the same.

`diff`'s `TreeBuilder` no longer turns a resolved block into text itself. It takes each node's text from `ascribe_resolve::reading`, and keeps what it alone needs: the kinds it pairs, the fingerprints, the files, the snippet's address. `node scripts/compare/outputs.ts --base main` shows `diff`'s text, JSON, and HTML report unchanged for every project and every case in its tests. Two of the code's three readings of "what a reader sees" are one; the third, Vale's, differs on purpose, and its module says so.

## Context

- README decision 1, and AGENTS.md's "A clean-up changes no output".
- `crates/ascribe-diff/src/tree.rs`: `Node::text`, `TreeBuilder::block`, `plain`, `push_inlines`, `directive_text`, and `collapse`; `compare.rs` and `words.rs`, which read `text` for the word-level diff.
- `ascribe_resolve::reading`, as phases 1 and 2 left it.
- `crates/ascribe-check/src/prose/extract.rs`'s header comment.
- `CONTRIBUTING.md`: the `optimization` label, and what CI runs with it.

## Design

A node's text today, and what the reading gives for it:

| Node | `diff`'s text | From the reading |
|---|---|---|
| A heading, a paragraph, a list item, a directive | Its inlines as plain text, white space collapsed | The block's text |
| A code block, a snippet's code | The code, collapsed | The block's text |
| An HTML block | Its HTML, collapsed | An `html` block, added to the reading here, which search never matches |
| A table | Every cell, joined by spaces, collapsed | Its rows' texts, joined by a space |
| A link with no text | Its target's title | The borrowed piece (phase 2) |

Where a row of this table turns out not to hold, the reading changes to agree with `diff`, only where that changes nothing search finds; otherwise the difference is listed in the pull request and the move waits for the maintainer, as decision 1 says. `diff`'s fingerprints don't move: they're built from `own`, not from `text`.

## Tasks

1. The `html` block kind in the reading, never matched by search, with its test.
2. A test, before the move, that compares each node's text from `TreeBuilder` with the reading's for every project in the repository and every case in `crates/ascribe-diff/tests/all/`. Run it, and settle each difference by the rule above.
3. `TreeBuilder` takes text from the reading; `plain`, `push_inlines`'s text, and `directive_text` go where nothing else uses them.
4. `prose/extract.rs`'s header says why Vale's reading is its own: one source file as parsed, markup kept, every arm.
5. `ascribe-diff`'s README and ARCHITECTURE.md, where they say how `diff` reads a page.

## Out of scope

Vale's map (decision 1). Any change to what `diff` reports.

## Acceptance criteria

- `node scripts/compare/outputs.ts --base main` shows every output, `diff`'s among them, byte for byte the same, in CI under the `optimization` label.
- `cargo test -p ascribe-diff` passes with no snapshot changed.

## Verify

```sh
cargo test -p ascribe-diff
cargo test -p ascribe-resolve reading
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Take diff's block text from the reading"
