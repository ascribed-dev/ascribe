# Phase 1: Decisions

Part of [Search and answers](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Five choices were made before anything shipped, and are recorded in the [README](README.md#decisions) as decisions 14 to 26. This file keeps the options each was chosen from.

Phase 2 puts each into a contract: a frontmatter key in the language, a table in `ascribe.toml`, a file in the site output, and a component of a published package. None can change afterwards without breaking what was promised.

This phase can't be folded into phase 2. A thread may narrow what it handles. It may not ship a shape the plan expects to change.

## Context

- The [proposal](https://github.com/ascribed-dev/ascribe/blob/097cc3d496fa2f3b08a69958ccacc50c76849126/project-docs/proposals/search-and-answers.md): "Managing how crawlers treat pages", and "The principle".
- `SPEC.md` §2.1: the reserved frontmatter keys, and what each one's value is.
- `docs/content/reference/content-model.md` §15, `[consumer]`: `agents`, and what it publishes.
- `docs/content/contracts/output-layout.md`: `_ascribe/permalinks.json`, the one list a build writes for a site today, with its `version` and its entries as objects.
- `project-docs/checklists.md`, "A new command or option" and "A new editor command, action, or setting": where a kind of choice lives, and how things are named.
- `site/astro.config.mjs`: three projects in one site.

## The decisions

On 11 October 2026 the maintainer asked for each to be answered, and to be asked only where the answer was truly uncertain. None was, so each recommendation is taken as written.

### 1. What the setting is called, and its values

One key, in a page's frontmatter and (from phase 3) in `ascribe.toml`.

| Option | Looks like | For and against |
|---|---|---|
| **`listing`, with `listed`, `unquoted`, `unlisted`** (recommended) | `listing: unlisted` | Says what happens in the reader's terms: whether the page is in the lists a search engine or an agent draws on. Three values in one order, from open to closed, which is what "restrict and never loosen" needs |
| `robots`, with the crawler's own words | `robots: noindex` | Familiar to someone who knows the tag. It names one output of several, and `nosnippet, noindex` are a set, not an order, so "stricter" has no plain meaning |
| `index`, a boolean, and a second key for quoting | `index: false` | The simplest for the common case. Two keys can contradict each other, and a third behavior needs a third key |

What each value means, whichever name is taken:

| Value | Search engines | The sitemap | `llms.txt`, and the pointer to it | Quoting |
|---|---|---|---|---|
| `listed` (when nothing is said) | May list it | In | In | Allowed |
| `unquoted` | May list it | In | In | Asked not to: `nosnippet` |
| `unlisted` | Asked not to: `noindex` | Out | Out | Not applicable |

A page that isn't listed is still built, still has its address, and still has a Markdown version. It isn't offered: nothing Ascribe writes points a crawler at it.

**Decided: `listing`, with `listed`, `unquoted`, and `unlisted`.** README decisions 14 and 15.

### 2. Where a project's words to crawlers live

`robots.txt` needs a home in `ascribe.toml` from phase 2, for a project's own lines, and from phase 4 for its choices by purpose.

| Option | Looks like |
|---|---|
| **A table of its own, `[crawlers]`** (recommended) | `[crawlers]` with `lines = ["Disallow: /drafts/"]`, and in phase 4 `training = "disallow"` beside it |
| Under `[consumer]` | `[consumer] robots = […]` |

The recommendation rests on what `[consumer]` is: how the site output fits one site generator. What a project says to crawlers is about the content, whoever builds the site.

**Decided: `[crawlers]`.** README decision 16.

### 3. The record's file, and its shape

| Option | Means |
|---|---|
| **`_ascribe/pages.json` in the site output, shaped as the permalinks list is** (recommended) | `version`, then `pages`, keyed by route, each entry an object. Phase 2's entries have `route`, `source`, `title`, `description`, `canonical`, `listing`, and `markdown` (the Markdown version's address, when the build publishes one). Later phases add keys and change none |
| In each page's frontmatter | No new file. A sitemap then means reading every page, and a site with another generator has no one list |

`source` is the page's path from the content root, the field [#274](https://github.com/ascribed-dev/ascribe/issues/274) names. The two should be one decision: the record and the collection entry call it the same thing.

`canonical` is the page's own address, absolute, when `[consumer] site` is set, and absent when it isn't. It becomes something else only with versions.

**Decided: `_ascribe/pages.json`, shaped as the permalinks list is.** README decisions 17, 18, and 19.

### 4. One site, several projects

Ascribe's own site holds three projects. A host has one `robots.txt` and, for this plan, one sitemap.

| Option | Means |
|---|---|
| **The integration writes one of each for the site, from every project's records** (recommended) | The sitemap holds every listed page of every entry that has routes. `robots.txt` holds each project's lines once, and the sitemap's address. Two projects that say contradictory things about one purpose (phase 4) are an error that names both |
| Each project writes its own, under its base path | Simple, and wrong: a `robots.txt` under a path does nothing, which is one of the mistakes the proposal sets out to catch |

A project whose pages a site draws itself and gives no route, as the site's `demo` is, has records and is in no sitemap. How an entry says so is the integration's to settle in phase 2, from what the list already knows.

**Decided: one of each for the site, from every project's records.** README decision 20.

### 5. What a layout includes

| Option | Means |
|---|---|
| **A component, `@ascribed/astro/Head.astro`, given the page's entry** (recommended) | `<Head entry={entry} />` writes what the record says. Phase 2: the title, the description, the canonical link, and the robots rule. Phase 6 adds to it. A layout's own tags sit beside it |
| A function that returns the record | The layout writes its own tags from it. Every site writes them, and can leave one out, which is how outputs come to disagree |

The title is the page's own. A site that wants "Page · Site" passes how: a property, named in phase 2 and kept.

**Decided: `Head.astro`, with a `title` property for how a site's titles read.** README decisions 21 and 22.

## Also confirmed here

Four smaller choices, recorded as README decisions 23 to 26: a fragment and a type can't use the key; `listing` governs the head and the sitemap when a build publishes nothing for agents; the sitemap has no dates; and phase 2's check is advice.

## Tasks

1. The maintainer decides each. Done: each recommendation is taken.
2. Record them in this plan's README, as decisions 14 on, with the reason for each. Done: 14 to 26.
3. If decision 1 or 3 isn't the recommendation, bring [phase 2](phase-2-thread.md)'s design in line before it starts. Not needed: phase 2 was written to the recommendations.

## Out of scope

Any code. The purposes' names and values, which phase 4 adds to the table decided here. The levels, which phase 3 adds without changing the key.

## Acceptance criteria

- The README's decisions list holds all five, and the confirmed choices.
- Phase 2's design uses the names decided.

## Commits

1. "Record the search and answers plan's decisions"
