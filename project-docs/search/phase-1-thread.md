# Phase 1: The thread

Part of [Search](README.md). Needs nothing.

## Goal

`ascribe search` finds words as readers read them in each page's own text, through phrases, in every build, and names the line to edit.

In `examples/quill`:

```console
$ ascribe search '"sign in to quill cloud"'
docs/install-agent.md:62: Sign in to Quill Cloud and copy an API key from Settings → Keys, then add it to…
    Connect to Quill #connect · deployment=cloud arm · through phrase cloud

1 place matches, on 1 page.
```

It exits with `0`. The line is in all three builds, so it names none. `grep -ri "sign in to quill cloud" docs` finds nothing. `ascribe search '"quill cloud"'` gives one place, the phrase's line in `ascribe.toml`, and says how many places use it and `ascribe refs phrase:cloud`. `ascribe search "zebra"` prints "No place matches." and exits with `1`.

## Context

- README decisions 5 to 12, 14, 15, and 18 to 22. The proposal's "A query", "What's searched", "A result", and "The seam".
- `crates/ascribe-resolve/src/build/tree.rs`: `ResolvedPage`, `ResolvedBlock` (`file`, `span`, `via`, `heading`, `rows`, `substitutions`, `links`, `snippet`), and how an arm's blocks are kept or dropped for a build.
- `crates/ascribe-check/src/prose/extract.rs`: `Prose`, its pieces (`Copied`, `Made`, `Phrase`), `locate`, and the property test at the end of the file. The method to follow; its code stays where it is (decision 1).
- `crates/ascribe-diff/src/tree.rs`: `TreeBuilder::block` and `push_inlines`, which turn a resolved block into text a reader reads, and `collapse`. What the reading must agree with, and phase 6 replaces.
- `crates/ascribe-query/src/render.rs`: resolving a page for a build (`project.for_build`, `resolve_page`, `dropped`). `crates/ascribe-query/src/refs.rs` and `crates/ascribe-cli/src/commands/refs.rs`: a question's entry, its `Place`, its limits, `next_command`, and its exit codes.
- `crates/ascribe-check/src/builds.rs`: `select_builds`.
- `crates/ascribe-cli/src/exit.rs`: the error codes, listed once in a test.
- `examples/quill/docs/install-agent.md`, lines 58 to 76, and `examples/quill/ascribe.toml`'s phrases and builds.

## Design

### The reading

`ascribe_resolve::reading` (decision 19) turns a resolved page into its blocks as a reader reads them. Each block of the reading has:

- **its kind,** as JSON will name it later: `heading`, `text` (a paragraph, a list item's own text, a block quote's, a directive's text and title), `table` (a row), or `code` (a code block written in the page);
- **its text,** with markup removed (decision 7) and white space collapsed to single spaces;
- **its pieces,** in order and touching, each a range of the text with the source span it stands for, `Copied`, `Made`, or `Phrase(key)`, as the Vale map has;
- **the heading it's under,** with that heading's id in the build, and **its arm,** when it's inside a variant group the build keeps more than one arm of, or the arm the build keeps.

A table row is one block: its cells' text joined by made spaces, as `diff` joins them. A code block's text is its lines, joined the same way. A list item's text is its own, not its children's. An HTML block isn't in the reading: the plain output leaves it out, and search never matches one (decision 8). Phase 6 adds it, for `diff`.

**A link with no text** reads as its target's title, as a reader sees it: a piece that stands for the link's span, marked as borrowed from the target's page, so a match in it is a place at the link, `through <page>'s title`. It's in the thread because the thread's own page has one, and the test against the plain output below can't pass with the title left out.

In this phase the reading has the blocks written in the page's own file: blocks that came through an include, and code a `@snippet` became, are left out. Phase 2 adds each.

**The property test,** after the Vale map's: for generated pages with phrases, links, emphasis, code spans, and tables, every character of every block's text is in exactly one piece, every `Copied` piece's text is the source's at its span, and every `Phrase` piece's text is its key's value.

**The test against the plain output:** for every project under `examples/` and `tests/conformance/cases/`, for every build and page, each block's text is found, in order, in that page's plain output read the same way: parsed, markup removed, white space collapsed. A block the reading has and the plain output doesn't is a bug in one of them.

### Matching

`ascribe_resolve::reading::Query` parses decision 6's grammar: words, and phrases in double quotes. An empty query, or one with an unclosed quote, is an error with its own code. Matching lowercases a copy of each block's text once, and the query once, and looks for each word, or each phrase with any run of spaces matching any run of white space, as a substring. A block matches when every word and phrase is in it. The match's range is from the first character any of them covers to the last.

### From a match to a place

`Reading::locate` turns a match's range into a place:

- **The line:** the source line where the match's first character is written. For a table row, the row's line; for a code block, the line in the page.
- **Through a phrase,** when the match covers any of a `Phrase` piece: `through phrase <key>`, each key once, in order.
- **Wholly inside one phrase's value:** the place is the phrase's line in `ascribe.toml`, not the page's, and it says how many places use the phrase and gives `ascribe refs phrase:<key>`.
- **After a phrase,** when the match is wholly inside its value: matching carries on in the block after the phrase. A later match wholly in a phrase is that phrase's place too; the first that isn't makes the line a place, and the block stops there.

### The places, and the entry

`ascribe_resolve::reading::find` takes the readings of the pages searched, each with its build, and the query, and returns the places, merged and in order. It's below both callers, so the command and the language server (phase 5) can't disagree about what a place is:

- **One place per source line,** however many builds or pages find it. Its builds are the builds that matched it; its text, the first of them, in `ascribe.toml`'s order. A phrase whose value differs by release line can match in one build and not another; the place lists only those it matched in.
- **A line with two matches** is one place, at the first that isn't wholly in a phrase.
- **The order** is decision 10's, without titles until phase 2: headings, then the rest, each in path order, then by line.
- **The count of pages** is of the pages a reader can open that show a place.

`ascribe_query::search(project, query, builds, limit)` is the command's entry: it chooses the builds with `select_builds`, resolves every page each build publishes, reads each, calls `find`, and returns the places, at most `limit`, the total, and the builds searched.

### The command

`ascribe search QUERY [--build NAME]... [--limit N]`, by decision 18. Paths after the query are phase 3's; in this phase, a second argument is a usage error that says to quote the query. Text output, as the goal shows. A cut list ends with the count and the command that gives the rest, as `refs` does. Exit codes by decision 14; an unknown build, an empty query, and a broken quote are `2`.

### A baseline

Decision 41 asks for a budget recorded on the runner before anything is sped up, and the thread is where a cold query first costs what it costs. `search/synthetic-3000` joins the corpora's metrics: a search for a common word across every build of the synthetic project, from cold. It's recorded with no target, so the phases after can see what each adds.

## Tasks

1. The reading of a page's own blocks, with its pieces, and the property test.
2. The query and the matching, with unit tests: words in any order, a phrase across a wrapped line, case, a word inside a longer word, a match that never crosses blocks, a row's cells, and a link with no text matched by its target's title.
3. `locate`, with tests for a match in copied text, one that starts in copied text and ends in a phrase's value (the thread's), one that covers two phrases, one wholly inside a value, and one in a borrowed title.
4. The test holding the reading to the plain output.
5. `find`, with tests: the thread's place, a line in one build of three, a phrase that differs by release line, and the order. `ascribe_query::search`, with a test of the limit.
6. The command, its new error codes in `exit.rs`, and its output in `crates/ascribe-cli/tests/output/` (`search.txt`, and `search-none.txt` for nothing found), observed again.
7. The metric, recorded with the corpora's bless step and its diff read.
8. The docs: `ascribe search` in `docs/content/command/questions.md`, `@id {permalink=true}: cli-search`, `@available: next`, with its options' fragment, the example's output, what it searches and what it can't (decision 8), that a fragment's text isn't searched yet, and its exit codes; the page's title and `docs/content/command/index.md`'s list name it; `ARCHITECTURE.md`'s list of commands' entries, and `ascribe-resolve`'s README for the reading; `changes/command/search.md`; and the surface map.

## Out of scope

Everything in the README's "Left out" table.

## Acceptance criteria

- The goal's three commands, in `crates/ascribe-cli/tests/`.
- The property test, and the test against the plain output, pass for every project in the repository.
- `search/synthetic-3000` is in `tests/corpora/baselines/perf.json`.
- `node scripts/compare/outputs.ts --base main` shows every project byte for byte the same.

## Verify

```sh
cargo test -p ascribe-resolve reading
cargo test -p ascribe-query search
cargo test -p ascribe-cli search
cargo run -p ascribe-cli -- search '"sign in to quill cloud"' --config examples/quill
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Read a resolved page as its readers do, with a map to the source"
2. "Find words in a build's pages and name the line to edit"
3. "Add ascribe search"
