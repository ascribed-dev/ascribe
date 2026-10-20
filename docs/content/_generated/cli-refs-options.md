@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `TARGET`: What to find the uses of: a page or fragment (`keys.md`), a heading (`keys.md#rotate-keys`), an image (`images/keys.png`), `phrase:<key>`, `feature:<key>`, `term:<id>`, `dimension:<name>`, `note:<type>`, or `widget:<name>`. A path is from the current directory, or from the content root.
- `--limit <N>`: List at most this many places. By default, 50 in text and 500 in JSON.
- `--format <FORMAT>`: How to show the answer.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
