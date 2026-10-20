---
type: how-to
title: Checks in CI
description: Check every pull request with the same command the editor runs, and the formatting check, in GitHub Actions.
---

The editor checks a page as you write it. CI checks every pull request the same way, so a problem nobody saw in the editor still fails the pull request. It's for a project that has `ascribe.toml` already, as [Getting started](../getting-started.md) sets one up.

## Add the workflow

Add `.github/workflows/check.yml`, which runs `ascribe check --deny-warnings` and `ascribe fmt --check` on every pull request and on each push to `main`:

@snippet {lang=yaml, title=.github/workflows/check.yml}: code:examples/docs-repository/.github/workflows/check.yml#workflow

It installs the latest release. To run the version your `package.json` pins instead, replace that step with `npm ci`, and run each command with `npx`. Where there's no Node, [install `ascribe` on the path](../install.md#without-node) instead.

[`ascribe check`, `build`, and `fmt`](check.md) has each command's options and exit codes.

## See also

- [The project report](report.md): the whole project's state, every week in CI.
- [Diagnostics](diagnostics.md): every problem Ascribe reports, and its fix.
