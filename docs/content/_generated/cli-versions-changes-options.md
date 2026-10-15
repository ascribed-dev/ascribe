<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `FROM`: The line compared from, named as a mark names it: "self-managed 3.4".
- `TO`: The line compared to, of the same target: 3.5.
- `--format <FORMAT>`: How to show what differs.
  - `text` (the default): What was added, removed, and changed, for people.
  - `json`: One JSON document, for tools.
  - `markdown`: A first draft of release notes, with a heading for each page.
