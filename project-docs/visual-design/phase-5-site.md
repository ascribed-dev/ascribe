# Phase 5: The docs site

Part of [Visual design](README.md). Requires phases 1 to 3; better after phase 4. `site/` only.

The site installs `@ascribed/elements`, `@ascribed/astro`, and `@ascribed/cli` from npm's `next` tag, not from the workspace. So phase 4's element colors reach the site the night after phase 4 merges, when the canary publishes. Until then the elements on the site keep their old defaults.

## Goal

The docs site (<https://ascribed-dev.com>) looks like Ascribe: the mark in the header, a favicon, a social card on shared links, and its colors and type from the tokens.

## Context

- `site/src/styles/site.css`: the site's styles. `site/src/layouts/Base.astro` and `Docs.astro`: the `<head>` and the page frame. `site/src/components/`: `Sidebar`, `Toc`, `Search`. `site/src/pages/404.astro`.
- `site/src/site.ts`: the site's name and address.
- `site/public/`: phase 3 put the favicon, touch icon, and social card here.
- `site/test/e2e.test.ts`: the browser tests, run by the `site.yml` workflow. `site/README.md` lists what they check.
- `CONTRIBUTING.md`, the docs site section: how to run it locally.
- The site is built with Ascribe and `@ascribed/elements`, so it's also the example people look at to see what a site built with Ascribe can look like.

## Design

- **Tokens.** The site's variables come from the same semantic tokens as the report's frame. Rename them where a clearer name helps; they're private (README decision 4).
- **The head.** Favicon (SVG, with the `.ico` fallback), touch icon, `theme-color` for light and dark, and Open Graph and Twitter card tags pointing at the social card with an absolute URL.
- **The header.** The mark and wordmark, linking home, as inline SVG using `currentColor` so it follows the scheme with no second file.
- **Type.** The type scale from phase 2: sizes, weights, line height, and a measure for prose (about 70 characters). System fonts (decision 7).
- **What's restyled:** the header, sidebar, table of contents, search box and results, prose, code blocks and their titles, tables, the 404 page. Layout stays as it is unless something is plainly broken; note anything like that in the pull request rather than redesigning it.
- **Light and dark** follow the system, as now. No picker.
- **Focus and motion.** A visible focus ring from the focus token on every interactive element, and no transition that ignores `prefers-reduced-motion`.

## Tasks

1. The site's variables generated from the tokens, with no literal colors left in `site.css`. The generator runs from the repository root and writes into `site/`, as in phase 1.
2. The head, the header, and the restyle.
3. `e2e.test.ts`: the favicon and social card resolve; the header's mark has an accessible name; the page has one `theme-color` per scheme. Update `site/README.md`'s list.
4. A contrast pass with a browser's accessibility checker on a guide page, a reference page with tables, and the 404 page, in light and dark. Say what was run in the pull request.
5. Screenshots, before and after, in light and dark, at phone and desktop widths.
6. `README.md` and `packages/vscode/README.md`: the header image from phase 3, with alt text.

## Out of scope

Navigation, content, or search changes; a landing page; a theme picker; changes to `@ascribed/elements` (phase 4).

## Acceptance criteria

- No color in the site's CSS is written outside the generated block.
- A link to the site pasted into a chat app shows the social card.
- The site passes the checker with no contrast failures, in both schemes.
- Lighthouse performance on a guide page is no lower than before; say both numbers.

## Verify

```sh
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
cd site && npm ci && npm run typecheck && npm run build && npm test
```

The site has its own lockfile and isn't in the pnpm workspace; `site/README.md` has the rest.

## Commits

1. "Take the docs site's colors and type from the tokens"
2. "Add the mark, favicon, and social card to the docs site"
