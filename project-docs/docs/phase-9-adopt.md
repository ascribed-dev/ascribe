# Phase 9: Our docs, kept current

Part of [Docs](README.md). Requires phases 5 and 7, and phase 8 if it was built. Docs, a workflow, and measurements.

## Goal

Our own docs use every drift check that exists: generated reference (phase 2), examples taken from tested files, and coverage. CI shows the drift report in its summary, by the recipe the docs give users. Then the measures are taken again, on real pull requests this time, and written down.

## Context

- The pages in `docs/content/`, and the code each describes.
- `examples/` (`quill`, `astro-site`, `monorepo`, `content-models`), the conformance fixtures, the CLI's test snapshots, and the repository's workflows: tested files that the docs' examples were copied from, or could come from.
- Phase 6's map and write-up; phase 7's sources and `@snippet`; phase 8's `covers` and `ascribe drift`, as far as it was built.
- The README's [measures](README.md#the-measures) and `measures.md`.
- [Decisions 1, 10, and 11](README.md#decisions).

## Design

### Examples

Declare the sources in `docs/ascribe.toml`. Then go through every code block in the docs. Each is one of:

- **Taken from a tested file.** Tag the file and use `@snippet`. An `ascribe.toml` example comes from `examples/`; a workflow from the repository's own; a site config from `examples/astro-site`; JSON output from a test's snapshot.
- **Generated** (phase 2 already covers it).
- **Illustrative,** with no real file behind it: a few lines showing syntax. Leave it, and where it could be wrong without anyone noticing, move it into a fixture that's checked and take it from there.

Record the count of each kind before and after.

Production builds on Netlify from the canary, so the snippets' files have to be there at build time. They are: Netlify builds from the same repository. Check it on a preview before merging, since this is the first time the site reads outside `docs/`.

### Coverage

If phase 8 built `covers`: start from phase 6's map, now that it's been tested against history, and write it into the pages as addresses. Prefer a region to a file, and a file to a glob. Pages that describe no code get none.

If phase 8 built the region-level report alone, there's nothing to write: snippets are the coverage.

### In CI

A job on pull requests that runs `ascribe drift --format summary` and writes it to the job's summary, following the recipe in `guides/drift.md`: Ascribe from npm, a full clone. It never fails the job and never comments (decision 11).

### The measures

After at least 15 pull requests have merged with the report on:

- **How often the report was right,** judged as in phase 6, per group. Compare with the back-test's number.
- **Whether anyone acted on it:** for each "right", did the page change in that pull request or a later one, and after how many days?
- **The host's build time,** now with snippets.
- **The time to a working site,** again, if the guides changed.
- **Issues filed from dogfooding,** in total.

Write them into `measures.md`, with what they suggest: whether this repository should turn on `--exit-code`; which item under [Later](README.md#later-not-in-this-plan) the numbers argue for building next (the pull request comment, history, sources in another repository, a setup command); and anything that should be undone.

### How we document Ascribe

A section in `CONTRIBUTING.md`: where a change's docs go, generated fragments and `ASCRIBE_BLESS`, tagging a file for a snippet, `covers`, reading the drift summary, and running the site. If the agents plan's `ascribe agents sync` is merged, run it for `docs/` and commit what that plan says to commit.

## Tasks

1. The sources, and the examples, with the before and after counts.
2. `covers`, if it exists.
3. The CI job, and `guides/drift.md` finished with what running it taught.
4. The measures, once enough pull requests have merged. This can be its own pull request.
5. `CONTRIBUTING.md`; the brainstorm updated (section 1 marked as built; the navigation gap from phase 4 written into section 6).

## Out of scope

Anything under Later; rewriting pages the report shows are stale (fix them in their own pull requests, and list them).

## Acceptance criteria

- No code block in the docs is a hand-kept copy of a file in the repository.
- Production still builds on Netlify, from npm, with the snippets in place.
- The drift summary appears on pull requests, built by the published recipe.
- `measures.md` has the numbers and says what they suggest.

## Verify

```sh
./target/debug/ascribe check --deny-warnings --config docs/ascribe.toml
./target/debug/ascribe drift --config docs/ascribe.toml
cargo test --workspace --locked
cd site && npm ci && npm run build
```

## Commits

1. "Take the docs' examples from tested files"
2. "Say which code each page covers"
3. "Show the drift report in CI"
4. "Record what keeping our docs current has shown"
