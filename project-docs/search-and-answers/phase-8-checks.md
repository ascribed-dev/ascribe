# Phase 8: Checks, and the table

Part of [Search and answers](README.md). Requires phases 3 and 6. `ascribe-check`, `ascribe report`, and the docs.

## Goal

What would disagree is reported before a deploy, and a team can see every page's record without crawling its own site:

```sh
$ ascribe check
[ASC…] Advice: two pages have the title `Configuration`
$ ascribe report pages
```

## Context

- The proposal: "Checks before deploy" and "For the team".
- README decisions 10 and 11: advice, and no score.
- The content checks contract, in `docs/content/reference/diagnostics.md` and decisions 60 to 63: a check's level, its kind of next step, `[checks]`, and acknowledgements.
- The checks that exist and are near these: `description-too-long`, `llms-section-large`, `page-orphan`, the missing-description check, and the two this plan has added.
- `ascribe report`: its sections, its JSON, `--exit-code`, and the `agents` section, which runs the delivery spec's checker against a published site. `crates/ascribe-check/` for where a section is built.
- The surface audit's finding 9 and [#270](https://github.com/ascribed-dev/ascribe/issues/270): what a report calls what it found. Use the word that issue settles on.
- The delivery spec's checks, one by one, sorted into what needs a server and what doesn't.

## Design

### The checks

Each is advice, configurable, with its kind of next step, a conformance case, and a line in the diagnostics reference. Each is decided on its own: one that can't be said in a sentence a writer would act on is left out, and the pull request says which and why.

**Titles and descriptions:**

- two pages of a build with the same title, or the same description (`review`: they may be two versions of one page);
- a title that's only the site's name;
- a title that disagrees with the page's own top heading, for a page that has one. Which heading: the `h1` in the page's content. Most docs pages have none, since the layout draws the title from the frontmatter, and they aren't looked at. A page that writes its own, as a landing page does inside its hero, is.

**The record against itself,** which are errors where the project can't be right and advice where it may be:

- a canonical address that isn't a page of the build;
- a page that isn't listed, and is another page's canonical;
- a page in a rule's folder that the rule can't reach, because the folder's name is misspelled (phase 3's validation may already have it).

**For agents, without a server:** the delivery spec's checks that read only what a build writes. Each one Ascribe can decide from the output is a check here, named for the spec's own, so `ascribe report agents` on the published site and `ascribe check` before it agree. Each that needs a response stays in `report`.

**Not here:** links through redirects, chains and loops, an address that went away. They need redirects, and the record of what was published (README, "Carved out").

### The table

A section of `ascribe report`, `pages`: every page of each build, with its address, title, description, canonical, listing, and where the listing came from.

- As text, a table cut at `--limit`, with what's left out counted.
- As JSON, the records, under the report's schema. Not a second shape: the same fields `_ascribe/pages.json` has, by the same names.
- Sorted by address. A filter for what isn't `listed`, since "what am I hiding, and why" is the question the table is for.

It reads nothing a build hasn't worked out, so it needs no network, and runs where `problems` and `inventory` do.

### What to measure

A page of the guide, and no code. Where a team looks, since Ascribe sees no traffic: the search engines' own reports; requests for the Markdown versions in a host's logs, which are the cleanest sign of agents reading; and, for whether answers are right, a team's own set of questions asked again after a change. Ascribe calls no model, so it describes that and doesn't run it.

The page says which claims have evidence and which don't, from the proposal's "Where the evidence is thin". It's the guide most likely to be read as a promise.

## Tasks

1. Each check: the registry, the check, its conformance cases, its reference line. One commit each, so one can be dropped.
2. The delivery spec's checks, sorted, with the ones a build can decide added and named for the spec's.
3. `ascribe report pages`: the section, its text, its JSON and schema, its tests, and the command reference.
4. Run all of it on Ascribe's docs. Fix what's wrong in the docs, acknowledge what's intended, and say in the pull request how many of each. A check that speaks on more than a few of Ascribe's own pages without cause is too loud, and its level or its rule changes before it ships.
5. The guide's page on measuring. `CHANGELOG.md`. The map of the surface. The decisions.

## Out of scope

A score. Checks of content style. Anything that needs a deployed site, a model, or what was published before.

## Acceptance criteria

- Each check has a conformance case that reports it and one beside it that doesn't.
- `ascribe check` on Ascribe's docs is clean, with each acknowledgement carrying its reason.
- `ascribe report pages --format json` validates against the schema, and its records equal `_ascribe/pages.json`'s for the same build. A test compares them.
- A project that sets nothing and has no problems of these kinds gets no new output from `check`.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- check docs
cargo run -p ascribe-cli -- report pages --config docs
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

## Commits

One for each check, then:

1. "List every page's record in the report"
2. "Say what a team can measure, and what nobody has"
