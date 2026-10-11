# Phase 7: For agents

Part of [Search and answers](README.md). Requires phase 2. The plain emitter, a command, and the docs.

## Goal

Two things the research found evidence for, and one question left by the website:

- **The Markdown version of a page has every variant, written out and labeled,** so what's in a tab that isn't showing isn't lost when an agent reads the page.
- **A product can ship a compact index of its docs,** for its own `AGENTS.md`, with the Markdown pages as a folder that can go inside a package.
- **Whether a site's root gets an `llms.txt`,** when its docs are under a path.

## Context

- The proposal: "What a build would write", and "What the research found": the one controlled benchmark (a pointer to the index cut dead fetches from 2.23 to 0.11 a task, which Ascribe ships), and the one first-party evaluation of an index in `AGENTS.md` (53% to 100%, with 8 KB).
- `crates/ascribe-emit/src/plain/`: how the plain output writes a variant group today, for a build with `variants = "switch"` and for one that filters. The surface audit's finding 56 is the same output seen from `ascribe render`: every arm in a row under a bold label, with nothing saying a reader sees one.
- `crates/ascribe-resolve`: `llms`, its sections, and its splitting.
- `ascribe agents sync`, and what it writes: it's the docs' own agents' files, not a product's. This phase's index is for the product the docs are about.
- `project-docs/checklists.md`: naming a command, and its `--format` words.
- The delivery spec's checks, as `ascribe report agents` runs them, for what they say about an index at a host's root.

## Design

### Variants, in the Markdown version

For a build that shows a switcher, the Markdown version writes each arm under a heading-level label that says the dimension and the value, and one line above the group that says a reader chooses one: "Shown one at a time, by operating system." `ascribe render` prints the plain output's text, so it gains the line too. That's what the surface audit's finding 56 asks of `render`, in [#271](https://github.com/ascribed-dev/ascribe/issues/271): that part of the issue closes with this change, or is already closed by the same line if #271 landed first, and then this phase takes the wording it chose.

For a build that filters to one value, nothing changes: the page has only that arm.

This changes the plain output for every page with a switcher, and what `ascribe render` prints for one. It's an output change by a recorded decision: the pull request carries the `outputs changed` label, and names each file for the comparison.

### The compact index

A command that writes, for one build, a block of Markdown small enough to paste into a product's `AGENTS.md`: one line a page, title and address, grouped as `llms.txt` is, with a size the command holds to (about 8 KB, the size the evidence is for) by leaving descriptions out before it leaves pages out, and saying what it left out.

- Its name follows the checklist. It's about the docs as a product's agents meet them, so it belongs with `ascribe agents`: `ascribe agents index`. Confirm against the map of the surface before it ships.
- With `--pages <DIR>`, it also copies the build's Markdown versions into a folder, and the index's addresses are paths into it. That's the folder a package ships.
- Only offered pages are in either.
- It writes to standard output, or to a block between markers in a file, as `ascribe agents sync` does for the rules, so a second run replaces the first.

### The root

A site whose docs are under `/docs/` has its `llms.txt` at `/docs/llms.txt`, and nothing at the root. The weekly Report workflow was pointed at `/docs/` when the docs moved there, so the delivery spec's checker has been judging the docs' own index and has said nothing about the root. Run `ascribe report agents --site` once against the root, to see what it says there, before deciding. Then decide, by a recorded decision, with that in hand:

| Option | Means |
|---|---|
| **The integration writes a root `llms.txt` that points at each project's** (recommended, if the checker accepts it) | One short file at the root, listing the indexes a site has. Each project's own stays where its pages are |
| The docs' index is also published at the root | One more copy, with addresses under `/docs/` |
| Nothing | The pointer on every page already leads to the right one |

The proposal's own finding weighs on this: `llms.txt` isn't how AI search finds a site. The pointer on each page is what had the measured effect, and it's there. So "nothing" is a fair answer, and the recommendation holds only if a root file costs a few lines.

## Tasks

1. The variants' labels and the line above them, in the plain emitter, with the outputs comparison's accepted files listed and the decision recorded.
2. `ascribe render`'s text follows, with its test, and #271's line about `render` is closed by it.
3. The command: its arguments, its size rule, `--pages`, the block between markers, its JSON, and its tests. The command reference, generated.
4. The root's `llms.txt`: the checker run against the root, the decision, then what it decides, including whether the Report workflow checks the root as well as `/docs/`.
5. `guides/agents.md`: a section for a product's team, saying what the evidence is and that it's one vendor's. `CHANGELOG.md`. The map of the surface.
6. Ascribe's own use: the index for `ascribe`, in the README the CLI's package ships, or a line saying why not.

## Out of scope

Version and status in the Markdown version's first lines, and an `llms.txt` for each version, which wait for versions. `llms-full.txt`. A docs server. Anything shown only to agents.

## Acceptance criteria

- In `examples/quill`, the Markdown version of the page with a switcher has each arm under its label, and the line above them.
- The index for Ascribe's docs is under the size, lists every offered page or says how many it left out, and no page that isn't offered.
- A second run of the command on a file with the block changes nothing.
- `node scripts/compare/outputs.ts --base main` shows the plain output's changed pages, each named with `--accept`, and nothing else.

## Verify

```sh
cargo test --workspace --locked
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Write every variant, labeled, in a page's Markdown version"
2. "Write a compact index of the docs for a product's agents"
3. "Say where a site's indexes are, at its root"
