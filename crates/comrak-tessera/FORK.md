# comrak-tessera: Tessera's fork of comrak

This crate is [comrak](https://github.com/kivikakk/comrak), vendored and changed to parse Tessera's block-level additions to CommonMark. comrak is licensed under the BSD 2-Clause license; its license is in [`COPYING`](COPYING) and applies to this crate, including Tessera's changes.

## Upstream version

| | |
|---|---|
| Upstream | <https://github.com/kivikakk/comrak> |
| Release | **v0.55.0**, tag commit `6fbe87fafde3953a9f3bc582804318593d703805` (2026-09-06) |
| Vendored in | the commit "Vendor comrak v0.55.0 as crates/comrak-tessera" (phase 04), which is the unmodified upstream source |
| CommonMark | 0.31.2 (upstream's target) |

## Why fork

Tessera changes CommonMark's block structure: a directive line interrupts a paragraph, is never a lazy continuation line, follows CommonMark's container rules, and, when it has a text primary, continues onto following lines as a paragraph does (SPEC §3.2, §3.4, §3.9). These rules have to live inside the block parser. Walking the tree an unmodified parser produces can't recover the right structure: unmodified comrak folds an unindented `@note` into the list item above it, where Tessera ends the list. comrak has no extension point for new block types, so the new block is added to a fork. See [`SPIKE.md`](SPIKE.md) for the evaluation and [PLAN.md](../../project-docs/PLAN.md#parser-comrak-tessera-tessera-syntax) for the design.

## What was vendored

From the v0.55.0 tag, unchanged except for the locations listed below:

- `src/` (all of it, including upstream's unit tests in `src/tests/` and their fixtures), except `src/main.rs`
- `build.rs`, `COPYING`, `rustfmt.toml`

Not vendored: `src/main.rs` (the CLI), `benches/`, `examples/`, `fuzz/`, `vendor/` (git submodules), `script/`, `www/`, `nix/`, and upstream's CI, changelog, and README.

## The manifest

`Cargo.toml` is Tessera's own, derived from upstream's:

- The package is `comrak-tessera`, version `0.55.0` (the upstream release), `publish = false`, license `BSD-2-Clause`. The library is `comrak_tessera`.
- Upstream's `cli` and `syntect` features, their dependencies, and the `comrak` binary are gone. `default = []`. The `bon`, `shortcodes`, `phoenix_heex`, `attributes`, and `arbitrary` features remain and still build and pass upstream's tests (`cargo test -p comrak-tessera --features bon,shortcodes,phoenix_heex,attributes,arbitrary`). The `syntect` `cfg`s left in the source are declared as expected `cfg` values so they don't warn.
- **Doctests are off** (`[lib] doctest = false`). Upstream's doctests import the crate as `comrak`, which no longer resolves. Upstream's unit tests still run.
- The crate doesn't use the workspace lints (upstream uses `unsafe` and `unwrap`). It allows `deprecated` and two clippy lints that only upstream's tests trip under the workspace's `clippy -D warnings`, so that upstream code stays untouched.
- `rustfmt.toml` is upstream's, so `cargo fmt` formats vendored code as upstream does.

## Changed locations

Every change to an upstream file is marked in the code with a `// TESSERA:` comment. `grep -rn '// TESSERA:' src` lists them. Two files are wholly Tessera's and have no upstream counterpart: [`src/tessera.rs`](src/tessera.rs) (the option, node, scanner, and HTML rendering) and [`src/parser/tessera.rs`](src/parser/tessera.rs) (the parser hooks). The test `tests/fork_md.rs` checks that this table's counts match the markers in the code.

| File | Markers | Where | What |
|---|---|---|---|
| `src/lib.rs` | 2 | module list | `pub mod tessera;` |
| | | re-exports | `parse_document_with_definitions` (phase 23) |
| `src/nodes.rs` | 5 | `NodeValue` | The `TesseraLine(Box<NodeTesseraLine>)` variant, after `FrontMatter` |
| | | `NodeValue::block` | A Tessera line is a block |
| | | `NodeValue::xml_node_name` | `tessera_line` |
| | | `NodeValue::accepts_lines` | A Tessera line takes its own line |
| | | `Node::can_contain_type` | A Tessera line contains only its primary's paragraph |
| `src/parser/options.rs` | 1 | `Extension` | The `tessera: Option<Arc<TesseraOptions>>` option, after `front_matter_delimiter` |
| `src/parser/mod.rs` | 13 | module list | `mod tessera;` |
| | | `check_open_blocks_inner` | A Tessera line stays open while its primary's paragraph does |
| | | `open_new_blocks` | `handle_tessera_line` in the chain of block starts, between block quotes and ATX headings |
| | | `detect_setext_heading` | A text primary never becomes a setext heading (changed condition) |
| | | `detect_table` | A text primary never becomes a table header (changed condition) |
| | | `finalize_borrowed` | A text primary has no link reference definitions (changed statement) |
| | | `finalize_borrowed` | A paragraph that starts with link reference definitions starts on the first line after them (phase 05: upstream leaves its start position, and so every inline position in it, on the definitions). **Candidate to upstream** (comrak bug; tested by `tests/sourcepos.rs`) |
| | | `handle_setext_heading` | The same, for a setext heading's text (phase 05). **Candidate to upstream**, with the row above |
| | | `parse_document` | Delegates to the new `parse_document_with_definitions`, which returns the link reference definitions the parser consumed along with the document (phase 23; the body is upstream's, with the parser kept so its list can be taken) |
| | | `Parser::parse` | Borrows the parser (`&mut self`, was `mut self`) so the caller can take the definitions |
| | | `Parser` | Two fields: the definitions found so far, and the last one `parse_reference_inline` read |
| | | `resolve_reference_link_definitions` | Takes the content's first line and column offsets, and records each definition with its positions (`tessera::locate`); the setext and paragraph call sites pass them |
| | | `parse_reference_inline` | `&mut self` (was `&self`); records the label, destination, and title ranges, and the cleaned values, in the field above. The parsing itself is unchanged |
| `src/parser/inlines.rs` | 1 | `close_bracket_match` | After an image, skips the attribute block that follows it directly (`![alt](src){width=600}`), so its contents are never parsed as emphasis, links, or code (phase 07). The scan is `tessera::image_attributes_len`, in Tessera's own file |
| `src/html.rs` | 1 | `format_node_default` | Renders a Tessera line with `tessera::render_html` |
| `src/cm.rs` | 2 | `CommonMarkFormatter::format_node` | Formats a Tessera line |
| | | `CommonMarkFormatter::format_tessera_line` | New method, after `format_front_matter` |
| `src/xml.rs` | 1 | `XmlFormatter::format_node` | A `raw` attribute on `tessera_line` |
| `src/tests/sourcepos.rs` | 3 | `TESSERA_LINE`, `node_values`, `sourcepos` | Upstream's test requires a case for every node type |

In total: 76 lines added and 3 changed in 8 upstream source files (48 of the added lines are code; the rest are comments), in 20 hunks, plus 13 lines in one upstream test. The new arms sit next to long-standing neighbors (`Document`, `FrontMatter`, `Paragraph`, block quotes) rather than at the end of each `match`, because upstream appends its own new node types at the end.

Phase 23 (link reference definitions) added 68 lines and changed 7 in two upstream files, in 7 hunks (`lib.rs` and `parser/mod.rs`; about half the added lines are comments), and the definition types and `locate` in `src/tessera.rs`. It adds no node and no `NodeValue` variant, and doesn't change what is parsed.

Also Tessera's, outside `src/`: `Cargo.toml`, this file, `SPIKE.md`, and the spike tests in `tests/spike.rs`.

## Merging an upstream release

The likeliest conflicts are the two upstream signatures phase 23 changed to return definitions: `Parser::parse` (`&mut self`, was `mut self`) and `resolve_reference_link_definitions` (an extra `origin` parameter, and its two call sites), plus `parse_reference_inline` (`&mut self`). Take upstream's version and reapply those changes from the table below.

Tessera's changes are a patch against a pristine upstream release. To move to a new release:

1. **Branch**, and check the new release's changelog for changes to the block parser (`src/parser/mod.rs`), `NodeValue`, or the renderers.
2. **Recreate the Tessera patch** against the release it's based on (the version in this file):

   ```sh
   git clone --depth 1 --branch v0.55.0 https://github.com/kivikakk/comrak /tmp/comrak-old
   rm /tmp/comrak-old/src/main.rs
   diff -ruN /tmp/comrak-old/src crates/comrak-tessera/src > /tmp/tessera.patch
   ```

   The patch holds only the changed locations above and the two Tessera files. If it holds anything else, stop: something was changed without being recorded here.
3. **Replace the vendored source** with the new release's, leaving Tessera's files in place:

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
6. **Check upstream's new block starts and paragraph rules.** A new block type that can interrupt a paragraph needs no change. A new rule that converts a paragraph into something else (as setext headings and tables do) needs the `tessera::is_text_primary` guard, and a new `NodeValue` variant needs nothing from Tessera.
7. **Run everything**: `cargo test -p comrak-tessera` (upstream's tests and the spike tests), `cargo test -p tessera-commonmark-suite` (the CommonMark suite against the fork, off and on), then the whole workspace with fmt and clippy. If the CommonMark version changed, update `tests/commonmark/spec.json` and rewrite the baselines.
8. **Update this file**: the version table, and the changed-locations table if anything moved.

A dry run of this procedure is recorded in `SPIKE.md`: the v0.55.0 patch applies without conflicts to v0.54.0, v0.52.0, and v0.50.0 (where it also builds and passes the tests with a one-line fix), and with two rejected hunks to v0.45.0.
