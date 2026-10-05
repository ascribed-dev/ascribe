<!-- Generated from the help text in crates/tessera-cli/src/ by crates/tessera-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p tessera-cli docs`. -->

- `[NAME]...`: The sources to update. By default, every source in another repository.
- `--to <REV>`: Move the pin to this revision instead of the head of the source's branch: a commit's full hash, a branch, or a tag. Only for one source: name it.
- `--format <FORMAT>`: How to show what changed.
  - `text` (the default): Each source's commits and copies, then the pages, for people.
  - `json`: One JSON document, for tools.
  - `summary`: Markdown for a pull request's description.
