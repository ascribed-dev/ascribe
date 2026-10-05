---
title: Drift
description: Keeping pages from falling behind the code they describe, with code examples taken from tested files and a report of the pages whose examples changed.
available: next
---

Docs drift when the code changes and the page that describes it doesn't. Ascribe catches what it can when you check, build, or diff, so a reader never sees the old version and a reviewer sees which pages a change reaches. In CI, `ascribe drift` says which pages' examples changed while their words didn't.

## Examples from tested code

A code example copied into a page goes stale the day its code changes, and nothing says so. With `@snippet`, a page takes the example from the file itself: the file that's built, tested, and run. There's no copy to fall behind. Ascribe reads the file whenever it checks, builds, or diffs, so changing the code changes every page that shows it.

### 1. Declare a source

A page can't read files outside its project's folder unless `ascribe.toml` names them. A **source** is a folder of code a page may read, with the files in it that are allowed:

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["service/**", "examples/**"]
ignore = ["**/node_modules/**"]
```

The folder must be in the same git repository as the project. See [`[sources.<name>]`](../reference/content-model.md#18-sourcesname) for every key.

### 2. Mark a region in the code

To show part of a file, mark it with tags in comments. Tag lines are left out of the example, and so are lines you mark to remove:

```python
def connect(host, port):
    # :snippet-start: connect
    client = Client(host, port)
    client.authenticate(load_token())  # :remove:
    client.open()
    # :snippet-end:
    return client
```

The tags are [Bluehawk](https://github.com/mongodb-university/Bluehawk)'s, so code already tagged for it works as it is. A file whose language Ascribe doesn't know the comment syntax of can still be used whole.

### 3. Use it in a page

```markdown
@snippet {title="Connect"}: code:service/client.py#connect
```

The address is the source, the file's path in it, and the region. The page gets a fenced code block with the region's lines, dedented, in the file's language:

```python
client = Client(host, port)
client.open()
```

Leave out `#connect` to show the whole file. The [directive reference](../reference/directives.md#snippet) has every attribute and tag.

### When the code changes

- **Check.** `ascribe check` and the editor report a snippet whose file, source, or region doesn't exist, and a file whose tags don't balance, at the `@snippet` line. Renaming a region or moving a file breaks the build instead of the example.
- **Build.** Every output has the code as it is now. In the JSON output, the code block names its address and the lines of the file it came from.
- **Diff and review.** [`ascribe diff`](../reference/cli.md#ascribe-diff) and review list every page whose snippet's code changed, with its address as what the change comes from: `install.md: 1 changed (through code:service/client.py#connect)`. The base revision's code is read from git, the same way the pages are.

## When an example changes

A snippet keeps the code on the page current, but not the sentence around it. When a pull request changes a region and not the page that shows it, the page now explains code that may work differently. `ascribe drift` lists those pages:

@snippet {lang=text}: code:crates/tessera-cli/tests/output/drift-guide.txt

A page covers the code it shows: the regions and whole files it takes snippets from, its fragments' included. Nothing else needs to be declared, and nothing else is reported. A region whose lines only moved, or whose file changed somewhere else, isn't a change. A page that changed in the same pull request, or through a fragment it includes, is in the second group: someone touched it, so it's listed but not flagged. An example that no longer resolves, because its region was renamed or its file moved and the page wasn't updated, is listed first, with why; `ascribe check` fails on it too.

The report says what changed and how much. Whether the words still hold is for whoever reads it.

### The report in CI

This GitHub Actions job writes the report to the run's summary on each pull request. It never fails the check: a changed example is something to read, not an error. Change `--config docs` to your project's folder (the one with `ascribe.toml`), or drop it when that's the repository's root, and install `ascribe` however your other jobs do.

@snippet {lang=yaml}: code:.github/workflows/drift.yml#job

- **History.** Like `ascribe diff`, `ascribe drift` compares with the merge base of the pull request's base branch and its head, so the checkout needs `fetch-depth: 0`. With less, it stops and says to fetch more.
- **The summary.** `--format summary` writes the groups as Markdown, each page linked to its route on `[consumer] site`, and nothing when no example changed, so the summary stays empty on most pull requests.
- **When it runs.** Run it on every pull request, not only on those that change your docs' folder: an example changes when its code does, so a filter on the docs' paths skips exactly the changes the report is for.
- **Failing the check.** To fail the job when an example broke, or changed while its page didn't, add `--exit-code`. Start without it, and see how often the report is right for your docs first.

This repository runs this job on these docs, in [`.github/workflows/drift.yml`]({repo}/blob/main/.github/workflows/drift.yml): the job above is taken from that file. It also installs `@ascribed/cli@next`, the nightly build of `main`, since these docs follow `main`.

See [`ascribe drift`](../reference/cli.md#ascribe-drift) for its options and JSON.
