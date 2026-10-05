<!-- Generated from the help text in crates/tessera-cli/src/ by crates/tessera-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p tessera-cli docs`. -->

- `--base <REV>`: The revision to compare with, anything git accepts (a branch, a tag, a commit). By default, the repository's default branch, the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master` that exists. The comparison starts from the merge base of that revision and `HEAD`, as a pull request shows its changes, so commits made on the base branch since you branched aren't listed. A shallow clone (what `actions/checkout` makes by default) may not have the merge base: fetch more history (`fetch-depth: 0`) or use `--base-exact`.
- `--base-exact`: Compare with the revision itself instead of the merge base.
- `--build <NAME>`: Compare only this build. Repeat it for several. By default, every build in `ascribe.toml`.
- `--format <FORMAT>`: How to show the changes.
  - `text` (the default): The changed pages of each build, with counts, for people.
  - `json`: One JSON document, for tools.
  - `html`: One self-contained HTML file that shows every changed page rendered, with its changes marked, for reviewers.
- `--exit-code`: Exit with 1 when anything changed, as `git diff --exit-code` does.
