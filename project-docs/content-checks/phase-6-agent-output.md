# Phase 6: Output agents can read

Part of [Content checks](README.md). Requires phase 1. Rust and `@ascribed/astro`. An addition to the outputs' contracts.

## Goal

A site built with Ascribe meets the [Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) wherever the build can decide the answer: an `llms.txt`, a Markdown version of every page, and a pointer to the index on each page. Most of the spec then holds by construction, and the spec's own checker proves it in CI.

This is in a checks project because it's what makes the checks pass: the alternative is reporting 20 problems an author can't fix.

## Context

- The brainstorm's section 7: the table of what the spec asks and what Ascribe has, and how `afdocs` runs.
- The spec and `afdocs`, both read at their current versions. The brainstorm quotes spec draft 0.6.0 and `afdocs` 0.22.2; both are pre-1.0 and their check ids may have changed. Pin the versions this phase targets, in one place.
- `crates/ascribe-emit`: the `plain` output ("for search indexes and LLMs") and the output layout. `docs/content/contracts/output-layout.md` and `site-render.md`: the contracts this phase adds to.
- `packages/astro`: the integration that serves the site output. `examples/astro-site`: the site the Astro end-to-end job builds and tests in `.github/workflows/js.yml`.
- `docs/content/reference/content-model.md` §15, `[consumer]`: `site`, the address absolute links need.
- `project-docs/checklists.md`: the lists for a change to an output and to the site output's markup. Two renderers must agree (`AGENTS.md`).

## Design

- **`llms.txt`, per build.** Written with the build's outputs: the site's name, a summary, and each published page as `[Title](url): description`, linking to the Markdown version. Sections come from folders until the content model has navigation. When it would pass the spec's size limit, it nests: a root file linking to one per section.
- **A Markdown version of each page.** The `plain` output, published beside the HTML at the URL the spec expects. `@ascribed/astro` serves it; a project using another generator gets the files and a documented layout.
- **The pointer.** A visually hidden element near the top of each page in the site output (a markup change: both renderers, the fixtures in `tests/render/`, and the elements' contract), and a blockquote at the top of each Markdown version, with an absolute URL.
- **Variants.** In the Markdown version, a variant's arms become labeled sections whose headings carry the variant ("Install the CLI (pnpm)"). A build that selects one value has no arms left, and already reads cleanly.
- **Opt-in.** `[consumer] agents = true`, or the name the content-model reference suggests on reading. It requires `[consumer] site`, since every link must be absolute; without it, a model error says so.
- **What stays the host's:** status codes, cache headers, content negotiation, bot protection. The guide says what to set for the hosts the docs already name, and the report (phase 7) says which of them a live site fails.

### Checks this adds

Few, because the rest hold by construction:

| Check | Kind | When |
|---|---|---|
| `description-too-long` for an `llms.txt` entry | `write` | With phase 4's checks, if it landed; else here |
| `llms-section-large`: one folder's index would pass the limit even nested | `write`: restructure, or `review` | `ascribe check` |

`page-size` and `page-description-missing` are phase 3's.

### Proving it

In the Astro end-to-end job: build `examples/astro-site` with the option on, serve it, and run `afdocs` through its vitest helpers, one test per check. The checks only a real host can pass are skipped by name, with the reason beside each. Record the score before this phase and after it in the pull request.

## Tasks

1. A baseline: `afdocs` on today's `examples/astro-site`, with the failing checks listed.
2. `llms.txt` and its nesting, in the emitter, with conformance cases and the output-layout contract.
3. Publishing the Markdown versions through `@ascribed/astro`.
4. The pointer, in both renderers, with the render fixtures and the elements' contract.
5. Variant headings in the plain output. This changes an existing output: name the changed files in the outputs comparison, and record the decision.
6. The `afdocs` tests in CI, pinned.
7. Turn it on for the docs site, and fix what `afdocs` finds there.
8. `docs/content/guides/astro.md` (turning it on, and the host's part), the contracts, `CHANGELOG.md`, and `project-docs/decisions.md`.

## Out of scope

Content negotiation and other server behavior; per-value pages for a dimension the site shows as tabs (the brainstorm's open question; decide it from `afdocs`'s result on labeled sections first); redirects.

## Acceptance criteria

- With the option off, every output is byte for byte what it was, apart from the variant headings of task 5.
- With it on, every `afdocs` check that doesn't need a real host passes on `examples/astro-site`, in CI.
- `llms.txt` lists exactly the pages the build publishes, by test.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test:e2e
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Write llms.txt with each build"
2. "Publish each page's Markdown beside it"
3. "Point each page at the index"
4. "Name the variant in a section's heading in the plain output"
5. "Check the example site against the delivery spec"
