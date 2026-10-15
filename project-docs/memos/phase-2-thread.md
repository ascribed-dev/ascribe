# Phase 2: The thread

Part of [Memos](README.md). Needs phase 1.

## Goal

A `@memo` in a page, in either form, is never published, never checked, never reformatted, and never a change. In `examples/quill/docs/keys.md`:

```markdown
## Rotate a key
@memo:
Rotation comes before revocation: support found that people who revoke
first lock out their own agents. See [the incident](notes/2026-08.md).

Keep the {product} name out of the heading; search shows it already.
@end
```

`ascribe check` reports nothing about the memo: `notes/2026-08.md` doesn't exist, and `{product}` isn't a use of the phrase. `ascribe fmt --check` passes with the memo as written. `ascribe build` writes the site, plain, and JSON outputs as they are without it, and `ascribe diff` against the commit before it lists no page.

This is the thinnest slice through every layer an in-page memo touches. It exists to find out whether a container's contents can be kept from every reader that walks a page before resolution (README, "The thread").

## Context

- README decisions 3 to 8, and phase 1's decision 1.
- SPEC §3.5 and §4.9.
- `crates/ascribe-core/src/schema.rs`: the built-in schemas, and the test that holds them to the SPEC's table.
- `crates/comrak-ascribe/src/ascribe.rs` and `src/parser/ascribe.rs`, and `FORK.md`'s table of changed locations.
- `crates/ascribe-syntax/src/structure/`: how openers and `@end` are paired, and `bind.rs`, where a following-block directive finds its block.
- `crates/ascribe-resolve/src/build/mod.rs`: `drop_acknowledgements`.
- What reads a page before resolution: the source index and the page checks in `crates/ascribe-check/src/`, Vale's extraction in `prose/extract.rs`, `ascribe refs` in `crates/ascribe-query/src/refs.rs`, `crates/ascribe-fmt/`, and the language server's links, ids, and tokens in `crates/ascribe-lsp/src/`.
- `tests/conformance/cases/directives/intended/`, for the shape of a case.

## Design

### The directive

`@memo` is a built-in: forms both, binding following block (section at the top of one, as `@available`'s), primary a line primary in line form and empty in container form, no attributes. SPEC gains §4.10 after `@intended`, and its table of built-ins a row. A memo in a fragment is part of the fragment.

### What's in a container

As phase 1 decides. With raw lines, the fork's scanner opens a raw container on `@memo:` and keeps every line to its `@end` as text, in one node that carries the lines' source span; SPEC §3.5 says what a raw container is and that `@memo` is the one built-in that has one. With parsed contents, every reader listed in the context skips a memo's children, and a test walks each of them over the same page.

Either way, `ascribe-syntax` gives a memo's text to anything that asks for it, as the source between its lines, prefixes stripped (`tree.rs` has the function). Phase 4's check is the first to ask.

### Removed at resolution

`drop_acknowledgements` removes `@memo` beside `@intended`, in either form, at any depth, and is renamed for what it now does. Nothing downstream changes: emit, both renderers, the preview, and the report never see a memo.

### `ascribe diff`

It compares resolved pages, so a change to a memo alone is no change already. A test holds it: a page whose only edit is to a memo isn't listed, and one with a memo added beside a changed paragraph lists the paragraph alone.

### The formatter

A memo's lines are left as written, and the directive line sits directly above its block, as `@intended`'s does.

### The editor

The language server gives no diagnostic, link, phrase, id, or semantic token from inside a memo. It gets that from the check and the index, so it needs no code of its own if they're right; a test of the server holds it. Completion and highlighting are phase 3's.

## Tasks

1. The schema and SPEC §4.10, and the table test.
2. The container, as phase 1 decides: the fork's scanner and `FORK.md`, or a skip in each reader.
3. Removal at resolution, and the diff test.
4. The formatter.
5. Conformance cases in `tests/conformance/cases/directives/memo/`: the line form, the container form, a memo at the top of a section, two stacked, one with nothing below it, one with attributes, one in a list item, one in a fragment, and the container from the goal: a link to a missing file, a `{word}`, an `@include`, a `# heading`, and an unclosed fence, none of which is read.
6. `examples/quill/docs/keys.md` gains the memo from the goal, and the compare script shows the example's outputs unchanged.
7. The directives reference (`docs/content/writing/directives.md`), and an entry in `changes/`.
8. The surface map.

## Out of scope

- Completion, highlighting, and hover (phase 3).
- `html-comment` (phase 3).
- Page templates (phase 4).
- Memo files (phase 5).

## Acceptance criteria

- `ascribe check`, `fmt --check`, `build`, and `diff` on `examples/quill` do what the goal says.
- `node scripts/compare/outputs.ts --base main` shows every project's outputs unchanged, `examples/quill` included.
- In the container case, no reader reports, indexes, or links anything inside the memo: `ascribe refs` on the missing file and on the phrase lists no use from it, and Vale is given none of its text.
- `ascribe fmt` on the case's page changes nothing.
- Changing only the memo's text, `ascribe diff` lists no page.
- In the editor, the memo's page has no diagnostics, and go to definition on the memo's link goes nowhere.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- check --config examples/quill
cargo run -p ascribe-cli -- fmt --check --config examples/quill
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Add @memo, which is never published"
2. "Keep a memo's contents from every reader"
3. "Leave a memo out of ascribe diff"
