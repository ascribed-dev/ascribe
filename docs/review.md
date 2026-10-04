# Review

Seeing what a change does to your pages as readers will see them, not as a diff of Markdown files. `ascribe diff` compares the working tree with a git revision page by page: a change to a fragment shows on every page that includes it, and reformatting shows on none. This guide is about putting that in front of reviewers.

## The report

`ascribe diff --format html` writes one HTML file showing every changed page rendered, with what changed marked: added and changed blocks with a bar and a label, the changed words highlighted, removed blocks where they were, and moved blocks linked to where they came from. **Show: Changes / As it will be / As it was** switches between the marks and the two versions of the page. The [command reference](cli.md#the-html-report) describes it in full.

```sh
ascribe diff --format html > review.html
```

The file holds everything it shows, images included, and makes no network requests, so it opens anywhere, with no checkout, no build, and no account. The pages are Ascribe's bare render, the same as the editor's page preview, without your site's layout, navigation, or styles.

## Report in CI

This GitHub Actions job writes the report on each pull request, uploads it, and links it from the run's summary, so a reviewer opens it from the pull request's checks. Change `--config docs` to your project's folder (the one with `ascribe.toml`), or drop it when that's the repository's root, and install `ascribe` however your other jobs do.

```yaml
name: Review

on:
  pull_request:

permissions:
  contents: read

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
        with:
          # The whole history, so `ascribe diff` finds where the branch left
          # its base. The default, one commit, isn't enough.
          fetch-depth: 0
      - uses: actions/setup-node@v7
        with:
          node-version: 24
      - run: npm install --global @ascribed/cli
      - name: Write the report
        run: >
          ascribe diff --config docs
          --base "origin/$GITHUB_BASE_REF" --format html > review.html
      - id: upload
        uses: actions/upload-artifact@v7
        with:
          name: review.html
          path: review.html
          # One file, not zipped, so it opens in the browser.
          archive: false
      - name: Link the report from the summary
        env:
          URL: ${{ steps.upload.outputs.artifact-url }}
        run: |
          {
            echo "### Review report"
            echo
            ascribe diff --config docs --base "origin/$GITHUB_BASE_REF" | sed 's/^/    /'
            echo
            echo "[Open the review report]($URL)"
          } >> "$GITHUB_STEP_SUMMARY"
```

- **History.** `ascribe diff` compares with the merge base of the pull request's base branch and its head, as the pull request itself does, so the checkout needs enough history to find it: `fetch-depth: 0`. With less, `ascribe diff` stops and says so; `--base-exact` compares with the base branch's tip instead, which also lists changes made on the base branch since the pull request branched.
- **The base.** `--base "origin/$GITHUB_BASE_REF"` names the pull request's base branch. Without it, `ascribe diff` uses the repository's default branch.
- **Opening it.** `archive: false` uploads the file as it is instead of in a zip, so the summary's link opens it in the browser. The link needs read access to the repository, like the rest of the run.
- **Large changes.** A report renders the first 300 changed pages and lists the rest by name, and leaves out images over 1 MB, so it stays small enough to open.

This repository runs the same job on `examples/quill`, in [`.github/workflows/review.yml`](../.github/workflows/review.yml).
