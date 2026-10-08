# Phase 3: Assets

Part of [Visual design](README.md). Requires phase 2's choice. A script and the files it writes.

## Goal

Every image of the mark that anything needs, generated from the sources in `design/` by one script, and put where each consumer reads it. This phase makes the files; phases 5 and 6 wire them in.

## Context

- `design/mark.svg`, `design/mark-color.svg`, `design/wordmark.svg` from phase 2.
- README decision 9: generated, committed, with a manifest and a test.
- `site/package.json` has `sharp`, which rasterizes SVG. If the script lives outside `site/`, move the dependency to where the script is rather than adding a second rasterizer.
- `site/public/`: holds only `_redirects` today. `packages/vscode/media/`: holds only `preview.svg`.
- `packages/vscode/package.json`: no `icon` and no `galleryBanner` yet. `.vscodeignore` or the `files` list decides what ships in the extension package.
- `project-docs/outside.md`: the accounts that hold an avatar or an image (the GitHub organization, the Marketplace publisher, npm), and who can change each.

## Design

### What's made

| Asset | For | Form |
|---|---|---|
| Favicon | The docs site | `favicon.svg` (one color, with a `prefers-color-scheme` rule inside so it shows on a dark tab), and `favicon.ico` at 32 pixels for browsers that want one |
| Touch icon | The docs site | 180 pixel PNG, full color on a solid background |
| Social card | Links to the docs site and the repository | 1200 × 630 PNG: mark, wordmark, and one line |
| Marketplace icon | The extension's listing | 256 pixel PNG, full color. The Marketplace doesn't take SVG |
| Activity bar icon | The extension's view container | 24 pixel one-color SVG, drawn on a 16 pixel safe area, which VS Code tints |
| README header | `README.md` and `packages/vscode/README.md` | Wordmark as SVG, in a light and a dark version for GitHub's `<picture>` |
| Avatar | The GitHub organization, the Marketplace publisher | 512 pixel PNG, full color, with padding for a circular crop |

The activity bar icon may need its own drawing: a mark that reads at 128 pixels often needs thicker strokes and fewer details at 16. If it does, it's a third source, `design/mark-small.svg`, and the favicon uses it too.

### The script

`scripts/design/assets.ts` writes each file to its consumer's folder (`site/public/`, `packages/vscode/media/`, `design/out/` for the avatar and social card) and writes `design/assets.json`: each output, its source, and the source's hash. Its test fails when a source's hash differs from the manifest, and when an output is missing or the wrong size. It doesn't compare pixels: rasterizers differ by platform, and the test runs on Linux and Windows.

SVG outputs are the source run through a fixed clean-up (no comments, no editor metadata, fixed precision), so they're the same bytes on every platform.

## Tasks

1. The script, the manifest, and the test.
2. The assets, checked by eye at their real size: the favicon in a browser tab on light and dark, the activity bar icon in a screenshot of VS Code's activity bar in a light, a dark, and a high-contrast theme. Put the screenshots in the pull request.
3. `design/README.md`: the table above, and how to regenerate.
4. `project-docs/outside.md`: note which accounts show the avatar or the social card, for the maintainer to upload. Uploading is theirs to do; the pull request lists the files and where each goes.
5. `ARCHITECTURE.md`: the assets in the `ASCRIBE_BLESS` table or beside it, as phase 1 did for tokens.

## Out of scope

Referencing the assets from the site or the extension (phases 5 and 6); walkthrough images (the editor UI plan's phase 9); uploading anything to an outside account.

## Acceptance criteria

- Changing a source SVG and not running the script fails a test that says which command to run.
- Every asset in the table exists at its size, and the one-color ones contain no color but `currentColor`.
- The extension package and the site's build output grow by no more than 150 KB together. Say the number in the pull request.

## Verify

```sh
node scripts/design/assets.ts && git status --short
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

## Commits

1. "Generate the mark's assets from its sources"
2. "Add the favicon, icons, social card, and README header"
