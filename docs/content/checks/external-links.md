---
type: how-to
title: Check external links
description: "Find the links to other sites that broke or moved, with `ascribe report links` and lychee, and fix, acknowledge, or leave out each one."
---

`ascribe check` never reaches the network, so it doesn't see a link to another site break or move. `ascribe report links` does: it hands every `http` and `https` address in your sources, once each, to [lychee](https://lychee.cli.rs), and reports each link that fails at its place in the source.

## Before you start

Install lychee from [its releases](https://github.com/lycheeverse/lychee/releases), or with `cargo install lychee`, and check that `lychee --version` works in the shell you run `ascribe` from. When it isn't on your `PATH`, set `command` in [`[checks.links]`](../content-model/checks.md#checkslinks) to where it is, from the project root.

## Check the links

@steps
1. Run the section:

   ```sh
   ascribe report links
   ```

   lychee runs in the project's folder, the one with `ascribe.toml`, and reads its own `lychee.toml` there, for headers, timeouts, or how often to retry. Ascribe turns lychee's own cache off, whatever `lychee.toml` says, and keeps its own: [What it remembers](#what-it-remembers).

2. Fix each link reported as **moved** (`link-external-moved`). Its address redirects permanently to another one, and the fix writes the new address. Check first that it's the page you meant, since some sites send every old address to their home page.

3. Fix each link reported as **broken** (`link-external-broken`): it answers with an error, doesn't answer in time, or can't be reached. Open it. When the page is gone, link to where its content went, or remove the link. A link whose server failed is reported differently: [A server's error](#a-servers-error).

4. Acknowledge a link that's fine as it is, such as one on a site that turns link checkers away and works in a browser. Write `@intended` above its block, with the reason, and the report stops listing it:

   ```markdown
   @intended {check=link-external-broken}: The site answers 403 to link checkers.
   See the [vendor's guide](https://vendor.example.com/guide).
   ```

   When the link starts working, the report says the acknowledgement is unused.

5. Leave out a host whose links you never want checked, such as a local server. A name with `*.` before it covers every host under it:

   ```toml
   [checks.links]
   ignore = ["localhost", "*.internal.example.com"]
   ```

6. Run `ascribe report links` again, and check that what's left is what you meant to leave.

To run it every day in CI, see [On a schedule](report.md#on-a-schedule). To hand the problems to an agent, `ascribe report links --format prompt` writes them as one prompt.

## A server's error

A link that answers with a server error, a `5xx` status such as `500 Internal Server Error` or `503 Service Unavailable`, isn't reported on the first answer. The server failed, and that often passes in a minute, so Ascribe asks lychee about the address again after 30 seconds, on every run, whatever it remembers. Only the second answer counts:

- **It answers:** the link isn't reported.
- **It fails again:** it's reported, so it isn't lost, but as the server's failure, not a page that's gone. The message says the server failed, which may be temporary:

  ```text
  agents/measuring.md:33: [ASC168] https://www.bing.com/webmasters answered 500 Internal Server Error when asked twice: the server failed, which may be temporary
  ```

  When asking again couldn't run, the first answer stands, and the message leaves out "when asked twice".

  When the link answered on an earlier run, it's [held as unconfirmed](#links-that-answered-before) instead, until it has kept failing for a few days.

A `404 Not Found` or a `410 Gone` says the page isn't there, so it's asked about once, and reported at once, as broken for readers. Any retries lychee makes itself, as `lychee.toml` sets them, come first, seconds apart; Ascribe's comes after them. Set `ASCRIBE_LINKS_RETRY_WAIT` to a number of seconds to wait another time, such as `0` in a test.

## What it remembers

Two folders in the project's `.ascribe/`, which belongs in `.gitignore`, keep what one run learns for the next: the answers it can reuse, and which links answered before. Deleting either never costs a missed problem.

### Answers it reuses

An address that answered with nothing to report isn't asked about again for a while: Ascribe keeps the answer in `.ascribe/cache/links/`, with the day, and reuses it for up to `[checks.links] reuse-days`, 14 days unless you set it. A link that moved, failed, or didn't answer in time is never kept, so every problem in a report was found in that run. The one thing reuse changes is when a link that has died since it last answered is reported: when its answer runs out, not before. Changing `lychee.toml` starts every address fresh.

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

- **Holding takes evidence.** A link is held only because the checker got an answer from that address before. A link that has never answered is reported, and so is every failure on a first run, which says so.
- **A page that's gone is reported at once.** A 404 or a 410 isn't held, and neither is a link that moved: a permanent redirect is an answer, not a failure.
- **An unconfirmed link isn't a problem.** It's listed, with what it answered and both days, but it isn't counted, doesn't fail `--exit-code`, and is left out of `--format prompt`. An acknowledgement covers it as it covers a reported link.
- **`--no-history`** reports every failure, as a first run would, and still remembers what it saw. **`confirm-days = 0`** reports every failure at once, always.
- **Deleting `.ascribe/history/`** costs one noisier report, as a first run's.

## See also

- [The project report](report.md): the rest of what `ascribe report` says about a project.
- [`ascribe report`](report-command.md#cli-report): every option, and the JSON.
- [`[checks.links]`](../content-model/checks.md#checkslinks): every setting.
- [Files Ascribe writes](../command/project-files.md): the rest of `.ascribe/`.
