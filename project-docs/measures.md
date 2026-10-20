# Measures

Numbers taken by running Ascribe, recorded as each is taken. The earlier ones (time to a working site, the host's build time, examples in the docs, and the update pull request's passes) are in [git history](https://github.com/ascribed-dev/ascribe/blob/7355bb43^/project-docs/docs/measures.md).

## The update pull request

The update workflow (`examples/docs-repository/.github/workflows/update-sources.yml`) on the sources fixture, `ascribed-dev/sources-fixture-docs` and `ascribed-dev/sources-fixture-code`, built by `scripts/sources-fixture/setup.ts`.

### Phase 10: two weeks on a schedule, 2026-10-06 to 2026-10-20

The update workflow ran each weekday at 06:17 UTC, with `@ascribed/cli@next`. The code repository made a scripted change each Monday, Wednesday, and Friday at 05:00 UTC (`change.yml`, in `changes.ts`'s order). Kyle rebuilt both repositories on 10-14 at 18:37 UTC, which closed the open pull request and started the changes' order again.

| Day | Scripted change | Run | Took | Pull request |
|---|---|---|---|---|
| 10-06 | | Passed | 20 s | None |
| 10-07 | A change no page shows | Passed | 21 s | None |
| 10-08 | | Passed | 20 s | None |
| 10-09 | A region renamed | Passed | 27 s | #8 opened. Description right; checks failed on `connect.md`, as they should |
| 10-12 | The code put back | Failed | 14 s | #8 should have closed. The run failed at `ascribe sources update --format summary`: `@next` had renamed the format to `markdown`, and the fixture's copy of the workflow hadn't changed |
| 10-13 | | Failed | 26 s | The same failure |
| 10-14 | A file moved | Failed | 16 s | The same failure. The rebuild that evening closed #8 |
| 10-15 | | Passed | 18 s | None |
| 10-16 | An example changed | Passed | 19 s | #9 opened. Description right: `login.md` to reread; checks passed |
| 10-19 | A change no page shows | Passed | 28 s | #9 updated: "Update sources: api to 1290c4c, examples to 1290c4c". Description right; checks passed |
| 10-20 | | Passed | 24 s | #9 left as it was |

**Every run that got to the pull request did the right thing.** In two weeks, the workflow opened two pull requests and updated one. Each description matched the day's change, and each pull request's checks started without anyone approving them. It closed none: the one close it should have made, on 10-12, was in the failed runs.

- **A run takes 18 to 28 seconds** when it passes, from the job's start to its end, including the tokens and installing Ascribe from npm.
- **GitHub starts scheduled runs late.** Every run started 5 to 9 hours after 06:17 UTC (between 11:27 and 15:29 UTC). A team whose update pull request should be there in the morning can't count on the time in the cron.
- **The three failures were the fixture, not the recipe.** The fixture installs `@ascribed/cli@next`, but its workflow is a copy from when it was built, so a renamed option broke it until it was rebuilt from the current recipe. A docs repository that installs a pinned version wouldn't see this; one that follows `@next` would, and should take the recipe again when `--format` changes.

Both schedules were turned off on 10-20, and #9 closed. Rebuilding the fixture with `setup.ts` turns them on again.
