<!-- Generated from the help text in crates/tessera-cli/src/ by crates/tessera-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p tessera-cli docs`. -->

- `--build <NAME>`: Build only this build. Repeat it for several. By default, every build in `ascribe.toml`.
- `--emit <OUTPUTS>`: Which outputs to write, separated by commas. All of them by default.
  - `site`: Markdown plus web components, for an Astro site.
  - `plain`: Fully resolved CommonMark with no HTML.
  - `json`: The resolved tree as JSON.
- `--format <FORMAT>`: How to show the checks' results, as for `ascribe check`.
  - `text` (the default): Diagnostics with source snippets, for people.
  - `json`: One JSON document, for tools.
- @available: next
  `--anchors`: Mark each block of the site output with the source file and lines it came from, for review. Each Markdown block gets an `<!--ascribe-anchor …-->` comment before it, and each element Ascribe writes gets `data-ascribe-source` (with `data-ascribe-via` for a block from a fragment). The site output's manifest records `"anchors": true`. Without it, the output has no anchors. The other outputs are the same either way.
