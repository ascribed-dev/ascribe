<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `[CODE]`: The diagnostic's code (`ASC036`) or name (`link-target-missing`).
- `--list`: List every diagnostic's code and name, one per line, instead.
- `--format <FORMAT>`: How to show the answer.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
