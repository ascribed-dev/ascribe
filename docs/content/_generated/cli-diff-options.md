@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
- @available: ascribe 0.3.0
  `[PAGE]`: Show only this page's changes, block by block, or, for a fragment, the pages that changed through it: a path from the current directory, or from the content root. With `--format prompt`, the prompt is about this page alone, in the first build it changed in, or about the pages that changed through a fragment.
- `--base <REV>`: The revision to compare with, anything git accepts (a branch, a tag, a commit). By default, the repository's default branch, the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master` that exists. The comparison starts from the merge base of that revision and `HEAD`, as a pull request shows its changes, so commits made on the base branch since you branched aren't listed. A shallow clone (what `actions/checkout` makes by default) may not have the merge base: fetch more history (`fetch-depth: 0`) or use `--base-exact`.
- `--base-exact`: Compare with the revision itself instead of the merge base.
- `--build <NAME>`: Compare only this build. Repeat it for several. By default, every build in `ascribe.toml`.
- `--format <FORMAT>`: How to show the changes.
  - `text` (the default): The changed pages of each build, with counts, for people; with a PAGE, its changes too, each block as it was and as it is.
  - `json`: One JSON document, for tools.
  - `html`: One self-contained HTML file that shows every changed page rendered, with its changes marked, for reviewers.
  - `prompt`: A prompt for an agent that reviews the changes as readers will see them: about every changed page, or about the PAGE named. Nothing when nothing changed.
- `--exit-code`: Exit with 1 when anything changed, as `git diff --exit-code` does.
- @available: ascribe 0.3.0
  `--pages-only`: List the changed pages without their block-level changes. With `--format json`, each page's `changes` is empty and its `counts` still count them, so the report stays short on a large change. With a PAGE, text leaves out the page's blocks; without one, it lists only pages anyway. HTML needs the blocks.
