<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--name <NAME>`: The observation to write: `examples.observations/<NAME>.toml`. Lowercase words of letters and digits joined by `-`, such as `node-22`, one for each environment the examples are tested in. Each observation is whole and separate, and running one doesn't touch another. By default, `default`.
- `--check`: Run nothing, and exit 1 when the observation is stale against the folder as it stands: for a pre-commit hook. It reads the config and the examples, and starts no program, the config's commands included.
- `--format <FORMAT>`: How to show what it did. With `json`, what the commands print goes to standard error, so standard output holds the JSON alone.
  - `text` (the default): What it runs and what it wrote, for people.
  - `json`: One JSON document, for tools.
