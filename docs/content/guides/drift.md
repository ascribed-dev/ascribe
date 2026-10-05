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

The folder must be in the same git repository as the project. When the code is in another repository, see [Docs kept apart from the code](#docs-kept-apart-from-the-code). See [`[sources.<name>]`](../reference/content-model.md#18-sourcesname) for every key.

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

## Docs kept apart from the code

When the docs have a repository of their own, a source can name the code's repository instead of a folder. Pages address it the same way, so moving the code from one repository to the other changes `ascribe.toml` and no page:

```toml
[sources.api]
git = "https://github.com/acme/api.git"
branch = "main"                           # the branch `update` follows
include = ["src/**", "examples/**"]
```

Ascribe doesn't read the other repository whenever it checks. It copies the files your snippets use, whole, into `sources/api/` beside `ascribe.toml`, and pins the commit they came from in `ascribe.lock`. You commit both with the docs:

1. Write the `@snippet`, then run `ascribe sources fetch`. It pins the source to the head of its branch the first time, and copies the file.
2. Commit `ascribe.lock` and `sources/`.
3. When the code moves on, run `ascribe sources update`. It moves the pin, copies the files again, and lists the pages whose examples changed, the way `ascribe drift` does, so the pull request that moves the pin says which pages to read. The pages are found by comparing with the docs' last commit, so in a new repository, commit once first.

Everything else reads the copies: `ascribe check`, `ascribe build`, the editor, and `ascribe diff` and `ascribe drift`, which read the copies at the base from the docs repository's own history. None of them run `git` against the code's repository or use the network, and `ascribe check` fails on a copy edited by hand, since the change belongs in the code. `ascribe sources status` shows each pin and copy.

- **What's published.** The copies are committed with the docs, so everyone who can read the docs repository can read them. When the code is private and the docs are public, that's a decision to make knowingly; `fetch` and `update` say so the first time they copy a source's files.
- **Access.** `ascribe sources fetch` and `update` run `git`, with whatever credentials let `git fetch` work in that shell. A CI job that only checks or builds needs no access to the code's repository, and neither does the host that builds your site. A job that runs `update` needs read access, for a private repository a token from a GitHub App or a read-only deploy key, given to `git` as any other job would, for example:

  ```sh
  git config --global url."https://x-access-token:${CODE_TOKEN}@github.com/".insteadOf "https://github.com/"
  ```

See [a source in another repository](../reference/content-model.md#a-source-in-another-repository) and [`ascribe sources`](../reference/cli.md#ascribe-sources).

## When an example changes

A snippet keeps the code on the page current, but not the sentence around it. When a pull request changes a region and not the page that shows it, the page now explains code that may work differently. `ascribe drift` lists those pages:

```text
compared with origin/main (3f9c2ab), from its merge base with HEAD (8d01e4c)

Examples that changed. The page shows the new code; check the words around it:
  install.md
    code:service/client.py#connect (+2 −1)

Examples that changed along with the page:
  guides/retries.md
    code:service/client.py#retry (+6 −0)
```

A page covers the code it shows: the regions and whole files it takes snippets from, its fragments' included. Nothing else needs to be declared, and nothing else is reported. A region whose lines only moved, or whose file changed somewhere else, isn't a change. A page that changed in the same pull request, or through a fragment it includes, is in the second group: someone touched it, so it's listed but not flagged. An example that no longer resolves, because its region was renamed or its file moved and the page wasn't updated, is listed first, with why; `ascribe check` fails on it too.

The report says what changed and how much. Whether the words still hold is for whoever reads it.

### The report in CI

This GitHub Actions job writes the report to the run's summary on each pull request. It never fails the check: a changed example is something to read, not an error. Change `--config docs` to your project's folder (the one with `ascribe.toml`), or drop it when that's the repository's root, and install `ascribe` however your other jobs do.

```yaml phrases=true
name: Drift

on:
  pull_request:

permissions:
  contents: read

jobs:
  drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
        with:
          # The whole history, so `ascribe drift` finds where the branch left
          # its base. The default, one commit, isn't enough.
          fetch-depth: 0
      - uses: actions/setup-node@v7
        with:
          node-version: {node}
      - run: npm install --global @ascribed/cli
      - name: Report changed examples
        run: >
          ascribe drift --config docs --base "origin/$GITHUB_BASE_REF"
          --format summary >> "$GITHUB_STEP_SUMMARY"
```

- **History.** Like `ascribe diff`, `ascribe drift` compares with the merge base of the pull request's base branch and its head, so the checkout needs `fetch-depth: 0`. With less, it stops and says to fetch more.
- **The summary.** `--format summary` writes the groups as Markdown, each page linked to its route on `[consumer] site`, and nothing when no example changed, so the summary stays empty on most pull requests.
- **Failing the check.** To fail the job when an example broke, or changed while its page didn't, add `--exit-code`. Start without it, and see how often the report is right for your docs first.

See [`ascribe drift`](../reference/cli.md#ascribe-drift) for its options and JSON.
