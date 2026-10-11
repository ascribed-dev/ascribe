# Proposal: a cache for slow outside checks

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail. It comes from "Derived: a cache" in the [brainstorm](../brainstorm.md#15-a-cache-a-search-index-and-notes), and starts where the [report's plan](https://github.com/ascribed-dev/ascribe/blob/76a46b26090c2dd0efbd7c918a29d58754f8eace/project-docs/content-checks/phase-7-report.md) stopped: "Results aren't cached in this phase."

## The goal

`ascribe report links` asks about an external link only when it has to, and reports a link as broken only when it has reason to believe it.

Those are two things, and this proposal keeps them apart:

- **A cache.** An address that answered a few days ago isn't asked about again. Deleting the cache costs time and nothing else.
- **A history.** A link that answered last week and fails today is held as unconfirmed, and reported once it has kept failing. This changes what a report says, on purpose, so it isn't called caching and doesn't live in the cache.

Ascribe's own checking isn't the problem and isn't touched. `ascribe check` takes 0.8 to 2 seconds at 3,000 pages and the language server answers in milliseconds, so nothing here makes checking incremental.

## What exists

- **Ascribe keeps nothing between runs.** `.ascribe/` holds the build's output and the files Vale runs with, both written again whenever they're needed.
- **`ascribe report links` runs lychee,** a program the project installs ([decision 66](../decisions.md)). Ascribe lists every `http` and `https` link with its place in the source, hands lychee each different address once, on standard input, with `--format json --verbose --no-progress -`, and maps the answers back to the lines by their position in that list (`crates/ascribe-check/src/links.rs`). The binary makes no request.
- **An answer is one of five:** fine, moved permanently, broken with a status, timed out, or unreachable. Moved is `link-external-moved`; the last three are `link-external-broken`, which `@intended {check=link-external-broken}` acknowledges at a block. `[checks.links]` takes `command` and `ignore`.
- **lychee reads its own `lychee.toml`** in the project's folder, for headers, timeouts, and retries. Ascribe doesn't read that file.
- **One cache exists already,** outside the project: `ascribe sources` keeps a bare repository per URL in the user's cache folder, and the agents' hook keeps its check server's files there.
- **The scheduled job starts cold.** [`report.yml`](../../.github/workflows/report.yml), which the docs give as the recipe, runs weekly and restores nothing.

## What was measured

All on 10 October 2026, with lychee 0.24.2 and the arguments Ascribe gives it.

| Addresses | Time | Failed |
|---|---|---|
| 14, Ascribe's docs, through `ascribe report links` | 0.8 to 4.5 s | 0 |
| 198, from this repository's research notes | 11.8 s, and 11.6 s later the same day | 10, then 5 |
| 1,786, every link in the research notes | 2 min 8 s | 76, and 2 timeouts |

About 14 addresses a second. Ascribe's share is negligible. A project with 2,400 links has been reported at 10 to 14 minutes ([Three cost classes](../reports/Ascribe%20fit%20for%20docs%20pain%20points.md)).

**Most failures weren't dead links.** Of the 76, 14 said the page is gone (thirteen 404, one 410). Forty-one were 503, three 429, one 522, thirteen turned the checker away (ten 403, two 401, one 405), three were network failures, and one was an address lychee couldn't read.

**The set changes from run to run.** The same 198 addresses failed 10 times in one run and 5 in the next. The five that differed were all `github.com/…/blob/…` pages answering 503.

**A smaller run doesn't clear them.** The 77 addresses that failed in the large run, asked again alone, took 19.8 s, and 72 failed again. 38 of the 41 503s are GitHub `/blob/` pages (the list has 48), and the same 38 failed both times. So the burst isn't the main cause on this evidence: one host turns the checker away most of the time, for hours.

What follows from that:

- **Speed is the cache's.** A run that asks only about the failures takes about 20 seconds where the full one takes 128. It can't go lower while a link fails: lychee retries, and one 429 took 11 seconds by itself.
- **Fewer false reports is the history's,** and only partly. It removes the failure that's gone tomorrow. It doesn't remove a host that refuses the checker for days; `[checks.links] ignore` and `@intended` stay the answer to that.

## Two stores, two promises

| | The link cache | The link history |
|---|---|---|
| Holds | Addresses that answered, and the day | For each address: the last day it answered, and the day it started failing |
| Changes | How long a run takes | Which failures are reported, and what's called new |
| If deleted | The next run asks about everything. Same report | The next run reports every failure, as a first run does |
| Where | `.ascribe/cache/links/` | `.ascribe/history/links.json` |
| Turned off for a run | `--recheck` | `--no-history` |

Neither is authored, and neither is committed. A committed list of findings a team has accepted is a baseline, which the brainstorm puts with the content checks, not here.

## The cache

**It holds only good news.** An entry says an address answered, with nothing to report, on a day. A link that moved, failed, or timed out is never an entry, so it's asked about on every run. That gives the rule a reader can rely on: **every finding in a report was asked about in that run.** Reuse can't show a stale finding or keep a fixed one alive.

The one way reuse changes a report is by delay. An address that died since it last answered is reported when its entry runs out, not before. That's the price of the feature, it's bounded by a number the project sets, and the report says how old its oldest answer is.

**An entry is a file,** named by a hash of its key, written by one run:

```json
{ "version": 1, "url": "https://docs.vale.sh/topics/installation", "asked": "sha256:9b1c…", "day": "2026-10-10" }
```

```
.ascribe/cache/links/4f/4f0c9a1b72d3e6a8.json
```

**The key** is what decides the answer, as far as Ascribe can know it:

- **The address exactly as it's handed to lychee,** fragment included. Two links to two headings of one page are two lines of lychee's input today, and two entries.
- **A hash of the project's `lychee.toml`,** or of nothing when there isn't one. A changed header or accepted status starts every address fresh. Ascribe hashes the file and doesn't interpret it.
- **Not the redirect chain.** A chain is part of an answer, and an answer with a permanent redirect isn't kept.
- **Not lychee's version.** An entry records what a server said. A new lychee asking again about everything at once is the burst this is meant to avoid.

**How long an answer is reused:**

```toml
[checks.links]
reuse-days = 14     # 0 asks about every address on every run
```

Each address's own limit falls between half of that and all of it, fixed by its hash. Addresses first checked together then don't all come due on one day, and with a daily run about a tenth of them are asked about each day.

**Days, not seconds.** The libraries don't read the clock. The binary hands them the day, as it does for `review-overdue`, so `ASCRIBE_TODAY` makes a test of expiry the same on any date.

**Forcing it.** `ascribe report links --recheck` asks about every address and replaces the entries. Deleting `.ascribe/cache/` does the same.

**Cleanup needs no command.** A run deletes each entry too old to be reused. So the folder never holds more than the addresses seen in the last `reuse-days`, and an address the pages dropped is gone within that time. There's no cap to set.

**Hosts in `[checks.links] ignore`** are never asked about and never entries.

### Several writers, and a crash

Only `ascribe report` writes link entries. The language server, the MCP server, and the hook's check server never run it. But two reports can run at once, and the later uses below add the editor and the hook as writers, so the store is built for that from the start:

- **One file per entry, written whole or not at all:** to a `tmp-` file beside it, then renamed into place, as the Vale files under `.ascribe/vale/` and the hook's files already are. There's no lock and no index file.
- **Two writers of one entry both wrote a true answer.** The last one stays.
- **A reader trusts nothing.** A file that doesn't parse, has another version, or holds a different address than was asked for is treated as absent.
- **A write that fails is skipped.** On Windows a rename can be refused while another process has the file open. The entry isn't written, the run goes on, and the next run asks again.
- **A crash leaves a `tmp-` file,** which cleanup removes once it's a day old.
- **Names are lowercase hex,** so case and path length don't differ by platform.

These files aren't the project's sources, so they're read outside `ascribe_resolve::FileSystem`, with the comment that says why. `lychee.toml` is a project file, and its read goes through `FileSystem`.

## The history

The rule, whole:

> A link that fails is reported, unless the history shows it answering before. Then it's **unconfirmed** until it has kept failing for `confirm-days`, with no answer in between.

```toml
[checks.links]
confirm-days = 3    # 0 reports every failure at once
```

| The history says | This run | The report |
|---|---|---|
| Nothing about the address | It fails | Broken, as today |
| It answered on 8 October | It fails, for the first time, on 10 October | Unconfirmed, until 13 October |
| It answered on 8 October, failing since 10 October | It fails, on 13 October or later | Broken |
| Failing since 10 October | It answers | Fine. The streak is forgotten |

- **A first run reports everything.** With no history there's no evidence, and nothing is held. That's what `ascribe report links` does today, and what CI does when nothing was restored. Nothing needs the history to be correct. It only needs it to be quieter.
- **Holding takes evidence.** A link is held only because this project's checker got an answer from that address before. A link that has never answered is never given the benefit of the doubt.
- **An unconfirmed link is shown, not hidden.** It has its own list, with its status and both dates. It isn't a finding: it's not counted, it doesn't make `--exit-code` fail, and it's left out of `--format prompt`, since an agent shouldn't fix a link that's probably fine.
- **An acknowledged link stays acknowledged.** `@intended {check=link-external-broken}` covers a failing link whether it's confirmed or not.
- **Moved links aren't held.** A permanent redirect is an answer, not a failure.
- **`--no-history`** reports as a first run would, and still records what it saw.

The history is one file, sorted, written whole through a rename after each run:

```json
{
  "version": 1,
  "run": "2026-10-10",
  "addresses": {
    "https://api.example.com/status": { "answered": "2026-10-08", "failing_since": "2026-10-10" },
    "https://old.example.com/setup": { "failing_since": "2026-10-03", "reported": true },
    "https://vale.sh/docs/install": { "answered": "2026-10-10", "reported": true }
  }
}
```

It forgets an address the pages no longer link to. When two runs finish together, one run's observations are lost, which can make a link's confirmation later and never earlier.

**It lives in a folder people delete.** The docs say `.ascribe/` is written again whenever it's needed, and that stops being quite true. Deleting the history costs one noisier report and never a missed finding, which is why it can live there. The reference says so plainly.

**It also answers "what's new".** The history knows what the last run reported, so the report can say which findings are new since then and how many went away. This is the brainstorm's "3 dead links since the last report", for links, and the report's plan left it out for want of exactly this file.

## What the report says

Today: `Checked 14 addresses.` With both stores:

```text
External links
  Checked 1786 addresses: asked about 64, and reused 1722 answers, the oldest from 28 September.
  3 links to look at, 1 of them new since 9 October:
    guides/install.md:40: [ASC168] `https://old.example.com/setup` answered 404 Not Found, so the link is broken for readers
    guides/vale.md:13: [ASC169] `https://vale.sh/docs/install` moved permanently to `https://docs.vale.sh/topics/installation`; link to the new address
    reference/sdk.md:88: [ASC168] `https://cdn.example.net/sdk.js` couldn't be reached: Network error: dns error
  2 links not confirmed. Each answered before and fails now, and is reported if it's still failing on its day:
    reference/api.md:12: `https://api.example.com/status` answered 503 Service Unavailable. Last answered 8 October; reported from 13 October
    guides/deploy.md:31: `https://status.example.org` didn't answer in time. Last answered 6 October; reported from 13 October
  1 link the last run reported is fine now, or no longer in the pages.
```

A first run says so: `Checked 1786 addresses, asking about each. Nothing is remembered from an earlier run, so every failure is reported.`

`--format summary` says the same in Markdown. The JSON's `links` gains fields and changes none, and stays at `schema_version` 1:

| Field | What it is |
|---|---|
| `asked` | How many addresses the checker was asked about in this run |
| `reused` | How many were answered from the cache. `checked` stays their sum |
| `oldest_answer` | The day of the oldest answer reused, or `null` |
| `previous_run` | The day of the run the history is from, or `null` on a first run |
| `new` | How many of `diagnostics` the previous run didn't report, or `null` |
| `resolved` | How many the previous run reported that this one doesn't, or `null` |
| `unconfirmed` | A capped list, as the others are: each link's file, line, address, status, `last_answered`, `failing_since`, and `reported_from` |

`diagnostics` is untouched: each entry is `check`'s diagnostic, and none of them is from a reused answer.

## Why not lychee's own cache

lychee has one: `--cache` writes `.lycheecache` in the folder it runs in, and `--max-cache-age` (one day by default) expires it. Tried on the 198 addresses:

| Run | Time | Asked | Permanent redirects reported |
|---|---|---|---|
| First, `--cache` | 11.6 s | 198 | 9 |
| Second | 11.0 s | 5 | 0 |
| Third | 11.2 s | 5 | 0 |

- **It forgets redirects.** Its file is `address,status,time`. A reused answer comes back as `OK (cached)` with no redirect chain, so every `link-external-moved` finding disappears on the second run. That's a cache changing a result.
- **It was no faster here.** It reused the 193 that answered and asked again about the 5 that failed, and one 429 was retried for 11 seconds.
- **It can't hold anything.** It keeps no failures and no last good answer, so the history would still be Ascribe's to keep.
- **It can't say what it reused** in terms of the project's lines.
- **Its file lands in the project's folder,** outside `.ascribe/`, where nothing ignores it.

Ascribe already hands lychee a list of addresses. Handing it a shorter list needs no option of lychee's and no knowledge of its file, and works the same with the stand-in checker the tests use.

One thing does change in what lychee is given: `--cache=false`. A project whose `lychee.toml` says `cache = true` gets lychee's cache under Ascribe today, and loses its moved links on the second run. The flag overrides the file; both were checked.

## In the project, not beside the sources' cache

The brainstorm said `.ascribe/`. The precedent says the user's cache folder. They differ for reasons:

| | Sources' repositories | Link cache and history |
|---|---|---|
| Keyed by | A repository's URL, the same for every project that names it | An address and this project's `lychee.toml` |
| Size | A git repository each | A line of JSON each |
| Locking and cleanup | git's own locks; never cleaned up | A rename per file; cleaned by every run |
| CI needs to keep it | No | Yes |

A path inside the checkout is what a platform's cache action saves most simply, `rm -rf .ascribe` resets everything Ascribe knows about a project, and the folder is already ignored. The cost is that a second clone or worktree of a project starts cold.

## In CI

A job gets nothing from either store unless it saves and restores them. With GitHub Actions, in the recipe the docs give:

```yaml
- uses: actions/cache@v4   # pinned to a commit, as the recipe's other actions are
  with:
    path: |
      docs/.ascribe/cache/links
      docs/.ascribe/history
    key: ascribe-links-${{ github.run_id }}
    restore-keys: ascribe-links-
```

- **The key is new on every run.** A saved cache can't be overwritten, so each run saves under its own id and restores the newest one by the prefix.
- **Nothing in the key names the content.** The entries expire by themselves, and a changed `lychee.toml` is in each entry's key.
- **Weekly is too slow for it.** GitHub removes a cache nobody has read for seven days, which a weekly job only just misses or doesn't. And three days of confirming takes two runs a week apart. With the cache, a daily run asks about a tenth of the addresses, so the recipe's schedule can become daily.
- **A job that restores nothing is a first run:** everything asked about, everything reported. Slow and noisy, as now, and not wrong.

## What never depends on it

- **`check`, `build`, `diff`, `drift`, `fmt`, the language server, and the hook never read or write either store.** Their outputs stay a function of the sources, and the determinism test and `scripts/compare/outputs.ts` stay as they are. Neither runs `report` today. A test holds the line the way `file_reads.rs` holds file reads: only the report's code opens `.ascribe/cache/` or `.ascribe/history/`.
- **`report`'s three local sections don't either.**
- **`report links` was never the same twice,** as the 10 and the 5 show. What it adds is two inputs that are files, so its tests are exact: a stand-in checker, a day from `ASCRIBE_TODAY`, and a store in a temporary folder give the same bytes every time.
- **A run's time depends on the cache. A run's findings depend on the history.** Nothing else does, and each has a flag that takes it out.

## Later uses

The store is general: a folder per kind under `.ascribe/cache/`, an entry per key. What goes in it later is a separate decision each time.

- **Vale's alerts, by the hash of a page's prose.** This is the other kind of entry: everything that decides the answer can be hashed (the prose, the configuration and vocabulary Ascribe writes, Vale's version), so it never expires and needs no history. It's also where several writers are real, since the editor runs Vale on save and the hook runs it with `in-check`. It waits on a measurement: nobody has timed Vale on a large project, and a project's own styles folder is an input that's awkward to hash.
- **The delivery spec's results.** A poor fit for a cache. `afdocs` checks a published site, which changes without a commit, so a page's hash doesn't say whether its answer still holds. "This check passed last week and fails now" would use the history, with nothing held.
- **New and gone findings for `problems`.** The same history folder, with `check`'s findings in it.
- **The search index** is derived too, and would sit under `.ascribe/` by the same rule, but it's an index, not this store. **Notes** are authored and never belong in either.

## What this leaves out

- Incremental checking, or anything that speeds up `check`, `build`, or the editor.
- A database, an index file, or a daemon. Entries are files.
- An HTTP client. Ascribe still asks nothing itself.
- Sharing a cache between machines, beyond what a CI platform's cache does.
- Trends over many runs. The history is the last state of each address, not a log.
- Judging failures by kind of host: "38 of these are one site answering 503" would be useful in the report, and is its own change.
- A command to clear or inspect the stores. A flag and a folder cover it.
- A baseline of accepted findings.

## Open questions

1. **Is a 404 held like a 503?** The proposal holds every kind of failure alike, which keeps the rule to one sentence. It means a page that really is gone, on a site that answered last week, is reported three days later instead of today. One of the thirteen 404s measured did pass when asked again. Should 404 and 410 be reported at once?
2. **Are 14 days of reuse and 3 days of confirming the right defaults,** and should either be a setting at first, or fixed until someone needs to change it?
3. **Should a run ask twice?** A first run has no history, so nothing quiets it. A second pass over that run's failures, before reporting, is the only thing that could. Measured, it cleared 5 of 77 and cost 20 seconds. Is that worth having, with the history or instead of part of it?
4. **Does the recipe, and Ascribe's own `report.yml`, move from weekly to daily?** The history works poorly at a week's spacing, and GitHub's cache may not survive it.

## The thread

This has a seam worth threading. The code path is short, a filter between listing the links and running the checker, in a pipeline that already works. What Ascribe has never done is read back its own state from an earlier run, and the place that matters most, the scheduled job, only works if a platform's cache restores it.

- **The use case:** the weekly job on Ascribe's own docs runs twice, and the second run's summary says it asked about 1 address and reused 13.
- **The layers:** `[checks.links]` in the content model, the store and the filter in `ascribe-check`, the section in `ascribe-query`, the text, summary, JSON, and the day in `ascribe-cli`, the workflow, and the docs.
- **The seam:** entries written by one process on one machine and trusted by another, through `actions/cache`, with the day decided by the binary. If it fails, the store's place or its key is wrong, and that's cheaper to learn with one field in the JSON than with seven.
- **Left out of it:** the history, `unconfirmed`, and what's new, which widen it next and are where the judgment is.
