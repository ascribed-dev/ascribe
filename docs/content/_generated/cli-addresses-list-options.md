@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `--build <NAME>`: Show only this build. By default, every build in `ascribe.toml`.
- `--format <FORMAT>`: How to show the redirects.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
