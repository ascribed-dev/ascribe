# Phase 2: Review

Part of [The editor's views](README.md). Needs phase 1.

## Goal

While review is on, a Review view lists the changed pages with their counts, their problems, and a badge for unsent comments, and opens the branch's source changes in VS Code's diff.

On a checkout of a pull request's branch, run **Ascribe: Start Review**. A **Review** view appears first in the sidebar: the changed pages under Added, Changed, and Removed, each with "3 changes, 1 open thread", and a page changed through a fragment saying `via _fragments/requirements.md`. Choosing a page opens it and its preview at its first change. Add a comment, and the view's badge says 1. **Problems in changed pages** lists the errors and warnings on those pages and nowhere else. **Open Source Changes** opens one diff of every changed source file against the base. **Ascribe: Stop Review**, and the view is gone.

## Context

- README decisions 6, 10, 12, and 13.
- `packages/vscode/src/preview/review.ts`: the base, `ascribe.reviewOn`, and **Changed Pages** (`ascribe.changedPages`), whose list of pages, counts, and "through" this view shows. `src/preview/threads.ts` for threads and unsent comments, and how review runs `git`.
- `crates/ascribe-lsp/src/review.rs`: `ascribe/review/setBase` and `ascribe/review/changes`. `crates/ascribe-diff/src/gitfs.rs` for reading the base.
- VS Code's Git extension API (`vscode.git`, version 1): `toGitUri(uri, ref)`. The command `vscode.changes(title, [[label, original, modified]])`, and its open issues past about 30 files and with read-only sides (in the [research](../reports/VS%20Code%20extension%20UI%20patterns.md)).
- `TreeView.badge`, which the extension doesn't use yet.
- `packages/vscode/test/integration/suite/review.it.ts` and `fakeGitHub.ts`.
- The docs: `docs/content/review/index.md` ("3. Read the changed pages" and "5. Submit"), and the review section of `docs/content/editor/index.md`.

## Design

### The view

`ascribe.review`, **Review**, first in the container, `when: ascribe.reviewOn`. Its items:

- **Three groups,** Added, Changed, and Removed, each page by title with its counts as Changed Pages has them, and "via …" when its own file didn't change. Choosing a page does what choosing it in Changed Pages does. A removed page opens the preview's **As it was**.
- **Problems in changed pages,** the last group: VS Code's diagnostics for the changed pages' files, errors and warnings, and advice under a toggle in the view's title bar, each opening its place. It's drawn from the same list of pages as the groups above, so it changes when they do.
- **The badge** is the unsent comments' count, with a tooltip saying so, and none at zero.
- **The title bar:** Submit Review…, which does what the preview's bar does; Open Source Changes; and Stop Review.
- **A page's inline button:** its own files' diff, the page's file and what it changed through.

Changed Pages stays as it is.

### Open Source Changes

The changed source files are what the server's changes name: each page's file and its "via" files, once each. For each, the base side is `toGitUri(file, base)` from the Git extension, whose language follows the file; an added file has no base side, and a removed one no working side.

Where the Git extension is off or has no repository for the file, the server answers the base side instead: a document provider for `ascribe-base:` URIs, answered by a request that reads the file at the base through `GitFs`, as `ascribe diff` does. Both are tested; the fallback by turning the Git extension off in an integration test.

Over 30 files, the diff opens with a note in its title that VS Code's diff may repeat headers past that many, and the page items' buttons stay the way to see one page's files.

### This page's Review section

While review is on, This page gains a **Review** section: whether the branch changes the page, its counts, and its open threads, each opening the thread in the preview. It's the Review view's data for one page, and adds no request.

## Tasks

1. The view, its groups, its badge, and its title bar, with its test API.
2. Problems in changed pages, and its advice toggle.
3. Open Source Changes and the page items' diff, through the Git extension, and the server's fallback with its request.
4. This page's Review section.
5. Tests: unit tests for the grouping and the file list; `review.it.ts` for the view, the badge after a comment, the problems section, and the diff with the Git extension and without it.
6. The docs: review's how-to (step 3 reads from the view, step 5 counts from its badge), the editor page's review section (the view, Open Source Changes, This page's Review section), `changes/review/review-view.md`, and the surface map.

## Out of scope

- A rendered diff in VS Code's diff editor. The preview is the rendered diff.
- Problems "new since the base". Problems in changed pages are every problem on them, new or not; telling new from old needs a finding's identity, which the [cache proposal](../proposals/cache.md) is about.
- Submitting from anywhere but Submit Review…'s dialog.

## Acceptance criteria

- The goal's steps, in `review.it.ts`, against the fake GitHub.
- The view's pages, counts, and "via" are the ones `ascribe diff --format json` lists for the same base.
- Open Source Changes shows the same files with the Git extension on and off.
- The view isn't there while review is off, and the sidebar's other views don't move.

## Verify

```sh
cargo test -p ascribe-lsp --locked
pnpm --filter ascribe-vscode test
```

The integration tests run in CI.

## Commits

1. "List a review's changed pages in a view of their own"
2. "Open the source changes of a review in one diff"
3. "Show a page's review in This page"
