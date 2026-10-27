@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `TYPE`: The page's type, as `ascribe model --section types` names it.
- `PATH`: Where the page goes, from the current folder, in the content root: a page's path, ending in .md, or a folder, where the page is named for its title.
- `--title <TITLE>`: The page's title, set in its frontmatter. Without one, the page keeps the template's, to fill in with the rest, and PATH names the page itself.
- `--dry-run`: Say where the page would go, and print its text, writing nothing.
- `--force`: Taken as by every command that writes. `new` changes no file that's there, only adds one, so the git guard has nothing to refuse.
- `--format <FORMAT>`: How to show what was written.
  - `text` (the default): Where the page went, and what to do next, for people and agents.
  - `json`: One JSON document, for tools.
