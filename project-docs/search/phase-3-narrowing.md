# Phase 3: Narrowing, and nothing found

Part of [Search](README.md). Needs phase 2.

## Goal

A search keeps to folders, pages, a type, case, or whole words, summarizes by page, and when nothing matches, says whether the words are written but not published.

In Ascribe's own docs, from the repository's root:

- `ascribe search sidebar docs/content/navigation/` finds only the pages in that folder, and a fragment's line only for the pages there that show it.
- `ascribe search sidebar --config docs --type how-to` finds only how-tos.
- `ascribe search limit --config docs --word` doesn't find "limits"; `ascribe search Vale --config docs --case` doesn't find "vale".
- `ascribe search sidebar --config docs --summary` lists the pages and how many places each has, most first.

In `examples/quill`, `ascribe search '"point the agent at your server"' --build cloud` prints no places, says the words are published by `site` and `self-managed-3.3`, which the search didn't name, and exits with `1`.

## Context

- README decisions 4, 6, 12, and 18. The proposal's "A query" (scope) and "For agents" (`--summary`).
- `crates/ascribe-check/src/scope.rs`, `Scope::of_paths`: how `check` keeps what counts for paths, a fragment's included text for the pages under them among it.
- `crates/ascribe-cli/src/commands/check.rs`: `--summary`, and the fields a cut list uses.
- The source index's text of each file, which the source scan reads; [decision #536c](../decisions/536c-nothing-reads-a-memo.md), which keeps every reader of the source out of memos.

## Design

### Scope

Paths after the query, from the current folder (decision 18), keep the search to pages in those folders, or those pages. A fragment's text counts for the pages there that show it, and its place lists only them. A path that names no source is exit code `2`; when every argument after the query is a plain word, the message says to quote the query. `--type <NAME>` keeps to pages of that type, and an unknown type is `2`, naming the closest.

### Case and words

`--case` matches case as written. `--word` matches a word or a phrase only where it's not inside a longer word: the characters on either side aren't letters, digits, or `_`.

### Summary

`--summary` replaces the list with the pages and how many places each has, most first, then by path, and the total, with `check --summary`'s fields for a cut list.

### Nothing found

When nothing matches, the source scan (decision 4) looks for the query, by the same rules, in the source index's text of every page, fragment, and `ascribe.toml`, leaving out memos. Then the last line says one of:

- "The words aren't in any page." Nothing in the source either.
- "No build searched publishes them. They're written at `docs/x.md:12` and 2 more places." The source has them where no searched build publishes, or where a phrase's value hides them, so the sentence says "written".
- "Builds not searched publish them: `site`, `self-managed-3.3`." With `--build`, when another build has a place.

The exit code stays `1`. The scan runs only when nothing matched, and only over what's already loaded.

## Tasks

1. Paths and `--type`, with tests: a folder, a page, a fragment counted for one page of two, an unknown type.
2. `--case` and `--word`, with tests at the edges: start and end of a block, punctuation, a phrase.
3. `--summary`, with a test.
4. The source scan, and the three sentences, with tests for each and for a word only in a memo, which counts as in no page.
5. The command's output for each, in `crates/ascribe-cli/tests/output/`, observed again.
6. The docs: `ascribe search`'s section gains scope, `--case`, `--word`, `--summary`, what nothing found says, and, by decision 2, how to run a pattern over resolved text with `ascribe build --emit plain` and `grep`, and what that loses; `changes/command/search-narrowing.md`; and the surface map.

## Out of scope

The JSON, the tool, and the editor.

## Acceptance criteria

- The goal's searches, in `crates/ascribe-cli/tests/`.
- `node scripts/compare/outputs.ts --base main` shows every project byte for byte the same.

## Verify

```sh
cargo test -p ascribe-query search
cargo test -p ascribe-cli search
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Keep a search to folders, pages, or a type"
2. "Match case, or whole words, when asked"
3. "Summarize a search by page"
4. "Say whether words nothing matched are written but not published"
