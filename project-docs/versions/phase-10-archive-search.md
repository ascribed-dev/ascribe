# Phase 10: Search in an archive

Part of [Versions](README.md). Needs phase 8.

## Goal

An archive can be searched, from a folder opened in a browser, with no server and no request to anywhere else. A box on every page finds pages and headings by their words, and the archive is still complete and readable with scripts off.

It's last because nothing waits for it, and because it's the one part of an archive that needs a script, which phase 8 decided an archive otherwise doesn't have.

## Context

- README decision 15: an archive "has search", and "depends on nothing".
- Phase 8's second decision: no script in an archive. This phase amends it, for one script.
- The [search proposal](../proposals/search.md): search across a live project. If it's built first, its index and its ranking are what this phase reuses, and this file should be read against it before starting.
- `crates/ascribe-diff/src/html/`, and `packages/review`'s `embed`: how a script built in a package is carried in the binary.
- Pagefind, which the docs site uses: its index is fetched in pieces, which a page opened with `file:` can't do.

## Decision first

**What the index is, and how a page opened from disk loads it.** A page at a `file:` address can't fetch another file, so the index can't be JSON read on demand.

| Option | Means |
|---|---|
| **A script file that holds the index, loaded by a `<script>` tag when the box is first used** (recommended) | Works from disk and from a host. One file, the size of the archive's text, compressed by what a browser can inflate itself |
| The index inside every page | No second file. Every page carries the whole archive |
| Pagefind's index | A component the docs already use. It can't be read from disk, which is the one place an archive must work |

Recorded before code, with the size it comes to for the docs.

## Design

### The index

Written by `ascribe versions archive` with the pages: each page's title, its headings with their ids, and its text by heading, from the resolved page and not from the HTML. Code blocks are in it, since a reader of old docs is often looking for a flag or an error.

### The box

In the sidebar of every page, and on the contents page.

- Typing lists pages and headings, best first: a match in a title, then in a heading, then in text, with the words around a match shown.
- A result links to the page, at the heading.
- It's reachable by keyboard, and says its results to a screen reader.
- With scripts off, the box isn't shown, and the contents page is the way around, as in phase 8.

### The script

One file, written in `packages/` and carried in the binary as the review report's is, with a test that the binary's copy is the built one. No dependency that fetches anything, and nothing stored in the browser.

It's the same script in every archive of one Ascribe version, so an archive made years ago keeps the script it was made with.

### What's said about size

The command's summary says the index's size. Past a size the phase picks by measuring, it says so plainly and still writes it: an archive without search is worse than one with a large index.

## Tasks

1. Record the decision, with the docs' index size.
2. The index, written with the archive.
3. The script and the box, with the embed and its test.
4. End-to-end tests from `file:`, with the network off: a search that finds a heading, one that finds a word in a code block, one that finds nothing, and the page with scripts off.
5. The print stylesheet leaves the box out.
6. The archive's contract, the guide, and an entry in `changes/`.

## Out of scope

- Search across live lines. That's the search proposal's.
- Search across several archives.
- Ranking beyond title, heading, and text.

## Acceptance criteria

- In an archive of `examples/quill`'s older line, opened from disk with the network off, a word from a heading finds that heading, and the result opens the page there.
- No request leaves the folder.
- With scripts off, every page is whole and the box is absent.
- An archive made twice from one commit is the same bytes, index included.

## Verify

```sh
cargo test --workspace --locked
pnpm test
cargo run -p ascribe-cli -- versions archive "self-managed 3.4" --to /tmp/quill-3.4 --config examples/quill
```

## Commits

1. "Record how an archive's search loads from disk"
2. "Write an archive's search index"
3. "Search an archive from any of its pages"
