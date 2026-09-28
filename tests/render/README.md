# Site-render fixtures

Shared fixtures for the [site-render contract](../../project-docs/contracts/site-render.md). Each pairs site markdown, as Tessera's site emitter writes it, with the HTML it must render to. Two implementations run them, and both must pass every fixture:

- `render_site_html()` in `tessera-emit` (phase 20), used by the editor preview;
- the markdown plugin in `@tessera/astro` (phase 21), used by the published site.

Phase 02 wrote these fixtures. They change only through the contract process (`project-docs/phases/README.md`, "Contracts"); phases 20 and 21 may add fixtures for cases they find.

## Layout

```
tests/render/
  fixtures.toml        every fixture, with the constructs it covers
  <name>/input.md      site markdown
  <name>/expected.html the HTML it must render to
```

`fixtures.toml` lists each fixture's `covers`: the constructs from the table in the contract's §6. `tests/conformance/tests/render_fixtures.rs` checks that every directory is listed, every listed fixture has both files, and every construct in the contract is covered.

## Running a fixture

1. Render `input.md` as CommonMark, with raw HTML passed through, applying the contract's attribute markers. GFM and typographic replacements (Astro's defaults) may be on: the fixtures contain nothing they change outside markers, and they must not change markers.
2. Compare the result with `expected.html` as HTML, not as text (below).

For the Astro plugin, render with a unified pipeline that has the plugin in the position it takes in Astro, not with a full Astro build: the fixtures test the transform, and Astro's image optimization would replace every `src`.

## Comparing HTML

Renderers serialize HTML differently (`<img />` or `<img>`, attribute order, quoting, newlines between blocks), so compare parsed trees:

1. Parse the expected and the actual HTML with an HTML5 fragment parser in a `<body>` context: for example `html5ever` in Rust, or `parse5` (through `hast-util-from-html` with `fragment: true`) in JavaScript. Character references are decoded, and void elements need no closing slash.
2. Remove text nodes that contain only whitespace, except inside `<pre>`.
3. Compare the trees node by node: element names, attributes as unordered sets of name and value, text exactly, and comments exactly.

A difference anywhere fails the fixture. Report it with the path to the first differing node.

The expected files are written in the form comrak renders, which is also how they were checked: each `input.md` was rendered with comrak 0.55 (CommonMark, raw HTML allowed), and the marker rules applied to the result.
