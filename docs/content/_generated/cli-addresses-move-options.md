@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `OLD`: Where it was: a page's path, from the current folder or from the content root; a file's path, `#`, and a heading's id; or an address from the host's root, starting with `/`, from before the project used Ascribe.
- `NEW`: Where it is now: a page's path, from the current folder or from the content root, and optionally `#` and a heading's id.
- `--format <FORMAT>`: How to show what was recorded.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
