---
type: how-to
title: The project report
description: Reading ascribe report and acting on each section, from the problems and pages it counts to broken external links and the published site's delivery to agents, and running it every week in CI.
available: ascribe 0.3.0
---

`ascribe check` answers "is this change fine?", quickly, on every edit and every pull request. `ascribe report` answers "what state is this project in?": it counts what the checks find across the whole project, lists what each build leaves out, and runs what's too slow or too networked for a check, such as following every external link. Every problem in it says what kind of next step it needs, as the editor does.

```sh
ascribe report                       # problems, inventory, and builds
ascribe report pages --not-listed    # the pages kept out of search, and why
ascribe report links                 # external links, with lychee
ascribe report agents --site https://docs.example.com
```

Without a section named, it runs the three that say what state the project is in and need nothing outside it. `pages`, the table of every page's record, needs nothing outside it either, and runs when it's named. The other two run programs you install, and reach the network; when one can't run, it says what's missing and how to get it, and the rest of the report is the same. [`ascribe report`](../reference/cli/diff.md#cli-report) lists every option, and the JSON.

This guide takes the report a section at a time and says what to do about what each one finds, then runs it every week in CI.

## Before you start

The sections that run by default, and `pages`, need only `ascribe`. The other two need a program each:

- [External links](#external-links) needs [lychee](https://lychee.cli.rs).
- [The site and agents](#the-site-and-agents) needs `afdocs`, and a built site at an address it can reach.

Each section says how to install its program.

## Problems

The same problems as `ascribe check`, counted three ways: by check, by file, and by kind of next step. The kind is what tells you how to spend the time:

| Next step | What it means |
|---|---|
| `fix` | Ascribe can make the edit: the editor offers it as a quick fix. |
| `choose` | You pick among things Ascribe can list: the heading a link meant, the new address of a page that moved. |
| `write` | It needs words, or judgment: a description, a split page. |
| `review` | It may be fine as it is. Look, then fix it or acknowledge it. |
| `outside` | Nothing in the source fixes it: a setting somewhere else, or a bug to report. |

It also lists every problem [acknowledged as intended](../reference/directives.md#intended), with its reason, so an acknowledgement that's no longer true is easy to find. `ascribe check --format concise` lists each problem.

## Inventory

How many pages the project has, by content type and by owner, and three lists from the checks across the project: the pages whose review date has passed, the pages nothing links to, and the fragments, phrases, features, glossary terms, and images nothing uses.

A page's owner is the value of the frontmatter field its type marks `role = "owner"`, such as a team's name:

```toml
[types.guide.frontmatter]
title = "string"
team = { type = "enum(platform, api)?", role = "owner" }
```

Without one, every page is counted as having no owner, and the report leaves that line out.

## Builds

For each build, the pages it doesn't publish and the content it takes out of the pages it does (a variant arm, a section marked available elsewhere), when another build keeps them, with why. It's what to read before a release of one build: whether what its readers won't see is what you meant to leave out. A project with one build has nothing to compare.

## Pages

Every page each build publishes, sorted by address, with what search engines and agents are told about it: its title, its description, and its listing, and where the listing comes from, the page's own `listing` or a rule of [`[listing]`](../reference/content-model/checks.md#22-listing) and its line in `ascribe.toml`. It's the table a team would otherwise crawl its own site to get, made from the build before anything is deployed. The line above it gives the site's address, which each page's canonical address starts with.

```text
Pages
  site publishes 4 pages on https://docs.example.com, 2 of them not listed:
    ADDRESS                LISTING   FROM                       TITLE    DESCRIPTION
    /docs/                 listed    default                    Home     Where to start.
    /docs/guides/install/  listed    default                    Install  Install the product.
    /docs/internal/notes/  unlisted  folder internal/, line 16  Notes
    /docs/roadmap/         unquoted  page                       Roadmap
```

`--not-listed` lists only the pages that aren't `listed`, which answers "what are we hiding, and why?". With `--format json`, each record has the fields of its entry in the build's `_ascribe/pages.json`, by the same names, for a script that checks them. What happens to the pages after a deploy, in search engines and in what agents read, is outside what Ascribe sees: [What to measure](measuring.md) says where to look.

## External links

`ascribe report links` hands every `http` and `https` address in the sources, once each, to [lychee](https://lychee.cli.rs), and reports each link at its place in the source:

- **Moved** (`link-external-moved`): the address redirects permanently to another one. The fix writes the new address; check first that it's the page you meant, since some sites send every old address to their home page. A site's root that sends you on to one of its own pages, such as its latest version's introduction, hasn't moved, and isn't reported.
- **Broken** (`link-external-broken`): the address answers with an error, doesn't answer in time, or can't be reached. Open it: when the page is gone, link to where its content went, or remove the link.

Some sites turn link checkers away and work in a browser, and some are down for a day. Acknowledge a link like that above its block, with the reason, and the report stops listing it; when the link starts working, the report says the acknowledgement is unused:

```markdown
@intended {check=link-external-broken}: The site answers 403 to link checkers.
See the [vendor's guide](https://vendor.example.com/guide).
```

To leave a whole host out, name it in `ascribe.toml`. A name with `*.` before it covers every host under it:

```toml
[checks.links]
ignore = ["localhost", "*.internal.example.com"]
```

Install lychee from [its releases](https://github.com/lycheeverse/lychee/releases), or with `cargo install lychee`. When it isn't on the path, `[checks.links] command` names where it is, relative to the project root. lychee runs in the project's folder, the one with `ascribe.toml`, and reads its own `lychee.toml` there, for headers, timeouts, or how often to retry. Ascribe turns lychee's own cache off, whatever `lychee.toml` says, because lychee keeps a link's status and not where it moved to, so a cached answer would hide every moved link.

### Answers it reuses

An address that answered with nothing to report isn't asked about again for a while: Ascribe keeps the answer in `.ascribe/cache/links/`, with the day, and reuses it for up to `[checks.links] reuse-days`, 14 days unless you set it. Each address comes due on its own day, between half of that and all of it, so a project's links aren't all asked about again at once. The report says what it did:

```text
External links
  Checked 1786 addresses: asked about 64, and reused 1722 answers, the oldest from 28 September.
```

A link that moved, failed, or didn't answer in time is never kept, so it's asked about on every run, and every problem in a report was found in that run. The one thing reuse changes is when a link that has died since it last answered is reported: when its answer runs out, not before. Changing `lychee.toml` starts every address fresh.

- **`--recheck`** asks about every address, and keeps what this run gets.
- **`reuse-days = 0`** asks about every address on every run, and keeps nothing.
- **Deleting `.ascribe/cache/`** costs the next run time, and nothing else.

### Links that answered before

Many failures aren't dead links: a server answers 503 for an afternoon, or turns the checker away for a day. So a link that answered on an earlier run and fails now isn't reported yet. It's **unconfirmed** until it has kept failing for `[checks.links] confirm-days`, 3 days unless you set it, with no answer in between, and reported from then on. Ascribe remembers, in `.ascribe/history/links.json`, the last day each address answered and the day it started failing.

```text
External links
  Checked 1786 addresses: asked about 64, and reused 1722 answers, the oldest from 28 September.
  3 links to look at, 1 of them new since 9 October:
    guides/install.md:40: [ASC168] https://old.example.com/setup answered 404 Not Found, so the link is broken for readers
    guides/vale.md:13: [ASC169] https://vale.sh/docs/install moved permanently to https://docs.vale.sh/topics/installation; link to the new address
    reference/sdk.md:88: [ASC168] https://cdn.example.net/sdk.js couldn't be reached: Network error: dns error
  2 links not confirmed. Each answered before and fails now, and is reported if it's still failing on its day:
    reference/api.md:12: https://api.example.com/status answered 503 Service Unavailable. Last answered 8 October; reported from 13 October
    guides/deploy.md:31: https://status.example.org didn't answer in time. Last answered 6 October; reported from 13 October
  1 link the last run reported is fine now, or no longer in the pages.
```

- **Holding takes evidence.** A link is held only because the checker got an answer from that address before. A link that has never answered is reported, and so is every failure on a first run, which says so: `Checked 1786 addresses, asking about each. Nothing is remembered from an earlier run, so every failure is reported.`
- **A page that's gone is reported at once.** A 404 or a 410 isn't held; every other failure, a 5xx, a 429, a refusal, a timeout, or a network error, is.
- **An unconfirmed link isn't a problem.** It's listed, with what it answered and both days, but it isn't counted, doesn't fail `--exit-code`, and is left out of `--format prompt`.
- **An acknowledged link stays acknowledged,** confirmed or not, and a link that moved is reported at once: a permanent redirect is an answer, not a failure.
- **What's new** is since the run before: the report says how many of its problems that run didn't report, and how many it reported that are fixed or no longer linked to.
- **`--no-history`** reports every failure, as a first run would, and still remembers what it saw. **`confirm-days = 0`** reports every failure at once, always.
- **Deleting `.ascribe/history/`** costs one noisier report, as a first run's, and never a missed problem.

## The site and agents

`ascribe report agents --site <URL>` runs the [delivery spec](https://agentdocsspec.com/spec/web/)'s checker, `afdocs`, on a built site, the published one or a preview of it, and lists each check as passing, warning, failing, or skipped. Install it with `npm install -g afdocs@0.22.2`.

A check that fails or warns is reported by who changes what it checks, and none of it asks you to edit a page:

- **The hosting** (`delivery-hosting`): status codes, caching, the type a `.md` file is served as, content negotiation, bot protection. The problem names the setting; [what your host does](astro.md#what-your-host-does) says how on the hosts the Astro guide covers. `content-start-position` is here too, since it's the site's page template: it measures where the content starts in the whole page converted to Markdown, so a template that writes the navigation first warns even with the content in `<main>`. Put the navigation after the content in the HTML, and place it with CSS. A check the checker added after the version Ascribe knows is counted here until Ascribe lists it.
- **Ascribe** (`delivery-output`): `llms.txt`, the Markdown pages, and the pointer on each page, which Ascribe writes with `[consumer] agents = true`. Check first that the site publishes what the build wrote; when it does, it's a bug in Ascribe, and the problem says where to report it.
- **The pages**, such as a page too long for an agent to read in one fetch, as Markdown or as HTML: `ascribe check` reports these on the page itself, so the report names the check and doesn't repeat it. The checker's `page-size-html` measures the whole page, navigation included, so a page over its limit that `ascribe check` doesn't report as too long is over it because of what the site's page template adds around the content.

## Handing a section to an agent

`--format prompt` writes one prompt about one section's problems, `problems`, `links`, or `agents`, for an agent to work through: each problem with what Ascribe knows about it, such as the sentence around a broken link or the setting to change, how to fix each kind, and the command that checks the work. It's nothing when the section finds nothing.

```sh
ascribe report links --format prompt
```

## On a schedule

External links break and sites change without a commit in your repository, and checking them takes minutes and the network, so run those sections on a schedule, not on every pull request. This GitHub Actions job runs them every day and writes the report to the run's summary. When it finds something, or a section can't run, it opens an issue, or updates the one that's open, and fails. Change `--config docs` to your project's folder, or drop it when that's the repository's root, and `--site` to your site.

@snippet {lang=yaml}: code:.github/workflows/report.yml#job

- **`actions/cache/restore` and `actions/cache/save`** keep the answers `links` reuses and what it remembers from one run to the next, so a daily run asks about roughly a tenth of the addresses, and a link that fails for a day isn't reported. The save comes right after the report, so a run that finds something, and fails, still saves. A saved cache can't be replaced, so each run saves under its own key and restores the newest by the prefix; GitHub removes a cache nobody has read for seven days. A run that restores nothing asks about every address and reports every failure, as a first run does. Its `path` is `.ascribe/cache` and `.ascribe/history` in the project's folder, so change `docs` there too.
- **`--format markdown`** writes Markdown, for the run's summary and the issue's body.
- **`--exit-code`** fails on any problem; `--exit-code=warning` only on warnings and errors. A section that couldn't run fails it too, with exit code 2: a report that didn't check the links isn't a pass.
- **The issue** is found by its title, so one stays open until someone closes it, and each run replaces its body with the latest report.

This repository runs this job on these docs, in [`.github/workflows/report.yml`]({repo}/blob/main/.github/workflows/report.yml): the job above is taken from that file. It also installs `@ascribed/cli@next`, the nightly build of `main`, since these docs follow `main`.

## See also

- [`ascribe report`](../reference/cli/diff.md#cli-report): every option, and the JSON.
- [What to measure](measuring.md): what happens to the pages after a deploy, which the report can't see.
- [`[checks.links]`](../reference/content-model/checks.md#checkslinks): the settings for the external links.
