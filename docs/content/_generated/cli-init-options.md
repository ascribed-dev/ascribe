@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- `[DIR]`: The folder to start the project in, from the current folder. It's made if it isn't there. By default, the current folder.
- `--templates`: Take the page templates too, as `ascribe templates eject` with no name does: six types, beside the type `page`.
- `--content-root <DIR>`: The folder the pages are in, from the project's folder, with `/` between its parts. By default, `docs`. When it holds Markdown already, `ascribe.toml` declares no page types, so no page needs `type:`, and `init` checks the pages and says what to run next. `--templates` declares types, so it's refused there.
- `--dry-run`: Say what it would change, and change nothing.
- `--force`: Change files even when git couldn't put them back as they are: ones with changes not committed, ones git doesn't track, or ones in no repository.
- `--format <FORMAT>`: How to show what was written.
  - `text` (the default): Short text, for people and agents to read.
  - `json`: One JSON document, for tools.
