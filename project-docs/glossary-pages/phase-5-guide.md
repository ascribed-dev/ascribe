# Phase 5: A guide

Part of [Glossary pages](README.md). Needs phases 2 to 4.

## Goal

A writer can set up a glossary from one page. Today nothing teaches it: the reference describes `[glossary]`, and no guide walks through it.

## Context

- Every earlier phase's changes to `docs/content/`: the directives reference, the content model reference and its contract, the diagnostics reference, and the editor's page.
- `docs/content/content-model/`: the feature folder the glossary's reference is in, and where the guide goes (decision #452).
- Ascribe's own glossary page, from phase 2, as the worked example.
- The Ascribe skill (`packages/cli/skills/ascribe/`) and the plugin's copy, which teach agents the directives.

## Design

A how-to in `docs/content/content-model/`, "Set up a glossary":

1. Write the page: a heading for each term with `@term` under it, and the definition as the section's first paragraph, a sentence that stands on its own.
2. List it in `[glossary] pages`.
3. Choose how terms are linked: `first`, `every`, or `marked`, for the glossary and for a term.
4. Aliases, and a term that's also an ordinary word.
5. A term that's new in a version, or differs by product, with `@available` and variants on its section.
6. A glossary in several pages, and terms shared through a fragment.
7. What the checks report, and what a site gets: `data-ascribe-term` and the definition as the link's title.

It links each reference rather than repeating it, and Ascribe's glossary page is its example.

The skill's guidance for agents gains a line: a term is defined once, on a glossary page, and a definition's first sentence is what a hover shows.

## Tasks

1. The guide, with `type: how-to`.
2. Links to it from the directives reference's `@term`, the content model reference's `[glossary]`, and the editor's page.
3. The skill's line, and the plugin's copy.
4. An entry in `changes/`.

## Out of scope

- A tutorial.

## Acceptance criteria

- `ascribe check --deny-warnings` passes on `docs`.
- Following the guide in an empty project started with `ascribe init` gives a glossary whose terms are linked in a second page, with no diagnostics.

## Verify

```sh
cargo run -p ascribe-cli -- check --deny-warnings --config docs
```

## Commits

1. "Add a guide to setting up a glossary"
