# Phase 5: Why Ascribe

Part of [The website](README.md). Requires phase 3, for the widgets and the finding. Mostly writing.

## Goal

`https://ascribed-dev.com/why/` makes the argument at length, for someone comparing tools: the problems teams that keep docs as code report, what Ascribe does about each, and what it deliberately doesn't do.

## Context

- The [proposal](../proposals/website.md): "Why Ascribe".
- Phase 1's decision 5: whether the page names other tools.
- `project-docs/reports/`: the research the problems come from. Each claim about what teams report should be traceable to one.
- README decision 17: the page describes approaches, and links to the comparison only once the comparison is fit to link.
- `examples/comparison/`: the same content written for Astro, Docker's docs, and Elastic's, beside Ascribe's, in `tessera.md`. Its README calls Ascribe "Tessera", cites `BRAINSTORM.md` sections, and describes gap markers the Ascribe page no longer has. The folder has no `ascribe.toml`, so nothing checks that page.
- `examples/monorepo`, and the test that keeps it valid: the pattern for the comparison's.
- `README.md`: the pitch as it's written today.
- Phase 3's finding: whether this page is an Ascribe page or an Astro one.
- `CONTRIBUTING.md`'s notes on how the docs are written.

## Design

The page has three parts.

- **The problems.** A few, each one a reader would recognize from their own work: an example that stopped matching the code, a link that broke in a reorganization, one page copied for two products and drifting, a review of a diff nobody can read as a page. Each is stated from the research, without a number the research doesn't have.
- **What Ascribe does about each,** with an example where one helps, read from real files as the landing page's are, and marked where no release has it.
- **What it doesn't do.** No hosting. No model. No navigation in the content model yet. One site generator supported. Each with a sentence on why, or on what's planned.

It's written to the standard of the docs: plain sentences, no superlatives, nothing a reader could check and find false.

Other tools are handled as README decision 17 has it. The page describes approaches and names no tool. It links to `examples/comparison/` on GitHub for the same content in each, and that link is this phase's to earn.

### The comparison, made fit to link

Done first, since the page's link waits on it:

- **The Ascribe version is rewritten in current Ascribe,** under an `ascribe.toml` in the folder, and renamed for what it is. What the other three files show stays as it is; they're other tools' formats and aren't Ascribe pages.
- **A test keeps it valid,** as `examples/monorepo`'s does: `ascribe check` on the folder passes, in the suite that runs the examples.
- **The README is brought current:** Ascribe by its name, no pointers into files that are gone, and what it says about each version checked against the version.
- **Nothing else references the folder today,** so nothing else moves.

If this can't be finished in the phase, the page ships without the link, and says nothing about a comparison.

The header gains Why Ascribe, and the landing page links to it.

## Tasks

1. The comparison: the Ascribe version under an `ascribe.toml`, its test, and its README.
2. The page, in the marketing project (or as phase 3's finding says).
3. Its examples and marks.
4. The header's link, and the landing page's.
5. The maintainer reads it before it merges. It's the site's longest claim about the product.

## Out of scope

A page per feature. Pricing. A comparison table of tools.

## Acceptance criteria

- Every problem the page names is in a report under `project-docs/reports/`, and the pull request says which.
- Every limit the proposal lists is on the page.
- `ascribe check` passes on `examples/comparison/`, a test runs it, and the README names nothing that doesn't exist. Or the page has no link to it.
- `ascribe check` passes on the marketing project, and the site's link test passes.
- The maintainer has approved the text.

## Verify

```sh
cargo run -p ascribe-cli -- check site/home
cargo run -p ascribe-cli -- check examples/comparison
cd site && npm run build:checkout && npm test
```

## Commits

1. "Make the comparison a valid Ascribe project"
2. "Add a page that says why Ascribe exists"
