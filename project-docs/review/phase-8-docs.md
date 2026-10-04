# Phase 8: Docs and a full pass

Part of [Review](README.md). Requires phases 1 to 7. Docs, and fixes for what the pass finds.

## Goal

- **A review guide** that a reviewer new to Ascribe can follow start to finish.
- **The whole flow tried by hand** on a real pull request, with what was found fixed or filed.

## Context

- `docs/review.md`, grown piece by piece through phases 3 to 7.
- `docs/README.md` (the docs' index), `docs/editor.md`, `docs/astro.md`, `docs/cli.md`, `packages/vscode/README.md` (the Marketplace page), `packages/astro/README.md`, `packages/review/README.md`.
- `CHANGELOG.md`'s unreleased section.
- [`mockup.html`](mockup.html): the reference the built UI is checked against in the pass, and the source of the guide's wording (Page preview, Site preview, unsent, detached, outdated).

## Design

### The guide

`docs/review.md`, in the order a reviewer meets things:

1. **What you get,** in a paragraph, with one screenshot of the site preview under review.
2. **Review a pull request:** check out the branch, start review, read the changed pages, comment, submit. The page preview first (it needs nothing else), then the site preview.
3. **What you need for each part:** a table. The diff: `git`. Comments in VS Code: a GitHub sign-in. Comments on the site preview: `gh`, signed in. The report in CI: nothing for the reader.
4. **How comments map to the pull request:** a comment on a block is a review comment on its file and line; a fragment's thread shows on every page that includes it; GitHub anchors comments only on changed lines and the few around them, so a comment on any other block goes in your review's summary and still shows on the block; new comments stay unsent, visible only to you, until you submit; a reply can go at once unless you have unsent comments; resolving acts at once.
5. **The report in CI.**
6. **When something's missing:** no pull request found, not signed in, anchors not arriving on the site, a thread that's detached, local commits not pushed.
7. **Privacy and security:** what runs where, that no token is stored or reaches a page, and that review mode is the only time Ascribe contacts GitHub.

Link it from `docs/README.md`, `docs/editor.md`, `docs/astro.md`, and both package READMEs, and remove what those repeat.

### The pass

In a scratch repository (not this one) with a site set up like `examples/astro-site`:

1. Open a pull request that changes a page, a fragment used by several pages, a phrase, and a variant arm; add a few review comments on GitHub, one on the fragment and one on a line later removed.
2. With the mockup open beside it, review it three ways: the CI report, the page preview, the site preview. Go through the mockup's **Try** buttons and **State** menu and reproduce each one in the real thing. List every difference in the pull request, and for each say whether the build or the mockup should change. In each, check the changed pages list, the marks, each thread's placement, and switching to the other views.
3. Comment from the page preview and from the site preview, including on a page whose file isn't in the pull request; submit; confirm on GitHub.
4. Repeat the editor steps on Windows if a machine is available, and say if not.
5. Time each step that makes the user wait (starting review, the diff rerun after an edit, loading threads) and record the numbers.

Fix what's small. File an issue for what isn't, and list the issues in the pull request.

## Tasks

1. The guide and the links.
2. The pass, with its findings in the pull request.
3. The CHANGELOG's unreleased section read as one release's notes.

## Out of scope

Releasing (a separate step, following `RELEASING.md`).

## Acceptance criteria

- A reviewer can follow `docs/review.md` from a fresh checkout to a submitted review without reading anything else.
- The pass's findings are fixed or filed, and its timings are recorded.

## Verify

```sh
pnpm lint && pnpm format:check
cargo test --workspace --locked
pnpm test
```

## Commits

1. "Write the review guide"
2. One per fix from the pass
