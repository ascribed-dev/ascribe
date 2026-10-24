---
type: reference
title: "`ascribe report`"
description: "Report on a project's health, section by section: options, output, and exit codes."
---

The command that reports what state a project is in. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## `ascribe report`
@id {permalink=true}: cli-report
@summary: Reports what state a project is in, section by section: what's too slow, needs the network, or is too broad for `ascribe check`, each problem with its kind of next step.

Reports what state a project is in, section by section: what's too slow, needs the network, or is too broad for `ascribe check`. Every problem in it carries its kind of next step, as in the editor. It's for a scheduled job, or for a look at the whole project; [Reading the report](report.md) says how to act on each section.

@include: ../_generated/cli-report-options.md

| Section | What it reports | Needs |
|---|---|---|
| `problems` | What `ascribe check` finds, counted by check, by file, and by kind of next step; and every problem acknowledged as intended, with its reason | Nothing |
| `inventory` | The pages by content type and by owner; the overdue reviews, the pages nothing links to, and the entries nothing uses | Nothing |
| `builds` | For each build, the pages and the content it leaves out that another build keeps, and why | Nothing |
| `pages` | Every page's record, by build and sorted by address: its title, description, canonical address, and listing, and where the listing comes from | Nothing |
| `links` | The external links that fail, time out, or moved permanently, each at its place in the source | [lychee](https://lychee.cli.rs), installed, and the network |
| `agents` | The [delivery spec](https://agentdocsspec.com/spec/web/)'s checks on a built site | `afdocs`, installed (`npm install -g afdocs@0.22.2`); the network; `--site` |

With no section named, it runs the first three; `pages` lists every page rather than saying what state the project is in, so it runs when it's named. Each section is independent: one that can't run, because its program isn't installed or `--site` is missing, says why and what would let it, and the others still run. `ascribe report` makes no request itself: `links` and `agents` run the programs the project installs, which do.

**`problems`** says, after its counts, how many of its problems are new since the last report and how many that report listed are gone, and lists the new ones; [What's new](report.md#whats-new-since-the-last-report) says how a problem is told from another. A first run and a run on other builds than the last one's say nothing about what's new. The last report's findings are kept in `.ascribe/history/report.json`; `--no-history` neither reads nor writes it.

**`pages`** lists, for each build, the record of every page it publishes: what every reader that isn't a person is told about the page, as the site output's [`_ascribe/pages.json`](../contracts/output-layout.md#13-the-page-records) holds it, read from the build's resolved pages, so nothing is built and nothing is fetched. As text, a table of each page's address, its listing, where that comes from (`default` when nothing sets it, `page` for its own `listing`, or a rule of [`[listing]`](../content-model/checks.md#22-listing) and its line in `ascribe.toml`), its title, and its description, cut at `--limit`, with what's left out counted; the line above it gives the site's address, how many pages the build publishes, and how many of them aren't `listed`. `--not-listed` lists only those, `unquoted` and `unlisted`: what a site keeps out of search, and why. As JSON, each build's `records`, sorted by `route`, each with the fields of its entry in `_ascribe/pages.json`, by the same names, and `route`, the key it's under there.

**`links`** lists every `http` and `https` link of the project's sources, hands the addresses to lychee once each, and reports each link it says is broken (`link-external-broken`, to review) or moved permanently with a `301` or `308` (`link-external-moved`, with a fix that writes the new address) at the link. A site's root that redirects to another of its own pages, a landing page, hasn't moved. An address that answered within `[checks.links] reuse-days`, 14 by default, with nothing to report, isn't asked about again; the section says how many it asked about and how many answers it reused, and how old the oldest is. A link that moved or failed is asked about on every run, so every problem was found in this run. An address that answers with a server error (`5xx`) is asked about again after 30 seconds, and only that answer counts; one that fails again is reported as the server's failure, which may be temporary, not as broken for readers. `--recheck` asks about every address. A link that answered on an earlier run and fails now, other than with a 404 or a 410, is listed as unconfirmed, not reported, until it has kept failing for `[checks.links] confirm-days`, 3 by default; it isn't counted, doesn't fail `--exit-code`, and is left out of `--format prompt`. The section says how many of its problems are new since the last run, and how many that run reported are gone. `--no-history` reports every failure, as a first run does. `[checks.links]` in `ascribe.toml` names the command, and the hosts to leave out; a link that's fine as it is, on a site that turns checkers away, is [acknowledged](../writing/directives.md#intended) with `@intended` above its block.

**`agents`** runs the delivery spec's checker on `--site` and reports each check that fails or warns by who changes what it checks: the hosting (`delivery-hosting`, with the setting to change), or what Ascribe writes for agents (`delivery-output`, a bug to report). A check about the pages, such as their size, is `ascribe check`'s to report, so it's named and not repeated. Nothing here asks you to edit a page. From its second run on the same site, it says how many checks are failing since the last report and names them, and how many that failed then no longer do; [The site and agents](report.md#the-site-and-agents) says how a check is told from another. It's kept in `.ascribe/history/report.json` too, and `--no-history` neither reads nor writes it.

### The summary and the prompt

`--format markdown` writes the same sections as Markdown, for a CI job's summary or an issue's body. `--format prompt` writes one prompt for an agent about the problems of one section, `problems`, `links`, or `agents`, with what each kind carries (the sentence around a link, the setting to change) and the command that checks the work; nothing when the section finds nothing. Each list holds at most `--limit` items; what's left out is counted, with the command that lists it.

### Exit codes
@id: report-exit-codes

| Code | Meaning |
|---|---|
| `0` | It ran, whatever it found |
| `1` | With `--exit-code`: a section found something at the level given, or above (`advice` when none is: any problem) |
| `2` | With `--exit-code`: a section asked for couldn't run. Without it, too: a usage error, an unknown build, or a project that doesn't load. The reason goes to standard error. |

### Report JSON

`--format json` writes one JSON document to standard output, with the [JSON output](check.md#json-output)'s rules: `schema_version` changes only when a field is removed or changes meaning, so **ignore fields you don't know**. Its schema is in the [JSON report contract](../contracts/json-reports/report.md#ascribe-report).

| Field | Type | Meaning |
|---|---|---|
| `schema_version` | number | `1` |
| `ascribe_version` | string | The version of `ascribe` that wrote the report |
| `error` | string or null | Why the report couldn't be made (exit code `2`); no section is there then |
| `sections` | array of strings | The sections asked for, in the report's order |
| `builds_checked` | array of strings | The builds `problems` checked and `builds` and `pages` report on |
| `not_run` | array of strings | The sections asked for that couldn't run; each says why in its own `not_run` |
| `summary` | object | How many `errors`, `warnings`, and `advice` the sections found, and how many of each kind of next step (`by_next`) |
| `next_command` | string or null | The command that lists everything a list leaves out |
| `problems`, `inventory`, `builds`, `pages`, `links`, `agents` | | Each section asked for |

A problem is a diagnostic as `ascribe check` writes it in [its JSON](check.md#json-output), with `next`, `help`, and `docs`. Each list is an object: `items`, at most `--limit` of them, `shown`, `total`, and `truncated`. A section that runs a program has `not_run`, `null` when it ran, or its `reason` and `how` to get it to run.
