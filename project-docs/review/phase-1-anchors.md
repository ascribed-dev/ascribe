# Phase 1: Source anchors

Part of [Review](README.md). Rust (`tessera-emit`, `tessera-lsp`, `tessera-cli`) and TypeScript (`packages/astro`, `packages/vscode`).

## Goal

In review mode, every block of a rendered page carries the source file and lines it came from, in both renderers. The page preview uses them at once, to scroll with the editor by block instead of by heading.

## Context

- `docs/contracts/site-render.md`: the attribute marker, where it applies, and why it's raw HTML (Astro's Markdown processing changes anything else before a plugin sees it). Its two implementations must agree.
- `tests/render/`: the shared fixtures (`fixtures.toml`, `README.md`), run by `crates/tessera-emit/tests/render_fixtures.rs` and `packages/astro/test/render-fixtures.test.ts`.
- `crates/tessera-emit/src/site/` (`blocks.rs`, `element.rs`, `inline.rs`): the site emitter. `crates/tessera-emit/src/render/mod.rs`: `render_site_html`.
- `packages/astro/src/attributes.ts`, `rehype.ts`, `satteri.ts`: the marker rules in each of Astro's Markdown processors.
- `tessera_resolve::ResolvedBlock`: each block's source (file, span, and the include chain), which the JSON emitter already writes (`crates/tessera-emit/src/json.rs`).
- `crates/tessera-lsp/src/preview.rs` (`PreviewPage.sections`), `packages/vscode/src/preview/controller.ts`, and `packages/vscode/src/webview/preview.ts`: today's heading-based scroll sync.
- `packages/elements/CONTRACT.md`: the elements the site output uses.
- [`mockup.html`](mockup.html): this phase has no review UI, but the mockup shows what the anchors are for. Hover or focus a block to see its source (`guides/rollouts.md:14-14`, and `… via guides/rollouts.md:8` for a block from a fragment); click a source line to find its block; double-click a block to open its source. Every block kind it lets you point at needs an anchor.

## Design

### What an anchor says

For a block: the content path of the file its text is written in, its first and last line, and the include chain when it came through one (each include's file and line, outermost first). The same facts as the JSON output's `source`, without byte spans.

In the HTML, on the block's own element:

```html
<p data-ascribe-source="guides/install.md:12-14">…</p>
<pre data-ascribe-source="_fragments/prereqs.md:3-9" data-ascribe-via="guides/install.md:20">…</pre>
```

Paths are percent-encoded so a path with a space or a colon is unambiguous. Write the exact grammar into the contract.

### Which blocks get one

Every block a reviewer could point at: headings, paragraphs, code blocks, lists and each list item, tables, block quotes, images that stand alone, thematic breaks, raw HTML blocks, and each Ascribe element (`ascribe-note`, `ascribe-steps`, `ascribe-tabs` and each `ascribe-tab`, `details`, `ascribe-availability`, project widgets, `ascribe-group`). Blocks inside containers get their own anchors too.

### How it gets through the Markdown pipeline

The site output is Markdown that the consumer renders, so the anchor has to survive that. The contract's reasoning applies: only raw HTML reaches a plugin unchanged. Design the mechanism in this phase, write it into the contract as a new section, and prove it with fixtures. It must meet all of these:

1. **Off by default.** Without review mode the site output is byte-for-byte unchanged. Add a test that compares.
2. **Same page.** With anchors on, each renderer's HTML equals its HTML with anchors off once the `data-ascribe-source` and `data-ascribe-via` attributes are removed: no extra elements, no changed nesting, no lists turning loose, no paragraphs appearing or disappearing. Add this comparison as a test over every fixture and over `examples/quill`.
3. **Both renderers agree,** on the shared fixtures.
4. **Degrades quietly.** A consumer without the plugin renders something invisible.

A starting point, to check and replace if it fails a requirement: extend the attribute marker so that a marker in a new position (for example, alone in a raw HTML block directly before a block) applies its attributes to the block that follows, and is removed. Watch for the cases where raw HTML changes how CommonMark reads its surroundings: a line with an open and close tag isn't an HTML block of its own by CommonMark's rules and becomes a paragraph, a block between list items can make a tight list loose, and anything inside a table cell or a heading is inline.

For elements Ascribe writes itself (`<ascribe-note …>`, `<details>`), put the attribute directly on the tag.

### Turning it on

- `ascribe build --emit site --anchors` writes anchors. Document it in `docs/cli.md`. The output's manifest records that anchors are on, so a consumer can tell.
- `@ascribed/astro` gets an `anchors` option: `false` by default, and `"dev"` to pass `--anchors` in `astro dev` only. `astro build` never has anchors unless set to `true`. Phase 7 turns this on for review.
- `ascribe/preview` always renders with anchors: the preview is never published.

### The preview's scroll sync

Replace heading-based sync with anchors: the editor's top visible line maps to the block whose anchor contains it (or the nearest before it), and clicking or scrolling in the preview maps back. A block from a fragment maps to the include line in the page being previewed (its `via`), since that's the file in the editor. Keep `sections` in the result for clients that use it.

## Tasks

1. The contract section, the emitter, and `render_site_html`, with new fixtures in `tests/render/` for every block kind, for nesting (a paragraph in a list item in steps in a variant arm), and for a block from a fragment.
2. The same rules in `packages/astro/src/attributes.ts` for both processors, passing the same fixtures.
3. The "off by default" and "same page" tests above.
4. `--anchors` on `ascribe build`, the manifest field, and the `anchors` option in `@ascribed/astro`, with tests.
5. Block-level scroll sync in the page preview, with unit tests for the line-to-block mapping and an integration test on a copy of `examples/quill`.
6. Docs: the contract, `docs/cli.md`, `docs/astro.md`, `packages/astro/README.md`, `docs/editor.md` (the preview follows the cursor by block), `CHANGELOG.md`.

## Out of scope

Anything about changes or comments. Inline anchors (words and phrases): blocks only.

## Acceptance criteria

- With anchors on, every block kind listed carries a correct anchor in both renderers, including through includes.
- With anchors off, output is unchanged; with them on, the page is the same page.
- The preview follows the editor by block, both ways.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
pnpm --filter ascribe-vscode test
cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm --filter @ascribed/example-astro-site test
```

## Commits

1. "Define source anchors in the site-render contract"
2. "Write and render source anchors"
3. "Apply source anchors in the Astro plugin"
4. "Scroll the preview with the editor by block"
