@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `TARGET`: What to rename: `phrase:<key>`, `value:<value>` (a dimension's value), a heading's id (`keys.md#rotate-keys`), or a page or fragment (`keys.md`), which moves. A path is from the current directory, or from the content root.
- `NEW`: The new name: a phrase's key, a value, or an id. For a page or fragment, where it moves to, read as TARGET is: from the current directory, or from the content root.
