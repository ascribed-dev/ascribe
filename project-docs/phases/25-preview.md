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
3. **Webview.** A preview panel beside the editor, opened with a command and an editor-title button. It loads `@tessera/elements` (bundled with the extension) and a default stylesheet, with a strict content security policy that allows only the extension's resources and the project's local resources.
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

_To be filled in by the implementing agent._
