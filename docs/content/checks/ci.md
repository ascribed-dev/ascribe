---
type: how-to
title: Check pull requests in CI
description: Check every pull request with the same command the editor runs, and the formatting check, in GitHub Actions.
---

The editor checks a page as you write it. CI checks every pull request the same way, so a problem nobody saw in the editor still fails the pull request. It's for a project that has `ascribe.toml` already, as [Getting started](../getting-started.md) sets one up.

## Add the workflow

@steps
1. Add `.github/workflows/check.yml`, which installs the project's dependencies and runs `npx ascribe check --deny-warnings` and `npx ascribe fmt --check` on every pull request and on each push to `main`, and shows each problem on the pull request's lines, as a linter's are. It checks out the whole history and names the base, which the project's `ascribe.addresses` needs ([below](#keep-the-addresses-youve-published)):

   @snippet {lang=yaml, title=.github/workflows/check.yml}: code:examples/getting-started/.github/workflows/check.yml#workflow

2. Commit it, and open a pull request. The check runs on it, and shows each problem on its line of the pull request.

`npm ci` installs the `@ascribed/cli` your `package.json` pins, so CI runs the `ascribe` the editor and `npx ascribe` run, and a new release changes the check only when you move the pin. With pnpm, run `pnpm install --frozen-lockfile` and `pnpm exec ascribe`.

Where there's no Node, [install `ascribe`](../install.md#without-node) with the install script, naming the release, add the folder it installs into, `~/.local/bin`, to the job's path, and run `ascribe` as it is:

```yaml
      - run: curl -fsSL https://ascribed-dev.com/install.sh | sh -s -- --version <version>
      - run: echo "$HOME/.local/bin" >> "$GITHUB_PATH"
      - if: github.event_name == 'pull_request'
        run: ascribe check --deny-warnings --format github --base origin/$GITHUB_BASE_REF
      - if: github.event_name == 'push' && !github.event.created
        run: ascribe check --deny-warnings --format github --base ${{ github.event.before }} --base-exact
      - if: github.event_name == 'push' && github.event.created
        run: ascribe check --deny-warnings --format github
      - run: ascribe fmt --check --format github
```

Name the release your team runs, as its number, such as `{version}` (a leading `v` is accepted too), and change it as you would a dependency: without `--version`, the script installs the latest release, and a release with a new check fails pull requests nobody changed.

[`ascribe check`, `build`, `preview`, and `fmt`](check.md) has each command's options and exit codes.

### The problems on the pull request's lines

`--format github` writes each problem as a GitHub Actions [annotation](check.md#output-for-github-actions), which the pull request shows on its line, with the diagnostic's name and message. It needs no permissions and no upload step, and works with a project in a folder of the repository. Without it, the problems are in the job's log alone. With a release that doesn't have it yet, leave it out: an unknown format fails the step.

## Keep the addresses you've published

Every project `ascribe init` started has [`ascribe.addresses`](../publishing/redirects.md), and `ascribe check` compares the addresses the docs publish with the base's, so a page that moved without a record fails the pull request. That's why the workflow gives `actions/checkout` `fetch-depth: 0`, for the history, and names the base: on a pull request, the branch it targets; on a push, the commit before the push, compared as it is. The push that creates `main`, a new repository's first, has no commit before it: there, the check names no base, and compares the pushed commit with itself.

Without the history, as `actions/checkout`'s default gives, or with a base that isn't there, as after a force-push or with `--base` on the push that creates the branch, the rest of the check runs and [`addresses-base-unreadable`](../publishing/redirects.md#addresses-base) says the addresses weren't compared, a warning `--deny-warnings` fails on. A workflow that leaves out `fetch-depth: 0` or `--base` fails the project's first pull request this way ([Troubleshooting](../troubleshooting.md#addresses-base-unreadable)).

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

## Everything in one workflow
@id: everything-in-one-workflow

For a project that publishes with [`@ascribed/astro`](../astro/index.md), this workflow runs every check on this page and the others, builds the site, and attaches the [HTML preview](../review/ci-report.md) to each pull request. Like the workflow above, it runs the `ascribe` your `package.json` pins, in every job:

@snippet {lang=yaml, title=.github/workflows/docs.yml}: code:examples/astro-site/.github/workflows/docs.yml#workflow

- **`check`** runs `ascribe check` with the base, as [above](#keep-the-addresses-youve-published), and `ascribe fmt --check`. Then `ascribe agents sync --check` fails when the [agents' instructions](../agents/instructions.md#keeping-them-current-in-ci) are out of date, and `ascribe observe --check` when an example changed without [being tested again](#keep-tested-examples-tested). Leave out the steps for what your project doesn't have: without `ascribe.examples.toml`, `ascribe observe --check` fails.
- **`build`** runs `astro build`, so a page the site can't build fails the pull request, not the deploy.
- **`preview`** writes the HTML preview of the pull request's changes, and links it from the run's summary. [Attach the HTML preview to every pull request](../review/ci-report.md) says what each step does.

With the project in a folder of the repository, give each `ascribe` command `--config` and the folder, and each job `defaults.run.working-directory` for `npm ci` and `astro build`.

## In another CI
@id: in-another-ci

`--format github` is for GitHub Actions alone. Elsewhere, such as GitLab CI or Jenkins, run the same commands, and turn [`--format json`](check.md#json-output) into what your CI shows. Each problem is one entry of `diagnostics`, with its `file`, its `range`, its `severity`, its `slug`, and its `message`. This prints each as a line, then fails as the check did:

```sh
npx ascribe check --deny-warnings --format json > check.json || status=$?
jq -r '.diagnostics[] | "\(.file):\(.range.start.line):\(.range.start.column): \(.severity): \(.message) [\(.slug)]"' check.json
exit "${status:-0}"
```

The same fields map onto a format your CI reads, such as a code quality or SARIF report. The check needs the history and the base there too, for [the addresses](#keep-the-addresses-youve-published): fetch the whole history, and give `--base` the branch the change targets.

## See also

- [Act on the project report](report.md): the whole project's state, every day in CI.
- [Diagnostics](diagnostics.md): every problem Ascribe reports, and its fix.
