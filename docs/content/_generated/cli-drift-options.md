<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

- `--base <REV>`: The revision to compare with, anything git accepts (a branch, a tag, a commit). By default, the repository's default branch, found as `ascribe diff` finds it. The comparison starts from the merge base of that revision and `HEAD`, as a pull request shows its changes. A shallow clone (what `actions/checkout` makes by default) may not have the merge base: fetch more history (`fetch-depth: 0`).
- `--build <NAME>`: Look only at this build. Repeat it for several. By default, every build in `ascribe.toml`.
- `--format <FORMAT>`: How to show the report.
  - `text` (the default): The groups of pages, with each example that broke or changed, for people.
  - `json`: One JSON document, for tools.
  - `summary`: Markdown for a CI job's summary, each page linked to the site; nothing when no example changed.
- `--exit-code`: Exit with 1 when a page's example no longer resolves, or changed while the page didn't.
