---
title: The project report
description: Reading ascribe report and acting on each section, from the problems and pages it counts to broken external links and the published site's delivery to agents, and running it every week in CI.
available: next
---

`ascribe check` answers "is this change fine?", quickly, on every edit and every pull request. `ascribe report` answers "what state is this project in?": it counts what the checks find across the whole project, lists what each build leaves out, and runs what's too slow or too networked for a check, such as following every external link. Every finding in it says what kind of next step it needs, as the editor does.

```sh
ascribe report                       # problems, inventory, and builds
ascribe report links                 # external links, with lychee
ascribe report agents --site https://docs.example.com
```

Without a section named, it runs the three that need nothing outside the project. The other two run programs you install, and reach the network; when one can't run, it says what's missing and how to get it, and the rest of the report is the same. [`ascribe report`](../reference/cli.md#ascribe-report) lists every option, and the JSON.

## Problems

The same findings as `ascribe check`, counted three ways: by check, by file, and by kind of next step. The kind is what tells you how to spend the time:

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

Install lychee from [its releases](https://github.com/lycheeverse/lychee/releases), or with `cargo install lychee`. When it isn't on the path, `[checks.links] command` names where it is, relative to the project root. lychee runs in the project's folder, the one with `ascribe.toml`, and reads its own `lychee.toml` there, for headers, timeouts, or how often to retry. Results aren't kept between runs.

## The site and agents

`ascribe report agents --site <URL>` runs the [delivery spec](https://agentdocsspec.com/spec/web/)'s checker, `afdocs`, on a built site, the published one or a preview of it, and lists each check as passing, warning, failing, or skipped. Install it with `npm install -g afdocs@0.22.2`.

A check that fails or warns is reported by who changes what it checks, and none of it asks you to edit a page:

- **The hosting** (`delivery-hosting`): status codes, caching, the type a `.md` file is served as, content negotiation, bot protection. The finding names the setting; [what your host does](astro.md#what-your-host-does) says how on the hosts the Astro guide covers. A check the checker added after the version Ascribe knows is counted here until Ascribe lists it.
- **Ascribe** (`delivery-output`): `llms.txt`, the Markdown pages, and the pointer on each page, which Ascribe writes with `[consumer] agents = true`. Check first that the site publishes what the build wrote; when it does, it's a bug in Ascribe, and the finding says where to report it.
- **The pages**, such as a page too long for an agent to read in one fetch, as Markdown or as HTML: `ascribe check` reports these on the page itself, so the report names the check and doesn't repeat it. The checker's `page-size-html` measures the whole page, navigation included, so a page over its limit that `ascribe check` doesn't report as too long is over it because of what the site's page template adds around the content.

## Handing a section to an agent

`--format prompt` writes one prompt about one section's findings, `problems`, `links`, or `agents`, for an agent to work through: each finding with what Ascribe knows about it, such as the sentence around a broken link or the setting to change, how to fix each kind, and the command that checks the work. It's nothing when the section finds nothing.

```sh
ascribe report links --format prompt
```

## On a schedule

External links break and sites change without a commit in your repository, and checking them takes minutes and the network, so run those sections on a schedule, not on every pull request. This GitHub Actions job runs them every Monday and writes the report to the run's summary. When it finds something, or a section can't run, it opens an issue, or updates the one that's open, and fails. Change `--config docs` to your project's folder, or drop it when that's the repository's root, and `--site` to your site.

@snippet {lang=yaml}: code:.github/workflows/report.yml#job

- **`--format summary`** writes Markdown, for the run's summary and the issue's body.
- **`--exit-code`** fails on any finding; `--exit-code=warning` only on warnings and errors. A section that couldn't run fails it too, with exit code 2: a report that didn't check the links isn't a pass.
- **The issue** is found by its title, so one stays open until someone closes it, and each run replaces its body with the latest report.

This repository runs this job on these docs, in [`.github/workflows/report.yml`]({repo}/blob/main/.github/workflows/report.yml): the job above is taken from that file. It also installs `@ascribed/cli@next`, the nightly build of `main`, since these docs follow `main`.
