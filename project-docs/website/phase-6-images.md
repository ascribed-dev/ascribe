# Phase 6: Images

Part of [The website](README.md). Requires phases 3 and 5. `site/`, `design/`, and a script.

## Goal

The two things rendered output can't show, the editor and review, are shown. Each marketing page has its own social card. A test says when a screenshot is older than what it claims to show.

## Context

- The [proposal](../proposals/website.md): "Design", on images.
- `design/README.md` and `design/assets.json`: how the mark's files and the existing social card are generated, and the test that holds them.
- `site/src/layouts/Base.astro`: the page's head, and the one card every page uses today.
- `packages/vscode/`: the extension's integration tests, which drive a real VS Code in CI. `examples/astro-site/test/e2e/`: the review toolbar in Chromium.
- The landing page's fourth claim, "reviewed as pages", which phase 3 left in words.

## Design

### Screenshots

As few as carry the point: the editor with a problem and its fix, and a review with a comment beside the block it's about.

- **Taken by a script where that's possible.** The review toolbar runs in a browser, so its picture can be taken by the end-to-end suite's own tools, from a fixture, and retaken with one command. Try the same for the editor; if a scripted VS Code can't give a clean picture, that one is taken by hand, and the steps are written beside it.
- **Each picture records what it shows:** the release it was taken at, in a file beside it.
- **A test fails** when a picture's release is older than the current one by more than the plan allows. Start with one minor release, and say so in the test. The failure names the command that retakes it.
- Light and dark each get a picture, and each has text that says what it shows.

### Social cards

A card for each marketing page, generated as the existing one is, from the page's title and the tokens. The docs' pages keep the one they have. The head names the page's own card, with an absolute address.

## Tasks

1. The script and the fixture for the review picture, and the editor's, or its written steps.
2. The record beside each picture, and the test.
3. The pictures on the landing page's fourth claim, and on Why Ascribe where it helps.
4. The cards: generation, `design/assets.json`, the head, and the site's test that each resolves.
5. `design/README.md` and `site/README.md`.

## Out of scope

Video. A gallery. Cards for docs pages.

## Acceptance criteria

- One command retakes each scripted picture, and its diff is empty when nothing changed.
- The test fails on a picture marked with a release two minors back, and passes on a current one.
- Each marketing page's head names its own card, and the built site's test finds it.

## Verify

```sh
pnpm test
cd site && npm run build:checkout && npm run typecheck && npm test
```

## Commits

1. "Show the editor and review on the site"
2. "Give each marketing page a social card"
