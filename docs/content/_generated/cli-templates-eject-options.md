<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `[NAME]...`: The templates to write, by name, as `ascribe templates list` shows them. Without a name, every one.
- `--to <DIR>`: The folder to write them in, from the current folder: inside the folder ascribe.toml is in, and outside the content root. By default, templates/ beside ascribe.toml.
- `--format <FORMAT>`: How to show what was written.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
