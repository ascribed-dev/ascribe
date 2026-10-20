@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `QUERY`: What to find: words, which must all be in one block in any order, or a phrase in double quotes, matched as written. Quote several words together: `ascribe search "rate limit"`, and a phrase inside them: `ascribe search '"rate limit"'`. Case is ignored, and a word matches inside a longer one.
- `--build <NAME>`: Search the pages of this build; repeat it for several. By default, every build.
- `--limit <N>`: List at most this many places. By default, 50.
