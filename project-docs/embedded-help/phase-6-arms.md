# Phase 6: The reader's arm

Part of [Embedded help](README.md). Needs phase 1.

## Goal

A product shows a section's `pnpm` arm to a reader it knows uses pnpm.

`help.json`'s entry for a section with a switcher on `pm` gains `choices`: for each value of `pm`, the section's text with that arm alone, as the plain output writes a build that selects it. A product that knows its reader's package manager shows `choices.pm.pnpm`; one that doesn't shows `text`, as before.

## Context

- README decision 4: a later version adds keys and changes none. `choices` is a key added to version 1.
- The plain output's switcher (SPEC §9.4): a line saying a reader sees one arm at a time, then each arm as a section. A build that selects an arm writes it as plain content (§9.3).
- Permalink addresses already take a dimension as a parameter (`?pm=pnpm`), which sets the reader's choice on the site: the same dimensions, for the same reason.
- Phase 1's renderer entry, which this calls once per value with that value selected.

## Design

- **`choices`,** present only when the section has a switcher: keyed by the switcher's dimension, then each value in declaration order, each the section's text with that value selected and every other group in the section as it was. A section with switchers on two dimensions has both, each varying its own.
- **The summary** isn't varied: it's one paragraph, and a marked summary inside an arm is phase 2's rule's to settle.
- **Size:** each value repeats the section. A fetch is one entry (phase 5), so the size matters only in the bundled file, which the README's "Carved out" answers: a smaller file, if a product finds it costs.

## Tasks

1. `choices` in `help.rs`, with emit tests: one dimension, two, a labeled arm, and a section with none.
2. The agreement test extended: each choice found in a build's plain output that selects it, where the project has one, and otherwise in the plain renderer's output with it selected.
3. §1.7's addition, and a decision file.
4. The embedded help page: choosing an arm; the program's `--choose pm=pnpm`; `changes/language/help-choices.md`.

## Out of scope

Choosing by availability: a product's version is its line's build (decision 8).

## Acceptance criteria

- The goal's entry, in an emit test.
- The program, run with `--choose`, prints the arm, in its test.

## Verify

```sh
cargo test -p ascribe-emit help
```

## Commits

1. "Give a section's help each arm of its switcher"
