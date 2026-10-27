@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `[NAME]...`: The sources to fetch. By default, every source in another repository.
- `--format <FORMAT>`: How to show what was copied.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
