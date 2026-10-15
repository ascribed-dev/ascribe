# Phase 3: The memo in every surface

Part of [Memos](README.md). Needs phase 2.

## Goal

The editor knows `@memo`, an HTML comment in a page is advice with a fix that makes it a memo, and the docs' generated fragments say what generates them in one.

```markdown
<!-- Keep this table in the order of the plans' prices. -->
| Plan | Seats | Price |
```

```console
$ ascribe check
pricing.md:3:1: advice[html-comment]: an HTML comment is in the published page's source; a memo isn't
```

The quick fix writes `@memo: Keep this table in the order of the plans' prices.` in its place.

## Context

- README decisions 3, 8, and 9, and phase 1's decision 3.
- `crates/ascribe-check/src/prose/extract.rs`: how a Vale comment is recognized.
- `crates/ascribe-check/src/intended.rs`: how a review check is acknowledged at a block.
- `crates/ascribe-lsp/src/complete.rs`, `hover.rs`, and `code_action.rs`.
- `packages/vscode/syntaxes/`: the grammars that color a page.
- The generators that write the note at the top of a generated fragment: `crates/ascribe-cli/src/docs.rs` and `shapes.rs`, `tests/conformance/tests/docs.rs`, and `packages/vscode/test/unit/docs.test.ts`.
- `crates/ascribe-query/src/rules.rs`: the rules `ascribe agents sync` writes, and their budget.

## Design

### In the editor

- **Completion** offers `@memo` where a directive can go, with both forms, from its schema.
- **Highlighting.** A memo's text is colored as a comment, in both forms, by the grammars. A container's lines are one region, so a link or a brace inside isn't colored as one.
- **Hover** on `@memo` says what it is: text for whoever edits next, never published.
- **Folding** closes a container memo to its first line.

### `html-comment`

Advice, a review check, configurable, with the next step `review`, as phase 1 names it. It reports each HTML comment in a page or a fragment, a block or inside a paragraph, at the comment. A comment that's Vale's, as `prose/extract.rs` recognizes it, isn't reported.

Its fix paragraph says the comment is in the published page's source, that `@memo` keeps the same text out of every output, and that a comment written for another tool is acknowledged with `@intended {check=html-comment}`.

The quick fix, **Rewrite as a memo**, is offered for an HTML block that holds one comment and nothing else. One line becomes a line memo, and more become a container. A comment inside a paragraph gets no quick fix: a memo is about a block, so where it goes is the writer's call.

### The generated fragments

Each generator writes its note as a line memo, `@memo: Generated from … by …; run … to rewrite it.`, where it wrote an HTML comment. The published pages lose the comment, which is the point, and the fragments are blessed. The rest of the HTML comments in `docs/content/` are rewritten as memos, or acknowledged where they're about another tool. `ascribe check --deny-warnings` on the docs reports none.

AGENTS.md says a generated file says what generates it. It still does, in a memo.

### The rules

In a project with a `@memo` in any page, the rules `ascribe agents sync` writes gain one sentence: read a block's `@memo` before changing it, and keep the two together. It's inside the budget, and cut with the rest when the rules are too long.

## Tasks

1. Completion, hover, and folding, and the grammars.
2. `html-comment`, its registry entry, its conformance cases (a block comment, an inline one, Vale's, one acknowledged, one in a fragment), and its quick fix with a test of the server.
3. The generators, and their fragments blessed.
4. The docs' other comments, and `ascribe check --deny-warnings` on `docs` and `site/home`.
5. The rules' sentence, with a test that it's there only when a page has a memo.
6. The editor's page (`docs/content/editor/index.md`), the diagnostics reference, `changes/`, and the surface map.

## Out of scope

- Converting every HTML comment in a project at once.
- Memo files.

## Acceptance criteria

- The page in the goal reports `html-comment`, and the quick fix leaves a memo and no report.
- A Vale comment and an acknowledged comment aren't reported.
- In VS Code, typing `@me` offers `@memo`, a memo's text is colored as a comment, and a container folds.
- No page under `docs/content/` has an HTML comment that's reported, and the published docs pages have none of the generators' notes in their source.
- `node scripts/compare/outputs.ts --base main` shows only the docs' pages that lost a comment, each named with `--accept`.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter ascribe-vscode test
cargo run -p ascribe-cli -- check --deny-warnings --config docs
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Complete, color, and fold @memo in the editor"
2. "Report an HTML comment in a page, with a fix that makes it a memo"
3. "Say what generates a fragment in a memo"
4. "Tell agents to read and keep a block's memo"
