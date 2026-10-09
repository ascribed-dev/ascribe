# Phase 3: Checks while writing

Part of [Content checks](README.md). Requires phase 1; offers acknowledgement once phase 2 has landed. Rust.

## Goal

Five checks an author sees as they write, each answering from one page and the content model, each with its next step.

## Context

- README decisions 1, 6, 7, and 8, and the inventory's first table.
- `crates/ascribe-check`: `check_file` (file-level checks, which the server runs on every keystroke) and the page-level checks (per build). `crates/ascribe-lsp/README.md` says which the server publishes and when.
- `crates/ascribe-emit` (`plain`): the resolved Markdown whose length the size check measures.
- `crates/ascribe-lsp/benches/keystroke.rs`: the benchmark decision 7 is measured by.
- The [Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/): read the current version for the size limit and what it asks of headings and code fences. The brainstorm quotes draft 0.6.0 (50,000 characters); confirm before writing the number into a message.
- `docs/content/reference/content-model.md`: page types and frontmatter fields (§4), field types including `date` (§5), and `[consumer]` (§15).
- `examples/monorepo`: its security policies already declare an owner and a review date.

## Design

All five are `advice` and `configurable`.

| Check | Reported when | Kind | Next step | Evidence in the prompt |
|---|---|---|---|---|
| `page-size` | A page's plain output, in a build, is over the limit | `write` | Split the page, or move bulk content after the prose | Each section's heading and size, largest first; the limit; which builds are over |
| `page-description-missing` | A page has no description and its type declares a description field | `write` | Write one | The page's title and first paragraph |
| `heading-level-skipped` | A heading is more than one level below the one before it, or a page has a second top-level heading | `fix` | Change the level | — |
| `code-language-missing` | A fenced code block has no language | `choose` | Pick one; `text` for output | The block's first lines |
| `review-overdue` | The page's review date has passed | `review` | Review the page and move the date, or acknowledge | The date, and the page's owner if it has one |

Details:

- **`page-size`** is a page-level check: the length depends on the build. The server reports it for the editor's build, on save rather than on every keystroke if measuring costs more than decision 7 allows. `ascribe check` reports it for every build, once per page, naming the builds. The limit is `[checks.page-size] limit`, defaulting to the spec's.
- **Which field is the description** and **which is the review date** aren't known to Ascribe today. Add `[checks.page-description] field` and `[checks.review] field`, each naming a frontmatter field; each check is off until its field is named, and naming a field no type declares, or one of the wrong type, is a model error. `review-overdue` compares with the current date, which makes `check` depend on the day it runs: say so in the reference, and give tests a fixed date through the existing clock, or add one.
- **`heading-level-skipped`** counts headings that arrive through includes, at the level they have on the page. Its fix changes the heading in the file it's written in. A page whose title is rendered as its top-level heading by the site has no top-level heading in its source; read how the outputs treat titles before deciding what "a second top-level heading" means.
- **`code-language-missing`** doesn't apply to `@snippet`, which takes its language from the file.

## Tasks

1. Each check, in its own commit, with its registry entry (every field of decision 1), conformance cases, and reference entry.
2. `[checks.page-size]`, `[checks.page-description]`, and `[checks.review]`, with validation.
3. The fixes for `heading-level-skipped` and `code-language-missing`, labeled `safe` or `unsafe` by the agents plan's rule.
4. The keystroke benchmark before and after, in the pull request.
5. Run all five on `docs/` and `examples/`, and fix or acknowledge what they find in the same pull request. What they find on Ascribe's own docs is the first test of whether each message says enough to act on.
6. `docs/content/`, `CHANGELOG.md`.

## Out of scope

Checks that need the whole project (phase 4); emitting `llms.txt` (phase 6).

## Acceptance criteria

- Each check has a conformance case for reporting, for not reporting, and for its level changed in `[checks]`.
- The keystroke benchmark's median moves by no more than 1 ms at 3,000 pages.
- `ascribe check --deny-warnings` on `docs/` still exits 0.

## Verify

```sh
cargo test --workspace --locked
cargo bench -p ascribe-lsp --bench keystroke
target/debug/ascribe check --deny-warnings --config docs
```

## Commits

One per check, then "Fix what the new checks find in the docs".
