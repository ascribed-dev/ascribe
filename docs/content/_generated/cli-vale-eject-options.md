@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `--dry-run`: Say what it would change, and change nothing.
- `--force`: Change files even when git couldn't put them back as they are: ones with changes not committed, ones git doesn't track, or ones in no repository.
- `--format <FORMAT>`: How to show what was written.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
