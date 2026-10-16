<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `LINE`: The line to retire, named as a mark names it: "self-managed 3.4". It's the oldest live line.
- `--url <URL>`: The address the line's archive is read at, recorded as its `url` in [releases].
- `--dry-run`: Say what the retirement would do, and write nothing.
- `--force`: Write the files even when git can't put them back, and rewrite the marks even when a build that isn't a line of the target changes.
- `--show-unused`: List each block, row, section, and page no live or planned line has, and each mark that says nothing once rewritten, with its page and first line. Works on a line retired already.
- `--remove-unused`: Delete what --show-unused lists. Works on a line retired already.
- `--format <FORMAT>`: How to show what the retirement did.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
