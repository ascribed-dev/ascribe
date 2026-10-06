# Site-render fixtures

Shared fixtures for the [site-render contract](https://ascribed-dev.com/contracts/site-render/). Each pairs site markdown, as Ascribe's site emitter writes it, with the HTML it must render to. Two implementations run them, and both must pass every fixture:

- `render_site_html()` in `ascribe-emit`, used by the editor preview;
- the markdown plugin in `@ascribed/astro`, used by the published site.

Both implementations must keep passing every fixture, so a change to how a construct renders changes the fixture and both implementations together. Add a fixture for any construct whose rendering needs pinning.

## Layout

```
tests/render/
  fixtures.toml          every fixture, with the constructs it covers
  <name>/input.md        site markdown
  <name>/expected.html   the HTML it must render to
  <name>/source/         for a source-anchor fixture: the page it's written from
  <name>/unanchored.md   for a source-anchor fixture: the page without anchors
  corpus/quill/<build>/  examples/quill's pages, with anchors (<page>.md) and without (<page>.unanchored.md)
```

The source-anchor fixtures' `input.md` and `unanchored.md`, and the corpus, are the site emitter's output, which `crates/ascribe-emit/tests/site_anchors.rs` checks. After a change to the emitter, run it with `ASCRIBE_BLESS=1` to rewrite them, and check the diff.

`fixtures.toml` lists each fixture's `covers`: the constructs it checks. `tests/conformance/tests/render_fixtures.rs` checks that every directory is listed, and every listed fixture has a description, at least one construct, and both files.

## Running a fixture

1. Render `input.md` as CommonMark, with raw HTML passed through, applying the contract's attribute markers. GFM and typographic replacements (Astro's defaults) may be on: the fixtures contain nothing they change outside markers, and they must not change markers.
2. Compare the result with `expected.html` as HTML, not as text (below).

For the Astro plugin, render with each markdown processor Astro can run (in Astro 7.3, Sätteri, its default, and `unified()`), with the plugin at the stage it takes in Astro, which is a user hast plugin that runs before Astro's image and heading-id passes, not with a full Astro build: the fixtures test the transform, and Astro's image optimization would replace every `src`.

## Comparing HTML

Renderers serialize HTML differently (`<img />` or `<img>`, attribute order, quoting, newlines between blocks), so compare parsed trees:

1. Parse the expected and the actual HTML with an HTML5 fragment parser in a `<body>` context: for example `html5ever` in Rust, or `parse5` (through `hast-util-from-html` with `fragment: true`) in JavaScript. Character references are decoded, and void elements need no closing slash.
2. Remove text nodes that contain only whitespace, except inside `<pre>`.
3. Compare the trees node by node: element names, attributes as unordered sets of name and value, text exactly, and comments exactly.

A difference anywhere fails the fixture. Report it with the path to the first differing node.

## The same page

With anchors on, a page must render to the same page (contract §7.5). For each source-anchor fixture and each corpus page, render the anchored and the unanchored markdown, remove `data-ascribe-source` and `data-ascribe-via` from the anchored HTML's tree, and compare the two trees as above.

The expected files are written in the form comrak renders, which is also how they were checked: each `input.md` was rendered with comrak 0.55 (CommonMark, raw HTML allowed), and the marker rules applied to the result.
