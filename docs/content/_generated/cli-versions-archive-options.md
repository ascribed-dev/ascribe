@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `LINE`: The line to archive, named as a mark names it: "self-managed 3.4". It's live or planned.
- `--to <DIR>`: The folder to write the archive in, from the current folder: an empty one, or one an archive was written in, whose files it replaces.
- `--build <NAME>`: The versioned build to archive the line of, when two builds are versioned by the line's target.
- `--current <URL>`: The address of the current documentation, which the statement at the top of every page links to.
- `--format <FORMAT>`: How to show what was written.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
