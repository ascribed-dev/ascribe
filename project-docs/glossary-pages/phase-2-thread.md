# Phase 2: The thread

Part of [Glossary pages](README.md). Needs phase 1.

## Goal

A glossary is a page: its terms are headings, its definitions are paragraphs, and every project in the repository has moved.

In `examples/monorepo/docs`:

```toml
[glossary]
match = "first"
pages = ["reference/glossary.md"]
```

```markdown
## Rollout
@id: rollout
@term {aliases="rollouts"}

A plan for turning a flag on for a growing share of users: by percentage, by segment, or on a schedule.
```

The `@id` is there to keep the table's key, `rollout`, as the term's id; a heading whose id from its text is already the key needs none ("Moving the projects", below).

`ascribe build` links "rollouts" on `getting-started.md` to `reference/glossary/#rollout`, with that paragraph as the link's title. In VS Code, rewording the paragraph changes the hover on "rollouts" in `getting-started.md` as you type.

This is the thinnest slice that can ship: decision 5 leaves no release with both forms, so every reader moves at once (README, "The thread").

## Context

- README decisions 1 to 8, 11 to 13, 15, 16, 18, and 20, and 21.
- SPEC §4.1, §5.4, and §7.2; `crates/ascribe-core/src/schema.rs`, where `@id` is declared.
- `crates/ascribe-model/src/model.rs` and `sections.rs`: `Glossary`, `GlossaryTerm`, and how `[glossary]` is loaded.
- `crates/ascribe-resolve/src/index/mod.rs`: the source index, which holds every page's headings, its `@available` directives, and its `@snippet` uses, and nothing else of the directives.
- `crates/ascribe-check/src/prose/setup.rs`: Vale's vocabulary of the project's own words, terms among them.
- `site/src/nav.ts` and `site/test/nav.test.ts`: the docs' sidebar, which names every page by hand.
- Every reader of `model.glossary.terms`, listed in the README's "What exists today".
- `crates/ascribe-emit/src/json.rs`, `site/blocks.rs`, and `render/mod.rs`: where a term's definition is written.
- The projects to move: `docs/`, `examples/monorepo/docs`, `examples/monorepo/handbook/pages/security`, `examples/content-models`, and the cases in `tests/conformance/cases/glossary/` and `checks/model-entries-unused/`.

## Design

### The directive and the model

`@term` is a built-in: line form, binding preceding heading (at the top of the section, as `@id`), no primary, and the attributes `aliases` (string), `match` (`first`, `every`, `marked`), and `case-sensitive` (boolean). SPEC gains its section after `@intended`, §5.4 is rewritten for terms on pages, and §7.2 loses `terms` and gains `pages`.

`[glossary]` takes `match`, `case-sensitive`, and `pages`. `terms`, or anything under it, is phase 1's error. A path in `pages` must be a page.

### The project's terms

A term is read from the source index. The index already holds every page's headings and its `@available` directives, and `@term` is added to it the way `@available` was: each one, with its attributes and the heading it's under. That's the "without a second parse" the thread asks about, so the seam's open question is this task. The terms are each heading with `@term` on a page in `pages`, or in a fragment such a page includes, with the heading's text, id, and attributes, and where it is. They're the project's, not the model's, and every reader in the README's list takes them from the project in this phase.

The definition is resolved for each build, by decision 15, when the build resolves the glossary's pages, and before any page is linked. Matching takes the terms and each one's link from there, as it takes them from the model today. A term is linked to the page that defines it, by decision 16 when more than one does.

**A link an author writes to a term's heading is a use of the term** (README decision 21): it gets the marker and the title an automatic link gets, `glossary-term-unused` counts it, and `ascribe refs term:` lists it. Exactly a link whose destination is the term's heading by its id; a link to the glossary page, or to another heading in the section, stays ordinary. It's how a `marked` term is linked, and the docs' three are the first case: their links gain the attribute and the title, named with `--accept`.

**Vale's vocabulary** takes the project's terms and aliases instead of the model's. It's written before each Vale run, only when its words differ (`prepare` in `prose/setup.rs`), so a term added on a glossary page reaches it on the next run with no trigger of its own.

Two terms or aliases with the same text are `glossary-term-duplicate`, at the second, naming the first, from the start: matching can't choose between them, and the model's error that caught it is gone. `@term` on a page that isn't in `pages` isn't a term, and nothing reports it until phase 3.

### The editor

The editor's incremental signature changes when a glossary page's terms change: a heading with `@term` added, removed, or renamed, an attribute, or a definition's text. A change to a glossary page that touches none of those leaves the other pages alone. Hover shows the definition for the editor's build.

### What's removed

`model-glossary-link`, `glossary-link-target-missing`, and `model-glossary-duplicate-term`, with their cases, their registry entries, and their rows in the diagnostics reference; and `ascribe rename`'s and the editor's rewriting of a term's `link`.

**`[[intended]] term` stays until phase 3,** naming the heading's id, which is the term's id. The repository's acknowledgements keep working, since "Moving the projects" keeps each term's id, and `glossary-term-unused` can be acknowledged at every step. Phase 3 adds `@intended` at the heading and removes `term` from `[[intended]]` in one change (decision 17).

### `ascribe model`

As decision 18 says, with its schema rewritten and `schema_version` at 3.

### Moving the projects

Each project lists its glossary pages in `pages`, marks each term's heading with `@term` and the aliases its table had, and drops the table. A term whose table key isn't its heading's id gets an `@id` with the key, so `data-ascribe-term` stays the same; where that would add an id no link uses, the term takes the heading's id instead, and the change is named with `--accept`. `docs/` gains its glossary page, `docs/content/glossary.md` (decision 20): a section for each of its eight terms, a sentence or two and a link on to the page that documents it, for the maintainer to read in this pull request. The site's sidebar names every page by hand, and its test fails on one it doesn't, so `site/src/nav.ts` lists the glossary in the Start group, with the other pages at the content root.

## Tasks

1. `@term`'s schema, SPEC, and the table test.
2. `[glossary] pages`, and `terms` an error, with conformance cases.
3. `@term` in the source index, as `@available` is; the project's terms from it; `glossary-term-duplicate` with its conformance cases; and every reader moved to the terms, Vale's vocabulary among them.
4. Definitions for each build, the title, and the JSON output.
5. The editor's incremental signature and hover, with a test that a definition's edit reaches another page's hover and that an unrelated edit to a glossary page resolves nothing else.
6. The removals.
7. `ascribe model`'s terms and schema.
8. Every project moved, `docs/`'s glossary page, and the page in the site's sidebar.
9. A link an author writes to a term's heading as a use of the term (README decision 21), with conformance cases for a `marked` term, a link to the glossary page, and a link to another heading in a term's section.
10. The content model reference and contract, the directives reference, the diagnostics reference, an entry in `changes/` that says **Behavior change**, and the surface map.

## Out of scope

- A term with no definition, a long definition, and `@term` off a glossary page (phase 3).
- `@intended` at a term's heading, and `term` leaving `[[intended]]` (phase 3).
- Go to definition, completion, quick fixes, and rename (phase 4).
- A guide (phase 5).

## Acceptance criteria

- In `examples/monorepo/docs`, `ascribe build` links "rollouts" on `getting-started.md` with the glossary paragraph as its title, and the JSON output lists the term on that block with the same text.
- A project with `[glossary.terms]` gets phase 1's error, naming `pages` and `@term`.
- No reader of the old terms is left: `git grep "glossary.terms"` finds only the error's message and the docs that say the table is gone.
- In the editor's tests, rewording a definition changes the hover on a use in another page, and editing a glossary page's other text doesn't resolve the other pages again.
- `node scripts/compare/outputs.ts --base main` shows differences only in the moved projects' pages, each named with `--accept`: titles where the paragraph's words differ from the old definition, and the docs' new glossary page and its links.
- `ascribe check --deny-warnings` passes on `docs` and on every example, and the site's tests pass with the glossary in the sidebar.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- build --config examples/monorepo/docs
cargo run -p ascribe-cli -- check --deny-warnings --config docs
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Declare a glossary term with @term on a page in [glossary] pages"
2. "Read every term from the glossary's pages"
3. "Move the repository's glossaries onto pages"
