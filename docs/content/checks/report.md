---
type: how-to
title: The project report
description: Reading ascribe report and acting on each section, from the problems and pages it counts to the external links and the published site, and running it every day in CI.
---

`ascribe check` answers "is this change fine?", quickly, on every edit and every pull request. `ascribe report` answers "what state is this project in?": it counts what the checks find across the whole project, lists what each build leaves out, and runs what's too slow or too networked for a check, such as following every external link. Every problem in it says what kind of next step it needs, as the editor does.

```sh
ascribe report                       # problems, inventory, and builds
ascribe report pages --not-listed    # the pages kept out of search, and why
ascribe report links                 # external links, with lychee
ascribe report agents --site https://docs.example.com   # the published site, with afdocs
```

Without a section named, it runs the three that say what state the project is in and need nothing outside it. `pages` needs nothing outside it either, and runs when it's named. `links` and `agents` run programs you install, and reach the network: when one can't run, it says what's missing, and the rest of the report is the same. [`ascribe report`](report-command.md#cli-report) lists every option, and the JSON.

This guide takes the report a section at a time and says what to do about what each one finds, then runs it every day in CI.

## Problems

The same problems as `ascribe check`, counted three ways: by check, by file, and by kind of next step. The kind is what tells you how to spend the time:

| Next step | What it means |
|---|---|
| `fix` | Ascribe can make the edit: the editor offers it as a quick fix. |
| `choose` | You pick among things Ascribe can list: the heading a link meant, the new address of a page that moved. |
| `write` | It needs words, or judgment: a description, a split page. |
| `review` | It may be fine as it is. Look, then fix it or acknowledge it. |
| `outside` | Nothing in the source fixes it: a setting somewhere else, or a bug to report. |

It also lists every problem [acknowledged as intended](../writing/directives.md#intended), with its reason, so an acknowledgement that's no longer true is easy to find. `ascribe check --format concise` lists each problem.

### What's new since the last report

From its second run on, the section says which problems are new since the last report, and how many that report listed are gone, and lists the new ones:

```text
Problems
  1 error, 13 warnings, 0 advice.
  14 problems, 2 of them new since 9 October; 5 the last report listed are gone:
    guides/install.md:40: [ASC036] `setup.md` doesn't exist
    guides/upgrade.md:12: [ASC036] `notes.md` doesn't exist
```

A problem is the same as one the last report listed when it has the same check, the same file, and the same text: the source text it's reported at, or its line's when it's reported at no text, with each run of spaces and line ends read as one space. Among problems alike in all three, as two links to one missing page in one paragraph, each is told from the others by its order in the file. So a problem stays the same when lines are added above it or around it, and is new when the text it's about is edited, which is usually what someone should look at: reword the sentence a prose alert is about, and its alert is new, and the old one is gone. Its line number, its message, and the builds it's in aren't part of it.

A first run says nothing about what's new, and neither does a run on other builds than the last one's, named with `--build` in any order, since that finds other problems. `--no-history` says nothing about what's new either, and keeps nothing: the next report compares with the last one before it, so a one-off run doesn't move what a scheduled report compares with. The last report's problems are kept in `.ascribe/history/report.json`, beside what [`links` remembers](external-links.md#what-it-remembers); deleting it costs one report that can't say what changed.

## Inventory

How many pages the project has, by content type and by owner, and three lists from the checks across the project: the pages whose review date has passed, the pages nothing links to, and the fragments, phrases, features, glossary terms, and images nothing uses.

A page's owner is the value of the frontmatter field its type marks `role = "owner"`, such as a team's name:

```toml
[types.guide.frontmatter]
title = "string"
team = { type = "enum(platform, api)?", role = "owner" }
```

Without one, every page is counted as having no owner, and the report leaves that line out.

For each source with an `ascribe.examples.toml`, it counts the snippets that show one of the source's [tested examples](../examples/tested.md), and how many of those are current, stale, and unobserved: each `@snippet` line once, in pages and fragments alike. A source in another repository is counted from the copies [`ascribe sources fetch`](../examples/sources.md#cli-sources-fetch) puts in its folder. [When an example changes](../examples/tested.md#when-an-example-changes) says what to do about a stale one.

```text
  Examples from code: 4 snippets, 2 current, 1 stale, 1 unobserved.
```

For each API in `[apis]`, it counts the spec's operations, how many of them a link names, and how many a checked example claims; [Find the operations no guide links to](../openapi/coverage.md) says what counts.

```text
  APIs: public: 24 operations, 19 linked, 6 with a checked example.
```

## Builds

For each build, the pages it doesn't publish and the content it takes out of the pages it does (a variant arm, a section marked available elsewhere), when another build keeps them, with why. It's what to read before a release of one build: whether what its readers won't see is what you meant to leave out. A project with one build has nothing to compare.

## Pages

Every page each build publishes, sorted by address, with what search engines and agents are told about it: its title, its description, and its listing, and where the listing comes from, the page's own `listing` or a rule of [`[listing]`](../content-model/checks.md#22-listing) and its line in `ascribe.toml`. It's the table a team would otherwise crawl its own site to get, made from the build before anything is deployed. The line above it gives the site's address, which each page's canonical address starts with.

```text
Pages
  site publishes 4 pages on https://docs.example.com, 2 of them not listed:
    ADDRESS                LISTING   FROM                       TITLE    DESCRIPTION
    /docs/                 listed    default                    Home     Where to start.
    /docs/guides/install/  listed    default                    Install  Install the product.
    /docs/internal/notes/  unlisted  folder internal/, line 16  Notes
    /docs/roadmap/         unquoted  page                       Roadmap
```

`--not-listed` lists only the pages that aren't `listed`, which answers "what are we hiding, and why?". A page that's listed and shouldn't be, or the other way round, is changed by its own `listing` or by the rule the table names. What happens to the pages after a deploy, in search engines and in what agents read, is outside what Ascribe sees: [What to measure](../agents/measuring.md) says where to look.

## External links

`ascribe report links` lists each link to another site that's broken or has moved, at its place in the source, and the links it holds back as unconfirmed because they answered on an earlier run. [Check external links](external-links.md) says what to do about each, and what the section remembers from one run to the next.

## The site and agents

`ascribe report agents --site <URL>` runs the delivery spec's checker on a built site and reports each check that fails or warns by who changes what it checks: the hosting, or what Ascribe writes. [What to measure](../agents/measuring.md#check-what-the-site-delivers-to-agents) says what to do about each.

From its second run on the same site, the section says which checks are failing since the last report, and how many that failed then no longer do:

```text
Agents
  The delivery spec's checks on https://docs.example.com: 21 pass, 2 fail.
  23 checks: 21 passing, 2 failing, 1 of them failing since 9 October: redirect-behavior.
```

A check is the same as one the last report had when it's on the same site, by the address given with `--site`, and has the same id. Failing, here, is failing or warning. A check is new when it's failing now and was passing on the last report, or wasn't in it, as a check the checker adds in a later version; one that was skipped or couldn't run then isn't counted as new, since there's nothing to say it was passing. One that failed then and passes now, or is gone, no longer fails. One that failed then and warns now is still failing, so it's neither.

Every check runs every time: nothing the checker said is reused, because a published site changes without a commit. The report only remembers how each check came out, to compare with. A first run says nothing about what's new, and neither does a run on another site than the last one's, such as a preview after the published site, since that's a different site. `--no-history` says nothing about what's new either, and keeps nothing. How each check came out is kept in `.ascribe/history/report.json`, beside the last report's problems; a report that doesn't run `agents` keeps what the file had for it.

## Handing a section to an agent

`--format prompt` writes one prompt about one section's problems, `problems`, `links`, or `agents`, for an agent to work through: each problem with what Ascribe knows about it, such as the sentence around a broken link or the setting to change, how to fix each kind, and the command that checks the work. It's nothing when the section finds nothing.

```sh
ascribe report links --format prompt
```

## On a schedule

External links break and sites change without a commit in your repository, and checking them takes minutes and the network, so run those sections on a schedule, not on every pull request. This GitHub Actions job runs them every day and writes the report to the run's summary. When it finds something, or a section can't run, it opens an issue, or updates the one that's open, and fails. Change `--config docs` to your project's folder, or drop it when that's the repository's root, and `--site` to your site.

@snippet {lang=yaml}: code:.github/workflows/report.yml#job

- **The `ascribe-links` artifact** keeps [what `links` remembers](external-links.md#what-it-remembers) from one run to the next, so a daily run asks about roughly a tenth of the addresses, and a link that fails for a day isn't reported. Each run downloads the newest one kept on its branch before the report, and uploads its own right after it, so a run that finds something, and fails, still keeps it. When there's none, or it can't be listed or downloaded, the report runs with nothing remembered, as on a first run, and every failure is reported; a failed download is a warning on the run, not a failure. The download needs `actions: read`. Its `path` is `.ascribe/cache` and `.ascribe/history` in the project's folder, so change `docs` there, and in `--dir`, too. It isn't kept with `actions/cache`: when a repository's caches pass 10 GB, GitHub evicts those read longest ago, and a small one read once a day goes first, so a busy repository's runs would often start with nothing remembered.
- **`--format markdown`** writes Markdown, for the run's summary and the issue's body.
- **`--exit-code`** fails on any problem; `--exit-code=warning` only on warnings and errors. A section that couldn't run fails it too, with exit code 2: a report that didn't check the links isn't a pass.
- **The issue** is found by its title, so one stays open until someone closes it, and each run replaces its body with the latest report.

This repository runs this job on these docs, in [`.github/workflows/report.yml`]({repo}/blob/main/.github/workflows/report.yml): the job above is taken from that file. It also installs `@ascribed/cli@next`, the nightly build of `main`, since these docs follow `main`.

## See also

- [`ascribe report`](report-command.md#cli-report): every option, and the JSON.
- [Check external links](external-links.md): acting on what `links` finds.
- [What to measure](../agents/measuring.md): what the published site delivers to agents, and what happens to the pages after a deploy, which the report can't see.
