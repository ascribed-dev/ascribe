---
type: how-to
title: Attach the review report to every pull request
description: Write the HTML review report in CI on each pull request, and link it from the run, so anyone can read the change without a checkout.
available: ascribe 0.2.0
---

`ascribe diff --format html` writes one HTML file showing every changed page rendered, with the same [marks](how-it-works.md#the-marks) as the previews and **Copy prompt** for an agent that reviews the page. It shows the changes, not the pull request's comments. The file holds everything it shows and makes no network requests, so a reviewer opens it with no checkout, no build, and no account, and nothing in it runs but its own script, so a report from a pull request you don't trust yet is safe to open. [The HTML report](diff.md#the-html-report) is its reference.

```sh
ascribe diff --format html > review.html
```

## Add the report to CI

This GitHub Actions job writes the report on each pull request, uploads it, and links it from the run's summary, so a reviewer opens it from the pull request's checks. Change `--config docs` to your project's folder (the one with `ascribe.toml`), or drop it when that's the repository's root, and install `ascribe` however your other jobs do.

@snippet {lang=yaml}: code:.github/workflows/review.yml#job

- **History.** `ascribe diff` compares with the merge base of the pull request's base branch and its head, as the pull request itself does, so the checkout needs enough history to find it: `fetch-depth: 0`. With less, `ascribe diff` stops and says so; `--base-exact` compares with the base branch's tip instead, which also lists changes made on the base branch since the pull request branched.
- **The base.** `--base "origin/$GITHUB_BASE_REF"` names the pull request's base branch. Without it, `ascribe diff` uses the repository's default branch.
- **Opening it.** `archive: false` uploads the file as it is instead of in a zip, so the summary's link opens it in the browser. The link needs read access to the repository, like the rest of the run.

This repository runs this job on these docs, when a pull request changes them or the code their examples come from, and the same job on `examples/quill`, in [`.github/workflows/review.yml`]({repo}/blob/main/.github/workflows/review.yml): the job above is taken from that file. It also installs `@ascribed/cli@next`, the nightly build of `main`, since these docs follow `main`.

## See also

- [](diff.md#the-html-report): what the report shows, and its limits on a large change.
- [](how-it-works.md): what review compares, and what its marks mean.
- [](index.md): reviewing the same change in the previews, with comments.
