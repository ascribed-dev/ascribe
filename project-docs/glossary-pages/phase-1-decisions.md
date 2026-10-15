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

**Decided: as the table has it.** The first block of the section's own content, and it must be a paragraph, so a definition is always a sentence a hover can show; what's kept of it is what a reader reads, and what's dropped is what only a renderer sees. A paragraph a fragment brings in counts, since it's on the page. README decision 15.

### 2. A term that two glossary pages show

A glossary split by product can keep shared terms in a fragment that each product's glossary page includes. Then one heading, one id, is on two pages.

| Option | Means |
|---|---|
| **One term, linked to the first page in `pages` that the build publishes** (recommended) | The same source is the same term. A build for one product publishes one of the pages, and links there; a build that publishes both links to the first listed. Not a duplicate, since it's one heading |
| An error | A term is on one page. The shared fragment can't hold terms, and each glossary repeats them |

Two different headings with the same term text stay decision 9's error.

**Decided: one term, linked to the first page in `pages` that the build publishes.** One source is one term, so it can't be the duplicate the error is for, and a glossary split by product needs its shared terms written once. The order of `pages` is already something a project writes, so it's the tie-break. README decision 16.

### 3. How `glossary-term-unused` is acknowledged

The term is a section of a page now. Today the acknowledgement is `[[intended]] term = "<id>"` in `ascribe.toml`.

| Option | Means |
|---|---|
| **`@intended` at the term's heading, and `term` leaves `[[intended]]`** (recommended) | The check reports at the heading, and a review check is acknowledged where it reports (SPEC §4.9). The reason sits beside the term it's about. `[[intended]]` keeps `phrase`, `feature`, and `image`, which have no page |
| `[[intended]] term`, naming the heading's id | No change to `[[intended]]`. The reason for a term is in `ascribe.toml`, away from it, and renaming the heading's id has to rewrite it |

**Decided: `@intended` at the term's heading, and `term` leaves `[[intended]]`.** A review check is acknowledged where it reports (SPEC §4.9), and the term reports at its heading. That takes one change to the language: `@intended` is a following-block directive, and a following-block directive can't bind a heading (SPEC §3.8), so it gains the binding `@available` has, heading-or-block: at the top of a section, under `@term` and `@id`, it acknowledges a check that reports at that heading, and nothing wider. The alternative, reporting at the definition's paragraph so today's binding serves, would point the message at the wrong line to keep the language still. `[[intended]]` keeps `phrase`, `feature`, and `image`, which have no page. README decision 17.

### 4. What `ascribe model` says about a term

`ascribe model --format json` lists each term with `term`, `aliases`, `definition`, `link`, and its settings. `link` goes, since every term is on its page, and a definition is now different for each build, while `ascribe model` describes no build.

| Option | Means |
|---|---|
| **The term's page and line, and no definition; `schema_version` goes up** (recommended) | Each term is `id`, `term`, `aliases`, `match`, `case_sensitive`, `page`, and `line`. A reader that wants the words opens the page, or reads the JSON output for a build. Removing two keys is a change to a contract, so the version moves, as it did for `type` (decision #450) |
| The definition for the editor's default build | Keeps `definition`. `ascribe model` is about the model, and this one key would depend on a build |

`ascribe model` keeps listing terms at all because agents read it to learn a project's vocabulary.

**Decided: the term's page and line, no definition, and `schema_version` goes to 3.** `ascribe model` describes the model and no build, and a definition is now a build's. `page` is the path from the content root, the form the audit's finding 13 settled for a page in JSON. Two keys removed is a contract change, so the version moves, as it did for `type` (decision #450). README decision 18.

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

**Decided: the names and levels in the table, and one constant for both limits.** `DESCRIPTION_LIMIT` serves `glossary-definition-long` as it serves `description-too-long`; a `limit` setting is added to both at once when a project asks, and not before, since this plan isn't the place to change the description check. `glossary-term-duplicate` takes a new code and ASC098's stays a gap, with ASC099 and ASC203, as decision #457a has it. README decision 19.

### 6. Where Ascribe's docs keep their glossary

Decision 14 gives the docs a glossary page. The proposal said `reference/glossary.md`, and the docs have no `reference/` folder now: each folder is a feature (decision #452).

| Option | Means |
|---|---|
| **`docs/content/glossary.md`, at the root** (recommended) | A glossary is about the whole product, as getting started is, and the root holds the pages that belong to no one feature. Its type is `reference` |
| `docs/content/content-model/glossary.md` | Beside the glossary's reference. But a reader looking up "fragment" isn't looking for the content model |

**Decided: `docs/content/glossary.md`, at the root, type `reference`.** The root holds what belongs to no one feature, as getting started does, and a glossary is about the whole product; a reader who follows "fragment" wants the word, not the content model's folder. README decision 20.

## Tasks

1. The maintainer reads the six and answers each. _Done,_ 15 October 2026.
2. Record each answer in this file, under its heading, as **Decided:** with the reason, and in the README's decisions, numbered from 15. _Done._
3. Record them in `project-docs/decisions/`, numbered by this plan's issue, as the repository does. _Waits:_ the plan has no issue and no pull request yet, and a decision file is named by one of the two.
4. Add to the proposal what the answers change in it. _Done:_ the acknowledgement, and the limit.

## Out of scope

- The wording of the diagnostics' messages and fixes. Each phase writes its own.
- What the docs' glossary says about each term. Phase 2 writes it, and the maintainer reads it there.

## Acceptance criteria

- Each of the six has an answer and a reason.
- Nothing in a later phase file contradicts one.
- Phase 2 can be started without asking the maintainer anything.

## Commits

1. "Record the decisions the glossary pages plan ships"
