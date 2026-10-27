@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `[NAME]`: The prompt's name.
- `--arg <KEY=VALUE>`: An argument of the prompt, as `key=value`. Repeat it for several. `--list` shows each prompt's arguments.
- `--list`: List the prompts, with their arguments, instead.
- `--format <FORMAT>`: How to show the prompt, or the list.
  - `text` (the default): Text, for people and agents to read.
  - `json`: One JSON document, for tools.
