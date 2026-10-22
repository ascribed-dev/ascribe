@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `LINE`: The line to release, named as a mark names it: "ascribe 0.4.0".
- `--next <LINE>`: The line planned after the released one, added to planned, so that `planned` names it from then on.
- `--dry-run`: Say what the release would do, and write nothing.
- `--force`: Write the files even when git can't put them back: untracked or changed files, or no repository at all.
- `--format <FORMAT>`: How to show what the release did.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
