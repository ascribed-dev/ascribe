@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `OLD`: What was removed: a page's path, from the current folder or from the content root; a file's path, `#`, and a heading's id; an address from the host's root, starting with `/`; or a permalink, `go:<name>`, nothing declares now.
- `--reason <TEXT>`: Why it was removed.
- `--format <FORMAT>`: How to show what was recorded.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
