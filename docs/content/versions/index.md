---
type: how-to
title: Versions
description: Publishing the docs of more than one version of a product from one source, a site for each version a reader can be on.
available: next
---

A product a team supports in more than one version needs docs for each: a reader on 3.4 should read about 3.4, and find the same page for 3.5. Ascribe does this from one source, the marks the pages already have, in four levels a project opts into one at a time:

1. **No versions.** The default. Nothing to configure, and nothing changes.
2. **One site that knows its versions.** `[releases]` in `ascribe.toml` lists a target's lines: the live ones, the planned ones, when a line's support ends, and the ones that have left. One site still, with checks that the marks name versions the list knows, and "planned" worked out instead of kept by hand.
3. **A window of live versions.** `versions = "<target>"` on a build gives one output for each live line, which this page covers, with links from one line to another, and an older line that says what it is and where the current one is, to a reader, a search engine, and an agent.
4. **An archive.** A line that's left the window, kept as a frozen copy. Not yet.

## List the live lines

A **line** is a version as you write it, and holds every version that starts with it: `3.4` holds `3.4.0`, `3.4.2`, and `3.4.2.1`. List the lines a reader can be on under the target they're versions of:

```toml
[releases.self-managed]
live = ["3.4", "3.5"]
```

The order doesn't matter: the newest is the **current line**. Write lines as finely as you release: `["3.4", "3.5"]` for a line for each minor release, `["0.2.0", "0.3.0"]` for one for each patch release. Every line of a target has the same number of parts. The [`[releases]` reference](../content-model/content.md#71-releasestarget) has the rules.

## Say what's planned, ending, and gone

The rest of the list says what a reader can't be on yet, and what they can't be on any more:

```toml
[releases]
planned-label = "planned"

[releases.self-managed]
live = ["3.4", "3.5"]
planned = ["3.6"]

[releases.self-managed.ends]
"3.4" = 2027-01-31

[releases.self-managed.archived]
"3.2" = { url = "https://docs.example.com/3.2/", ended = 2026-03-31 }
"3.3" = {}
```

- **`planned`** lines haven't shipped. Content marked for a version in one reads "planned" in every output, `Self-managed (planned, 3.6)`, with `planned-label`'s word if you set one. When 3.6 ships, move it to `live`, and the same marks read "since 3.6" with no other edit.
- **`ends`** gives the last day each live line is supported. Once it's passed, `ascribe check` reports [`release-line-ended`](../checks/diagnostics/content-checks.md): move the line to `archived`.
- **`archived`** lines have left. `url` says where one can still be read, and `ended` when its support ended; both are optional.

Every line of a target has the same number of parts, planned lines are newer than every live line, and archived lines older.

When a planned line ships, [`ascribe versions release`](release.md#release-day) moves it to `live` and rewrites the marks that named it through a feature; `ascribe versions list` shows the lines, and `ascribe versions changes` what differs between two.

## What the checks find

With the list, `ascribe check` checks each version a mark names, in pages and in `ascribe.toml`'s features and builds:

- **A version newer than every line** is an error, [`available-version-ahead`](../checks/diagnostics/directives.md): `self-managed 3.7` when 3.6 is the newest line is likely a typo, or a line to plan. The fix changes it to the newest line.
- **A version between two declared lines** but in none, such as `3.5` when the lines are `3.4` and `3.6`, is [`available-line-undeclared`](../checks/diagnostics/content-checks.md), with a fix to each line beside it.
- **A mark that no output shows** is reported when nothing builds the versions it names. On a versioned build, a version at or before the oldest live line isn't shown, since every reader has it: `self-managed 3.3` reads the same as `self-managed`, and [`available-same-in-every-line`](../checks/diagnostics/content-checks.md) offers to write it so. Content that's in no live or planned line at all is [`available-in-no-line`](../checks/diagnostics/content-checks.md). A build that badges or filters, and isn't versioned by the target, still shows every version, so while a project has one, these two report nothing.
- **`availability-left-behind`** compares a feature's versions with its target's current line.

A version older than every declared line isn't a mistake the first two report: it's before the window, so you needn't declare the releases before the ones you support.

A phrase can follow the list too: `release = { line = "self-managed" }` makes `{release}` the current line, and in an older line's output of a versioned build, that line.

## Build a site for each line

Name the target on a build:

```toml
[builds.site]
availability = "badge"
versions = "self-managed"
```

`ascribe build` then writes an output for each live line. The current line keeps the build's name and its addresses; each other line is `<build>@<line>`, under its number after the base path:

| Line | Output | A page at `install.md` |
|---|---|---|
| 3.5, current | `.ascribe/build/site/` | `/docs/install/` |
| 3.4 | `.ascribe/build/site@3.4/` | `/docs/3.4/install/` |

So turning versions on moves no address, and a link to the docs always shows the newest line. Every address in an older line's output starts with its folder: links between pages, the files they link to, and the canonical addresses in its page records. `--build site@3.4` builds that line alone, and `--build site` the current one.

A versioned build's `availability` is `badge`: a build that filters is already one version of one target. With one live line, the build writes what it would without `versions`, apart from the file the lines share (below), and older marks no longer show their versions.

## What each line has

A line keeps content whose `available` doesn't name the target, and content that's available for the target at some version in the line. Content that begins at `3.4.2` is in 3.4; so is content removed at `3.4.2`, since it was there at `3.4.0`. What isn't in a line is removed from its output, as a filtered build removes it: a section, a table row, or a whole page. The current line also keeps content marked for a version after it, shown as it is in any badge build, such as "3.6+".

What's left is marked as a badge build marks it, but a version at or before the oldest live line isn't shown, since every reader has it: with 3.4 and 3.5 live, `self-managed 3.3` shows the target alone, `self-managed deprecated 3.2` shows "deprecated", and `self-managed 3.4.2` shows its version. A build that isn't versioned shows them all.

`ascribe check` checks each line as a build. A problem in more than one line is reported once, saying so.

## Links between lines

A link to a page that isn't in a line still goes somewhere, in three steps:

1. **The page that takes its place.** A page that's rewritten under a new name says which page it replaces, with `replaces` in its frontmatter:

   ```yaml
   ---
   title: Configure the agent
   available: self-managed 3.5
   replaces: configure-legacy.md
   ---
   ```

   A link to either page lands on whichever one the line has. Neither page links to the other by hand, and a link keeps its `#id` when the page it lands on has a heading with that id.
2. **The page in another line.** Otherwise the link leads to the page in the nearest line that has it, the closest newer line first. In 3.4, a link to a page new in 3.5 goes to `/docs/webhooks/`, and the line follows the link: "Webhooks (3.5)". A site styles it by its `data-ascribe-release-line` attribute; the plain output writes the line after the link.
3. **Nowhere.** When no live line has the page, the link is [`link-page-dropped`](../checks/diagnostics/pages.md), as in any build.

A link from the current line to a page only an older line has is [advice](../checks/diagnostics/pages.md): it's a reader on the newest docs sent back to an older line, which is usually a link the page no longer needs. Two pages that replace each other, or a page that replaces a page its own line still has, are errors. [`replaces`](../writing/directives.md#a-page-that-replaces-another) has the rules, and `ascribe link` says where a link goes in each line.

## What an older line says about itself

A reader who lands on an older line from a search should know it's older, and find the current one. Each output says so in its own way:

- **The canonical address** of a page in an older line is the same page in the current line, or the page that replaced it there, following `replaces` as a link does. A page the current line has neither of keeps its own. So a search engine indexes the current line's page, and an older line's page only when it's the one place the page is.
- **The sitemap** lists a page when it's listed and its canonical address is its own: each current page once, and an older line's page only when the current line has no such page.
- **The page's Markdown** opens with a line that says which version it is:

  > Documentation for Self-managed 3.4. Support ends on 31 January 2027. The current version is 3.5: <https://docs.example.com/docs/install/>.

  The current line's says its own version alone. The date comes from `ends`, and the sentence is left out without one; the target is named by its [label](../content-model/content.md#6-dimensionsname).
- **A site's banner** says the same to a person: [`VersionBanner.astro`](#publish-with-astro) with Astro, or the site's own from the files below.

Nothing is added to `robots.txt`. An older line is kept out of an index by its pages, not by a rule over its folder, which would also stop a crawler reading the canonical link.

### Keep the older lines out of an index

A project that doesn't want its older lines found at all says so under `[listing]`:

```toml
[listing]
older-lines = "unlisted"
```

Every page of every line but the current one is then `unlisted`: it asks not to be indexed, and it's left out of the sitemap and `llms.txt`. `unquoted` asks that they not be quoted. The rule only restricts, as every listing rule does: a page in an older line that sets a looser `listing`, such as `listing: listed`, is [`listing-loosened-in-line`](../checks/diagnostics/pages.md). A folder's or a type's rule still applies on top of it.

## What the lines know about each other

Each `ascribe build` of a project with `[releases]` writes `.ascribe/build/_ascribe/versions.json`: for each versioned build, the target's label, its lines, and for each page the lines that publish it, with its route in each, and the page it replaces or that replaces it. A site reads it to link a page to the same page in another line. The [output layout contract](../contracts/output-layout.md#15-the-versions-file) has its format.

A site that isn't built with Astro has all a banner and a list of versions need in it and in each line's page records (`_ascribe/pages.json`):

| To write | Read |
|---|---|
| "Documentation for Self-managed 3.4" | `builds.<name>.label`, and the line from `lines[]`, the one whose `base` the page is under |
| "Support ends on 31 January 2027" | that line's `ends` |
| The link to the current line | the page's `canonical` in its record when it isn't the page's own address; else the `base` of the line whose `status` is `current` |
| A page's other lines | `pages.<path>.lines`, following `replaces` and `replaced-by` |
| The lines that have left | the `lines[]` whose `status` is `archived`, and their `url` |

## Publish with Astro

[`@ascribed/astro`](../astro/index.md) serves every line from one site, with no change to `astro.config.mjs` or `src/content.config.ts`. The collection has every line's pages: the current line's with their usual ids, and an older line's with the line in front, `3.4/install`, so the route `[...slug].astro` gives each is where Ascribe's links point. Each entry's data has `line` and `current`.

`<Versions entry={entry} />`, from `@ascribed/astro/Versions.astro`, lists the lines, each a link to the page there, with the page's own line marked `aria-current="page"`. A page that replaces another lists the lines of the page it replaces too, and the other way around, so `configure/` in 3.5 and `3.4/configure-legacy/` link to each other. A line that has neither is shown unlinked, "(not in this version)", and the archived lines that have a `url` follow, each linking to it, "(archived)". It renders nothing for a build that isn't versioned, or one with a single line to list.

`<VersionBanner entry={entry} />`, from `@ascribed/astro/VersionBanner.astro`, writes the banner on an older line's pages, and nothing on the current line's:

> Documentation for Self-managed 3.4. Support ends on 31 January 2027. [See this page for 3.5.](/docs/install/)

The link goes to the page in the current line, or the page that replaced it, and to the current line's home otherwise, as "See the documentation for 3.5." Each sentence's words are a property, `documentation`, `supportEnds`, `samePage`, and `home`, with `locale` for the date, so a site can translate them; `Versions.astro`'s are `label`, `missing`, and `archived`.

The sidebar shows the pages of the page's own line when the layout passes the entry: `sidebar(nav, entries, { entry })`. `npx @ascribed/astro init` writes all three into the layout.

`astro build` and `astro dev` build every line, and `astro dev` rebuilds them all on a change. Two things stay with the current line for now:

- **A permalink forwards to the current line,** whichever lines have the page: it's the address a product prints, and it should land on the newest docs.
- **Only the current line has a Markdown version and `llms.txt`.** An older line's pages point to neither, and the site's sitemap lists the current line's pages alone.
