<!-- Generated from the help text in crates/tessera-cli/src/ by crates/tessera-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p tessera-cli docs`. -->

- `--check`: Change nothing, and list each file that would change. Exits with 1 if any would. Use it in CI.
- `[PATHS]...`: Files and directories to format: the `.md` files among them. By default, every `.md` file under the content root.

  Directories whose names start with `.`, `node_modules`, and directories inside the searched ones that hold an `ascribe.toml` other than the project's own (another project, formatted under its own model) are skipped.
