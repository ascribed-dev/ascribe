---
type: how-to
title: Report the pages whose examples changed
description: Run ascribe drift on each pull request, so the run's summary lists the pages whose code examples changed while their words didn't.
available: ascribe 0.2.0
---

A snippet keeps the code on the page current, but not the sentence around it. When a pull request changes a region and not the page that shows it, the page now explains code that may work differently. `ascribe drift` lists those pages:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/drift-guide.txt

A page covers the code it shows: the regions and whole files it takes [snippets](snippets.md) from, its fragments' included. Nothing else needs to be declared, and nothing else is reported. A region whose lines only moved, or whose file changed somewhere else, isn't a change. A page that changed in the same pull request, or through a fragment it includes, is in the second group: someone touched it, so it's listed but not flagged. An example that no longer resolves, because its region was renamed or its file moved and the page wasn't updated, is listed first, with why; `ascribe check` fails on it too.

The report says what changed and how much. Whether the words still hold is for whoever reads it.

## Add the report to CI

This GitHub Actions job writes the report to the run's summary on each pull request. It never fails the check: a changed example is something to read, not an error. Change `--config docs` to your project's folder (the one with `ascribe.toml`), or drop it when that's the repository's root, and install `ascribe` however your other jobs do.

@snippet {lang=yaml}: code:.github/workflows/drift.yml#job

- **History.** Like `ascribe diff`, `ascribe drift` compares with the merge base of the pull request's base branch and its head, so the checkout needs `fetch-depth: 0`. With less, it stops and says to fetch more.
- **The summary.** `--format markdown` writes the groups as Markdown, each page linked to its route on `[consumer] site`, and nothing when no example changed, so the summary stays empty on most pull requests.
- **When it runs.** Run it on every pull request, not only on those that change your docs' folder: an example changes when its code does, so a filter on the docs' paths skips exactly the changes the report is for.
- **Failing the check.** To fail the job when an example broke, or changed while its page didn't, add `--exit-code`. Start without it, and see how often the report is right for your docs first.

This repository runs this job on these docs, in [`.github/workflows/drift.yml`]({repo}/blob/main/.github/workflows/drift.yml): the job above is taken from that file. It also installs `@ascribed/cli@next`, the nightly build of `main`, since these docs follow `main`.

## See also

- [`ascribe drift`](../review/diff.md#cli-drift): its options and JSON.
- [Keep the docs in another repository](docs-repository.md#the-update-pull-request): the pull request that moves a source's pin lists the same pages.
