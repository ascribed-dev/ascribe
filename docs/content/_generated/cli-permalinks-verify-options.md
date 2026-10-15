<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `[NAME]...`: The names to check.
- `--from <FILE>`: Also check the names in this file, one on each line. Blank lines and lines that start with `#` are skipped.
- `--against <PATH>`: The list to check against. By default, the ascribe.permalinks beside the project's ascribe.toml.
- `--format <FORMAT>`: How to show the answer.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
