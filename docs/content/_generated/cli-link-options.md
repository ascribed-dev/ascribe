<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `TARGET`: The link's destination, as the page would write it: `keys.md`, `keys.md#rotate-keys`, `#install`.
- `--from <PAGE>`: The page the link is on: a path from the current directory, or from the content root.
- `--format <FORMAT>`: How to show the answer.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
