# Phase 1: Decisions

Part of [Glossary pages](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Six choices are made before anything ships, because each goes into a contract: the language, the content model, the JSON output, `ascribe model`'s JSON, the diagnostics, and where Ascribe's docs put a page.

## Context

- The [proposal](../proposals/glossary-pages.md), whole, with its settled answers.
- SPEC §4.1 (`@id`, the model for a directive at the top of a section), §4.9 (`@intended`), and §5.4 (glossary terms).
- `crates/ascribe-resolve/src/build/glossary.rs`: what's linked and when.
- `crates/ascribe-emit/src/json.rs` (a block's terms), and `crates/ascribe-query/src/model.rs` (a term in `ascribe model`).
- `crates/ascribe-model/src/intended.rs`: `[[intended]] term`.
- `docs/content/checks/diagnostics.md`, and `description-too-long`'s entry in `tests/conformance/diagnostics.toml`.

## The decisions

Each has a recommendation. The maintainer may take it, change it, or ask for more.

### 1. What a definition is, exactly

Decision 8 says it's the first paragraph as plain text. What's left is which paragraph, and what plain text keeps.

| Question | Recommended |
|---|---|
| Which paragraph | The first block of the section's own content, after the heading's directives, and it must be a paragraph. A section that opens with a list, a note, or a code block has no definition, which is decision 9's error: the fix is a sentence before it |
| Inside a fragment | A paragraph included at that place counts, as it would on the page |
| Phrases | Substituted, for the build |
| Emphasis and strong | Their text |
| Inline code | Its text, without the backticks |
| A link | Its text |
| An image | Its alt text |
| Inline HTML | Dropped |
| Line breaks | One space, and whitespace runs one space |
| A paragraph the build removes | The term has no definition in that build, and isn't linked in it (phase 3) |

### 2. A term that two glossary pages show

A glossary split by product can keep shared terms in a fragment that each product's glossary page includes. Then one heading, one id, is on two pages.

| Option | Means |
|---|---|
| **One term, linked to the first page in `pages` that the build publishes** (recommended) | The same source is the same term. A build for one product publishes one of the pages, and links there; a build that publishes both links to the first listed. Not a duplicate, since it's one heading |
| An error | A term is on one page. The shared fragment can't hold terms, and each glossary repeats them |

Two different headings with the same term text stay decision 9's error.

### 3. How `glossary-term-unused` is acknowledged

The term is a section of a page now. Today the acknowledgement is `[[intended]] term = "<id>"` in `ascribe.toml`.

| Option | Means |
|---|---|
| **`@intended` at the term's heading, and `term` leaves `[[intended]]`** (recommended) | The check reports at the heading, and a review check is acknowledged where it reports (SPEC §4.9). The reason sits beside the term it's about. `[[intended]]` keeps `phrase`, `feature`, and `image`, which have no page |
| `[[intended]] term`, naming the heading's id | No change to `[[intended]]`. The reason for a term is in `ascribe.toml`, away from it, and renaming the heading's id has to rewrite it |

### 4. What `ascribe model` says about a term

`ascribe model --format json` lists each term with `term`, `aliases`, `definition`, `link`, and its settings. `link` goes, since every term is on its page, and a definition is now different for each build, while `ascribe model` describes no build.

| Option | Means |
|---|---|
| **The term's page and line, and no definition; `schema_version` goes up** (recommended) | Each term is `id`, `term`, `aliases`, `match`, `case_sensitive`, `page`, and `line`. A reader that wants the words opens the page, or reads the JSON output for a build. Removing two keys is a change to a contract, so the version moves, as it did for `type` (decision #450) |
| The definition for the editor's default build | Keeps `definition`. `ascribe model` is about the model, and this one key would depend on a build |

`ascribe model` keeps listing terms at all because agents read it to learn a project's vocabulary.

### 5. The diagnostics' names and levels

| Reports | Recommended | Level | Next step |
|---|---|---|---|
| `[glossary] terms`, or a key under it | `model-unknown-key`, whose message names `pages` and `@term`, as `files` on a type's does (decision #450) | Error | `write` |
| A path in `pages` that isn't a page | `model-glossary-page` | Error | `choose` |
| Two terms or aliases with the same text | `glossary-term-duplicate`, at the second, naming the first; `model-glossary-duplicate-term` (ASC098) goes | Error | `write` |
| A term with no definition | `glossary-definition-missing`, at the heading | Error | `write` |
| A definition over the limit | `glossary-definition-long`, at `description-too-long`'s limit | Advice | `write` |
| `@term` on a page not in `pages` | `term-off-glossary-page`, with the fix that adds the page | Error | `choose` |
| `model-glossary-link` (ASC099) and `glossary-link-target-missing` (ASC203) | Removed. Their codes stay gaps (decision #457a) | | |

**The limit.** The proposal settled that a definition's limit is `description-too-long`'s, "set the same way", and assumed that check had a `limit`. It doesn't: the limit is a constant, 300 characters. Recommended: keep one constant for both, as `description-too-long` has it, and add a `limit` to both at once if a project ever needs one. The alternative is to add `[checks.<name>] limit` to both now, which changes `description-too-long` in a plan about the glossary.

### 6. Where Ascribe's docs keep their glossary

Decision 14 gives the docs a glossary page. The proposal said `reference/glossary.md`, and the docs have no `reference/` folder now: each folder is a feature (decision #452).

| Option | Means |
|---|---|
| **`docs/content/glossary.md`, at the root** (recommended) | A glossary is about the whole product, as getting started is, and the root holds the pages that belong to no one feature. Its type is `reference` |
| `docs/content/content-model/glossary.md` | Beside the glossary's reference. But a reader looking up "fragment" isn't looking for the content model |

## Tasks

1. The maintainer reads the six and answers each.
2. Record each answer in this file, under its heading, as **Decided:** with the reason, and in the README's decisions, numbered from 15.
3. Record them in `project-docs/decisions/`, numbered by this plan's issue, as the repository does.
4. Add to the proposal what the answers change in it.

## Out of scope

- The wording of the diagnostics' messages and fixes. Each phase writes its own.
- What the docs' glossary says about each term. Phase 2 writes it, and the maintainer reads it there.

## Acceptance criteria

- Each of the six has an answer and a reason.
- Nothing in a later phase file contradicts one.
- Phase 2 can be started without asking the maintainer anything.

## Commits

1. "Record the decisions the glossary pages plan ships"
