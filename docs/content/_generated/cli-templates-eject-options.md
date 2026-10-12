<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `[NAME]...`: The templates to write, by name, as `ascribe templates list` shows them. Without a name, every one.
- `--to <DIR>`: The folder to write them in, from the current folder: inside the folder ascribe.toml is in, and outside the content root. By default, templates/ beside ascribe.toml.
- `--keep-type`: When ascribe.toml declares a template's type already, keep the type and add only its template and guide to it, leaving its files, frontmatter, other keys, and comments as they are. Refuses a type whose frontmatter doesn't declare the template's fields as strings, or that names a template or guide already.
- `--format <FORMAT>`: How to show what was written.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
