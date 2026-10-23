@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `PAGE`: The page: a path from the current directory, or from the content root. Or `go:` and a permalink's name, for the help a product shows for it: its title, summary, and text, as the build's `_ascribe/help.json` has them, and its address.
- `--build <NAME>`: The build whose reader to show it as. Needed when `ascribe.toml` has more than one build.
- `--frontmatter`: Write the page's frontmatter first, between `---` lines. Nothing for `go:<name>`, which has no frontmatter.
- `--format <FORMAT>`: How to show the answer.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
