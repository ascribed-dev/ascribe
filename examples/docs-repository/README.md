# A docs repository

The docs for the Lantern SDK, a made-up client library, kept in a repository of their own, as the [drift guide](https://ascribed-dev.com/docs/examples/#docs-kept-apart-from-the-code) describes. The examples come from two sources in the code's repository, `ascribed-dev/sources-fixture-code`, whose quickstart is a tested example: `ascribe sources fetch` copies its `ascribe.examples.toml` and observation with it, and the quickstart page says what it was tested with.

```
ascribe.toml                         the sources: `api` and `examples`, in another repository
content/                             four pages, three with a snippet
.github/workflows/update-sources.yml the update pull request: the guide's recipe
.github/workflows/check.yml          `ascribe check` and `ascribe fmt --check` on each pull request: getting started's recipe
```

The copies (`sources/`) and `ascribe.lock` aren't here: `scripts/sources-fixture/setup.ts` copies this folder into `ascribed-dev/sources-fixture-docs`, or into a local repository with `--local`, and fetches them there. The guide takes the update workflow from this folder, and the script's pass runs it, so the recipe users copy is the one that's tested.
