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

- README decisions 3 to 8 and 21.
- SPEC §3.5 (forms), §3.8 (binding), and §4.9.
- `crates/ascribe-core/src/schema.rs`: the built-in schemas, and the test that holds them to the SPEC's table.
- `crates/comrak-ascribe/src/ascribe.rs` and `src/parser/ascribe.rs`, and `FORK.md`'s table of changed locations.
- `crates/ascribe-syntax/src/structure/`: how openers and `@end` are paired, and `bind.rs`, where a following-block directive finds its block.
- `crates/ascribe-resolve/src/build/mod.rs`: `drop_acknowledgements`.
- What reads a page before resolution: the source index and the page checks in `crates/ascribe-check/src/`, Vale's extraction in `prose/extract.rs`, `ascribe refs` in `crates/ascribe-query/src/refs.rs`, `crates/ascribe-fmt/`, and the language server's links, ids, and tokens in `crates/ascribe-lsp/src/`.
- `tests/conformance/cases/directives/intended/`, for the shape of a case.

## Design

### The directive

`@memo` is a built-in: forms both, binding following block (section at the top of one, as `@available`'s), primary a line primary in line form and empty in container form, no attributes. SPEC gains §4.10 after `@intended`, and its table of built-ins a row. A memo in a fragment is part of the fragment.

### A container that binds

Today a binding belongs to line form (SPEC §3.8), and a container's content is what it holds. A container memo is both: it holds its text, and by decision 4 it's about something. No directive has done that, so SPEC §3.5 and §3.8 each gain a sentence:

- **What it's about** is decided from its opening line, as a line memo's is: at the top of a section, the section; anywhere else, the next block after its `@end` in the same container.
- **Touching** is measured from the `@end`: the block it's about starts on the line after it. A blank line there is the same warning a following-block directive gets, and `ascribe fmt` removes it.
- **Nothing after the `@end`** in its container, outside the top of a section, is the error any following-block directive with nothing below it gets, reported at the opening line.
- **Stacking** works across forms: a line memo directly above a container memo is about what the container is about.

### What's in a container

The fork's scanner opens a raw container on `@memo:` and keeps every line to its closing line as text, in one node that carries the lines' source span. The closing line is a line that's only `@end` at the memo's indent, inside the container's prefixes; so a memo can't hold such a line, and SPEC §3.5 says so. SPEC §3.5 also says what a raw container is, and that `@memo` is the one built-in that has one.

**The `@end` stays an `AscribeLine` of its own.** The raw node ends on the line before it, and the scanner then reads the `@end` as it reads any other, so `ascribe-syntax` pairs the opener with it exactly as it pairs every container today, and the pairing learns no exception. This is the one place the fork and the syntax crate meet, and the conformance cases below hold both sides of it.

`ascribe-syntax` gives a memo's text to anything that asks for it, as the source between its lines, prefixes stripped (`tree.rs` has the function). Phase 4's conversion tests are the first to ask.

### Removed at resolution

`drop_acknowledgements` removes `@memo` beside `@intended`, in either form, at any depth, and is renamed for what it now does. Nothing downstream changes: emit, both renderers, the preview, and the report never see a memo.

### `ascribe diff`

It compares resolved pages, so a change to a memo alone is no change already. A test holds it: a page whose only edit is to a memo isn't listed, and one with a memo added beside a changed paragraph lists the paragraph alone.

### The formatter

A memo's lines are left as written, and the directive line sits directly above its block, as `@intended`'s does.

### The editor

The language server gives no diagnostic, link, phrase, id, or semantic token from inside a memo. It gets that from the check and the index, so it needs no code of its own if they're right; a test of the server holds it. Completion and highlighting are phase 3's.

## Tasks

1. The schema, SPEC §4.10, the sentences in §3.5 and §3.8, and the table test.
2. The raw container: the fork's scanner and `FORK.md`, with the `@end` left to the scanner.
3. Removal at resolution, and the diff test.
4. The formatter.
5. Conformance cases in `tests/conformance/cases/directives/memo/`: the line form, the container form, a memo at the top of a section, two stacked, a line memo stacked on a container one, one with nothing below it, a container with nothing after its `@end`, a container with a blank line after its `@end`, one with attributes, one in a list item, one in a fragment, and the container from the goal: a link to a missing file, a `{word}`, an `@include`, a `# heading`, a nested `@note:`, and an unclosed fence, none of which is read, with the `@end` after them still closing the memo.
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
