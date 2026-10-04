# Phase 3: The static report

Part of [Review](README.md). Requires phases 1 and 2. Rust, a little browser JavaScript and CSS, and docs.

## Goal

`ascribe diff --format html` writes one self-contained HTML file showing every changed page rendered, with what changed marked. CI uploads it, and a reviewer opens it from the pull request with no checkout, no build, and no account beyond GitHub's.

It's read-only: no comments. It's also where the change marks' look and behavior are settled, for the overlay to reuse.

## Context

- Phase 2's comparison and JSON; phase 1's anchors.
- `crates/tessera-emit/src/render/mod.rs` (`render_site_html`) and `crates/tessera-lsp/src/preview.rs`: rendering one page to HTML without a site generator, and how the preview lists a page's assets.
- `packages/elements` (`css/`, `dist/`): the element library's stylesheet and script, which the rendered HTML needs. `packages/vscode/src/webview/elements.ts` and `elements.css` show how the preview loads them.
- [`mockup.html`](mockup.html): the reference for how changes look, which the report shares with both previews. Build the marks as the page preview shows them in its **Changes** mode: a label on every mark (Added, Changed, Removed, Moved) as well as a bar that differs in shape, word highlights inside changed blocks, a removed block collapsed to one line with **Show**, and a move as a linked stub at the origin and the paragraph at the destination. The three show modes (**Changes / As it will be / As it was**) and the change position ("3 of 10 on this page", then the next changed page) are the mockup's too. The report has no comments, so ignore the threads.
- GitHub Actions: `actions/upload-artifact` can upload a single file unzipped so it opens in the browser (added in 2026; check the current action's documentation for the option and its limits).

## Design

### The file

- One HTML file with everything inline: styles, the element library, a small script, and images as data URIs up to a size limit (above it, a placeholder with the image's path). No network requests.
- A sidebar lists changed pages per build, with counts, and marks pages that changed only through something they use ("via `_fragments/prereqs.md`").
- A build picker when more than one build has changes. Pages identical across builds are stored once.
- Each page is rendered as the working tree has it, with marks:
  - **added** and **changed** blocks: a bar in the margin, in distinct colors, with words added inside a changed block highlighted;
  - **removed** blocks: rendered from the base, struck through and collapsed to one line with a control to expand, placed after the block they followed;
  - **moved** blocks: a bar and a "moved from" note linking to the old place.
- A toggle, **Show: changes / page as it will be / page as it was**, and next and previous change buttons.
- Each block shows its source on hover (`guides/install.md:12`), from its anchor.
- Colors meet contrast requirements in light and dark, and no information is carried by color alone (the bars differ in shape or carry a label).

### The marks as a shared piece

Put the marks' CSS and the script that applies a phase 2 `changes` list to anchored HTML in one place the report inlines and `@ascribed/review` (phase 6) will import: `packages/review/src/marks/`. Create the package in this phase with just that, private until phase 5 decides its published shape. The Rust side embeds the built files with `include_str!`; add a check, like the existing ones for generated files, that the embedded copies are current.

### Limits

Above a number of changed pages (choose one; a few hundred), the report includes the first N in full and lists the rest by name, saying so at the top. A report must stay openable.

### The CI recipe

`docs/review.md` (new, started here and finished in phase 8) gets a "Report in CI" section with a GitHub Actions job: fetch enough history for the merge base, run `ascribe diff --format html > review.html`, upload it, and link it from the job summary. Add the job to this repository's own workflow for `examples/quill`, as the working example.

## Tasks

1. `packages/review` with the marks' CSS and script, and unit tests (vitest, jsdom) applying each change kind to anchored HTML.
2. `--format html` in `ascribe diff`: rendering both sides, inlining, the sidebar, and the limits. Snapshot tests on a small fixture repository, and a test that the file makes no external requests (no `http` URL in a `src`, `href` to a stylesheet, or `url()`).
3. The CI recipe, in the docs and in this repository's workflow.
4. `docs/cli.md`, `docs/review.md`, `CHANGELOG.md`.

## Out of scope

Comments and anything from GitHub; the site's own layout (the report is Ascribe's bare render, and says so in its header).

## Acceptance criteria

- The report for a pull request touching a fragment shows every page that includes it, with the change marked in each.
- It opens from a GitHub Actions artifact in a browser with no other files.
- Removed content is visible, and the three "show" modes work.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/review test
pnpm lint && pnpm format:check && pnpm typecheck
```

## Commits

1. "Mark changed blocks in rendered pages"
2. "Write the diff as a static HTML report"
3. "Upload the review report in CI"
