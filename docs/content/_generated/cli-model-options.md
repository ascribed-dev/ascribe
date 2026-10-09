<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `[PATH]`: A file or folder in the project, to find its `ascribe.toml` from. By default, the current directory.
- `--section <NAME>`: Show only this section, in full.
  - `types`: Page types, with their files and frontmatter fields.
  - `dimensions`: Dimensions and their values.
  - `phrases`: Phrases, with their values.
  - `features`: Features, with their availability.
  - `glossary`: Glossary terms.
  - `widgets`: Project widgets, with their attributes.
  - `builds`: Builds, with their variant and availability modes.
- `--format <FORMAT>`: How to show the answer.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
