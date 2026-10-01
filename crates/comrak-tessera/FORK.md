# comrak-tessera: Ascribe's fork of comrak

This crate is [comrak](https://github.com/kivikakk/comrak), vendored and changed to parse Ascribe's block-level additions to CommonMark. comrak is licensed under the BSD 2-Clause license; its license is in [`COPYING`](COPYING) and applies to this crate, including Ascribe's changes.

## Upstream version

| | |
|---|---|
| Upstream | <https://github.com/kivikakk/comrak> |
| Release | **v0.55.0**, tag commit `6fbe87fafde3953a9f3bc582804318593d703805` (2026-09-06) |
| Vendored in | the commit "Vendor comrak v0.55.0 as crates/comrak-tessera", which is the unmodified upstream source |
| CommonMark | 0.31.2 (upstream's target) |

## Why fork

Ascribe changes CommonMark's block structure: a directive line interrupts a paragraph, is never a lazy continuation line, follows CommonMark's container rules, and, when it has a text primary, continues onto following lines as a paragraph does (SPEC §3.2, §3.4, §3.9). These rules have to live inside the block parser. Walking the tree an unmodified parser produces can't recover the right structure: unmodified comrak folds an unindented `@note` into the list item above it, where Ascribe ends the list. comrak has no extension point for new block types, so the new block is added to a fork.

The alternative was markdown-rs, a state-machine CommonMark parser built for MDX. Adding a block construct there means writing a new construct in its tokenizer (states, events, and the resolver), integrating it with its document and flow content types, and teaching its event-to-tree conversion about it, and its extension points are designed around MDX's needs. It would also need its own fork, with its own merge burden, and the CommonMark baseline would have to be redone. In the comrak fork, every block-level requirement fits the existing block-parser model, which is a port of the CommonMark reference implementation: the CommonMark 0.31.2 suite passes 652 of 652 with the Ascribe option off and on (see [`tests/commonmark/README.md`](../../tests/commonmark/README.md)), upstream's own unit tests pass, and parsing a 7 MB document takes about 120 ms with the option on or off.

## What was vendored

From the v0.55.0 tag, unchanged except for the locations listed below:

- `src/` (all of it, including upstream's unit tests in `src/tests/` and their fixtures), except `src/main.rs`
- `build.rs`, `COPYING`, `rustfmt.toml`

Not vendored: `src/main.rs` (the CLI), `benches/`, `examples/`, `fuzz/`, `vendor/` (git submodules), `script/`, `www/`, `nix/`, and upstream's CI, changelog, and README.

## The manifest

`Cargo.toml` is Ascribe's own, derived from upstream's:

- The package is `comrak-tessera`, version `0.55.0` (the upstream release), `publish = false`, license `BSD-2-Clause`. The library is `comrak_tessera`.
- Upstream's `cli` and `syntect` features, their dependencies, and the `comrak` binary are gone. `default = []`. The `bon`, `shortcodes`, `phoenix_heex`, `attributes`, and `arbitrary` features remain and still build and pass upstream's tests (`cargo test -p comrak-tessera --features bon,shortcodes,phoenix_heex,attributes,arbitrary`). The `syntect` `cfg`s left in the source are declared as expected `cfg` values so they don't warn.
- **Doctests are off** (`[lib] doctest = false`). Upstream's doctests import the crate as `comrak`, which no longer resolves. Upstream's unit tests still run.
- The crate doesn't use the workspace lints (upstream uses `unsafe` and `unwrap`). It allows `deprecated` and a few clippy lints that only upstream's code and tests trip under the workspace's `clippy -D warnings`, so that upstream code stays untouched.
- `rustfmt.toml` is upstream's, so `cargo fmt` formats vendored code as upstream does.

## Changed locations

Every change to an upstream file is marked in the code with a `// TESSERA:` comment. `grep -rn '// TESSERA:' src` lists them. Two files are wholly Ascribe's and have no upstream counterpart: [`src/tessera.rs`](src/tessera.rs) (the option, node, scanner, and HTML rendering) and [`src/parser/tessera.rs`](src/parser/tessera.rs) (the parser hooks). The test `tests/fork_md.rs` checks that this table's counts match the markers in the code.

| File | Markers | Where | What |
|---|---|---|---|
| `src/lib.rs` | 2 | module list | `pub mod tessera;` |
| | | re-exports | `parse_document_with_definitions` |
| `src/nodes.rs` | 5 | `NodeValue` | The `TesseraLine(Box<NodeTesseraLine>)` variant, after `FrontMatter` |
| | | `NodeValue::block` | An Ascribe line is a block |
| | | `NodeValue::xml_node_name` | `tessera_line` |
| | | `NodeValue::accepts_lines` | An Ascribe line takes its own line |
| | | `Node::can_contain_type` | An Ascribe line contains only its primary's paragraph |
| `src/parser/options.rs` | 1 | `Extension` | The `tessera: Option<Arc<TesseraOptions>>` option, after `front_matter_delimiter` |
| `src/parser/mod.rs` | 13 | module list | `mod tessera;` |
| | | `check_open_blocks_inner` | An Ascribe line stays open while its primary's paragraph does |
| | | `open_new_blocks` | `handle_tessera_line` in the chain of block starts, between block quotes and ATX headings |
| | | `detect_setext_heading` | A text primary never becomes a setext heading (changed condition) |
| | | `detect_table` | A text primary never becomes a table header (changed condition) |
| | | `finalize_borrowed` | A text primary has no link reference definitions (changed statement) |
| | | `finalize_borrowed` | A paragraph that starts with link reference definitions starts on the first line after them (upstream leaves its start position, and so every inline position in it, on the definitions). **Candidate to upstream** (comrak bug; tested by `tests/sourcepos.rs`) |
| | | `handle_setext_heading` | The same, for a setext heading's text. **Candidate to upstream**, with the row above |
| | | `parse_document` | Delegates to the new `parse_document_with_definitions`, which returns the link reference definitions the parser consumed along with the document (the body is upstream's, with the parser kept so its list can be taken) |
| | | `Parser::parse` | Borrows the parser (`&mut self`, was `mut self`) so the caller can take the definitions |
| | | `Parser` | Two fields: the definitions found so far, and the last one `parse_reference_inline` read |
| | | `resolve_reference_link_definitions` | Takes the content's first line and column offsets, and records each definition with its positions (`tessera::locate`); the setext and paragraph call sites pass them |
| | | `parse_reference_inline` | `&mut self` (was `&self`); records the label, destination, and title ranges, and the cleaned values, in the field above. The parsing itself is unchanged |
| `src/parser/inlines.rs` | 1 | `close_bracket_match` | After an image, skips the attribute block that follows it directly (`![alt](src){width=600}`), so its contents are never parsed as emphasis, links, or code. The scan is `tessera::image_attributes_len`, in Ascribe's own file |
| `src/html.rs` | 1 | `format_node_default` | Renders an Ascribe line with `tessera::render_html` |
| `src/cm.rs` | 2 | `CommonMarkFormatter::format_node` | Formats an Ascribe line |
| | | `CommonMarkFormatter::format_tessera_line` | New method, after `format_front_matter` |
| `src/xml.rs` | 1 | `XmlFormatter::format_node` | A `raw` attribute on `tessera_line` |
| `src/tests/sourcepos.rs` | 3 | `TESSERA_LINE`, `node_values`, `sourcepos` | Upstream's test requires a case for every node type |

In total: 76 lines added and 3 changed in 8 upstream source files (48 of the added lines are code; the rest are comments), in 20 hunks, plus 13 lines in one upstream test. The new arms sit next to long-standing neighbors (`Document`, `FrontMatter`, `Paragraph`, block quotes) rather than at the end of each `match`, because upstream appends its own new node types at the end.

Link reference definition support added 68 lines and changed 7 in two upstream files, in 7 hunks (`lib.rs` and `parser/mod.rs`; about half the added lines are comments), and the definition types and `locate` in `src/tessera.rs`. It adds no node and no `NodeValue` variant, and doesn't change what is parsed.

Also Ascribe's, outside `src/`: `Cargo.toml`, this file, and the spike tests in `tests/spike.rs`.

## Merging an upstream release

The likeliest conflicts are the upstream signatures changed to return link reference definitions: `Parser::parse` (`&mut self`, was `mut self`) and `resolve_reference_link_definitions` (an extra `origin` parameter, and its two call sites), plus `parse_reference_inline` (`&mut self`). Take upstream's version and reapply those changes from the table below.

Ascribe's changes are a patch against a pristine upstream release. To move to a new release:

1. **Branch**, and check the new release's changelog for changes to the block parser (`src/parser/mod.rs`), `NodeValue`, or the renderers.
2. **Recreate the Ascribe patch** against the release it's based on (the version in this file):

   ```sh
   git clone --depth 1 --branch v0.55.0 https://github.com/kivikakk/comrak /tmp/comrak-old
   rm /tmp/comrak-old/src/main.rs
   diff -ruN /tmp/comrak-old/src crates/comrak-tessera/src > /tmp/tessera.patch
   ```

   The patch holds only the changed locations above and the two Ascribe files. If it holds anything else, stop: something was changed without being recorded here.
3. **Replace the vendored source** with the new release's, leaving Ascribe's files in place:

   ```sh
   git clone --depth 1 --branch vX.Y.Z https://github.com/kivikakk/comrak /tmp/comrak-new
   rm -rf crates/comrak-tessera/src
   cp -R /tmp/comrak-new/src crates/comrak-tessera/src
   rm crates/comrak-tessera/src/main.rs
   cp /tmp/comrak-new/{build.rs,COPYING,rustfmt.toml} crates/comrak-tessera/
   ```

   Commit this state on its own ("Vendor comrak vX.Y.Z"), so the next merge has a pristine base to diff against.
4. **Reapply the patch**: `patch -p1 -d crates/comrak-tessera -i /tmp/tessera.patch`. Resolve any rejected hunk by hand at the equivalent place (`*.rej` files show them), keeping its `// TESSERA:` marker.
5. **Update the manifest** from the new release's `Cargo.toml`: dependency versions, `rust-version`, new features (declare any feature that pulls in the CLI or syntect as an expected `cfg` instead), and `version`.
6. **Check upstream's new block starts and paragraph rules.** A new block type that can interrupt a paragraph needs no change. A new rule that converts a paragraph into something else (as setext headings and tables do) needs the `tessera::is_text_primary` guard, and a new `NodeValue` variant needs nothing from Ascribe.
7. **Run everything**: `cargo test -p comrak-tessera` (upstream's tests and the spike tests), `cargo test -p tessera-commonmark-suite` (the CommonMark suite against the fork, off and on), then the whole workspace with fmt and clippy. If the CommonMark version changed, update `tests/commonmark/spec.json` and rewrite the baselines.
8. **Update this file**: the version table, and the changed-locations table if anything moved.

A dry run of this procedure measured how the patch survives upstream churn, by applying the v0.55.0 patch to older releases (the same distance a merge forward would cover):

| Release | Released | Lines upstream changed between it and v0.55.0, in the six files Ascribe hooks into | Patch |
|---|---|---|---|
| v0.54.0 | 2026-07 | 6 | Applies cleanly |
| v0.52.0 | 2026-04 | 487 | Applies cleanly |
| v0.50.0 | 2026-01 | 1,169 | Applies cleanly, **builds with a one-line fix** (a renderer helper, `Context::lf`, that v0.50.0 lacks), and passes all of v0.50.0's unit tests (397, plus the 5 scanner tests) and all 22 spike tests |
| v0.45.0 | 2025-10 | 3,836, including 1,519 in `parser/mod.rs` | 20 of 22 hunks apply; 2 in `nodes.rs` are rejected |

The first version of the patch added each new `match` arm at the end, next to upstream's newest variant, and conflicted with every older release, because upstream appends its own new node types there. Moving Ascribe's arms next to long-standing neighbors (`Document`, `FrontMatter`, `Paragraph`) is what made it apply cleanly. Expect a merge to take an hour or two of mechanical work, plus reading upstream's changelog for new rules that convert a paragraph into something else, which would need the `is_text_primary` guard.

comrak is actively maintained (a minor release roughly monthly) and makes breaking API changes in minor releases. Vendoring insulates Ascribe from those until it chooses to merge, and nothing forces frequent merges: the fork only needs upstream for bug fixes and CommonMark spec updates.

## Things to know when changing the fork

- **Positions are line and column, not byte offsets.** comrak's sourcepos is 1-based line and byte column. `tessera-syntax` converts them to byte offsets with its line index, and computes sub-spans (name, attributes, colon, primary) from `NodeTesseraLine::raw`, the node's start column, and `text_primary`. The edge cases are tabs, CRLF, and blockquote markers inside a multi-line primary. The primary's inline nodes already carry correct positions.
- **The primary is a paragraph in the tree.** Consumers must not treat an Ascribe line's child as a block of content. It's the directive's primary.
- **Head parsing is deliberately minimal here.** The fork only finds where a text primary starts: past an attribute block with quoted strings, and a `:`. A head it can't read (an unclosed `{`, text after the name) gets no primary, so it doesn't continue onto the next line. `crates/tessera-syntax/src/head.rs` parses the whole head, and `tests/agreement.rs` there checks that it and this scanner agree on where the primary starts. Keep them in agreement.
- **Inline extensions** go into `parser/inlines.rs`, a 2,700-line hand-written inline parser with a byte-dispatch `match`. `{` is already dispatched there for another extension (Phoenix HEEx, off in Ascribe), and comrak's `attributes` feature already parses `{…}` after images and links, though with a different grammar (Pandoc style, not Ascribe's `key=value, key=value`).
- **comrak's CommonMark renderer isn't Ascribe-aware.** It drops the escape in `\@note`, so its output of that text is a directive. Ascribe's formatter must escape a line-initial `@keyword` (and `.` title lines and `{key}` phrases) itself if it reuses any of comrak's rendering.
- **Extensions Ascribe doesn't use aren't guarded.** With comrak's description-lists extension on, a text primary could still become a description term. Ascribe doesn't enable it; if it ever does, it needs the same guard.
