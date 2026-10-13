# Phase 3: The six

Part of [Page templates](README.md). Requires phase 2. Mostly the conversion, and writing.

## Goal

The whole starter set: concept, how-to, tutorial, reference, troubleshooting, and release notes, each with its guide and Ascribe's notes, and `ascribe templates eject` with no name writes all six and a content model that declares them.

## Context

- README decisions 1 to 4 and 10: convert, don't rewrite; three layers; a rule that doesn't match stops the script.
- Their six templates and guides at the pinned release. Read each whole. The proposal counted 561 placeholders across the set, and 5 errors and 18 warnings from six templates checked unchanged.
- The script and its rule file, as phase 2 left them.
- What Ascribe has that their templates don't know about: `@steps`, `@note` and a project's note types, `@variant`, `@available`, `@snippet`, `@details`, phrases, and the glossary.
- `docs/content/writing/directives.md`, for how the docs describe each.

## Design

### Five more through the conversion

Each template is taken through the script as the how-to was. What's known of the five, from their v1.6.0 files:

- **The reference template is tables:** six rows with a placeholder in each cell. That's not a risk, it's the template. The escape has to reach a table's cells, and it does more there than elsewhere: `\{` in a row's first cell also keeps the cell from being read as the row's attribute block.
- **The release notes' title has two placeholders in it,** `Release notes - {Product name} {Release number - 0.0.0}`, which goes to frontmatter whole (README decision 21).
- **None of the six has a code fence,** so "a brace inside code isn't escaped" has no case among them. The rule stays, held by the script's own test, and its first real case will come from a later pack.

The risk is a template the steps don't fit in some way nobody has read for yet: a nested list, a heading that is a placeholder, a placeholder across blocks that isn't shaped like the how-to's.

- **A rule is added only where a template needs one,** and each is the smallest that works: "the numbered list under this heading is a procedure", "this quoted paragraph beginning 'Note:' is a note".
- **A rule changes structure, never words.**
- **If a template needs a step that isn't mechanical,** stop and report before writing it by hand. The choices then are to ship the set without that template, or to send a change to their repository.

The count of rules is worth watching. A handful across six templates is the design working. Twenty is a sign the set is being rewritten by other means, and that's reported too.

### Ascribe's notes on each type

Six short files, written by hand, one for each type: what's particular to writing that kind of page in Ascribe, and nothing their guide already says.

- For a how-to: alternatives by platform are a variant group; a step's command comes from tested code with `@snippet`.
- For a reference: what's generated doesn't belong in a hand-written page.
- For release notes: availability marks, and what `ascribe report` already says about a release.
- For a concept, a tutorial, and troubleshooting: found by writing one page of each first.

Each is a page a person would read in a minute. They're the only words in the set that are Ascribe's, so they're held to the docs' standard, and the maintainer reads them.

### The content model for six types

`eject` with no name adds six types, each with its `files` folder and the frontmatter a page of that kind needs: a title and a description for all, and no more unless their template's own front matter asks for it.

The folders are a starting structure, and the output says so: where each type's pages go, and that `files` is the line to change.

### The docs' page on templates

Complete now: what the set is, where it's from and at which release and license, how to take it, how to change a template, what the check finds and what it can't, and what Ascribe's notes are.

## Tasks

1. Each of the five, converted, with its rules, one commit each, so one can be held back.
2. The tests of phase 2, now over six.
3. The six notes.
4. `eject` with no name, and the six types' declarations, with a test that a project made this way checks clean.
5. The example project grows to the six.
6. The docs' page, `CHANGELOG.md`, and the map of the surface.

## Out of scope

Any template outside the six. A rule that changes their words. Checking a page's shape against its type.

## Acceptance criteria

- All six templates pass `ascribe check` in a project that declares their types, and `ascribe fmt --check`.
- Every rule in the data file is used by exactly one place, and removing any one makes the script stop.
- A page copied from each template reports its placeholders, and reports nothing once they're filled.
- The maintainer has read the six notes.
- The pull request says how many rules the six needed, and names any template that needed a step by hand.

## Verify

```sh
cargo test --workspace --locked
pnpm test
```

## Commits

One for each template, then:

1. "Write Ascribe's notes on each page type"
2. "Give a project the six types with one command"
