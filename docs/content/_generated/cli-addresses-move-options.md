@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `[OLD]`: Where it was: a page's path, from the current folder or from the content root; a file's path, `#`, and a heading's id; or an address from the host's root, starting with `/`, from before the project used Ascribe.
- `[NEW]`: Where it is now: a page's path, from the current folder or from the content root, and optionally `#` and a heading's id.
- `--from-git`: Record every move git sees between the base and the working tree whose old path is gone, one entry for each page, as after a reorganization made with `git mv`. Git sees a file renamed and added, and only when the old and new files are mostly the same.
- `--base <REV>`: With `--from-git`, the revision git's renames are seen from: anything git accepts. By default, the repository's default branch, from its merge base with `HEAD`, as `ascribe check` chooses it.
- `--base-exact`: With `--from-git`, see the renames from the base revision itself instead of its merge base with `HEAD`.
- `--format <FORMAT>`: How to show what was recorded.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
