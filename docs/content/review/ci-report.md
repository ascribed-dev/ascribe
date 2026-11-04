---
type: how-to
title: Attach the HTML preview to every pull request
description: Write the HTML preview in CI on each pull request, and link it from the run, so anyone can read the change without a checkout.
---

`ascribe diff --format html` writes one HTML file showing every changed page rendered, with the same [marks](how-it-works.md#the-marks) as the previews and **Copy prompt** for an agent that reviews the page. It shows the changes, not the pull request's comments. The file holds everything it shows and makes no network requests, so a reviewer opens it with no checkout, no build, and no account. Nothing in it runs but its own script, so a preview from a pull request you don't trust yet is safe to open. [The HTML preview](diff.md#the-html-report) is its reference.

```sh
ascribe diff --format html > review.html
```

## Add the preview to CI
@id: add-the-report-to-ci

This GitHub Actions job writes the preview on each pull request, uploads it, and links it from the run's summary, so a reviewer opens it from the pull request's checks. Add it to the `jobs` of the workflow that checks your docs, as [Everything in one workflow](../checks/ci.md#everything-in-one-workflow) does. It runs the `ascribe` your `package.json` pins, with `npm ci` and `npx`, as the check does, so a new release changes the preview only when you move the pin. When your project is in a folder of the repository, give both `ascribe diff` commands `--config` and the folder, the one with `ascribe.toml`.

@snippet {lang=yaml}: code:examples/astro-site/.github/workflows/docs.yml#preview

- **History.** `ascribe diff` compares with the merge base of the pull request's base branch and its head, as the pull request itself does, so the checkout needs enough history to find it: `fetch-depth: 0`. With less, `ascribe diff` stops and says so; `--base-exact` compares with the base branch's tip instead, which also lists changes made on the base branch since the pull request branched.
- **The base.** `--base "origin/$GITHUB_BASE_REF"` names the pull request's base branch. Without it, `ascribe diff` uses the repository's default branch.
- **Opening it.** `archive: false` uploads the file as it is instead of in a zip, so the summary's link opens it in the browser. The link needs read access to the repository, like the rest of the run.

This repository runs the same job on these docs, when a pull request changes them or the code their examples come from, and on `examples/quill`, in [`.github/workflows/review.yml`]({repo}/blob/main/.github/workflows/review.yml). It installs `@ascribed/cli@next` there, the nightly build of `main`, since these docs follow `main`.

## See also

- [](diff.md#the-html-report): what the preview shows, and its limits on a large change.
- [](how-it-works.md): what review compares, and what its marks mean.
- [](index.md): reviewing the same change in the previews, with comments.
