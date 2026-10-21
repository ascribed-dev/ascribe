---
type: concept
title: Examples and drift
description: Keeping pages from falling behind the code they describe, with code examples taken from tested files and a report of the pages whose examples changed.
---

Docs drift when the code changes and the page that describes it doesn't. A code example copied into a page goes stale the day its code changes, and nothing says so. Ascribe takes the example from the file itself, the one that's built, tested, and run, so there's no copy to fall behind, and it catches what it can when you check, build, or diff, so a reader never sees the old version and a reviewer sees which pages a change reaches.

- **Check.** `ascribe check` and the editor report a snippet whose file, source, or region doesn't exist, and a file whose tags don't balance, at the `@snippet` line. Renaming a region or moving a file breaks the build instead of the example.
- **Build.** Every output has the code as it is now. In the JSON output, the code block names its address and the lines of the file it came from.
- **Diff and review.** [`ascribe diff`](../review/diff.md#cli-diff) and review list every page whose snippet's code changed, with its address as what the change comes from: `install.md: 1 changed (through code:service/client.py#connect)`. The base revision's code is read from git, the same way the pages are.

A snippet keeps the code on the page current, but not the sentence around it, and it can't say whether the code still works. Two more things cover that: a report of the pages whose examples changed while their words didn't, and a line under each example saying what it was tested with.

## Tasks

- [](snippets.md): mark a region in the code and show it on a page.
- [](docs-repository.md): take the examples from the code's repository when the docs have one of their own, and keep them up to date with a pull request.
- [](drift.md): list, on each pull request, the pages whose examples changed.
- [](tested.md): have each page say what its examples were tested with.

## See also

- [`ascribe sources` and `ascribe observe`](sources.md): their options, exit codes, and JSON.
- [`@snippet`](../writing/directives.md#snippet): every attribute and tag.
