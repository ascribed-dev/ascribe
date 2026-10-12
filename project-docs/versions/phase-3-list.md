# Phase 3: A site that knows its versions

Part of [Versions](README.md). Needs phase 2's first commit, the list in the model, and nothing else of it: it can be built beside the rest of the thread.

## Goal

The second level, whole. With `[releases]` alone and one site, a mistyped version is an error, "planned" is computed, a mark that no longer does anything is found, and a support date that's passed is reported. Ascribe's own docs take it, and stop keeping `unreleased` by hand.

Nothing here needs a versioned build, and its seam isn't the thread's. It's numbered after the thread because the thread comes first in a plan, and it's what Ascribe's own docs wait for, so it shouldn't wait on the thread's Astro half.

## Context

- README decisions 2 to 6 and 22, and phase 1's decisions 1 to 4, 10, and 11.
- `project-docs/decisions.md`, decision 67: `[versions] current` and `availability-left-behind`, which this phase amends.
- `crates/ascribe-check/src/across.rs`: the checks across a project, `availability-left-behind` among them.
- `packages/elements/CONTRACT.md`: the availability element. `crates/ascribe-emit/src/labels.rs`: the text the element and the plain output share.
- `docs/content/reference/cli.md`: `ASCRIBE_TODAY`, the one rule for a check that depends on the day.
- `docs/ascribe.toml`, `site/home/ascribe.toml`, and `scripts/docs-site/facts.test.ts`, which holds the two equal.
- `RELEASING.md`: the step after a release.

## Design

### The rest of the list

```toml
[releases]
planned-label = "planned"

[releases.self-managed]
live = ["3.4", "3.5"]
planned = ["3.6"]

[releases.self-managed.ends]
"3.4" = 2027-01-31

[releases.self-managed.archived]
"3.3" = { url = "https://docs.example.com/archive/3.3/", ended = 2026-06-30 }
"3.2" = {}
```

- `planned` lines are newer than every live line. One that isn't is an error.
- `ends` gives a date for a live line. A key that isn't a live line is an error.
- `archived` lines are older than every live line. `url` is optional (README decision 22), and `ended` is a date.
- A line is in one list.
- A target may declare `planned` or `archived` with no `live` line only if nothing builds it: a build's `versions` still needs one.

### What the list knows about a version in a mark

For a version written in a mark, a feature, or a build's filter, for a target with a list:

| The version is | Reported as | Fix offered |
|---|---|---|
| In a declared line | Nothing | |
| Newer than every declared line | An error | The closest declared line |
| Between two declared lines | Advice: the line isn't declared | Declare it as archived, or the closest declared line |
| Older than every declared line | Nothing: it's before the window, and a project doesn't declare every line it ever shipped. A mark that does nothing is reported by its own check (below) | |

A target with no list reports none of these.

### Marks that no longer do anything

A mark does nothing only when no output shows it. By phase 1's decision 11, a line's output leaves out a version from before the window and every other build shows it. So the first two checks below report only when every build that publishes the content is versioned by the mark's target, or filtered for it. A project with one badge build, Ascribe's docs among them, gets neither: `ascribe 0.1.1` still reads "since 0.1.1" there, and that's the mark doing its job.

Three kinds, each exact because marks are positive and ordered:

- **A mark that's the same in every declared line.** `self-managed 3.1`, with 3.4 the oldest: every line has the content, and no line shows the version. Advice, with next step `write`: the mark can be `self-managed`. Phase 9's `retire` applies it; here it's reported.
- **Content in no declared line.** Removed before the oldest line, with nothing after. A warning: no output has it at all.
- **A feature left behind.** `availability-left-behind` reads each target's current line where it read `[versions] current`.

### `[versions] current` goes

As phase 1's decision 1 has it. A project that sets it gets an error naming `[releases]`. The reference's row, decision 67's amendment, and the two projects in this repository change together.

### Planned

A version in a planned line is planned. In every output, as phase 1's decision 4 has it: the element's attribute and its text, the plain output's availability line, and the JSON output's field. The label is `[releases] planned-label`.

The fixtures in `tests/render/` gain a planned entry, and both renderers pass them. The element's styles for it come from the tokens.

A badge build, a filter build, and the current line of a versioned build all say it. An older line never holds planned content, so never says it.

### A line's `ends`

A live line whose date has passed is advice, as of today in UTC, and `ASCRIBE_TODAY` sets the day, the rule `review-overdue` already follows. The reference's sentence about checks that depend on the day names both.

### The phrase

As phase 1's decision 3 has it. In a line's output it's that line, and elsewhere the target's current line. A phrase that names a target with no list is an error in `ascribe.toml`.

### Ascribe's own docs

- `docs/ascribe.toml` and `site/home/ascribe.toml` gain `[releases.ascribe]`, with the current release live and the next planned. The releases before it aren't declared: a version older than every declared line is before the window and reports nothing, and with one badge build no mark is inert.
- The docs keep one live line. Phase 1's decision 10 is why that holds at the next release, and phase 7 builds it; until then `RELEASING.md`'s step moves the old line to `archived` by hand.
- `[lifecycle.unreleased]` goes. `[features.next]` names the planned line.
- `version` becomes the phrase above, and its test in `scripts/docs-site/facts.test.ts` checks the list against `Cargo.toml` instead.
- The label a reader sees changes from "in main, not yet released" to the planned label. Set `planned-label` so the docs read as they should.
- `RELEASING.md`'s step names the list. Phase 7 replaces it with a command.

## Tasks

1. `planned`, `ends`, `archived`, and `planned-label`, with their rules.
2. The four cases for a version in a mark, each with a conformance case, and the two fixes.
3. The marks that no longer do anything: two new checks, and `availability-left-behind` moved to the list.
4. `[versions] current` removed, with its error.
5. Planned in the element, the plain output, and the JSON output, with the fixtures.
6. The `ends` check.
7. The phrase.
8. The two projects in this repository, and `RELEASING.md`.
9. The reference, the element contract, the guide's section on the second level, entries in `changes/`, and the decisions recorded. The entry for `[versions] current` says the key was in no release, so nobody reading it later has to check whether its removal broke anything.

## Out of scope

- Rewriting a mark. Phase 9.
- `ascribe versions release`. Phase 7.
- An archived line appearing anywhere a reader sees. Phase 5 lists it; phase 8 makes one.

## Acceptance criteria

- `@available: self-managed 3.45` in `examples/quill` is an error that offers the closest line.
- A mark naming a line between two declared ones is advice with both fixes, and declaring the line as archived with no address clears it.
- Content marked for the planned line reads "planned" in the site output, the plain output, and the JSON, and stops when the line moves to `live`, with no other edit.
- With `ASCRIBE_TODAY` set after a line's `ends`, `ascribe check` reports it, and before it, doesn't.
- `docs/` builds with no `unreleased` state and no hand-kept `version`, and its pages read as intended.
- `ascribe check` in `docs/` reports no mark as doing nothing, and none of its marks for earlier releases is touched. In `examples/quill`, whose site build is versioned, a mark naming a line before the window is reported.
- A project with no `[releases]` is byte for byte the same.

## Verify

```sh
cargo test --workspace --locked
pnpm test
cargo run -p ascribe-cli -- check --config docs
node scripts/compare/outputs.ts --base main --accept 'docs: *' --accept 'quill: *'
```

## Commits

1. "Let the list say which lines are planned, ending, and archived"
2. "Report a version the list can't place"
3. "Find the marks that no longer do anything"
4. "Say planned where a version hasn't shipped"
5. "Report a live line whose support has ended"
6. "Give a target's line a phrase"
7. "Take the list in Ascribe's own docs"
