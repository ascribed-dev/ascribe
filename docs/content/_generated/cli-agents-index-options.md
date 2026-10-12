<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--build <NAME>`: The build whose pages to index. Needed when `ascribe.toml` has more than one build.
- `--into <FILE>`: Write the index into this file, between its markers, instead of printing it. The rest of the file is left as it is, and a file without the markers gets the block at its end.
- `--pages <DIR>`: Also copy each page's Markdown version into this folder, and point the index's addresses into it: for a package that ships its docs. The addresses are from the folder of the file the index is written into, or from the current directory when it's printed.
- `--format <FORMAT>`: How to show the answer.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
