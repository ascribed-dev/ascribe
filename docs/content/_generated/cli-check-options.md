<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--build <NAME>`: Check only this build. Repeat it for several. By default, every build in `ascribe.toml`. An unknown build name is a usage error, and the message lists the builds.
- `--format <FORMAT>`: How to show the results.
  - `text` (the default): Diagnostics with source snippets, for people.
  - `json`: One JSON document, for tools.
- `--deny-warnings`: Make warnings fail the command too (exit code 1), for CI.
