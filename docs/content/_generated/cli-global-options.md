<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--config <PATH>`: The content model, `ascribe.toml`, or a directory that holds one. By default, the nearest `ascribe.toml` in the current directory or a parent, so the commands work from anywhere inside a project. Not for `lsp`, whose project comes from the editor.
- `--color <WHEN>`: When to color text output.
  - `auto` (the default): When writing to a terminal, unless `NO_COLOR` is set.
  - `always`: Always.
  - `never`: Never.
