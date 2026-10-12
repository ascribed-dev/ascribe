# Phase 7: Repositories to copy

Part of [Tested examples](README.md). Needs phases 3, 4, and 6.

## Goal

A team that wants tested examples starts from a repository that already has them: examples as plain functions, an expected-output file for each, tests that compare the two, the config, committed observations, and a docs project that shows them. Two to begin with: Node with TypeScript, and Python.

Being opinionated about testing without building a test product means showing the pattern where it can be copied whole.

## Context

- README decision 22, and the proposal's "Example repositories" and "Shown output".
- The research, [Code examples tied to tested code](../reports/Code%20examples%20tied%20to%20tested%20code.md): the pattern that held up at scale, and its clearest warning, sample infrastructure left to rot.
- `examples/tested`, from phase 2, which is the Node pattern in small, inside this repository.
- `examples/docs-repository`, and its workflow that follows another repository and opens a pull request.
- [outside.md](../outside.md).

## Design

### What each repository holds

```
examples/            each example a plain function in a file, tagged, with its expected output beside it
tests/               one test for each example: call it, compare with the output file
ascribe.examples.toml
examples.observations/
docs/                an Ascribe project that shows each example and its output
README.md            how to add an example, run the tests, and observe
.github/workflows/   the tests, on a schedule and on a pull request
```

- **No custom code.** Nothing in a repository is there to make Ascribe work: the CLI writes the observations, and the tests are the language's usual framework (`node --test` or Vitest, and pytest).
- **Two observations** in each, under two versions of the runtime, so the docs show a merged line.
- **The docs are a source in the same repository.** The README says what changes when the docs live in another: one `[sources.<name>]` with `git`, and `ascribe sources update`.
- **Python's repository has no `package.json`.** It installs `ascribe` with phase 6's script, which is what proves that path.

### Kept working

Each repository's own CI runs its tests weekly and on a pull request, and builds its docs. It doesn't write observations: README decision 6. It does run `ascribe observe --check`, so a pull request that edits an example without observing again fails there.

When the weekly run fails, GitHub tells the maintainer. That's the upkeep this adds, and `outside.md` records each repository, its scheduled job, and who is told.

### Held to Ascribe's releases

A repository pins a release of `ascribe`. After each release, the step in `RELEASING.md` that tries the release as a user gains one line: build each example repository's docs with it. A change in Ascribe that breaks the pattern is found then, and not by someone copying the repository.

### Where the docs point

The tested examples guide ends with the two repositories, and says to copy one. Getting started mentions them once.

## Tasks

1. The Node and TypeScript repository, with the maintainer creating it under the organization.
2. The Python repository.
3. Each one's CI, and `ascribe observe --check` in it.
4. `outside.md`, and the line in `RELEASING.md`.
5. The guide's last section, and an entry in `changes/`.

## Out of scope

- More languages. Go and Rust are the likely next two, each when someone asks.
- A template repository or a generator. A repository to copy is enough, and has nothing to keep in step with.
- Reading the tests' results. That's the first thing carved out of this plan.

## Acceptance criteria

- Cloning either repository, installing its dependencies, and running `ascribe observe` rewrites the committed observation with no change to `[files]`.
- Each repository's built docs show every example with its output and a tested line naming two versions.
- Editing an example in a pull request fails that repository's CI until it's observed again.
- The Python repository's CI has no Node step.
- `outside.md` lists both, with their scheduled jobs.

## Verify

In each repository:

```sh
ascribe observe --check
ascribe check --deny-warnings --config docs
ascribe build --config docs
```

## Commits

In this repository:

1. "Point the tested examples guide at repositories to copy"
2. "Record the example repositories among what's outside"
