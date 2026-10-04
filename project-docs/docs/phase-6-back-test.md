# Phase 6: The back-test

Part of [Docs](README.md). Needs no other phase, and can run at the same time as phases 1 and 3. A script and a write-up. No product code.

## Goal

Before building a coverage report, find out whether one would be worth reading. Over this repository's past pull requests, with a hand-written map from pages to the code they describe: how often would "this pull request changed code a page covers, and didn't change the page" have been right?

The answer decides phase 8.

## Context

- `git log` on `main`: about 60 merged pull requests, many of which changed code and docs together because their plans said to.
- `docs/`: the pages, at whatever paths they had at the time.
- [Decisions 10 and 11](README.md#decisions).

## Design

### The map

A file beside the script: for each docs page, the globs of the code it describes. The command reference covers the commands' sources; the editor guide covers the extension's manifest and preview code; a contract covers the emitter that implements it. Write it as you would if you were being specific: a page that covers a whole crate will be reported on everything.

Write the map before looking at any results, and don't tune it afterwards. A map tuned to the history proves nothing. If it turns out wrong in an obvious way, say so, fix it once, and report both runs.

### The script

For each pull request merged to `main` (at least the last 40; all of them if it's easy): the files it changed, the pages whose globs match a changed file, and whether each of those pages (or anything it includes) changed too. Output: one row per pull request and page reported. Plain `git`, a few dozen lines, kept in `project-docs/docs/back-test/` so it can be run again.

### Judging

Read each report and mark it:

- **right:** after the pull request, the page said something untrue or left out something it should have had;
- **fine:** the change didn't affect what the page says (a refactor, a test, an internal change);
- **unclear.**

Then look the other way, for misses: find drift that was fixed later ("fix the docs for…" commits, corrections made in reviews) and check whether the report would have flagged the pull request that caused it.

Also try the map at the grain snippets would give: for a handful of pages, name the functions or blocks the page is really about, and judge by hand whether reporting only changes to those would have cut the "fine" rows. This is an estimate, and say so.

### The write-up

`project-docs/docs/back-test.md`: the table; the share of reports that were right; the misses; the commonest causes of "fine"; the region-level estimate; and for each "right", how many days until the page was fixed, if it was.

### What the result decides

- **Half or more right:** build phase 8 as written.
- **Between about a third and a half:** build phase 8 with the file-level report off by default, and only the region-level report (from snippets) on.
- **Under about a third:** don't build file-level coverage. Phase 8 becomes region-level only, and says so.

Put the decision at the top of the write-up, and change phase 8's file to match in the same pull request.

## Tasks

1. The map, committed before the script is run.
2. The script and its output.
3. The judging, and the write-up with the decision.
4. Phase 8's file, adjusted.

## Out of scope

Any change to Ascribe; fixing the stale pages it finds (list them; fix them in their own pull requests).

## Acceptance criteria

- At least 40 pull requests covered, every report judged.
- The map's commit comes before the results', and any retuning is declared.
- The write-up states the decision and the numbers behind it.

## Verify

```sh
pnpm format:check
```

## Commits

1. "Map the docs to the code they describe"
2. "Back-test a coverage report on past pull requests"
