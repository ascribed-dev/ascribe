# Phase 10: The update pull request

Part of [Docs](README.md). Requires phase 9. A workflow, a fixture, and docs.

## Goal

On a schedule, a job moves each remote source to its latest commit and, when that changes anything, opens one pull request in the docs repository. The pull request's description is the drift report: which commits came in, and which pages' examples changed. Merging it is how the docs catch up with the code; reading it is how a team finds out they'd fallen behind.

Nothing here runs in the binary beyond phase 9's `ascribe sources update`. It's a recipe a team copies, and that we run.

## Context

- Phase 9's `ascribe sources update --format summary`.
- Renovate and Dependabot: one pull request that's updated in place beats one per night; grouping and a schedule are how they keep the noise down. Read how each describes its pull requests, and what people ask them to stop doing.
- GitHub Actions, to check: scheduled workflows (and that they stop after 60 days without activity in a repository); what a workflow needs to push a branch and open a pull request (`contents: write`, `pull-requests: write`, and the repository setting that lets Actions create pull requests); that a pull request opened with the default token doesn't start other workflows, and the ways around it (a GitHub App token, a deploy key); reading a private repository from another repository's workflow.
- The review plan's decision that the binary never talks to GitHub: the workflow does, with `gh`.
- `ascribed-dev/review-fixture` and `scripts/review-fixture/setup.ts`: a fixture built from nothing by a script, for work that needs real GitHub.
- [Decisions 11, 15, and 16](README.md#decisions).

## Design

### The recipe

A GitHub Actions workflow, in `guides/drift.md` for users to copy, of a few dozen lines:

1. On a schedule the team chooses (the recipe's default is weekdays, once), and by hand.
2. Check out the docs repository; install Ascribe from npm; give `git` read access to the sources.
3. `ascribe sources update --format summary > update.md`.
4. If no file changed, stop: no branch, no pull request, a green run.
5. Otherwise run `ascribe check`, and keep its result for the description.
6. Commit the lock and the copies to one fixed branch (`ascribe/update-sources`), force-pushed, and open a pull request from it, or update the one that's open.

### The pull request

One at a time, updated in place. Its title names what moved ("Update sources: api to 9f2c41d, cli to 77ab0e3"). Its description, from `update.md`:

- per source, the old and new commits as a link to the comparison on the code's host, and how many commits;
- **Examples that changed**, each page linked, with how much each example changed: the pages to reread;
- **Examples that broke** (a region that's gone, a file that moved), with the `check` error for each: the pages that must be fixed before this can merge;
- a line saying what merging does, and that the next run updates this pull request if it's still open.

Because the copies are in the diff, the docs repository's own checks, the review report, and the site preview all run on the pull request and show the changed pages as readers will see them. That's the point of copying the files: a change to someone else's code arrives as a reviewable change to the docs.

A pull request with broken examples fails `ascribe check` like any other. Nothing new is failed or commented beyond that (decision 11).

### The fixture

Two repositories in the org, built from nothing by a script like the review fixture's (`scripts/sources-fixture/setup.ts`):

- `ascribed-dev/sources-fixture-code`: a small made-up code project with tagged regions;
- `ascribed-dev/sources-fixture-docs`: an Ascribe project that takes snippets from it and from a second source, with the recipe's workflow.

The script can also push a scripted change to the code repository: an example changed, a region renamed, a file moved, nothing relevant changed. The pass below uses those.

### Our own docs

Our docs' code is in this repository, so they have no remote source of their own to follow, and inventing one would be the special-casing decision 1 rules out. The fixture is where this is run for real. If, when this phase starts, the docs do quote something kept elsewhere in the org, use it and say so.

### The pass

On the fixture, by hand, with each scripted change: run the workflow, read the pull request it opens or updates, and check it against the change. Then leave the schedule on for two weeks with a scripted change every few days, and record: how many pull requests were opened and updated, whether each description was right, and how long a run takes. Put the results in `measures.md`.

## Tasks

1. The recipe, working on the fixture.
2. The fixture's script, with `--local` for the parts that need no GitHub.
3. The pass, and its numbers.
4. `guides/drift.md`: "Docs kept apart from the code", with the recipe, the access it needs, and what the pull request means. `measures.md`.

## Out of scope

A GitHub Action that wraps the recipe (Later, with the one for check and review); GitLab and other hosts (the commands work anywhere; only the recipe is GitHub's); a pull request or a comment in the code repository; merging automatically; one pull request per source.

## Acceptance criteria

- A change to an example in the code repository becomes, at the next run, a pull request in the docs repository that names the page.
- A second change before it's merged updates that pull request, and doesn't open another.
- A night with no relevant change opens nothing and passes.
- A removed region gives a pull request whose checks fail, with the page and the reason in its description.
- The recipe in the guide is the workflow the fixture runs, checked by a test that compares them.

## Verify

```sh
node scripts/sources-fixture/setup.ts --local
pnpm typecheck && pnpm lint && pnpm format:check
```

## Commits

1. "Add a script that builds the sources fixture"
2. "Open a pull request when a source moves"
3. "Document docs kept apart from the code"
