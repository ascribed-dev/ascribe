<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `[DIR]`: The folder to start the project in, from the current folder. It's made if it isn't there. By default, the current folder.
- `--templates`: Take the page templates too, as `ascribe templates eject` with no name does: six types, beside the type `page`.
- `--format <FORMAT>`: How to show what was written.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
