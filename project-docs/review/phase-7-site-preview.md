# Phase 7: The site preview

Part of [Review](README.md). Requires phase 6. TypeScript: `packages/astro` and `packages/vscode`.

## Goal

The overlay on the real site, in `astro dev`: the page in the site's own layout, with changes marked and threads beside their blocks. And switching between the three views: the site, the page preview, and the source.

## Context

- `packages/astro/src/index.ts` (the integration's hooks: `astro:config:setup`, `astro:server:setup`), `dev.ts` (rebuilding on edits), `run.ts` and `binary.ts` (running `ascribe`), `project.ts`.
- Phase 1's `anchors` option.
- Astro's dev toolbar: an integration adds an app (`addDevToolbarApp`) whose client code runs in the page in dev only, and which talks to the dev server over the toolbar's own client-server channel. Check the current API in Astro's documentation for the supported version.
- `packages/review`: `ReviewSession` with `ghTransport` (phase 5) and the overlay (phase 6).
- `examples/astro-site`: the example site and its end-to-end tests (`test/e2e`).
- `packages/vscode/src/preview/`: the preview panel.
- [`mockup.html`](mockup.html): the reference for the site preview. Switch the preview to **Site** to see it: the page in the site's own layout, the toolbar app as a button that opens a one-row panel of controls, the unsent count on the button, and threads as markers when the nav leaves no room for the column. Its **State** menu shows "Site preview: anchors didn't arrive" and "Site preview: not an Ascribe page". The **Page | Site** switch and the **Open Page Preview** and **Open Site Preview** actions on the source tab are the switching this phase builds.

## Design

### The dev toolbar app

- `@ascribed/astro` gets a `review` option (`true` by default in dev; `false` removes the app). With it, `astro dev` builds with anchors, and the toolbar has an **Ascribe review** app.
- Turning the app on starts review for the session: the dev server runs `ascribe diff --format json` for the integration's build, against the pull request's base if `gh` finds a pull request and the default base otherwise, and makes a `ReviewSession` with `ghTransport`. Nothing runs before the user turns it on (README decision 8).
- The app's panel starts collapsed to its toolbar button, which shows the unsent count, and opens from it. Open, it holds the review controls in one row and can be closed; the page gets bottom scroll padding so the panel never hides what's focused. It carries its own **Refresh**, since a browser has no editor title bar.
- The app hosts the overlay on the page. `OverlayHost` is implemented over the toolbar's client-server channel; all `git`, `ascribe`, and `gh` access is in the dev server. No token reaches the page.
- **No HTTP endpoints.** Don't add routes that post to GitHub: any web page open in the same browser could call a route on `localhost`. Use the toolbar's channel only, and check how it authenticates its client before relying on it; if it doesn't, stop and report.
- The diff reruns after each rebuild (`dev.ts`'s loop), debounced, and the overlay updates. Record how long the rerun takes on `examples/astro-site`.
- **Anchors that don't arrive.** A layout or component can drop attributes. When a page has threads or changes but few or no anchors, the app says so, lists the threads and changes in a panel anyway, and names the likely cause.
- **Pages that aren't Ascribe pages** (the site's own routes): the app says there's nothing to review here and offers the changed pages list, each linking to its route.
- Without `gh`, or with no pull request, the app shows changes only, and says how to get comments.

### Switching views

- **Site to source:** each block's and thread's **Open source** opens the file at the line in the editor. Use the dev server's open-in-editor support if Astro exposes it, and a `vscode://file/…` link otherwise; make the editor configurable the way Astro's own tooling does.
- **Editor to site:** the dev server writes its URL and build to a small file in the project's output directory (`.ascribe/dev.json`) when it starts, and removes it when it stops. **Ascribe: Open Site Preview** in VS Code reads it, maps the active page to its route (`route` in `ascribe/preview`'s result), and opens the browser there. With no dev server, it says to start one.
- **In the preview panel:** a **Page | Site** switch. **Site** shows the dev server's page for the same file in the panel (a frame on the dev server's URL), following the active file as **Page** does. Check what VS Code allows a webview to frame, and that it works in remote workspaces (ports forwarded); if it can't work somewhere, the switch opens the browser there instead.
- A stale `dev.json` (the server crashed) must not break anything: check the URL answers before using it.

## Tasks

1. The `review` option, the toolbar app, and the dev-server side, with unit tests for the server side (a fake session and a fake `ascribe`), and an end-to-end test in `examples/astro-site` (its existing browser tests): with a working-tree change in a temporary copy, the changed block is marked on the real page.
2. The "anchors didn't arrive" and "not an Ascribe page" states, with tests.
3. Retitle the existing preview commands to the README's names (**Open Page Preview**, **Open Page Preview to the Side**), keeping their command IDs, and update `docs/editor.md` and the extension's README to match.
4. `dev.json`, **Open Site Preview**, and the **Page | Site** switch in the extension, with unit and integration tests (a fake dev server).
5. A manual pass on a real pull request in a scratch repository with `examples/astro-site`'s setup: comment on the site preview, see it in the page preview and on GitHub after submit. Say in the pull request what you checked.
6. `docs/astro.md`, `packages/astro/README.md`, `docs/editor.md`, `docs/review.md`, `CHANGELOG.md`.

## Out of scope

Deployed previews (`astro build` output with the overlay) and any hosted piece; other site generators.

## Acceptance criteria

- On `examples/astro-site`, the toolbar app marks changes and shows threads on the real page, and a comment made there reaches the pull request on submit.
- Each of the three views opens the other two at the same place.
- `astro build` output has no anchors, no overlay, and no review code, by default. Add a test.

## Verify

```sh
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/review test
pnpm --filter ascribe-vscode test
pnpm --filter @ascribed/example-astro-site test
pnpm --filter @ascribed/example-astro-site test:e2e
cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm typecheck && pnpm lint && pnpm format:check
```

## Commits

1. "Review in the site preview, in Astro's dev toolbar"
2. "Open a page's site preview from the editor"
3. "Show the site preview in the preview panel"
