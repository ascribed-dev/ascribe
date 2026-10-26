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

[`ascribe check`, `build`, `preview`, and `fmt`](check.md) has each command's options and exit codes.

## Keep the addresses you've published
@available: ascribe 0.4.4

A project with [`ascribe.addresses`](../publishing/redirects.md) has `ascribe check` compare the addresses the docs publish with the base's, and the base needs the history: give `actions/checkout` `fetch-depth: 0`, and name the base. On a pull request, it's the branch the pull request targets; on a push, it's the commit before the push, compared as it is:

```yaml
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          fetch-depth: 0
      # ...
      - if: github.event_name == 'pull_request'
        run: ascribe check --deny-warnings --base origin/$GITHUB_BASE_REF
      - if: github.event_name == 'push'
        run: ascribe check --deny-warnings --base ${{ github.event.before }} --base-exact
```

Without the history, or with a base that isn't there, as after a force-push, the rest of the check runs and [`addresses-base-unreadable`](../publishing/redirects.md#the-base) says the addresses weren't compared, a warning `--deny-warnings` fails on.

## See also

- [The project report](report.md): the whole project's state, every day in CI.
- [Diagnostics](diagnostics.md): every problem Ascribe reports, and its fix.
