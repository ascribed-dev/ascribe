---
type: how-to
title: Checks in CI
description: Check every pull request with the same command the editor runs, and the formatting check, in GitHub Actions.
---

The editor checks a page as you write it. CI checks every pull request the same way, so a problem nobody saw in the editor still fails the pull request. It's for a project that has `ascribe.toml` already, as [Getting started](../getting-started.md) sets one up.

## Add the workflow

Add `.github/workflows/check.yml`, which installs the project's dependencies and runs `npx ascribe check --deny-warnings` and `npx ascribe fmt --check` on every pull request and on each push to `main`, and shows each problem on the pull request's lines, as a linter's are. It checks out the whole history and names the base, which the project's `ascribe.addresses` needs ([below](#keep-the-addresses-youve-published)):

@snippet {lang=yaml, title=.github/workflows/check.yml}: code:examples/getting-started/.github/workflows/check.yml#workflow

`npm ci` installs the `@ascribed/cli` your `package.json` pins, so CI runs the `ascribe` the editor and `npx ascribe` run, and a new release changes the check only when you move the pin. With pnpm, run `pnpm install --frozen-lockfile` and `pnpm exec ascribe`.

Where there's no Node, [install `ascribe` on the path](../install.md#without-node) with the install script, naming the release, and run `ascribe` as it is:

```yaml
      - run: curl -fsSL https://ascribed-dev.com/install.sh | sh -s -- --version <version>
      - if: github.event_name == 'pull_request'
        run: ascribe check --deny-warnings --format github --base origin/$GITHUB_BASE_REF
      - if: github.event_name == 'push'
        run: ascribe check --deny-warnings --format github --base ${{ github.event.before }} --base-exact
      - run: ascribe fmt --check --format github
```

Name the release your team runs, and change it as you would a dependency: without `--version`, the script installs the latest release, and a release with a new check fails pull requests nobody changed.

[`ascribe check`, `build`, `preview`, and `fmt`](check.md) has each command's options and exit codes.

### The problems on the pull request's lines

`--format github` writes each problem as a GitHub Actions [annotation](check.md#output-for-github-actions), which the pull request shows on its line, with the diagnostic's name and message. It needs no permissions and no upload step, and works with a project in a folder of the repository. Without it, the problems are in the job's log alone. With a release that doesn't have it yet, leave it out: an unknown format fails the step.

## Keep the addresses you've published

Every project `ascribe init` started has [`ascribe.addresses`](../publishing/redirects.md), and `ascribe check` compares the addresses the docs publish with the base's, so a page that moved without a record fails the pull request. That's why the workflow gives `actions/checkout` `fetch-depth: 0`, for the history, and names the base: on a pull request, the branch it targets; on a push, the commit before the push, compared as it is.

Without the history, as `actions/checkout`'s default gives, or with a base that isn't there, as after a force-push, the rest of the check runs and [`addresses-base-unreadable`](../publishing/redirects.md#addresses-base) says the addresses weren't compared, a warning `--deny-warnings` fails on. A workflow that leaves out `fetch-depth: 0` or `--base` fails the project's first pull request this way.

## Keep tested examples tested

A project with [tested examples](../examples/tested.md) fails the check when one changed after its last test run: the page has lost its "Tested with" line, and an expected output beside it may no longer be what the code prints. [`example-stale`](diagnostics/output-checks.md#ASC199) is a warning, which `--deny-warnings` fails on, at each `@snippet` of the file, naming the folder to run `ascribe observe` in. An example that was never tested, `example-unobserved`, is advice, and passes.

To test it again, run `ascribe observe` in that folder and commit the observation it writes. CI can run it too, as `ascribe observe --yes`: without a terminal, nobody is there to answer its question, so it runs nothing without `--yes`. It writes the observation only when the test passes. In the code's own workflow, `ascribe observe --check` runs nothing and fails a pull request that changed an example without testing it again, before the docs see it:

```yaml
      - name: Check the examples were tested
        working-directory: code
        run: ascribe observe --check
```

To have CI write a new observation, run `ascribe observe --yes` in a job that has what the test needs, and commit `examples.observations/` from there, or keep it as an artifact to commit by hand. When the code is [in another repository](../examples/tested.md#when-the-code-is-in-another-repository), it's that repository's CI that runs it, and the next `ascribe sources update` brings the observation here; until then, an update pull request that brings a stale example fails this check.

To let a stale example pass, set `example-stale = "advice"` in [`[checks]`](../content-model/checks.md#22-checks).

## See also

- [The project report](report.md): the whole project's state, every day in CI.
- [Diagnostics](diagnostics.md): every problem Ascribe reports, and its fix.
