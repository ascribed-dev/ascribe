<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `[NAME]...`: Names to retire: permalinks the content no longer declares, which ascribe.permalinks has.
- `--reason <TEXT>`: Why they were retired, for a reader whose link leads there.
- `--to <PAGE>`: Where their addresses lead now: a page, by its path from the current folder or from the content root, and `#` and a heading's id. Without it, an address shows the reason.
- `--page <PAGE>`: Retire every permalink this page declares, or that ascribe.permalinks has on it: a path from the current folder, or from the content root.
- `--under <FOLDER>`: Retire every permalink declared in a file under this folder, or that ascribe.permalinks has there: a path from the current folder, or from the content root.
- `--format <FORMAT>`: How to show what was retired.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
