<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--check`: Change nothing, and list each file that would change. Exits with 1 if any would. Use it in CI.
- `[PATHS]...`: Files and directories to format: the `.md` files among them. By default, every `.md` file under the content root. Directories whose names start with `.`, `node_modules`, and directories inside the searched ones that hold an `ascribe.toml` other than the project's own (another project, formatted under its own model) are skipped. So is a file in the content root reached through a symbolic link that leads to a file that isn't a source file of the content root, and it's reported.
- @available: next
  `--format <FORMAT>`: How to show what changed. `json` lists each file with the edits that format it; with `--check`, that's the edits without writing them.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
