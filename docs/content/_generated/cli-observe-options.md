<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--format <FORMAT>`: How to show what it did. With `json`, what the commands print goes to standard error, so standard output holds the JSON alone.
  - `text` (the default): What it runs and what it wrote, for people.
  - `json`: One JSON document, for tools.
