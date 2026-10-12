<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--format <FORMAT>`: How to show what's changed.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
- `--exit-code`: Exit with 1 when the set has changed a template the project took since the copy, or no longer has one.
- `--diff <NAME>`: Print the change to one template, by name, as a unified diff from the project's files to the set's, instead of the status. It applies nothing.
