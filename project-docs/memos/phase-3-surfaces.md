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

- README decisions 3, 4, 8, 9, and 23.
- `crates/ascribe-check/src/prose/extract.rs`: how a Vale comment is recognized.
- `crates/ascribe-check/src/intended.rs`: how a review check is acknowledged at a block.
- `crates/ascribe-lsp/src/complete.rs`, `hover.rs`, and `code_action.rs`.
- `packages/vscode/syntaxes/`: the grammars that color a page.
- The five generators that write the note at the top of a generated fragment: `crates/ascribe-cli/src/docs.rs` and `shapes.rs`, `tests/conformance/tests/docs.rs`, `packages/vscode/test/unit/docs.test.ts`, and `scripts/docs-site/elements.test.ts`.
- `crates/ascribe-fmt/src/blank.rs`: the blank line between a following-block directive and its block, which the formatter removes.
- `crates/ascribe-query/src/rules.rs`: the rules `ascribe agents sync` writes, and their budget.

## Design

### In the editor

- **Completion** offers `@memo` where a directive can go, with both forms, from its schema.
- **Highlighting.** A memo's text is colored as a comment, in both forms, by the grammars. A container's lines are one region, so a link or a brace inside isn't colored as one.
- **Hover** on `@memo` says what it is: text for whoever edits next, never published.
- **Folding** closes a container memo to its first line. The server answers no folding ranges today: a page's folding is VS Code's Markdown extension's, which folds headings, lists, and code blocks and knows nothing of a container. VS Code merges the ranges of every provider, so the server's add the memo's to the Markdown extension's.

### `html-comment`

Advice, a review check, configurable, with the next step `review` (decision 23). It reports each HTML comment in a page or a fragment, a block or inside a paragraph, at the comment. A comment that's Vale's, as `prose/extract.rs` recognizes it, isn't reported.

Vale's is the only exemption. Ascribe's docs have no other tool's comments: outside the generated notes, their HTML comments are two of Vale's and two examples inside a code block. If projects turn out to write markdownlint's or Prettier's, a second exemption is cheaper than an acknowledgement for each, and is its own change.

Its fix paragraph says the comment is in the published page's source, that `@memo` keeps the same text out of every output, and that a comment written for another tool is acknowledged with `@intended {check=html-comment}`.

The quick fix, **Rewrite as a memo**, is offered for an HTML block that holds one comment and nothing else. One line becomes a line memo, and more become a container. Where the memo goes depends on what follows the comment, since a memo is about the block it touches and binding a heading is an error (SPEC §3.8):

- **A block follows it in its container:** the memo replaces the comment, touching that block.
- **It's the last block of a section:** the memo goes to the top of the section, about the section, and the comment is removed.
- **It's the last block of a container with no heading,** such as a list item, or of a page before its first heading: no quick fix. Where the memo belongs is the writer's call.

A comment inside a paragraph gets no quick fix either, for the same reason.

### The generated fragments

Each of the five generators writes its note as a line memo, `@memo: Generated from … by …; run … to rewrite it.`, where it wrote an HTML comment, and **touching the fragment's first block**, with no blank line between. They write a blank line after the comment today; under a memo that's the warning for a following-block directive's blank line, and `ascribe fmt --check` on the docs would fail on every fragment. The published pages lose the comment, which is the point, and the fragments are blessed. The docs' two Vale comments stay. `ascribe check --deny-warnings` and `ascribe fmt --check` on the docs pass.

AGENTS.md says a generated file says what generates it. It still does, in a memo.

### The rules

In a project with a `@memo` in any page, the rules `ascribe agents sync` writes gain one sentence: read a block's `@memo` before changing it, and keep the two together.

**The order the rules are cut in** when they're over the budget: the memo sentences go first, before the page types' lines, since they're the newest and the least needed in a project with few memos. This sentence is the first; phase 7 adds the second, which is cut with it.

## Tasks

1. Completion, hover, and folding, and the grammars.
2. `html-comment`, its registry entry, its conformance cases (a block comment, an inline one, Vale's, one acknowledged, one in a fragment), and its quick fix with a test of the server for each of the three positions.
3. The five generators, each writing a memo that touches its block, and their fragments blessed.
4. `ascribe check --deny-warnings` and `ascribe fmt --check` on `docs` and `site/home`. If a page shows another tool's comments by then, say so in the pull request.
5. The rules' sentence, the order they're cut in, and a test that it's there only when a page has a memo and is the first thing cut.
6. The editor's page (`docs/content/editor/index.md`), the diagnostics reference, `changes/`, and the surface map.

## Out of scope

- Converting every HTML comment in a project at once.
- Memo files.

## Acceptance criteria

- The page in the goal reports `html-comment`, and the quick fix leaves a memo and no report.
- A comment that's the last block of a section becomes a memo at the section's top, and one that's the last block of a list item gets no quick fix.
- A Vale comment and an acknowledged comment aren't reported.
- In VS Code, typing `@me` offers `@memo`, a memo's text is colored as a comment, and a container folds.
- No page under `docs/content/` has an HTML comment that's reported, the published docs pages have none of the generators' notes in their source, and `ascribe fmt --check` on the docs passes.
- `node scripts/compare/outputs.ts --base main` shows every project the same: the binary changes no output, since it builds the same sources on both sides. The docs' pages lose the generators' comments through their source, which a build of `docs/` at `main` and at the branch, with one binary, shows: their site and JSON outputs differ only by the comments and the blank lines after them, beside the pages whose prose this phase changes.

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
