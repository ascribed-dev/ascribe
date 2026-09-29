# Phase 25: Preview

**Track:** Editor · **Start after:** 15, 17, 19, 20 · **Finish after:** 21 · **Parallel with:** 16, 22, 24 · **Unblocks:** 27

## Goal

Show a live preview of the current page in VS Code that matches the published site. It renders through the same site emitter, the same site-render transforms, and the same element library as the Astro site, and shows the project's own images.

## Read first

- [PLAN.md](../PLAN.md): VS Code extension, Preview.
- [SPEC.md](../../SPEC.md): §9.3, §9.4 (including Assets), §9.7.
- `project-docs/contracts/site-render.md` and `project-docs/contracts/assets.md` (phase 02).
- Handoff notes from phases 15, 17, 19, 20, and 21.

## Deliverables

- A custom request in `crates/tessera-lsp`, for example `tessera/preview`, that returns rendered HTML for a document and a build.
- A preview webview in `packages/vscode`.

## Tasks

1. **Rendering.** For a document (including unsaved changes) and a build, resolve the page (phase 12), emit site output (phase 20), and render it with phase 20's `render_site_html()`. Because that renderer implements the same site-render contract as the Astro plugin, and both pass the same `tests/render/` fixtures, heading ids and image attributes come out the same as on the site. Return the HTML plus the page's title and frontmatter.
2. **Assets.** Instead of copying assets into an output directory, the preview rewrites asset references to webview-safe URLs pointing at the source files (`webview.asWebviewUri`). Include the project's content root and asset locations in the webview's `localResourceRoots`, alongside the extension's own resources. Resolve references from the file they're written in, as the asset contract requires, so a fragment's image shows.
3. **Webview.** A preview panel beside the editor, opened with a command and an editor-title button. It loads `@ascribed/elements` (bundled with the extension) and a default stylesheet, with a strict content security policy that allows only the extension's resources and the project's local resources.
4. **Live updates.** Re-render on edits, debounced. Keep the scroll position, and follow the editor's cursor to the matching section.
5. **Build picker.** A control listing the content model's builds, so the author can preview, for example, the tabbed site build or the cloud-only build.
6. **Links.** Clicking a link in the preview opens the target file in the editor.
7. **Parity test.** For the phase 21 slice's pages under each build, the preview's HTML matches the Astro site's built HTML for the page content: the same elements, attributes, heading ids, and image attributes. Asset URLs differ by design; compare them by the source file they resolve to.

## Acceptance criteria

- [ ] The preview renders the Quill page with working tabs, notes, steps, badges, and its images, including the fragment's image.
- [ ] The parity test against phase 21's built site passes.
- [ ] Edits appear in the preview within about half a second.
- [ ] Switching builds in the picker changes which variant arms and content appear.

## Out of scope

- Rendering the consumer's full page layout. The preview shows the page's content with default styles.

## Handoff notes

### What was built

- **`crates/tessera-lsp/src/preview.rs`**: the custom request `ascribe/preview` (`PREVIEW_METHOD`), registered by one arm in `server.rs` (`handle_request`) and one `mod` and `pub use` in `lib.rs`. For a document and an optional build it takes the current snapshot (unsaved edits included), resolves the page (`Project::resolve_page`, `AstroRouter`), writes it with `SiteEmitter` through the new `tessera_emit::emit_page`, strips the frontmatter, and renders it with `render_site_html`. The answer is documented in `crates/tessera-lsp/README.md`: `build`, `builds` (`name`, `editor`, `description`), `projectRoot`, `contentRoot`, `documentVersion`, `problems`, and `page` (`path`, `route`, `title`, `frontmatter`, `html`, `assets`, `links`, `sections`). It has 12 tests (`tests/preview.rs`).
- **`packages/vscode`**:
  - `src/preview/controller.ts` (`PreviewController`): the panel, the debounce, the build picker, cursor following, link handling, the roots. `src/preview/html.ts`: the shell and the content security policy. `src/preview/refs.ts`: asset references compared the way both sides do. `src/preview/protocol.ts`: the types.
  - `src/webview/preview.ts` (+ `preview.css`, `elements.ts`, `elements.css`): the script the webview runs, and the element library bundled from `@ascribed/elements` (its source and stylesheet, so bundling doesn't wait on the package's build) into `dist/webview/`.
  - Commands `ascribe.openPreview` (also the title-bar button of a markdown editor) and `ascribe.selectPreviewBuild`; `onWebviewPanel:ascribe.preview` restores the panel after a reload. `ServerController` gained `request()` and `onDidStart`; `AscribeApi` gained `preview` for tests.
- **Tests.** Rust: `crates/tessera-lsp/tests/preview.rs` (12). Unit: `test/unit/preview.test.ts` (6). Webview, in Chromium under the real policy: `test/webview/preview.test.ts` (7). Integration, in real VS Code: the `preview` suite (8). Parity: `test/parity/parity.test.ts` (16), `pnpm --filter ascribe-vscode test:parity`.

### Interfaces later phases use

- **`ascribe/preview`**: the README of `tessera-lsp`. Other clients can use it as is.
- **`tessera_emit::emit_page(emitter, cx, page) -> EmittedPage`** (`output`, `text`, `assets: Vec<PlacedAsset>`): one page of `emit`. `emit` now calls it, and its output is unchanged.
- **The webview's messages** (`ToWebview`, `FromWebview` in `protocol.ts`) and the test surface, `AscribeApi.preview` (`whenDrawn`, `renders`, `receive`, `selectBuild`, `reveals`, `localResourceRoots`, `shell`).

### Changes outside the preview's own files

Both additive, and listed in the pull request:

- `crates/tessera-emit/src/emitter.rs`, `lib.rs`: `emit_page`, `EmittedPage`, `PlacedAsset`; `emit` is refactored to call `emit_page`. Its tests (`site`, `plain`, `json`, snapshots) pass unchanged.
- `crates/tessera-lsp/Cargo.toml`: dependencies on `tessera-emit` and `serde_yaml`.
- `.github/workflows/js.yml` (still `workflow_dispatch` only): the `astro` job runs the parity test.
- `packages/vscode/package.json`, `tsconfig*.json`, `esbuild.mjs`, `vitest*.config.ts`, `test/unit/manifest.test.ts`: the commands, the webview build, a second tsconfig with the DOM library for the webview code and its tests.
- `project-docs/questions.md`: Q181 to Q187.

### Decisions

Q181 (which document, and a fragment), Q182 (assets outside the content root), Q183 (the policy is stricter than the site), Q184 (what the parity test leaves out), Q185 (the preview draws the title and page availability), Q186 (the picker's default and memory), Q187 (which links open a file). All `open`, implemented as proposed, marked `SPEC-QUESTION`.

Also:

- **The server answers on the request thread** and holds the lock only to clone the snapshot; rendering is outside it. A page takes about 4 ms (median over 20 edits, `an_answer_takes_milliseconds`).
- **Ordering.** The client sends the change before the request, but the answer says which version it used (`documentVersion`) and the client asks again if that is older than the document's; it never draws a stale page.
- **References.** The server sends each asset's reference as the emitter wrote it (percent-encoded with `encode_path`); comrak escapes it again when it writes `src` (a space becomes `%20`), so the client compares references after resolving them against a dummy base (`canonicalReference`). Asset URLs get `?v=<mtime>` so a replaced image is fetched again, and a watcher on the content root asks for a render when a file changes on disk.
- **The page HTML is parsed into an inert `<template>`**, its `<img src>` rewritten there, and then moved into the document, so no request is made for a reference before it is rewritten.

### How "edits appear within about half a second" was measured

- **In real VS Code** (`preview` suite, "follows edits within half a second"): the test inserts text into the open editor with `editor.edit`, takes `Date.now()` before the call, and waits for the webview's `rendered` message for the render whose HTML has the inserted marker; the time is `drawnAt` (when the extension host got that message) minus the start. It does this 12 times, each after 150 ms of quiet, with the buffer unsaved, and asserts every one is under 500 ms. Three runs in this container: medians 113, 116, and 113 ms; maximums 167, 213, and about 170 ms. The floor is the debounce (100 ms); the server, the render, and the webview take about 10 to 15 ms.
- **The server alone** (`tests/preview.rs`): 20 edits, each then a request; median 4.1 ms, maximum 4.8 ms, asserted under 100 ms.
- A person typing continuously resets the debounce on each key, so the preview updates 100 ms after they pause.

### Acceptance criteria

See the pull request for the status and evidence of each.

### Left open

- **Q181 to Q187.**
- **A glossary term's link** in the preview isn't clickable, and a link to a heading in another page opens the file, not the heading (Q187).
- **Assets outside the content root** show as broken images, with a warning (Q182).
- **Windows and macOS** weren't run: everything ran on Linux (VS Code 1.139.1 under `xvfb-run`, Chromium). Paths reach the webview through `Uri.file` and `asWebviewUri`, and the server sends absolute paths it has normalized, which `uri.rs` already handles for drive letters, but nothing here has run on those platforms.
- **The one flake seen.** Once, in the first full run of the integration suites after a fresh bundle, the `quill` suite's "updates diagnostics after the file changes on disk" timed out (30 s); it passed alone three times and in the next two full runs. It is phase 17's test of the file watcher and doesn't touch the preview; it wasn't investigated further.
- **The panel serializer** (restoring the panel after a reload) is written but isn't covered by an integration test: the test host can't reload its own window.
- **`retainContextWhenHidden`** is on, so a hidden preview keeps its DOM (and its memory).

