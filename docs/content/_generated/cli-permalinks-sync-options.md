<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--check`: Write nothing, and exit with 1 when the list would change: for CI.
- `--format <FORMAT>`: How to show what changed.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
