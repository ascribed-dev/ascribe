---
type: how-to
title: Versions
description: Publishing the docs of more than one version of a product from one source, a site for each version a reader can be on.
available: next
---

A product a team supports in more than one version needs docs for each: a reader on 3.4 should read about 3.4, and find the same page for 3.5. Ascribe does this from one source, the marks the pages already have, in four levels a project opts into one at a time:

1. **No versions.** The default. Nothing to configure, and nothing changes.
2. **One site that knows its versions.** `[releases]` in `ascribe.toml` lists a target's live lines. So far it takes `live` alone; what's planned, when a line ends, and what's archived, with the checks they make possible, are to come.
3. **A window of live versions.** `versions = "<target>"` on a build gives one output for each live line, which this page covers. Links from one line to another, a banner on an older line's pages, and canonical links to the current line are to come.
4. **An archive.** A line that's left the window, kept as a frozen copy. Not yet.

## List the live lines

A **line** is a version as you write it, and holds every version that starts with it: `3.4` holds `3.4.0`, `3.4.2`, and `3.4.2.1`. List the lines a reader can be on under the target they're versions of:

```toml
[releases.self-managed]
live = ["3.4", "3.5"]
```

The order doesn't matter: the newest is the **current line**. Write lines as finely as you release: `["3.4", "3.5"]` for a line for each minor release, `["0.2.0", "0.3.0"]` for one for each patch release. Every line of a target has the same number of parts. The [`[releases]` reference](../content-model/content.md#71-releasestarget) has the rules.

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

`ascribe check` checks each line as a build. A problem in more than one line is reported once, saying so. A link to a page that isn't in a line is [`link-page-dropped`](../checks/diagnostics/pages.md), as in any build.

## What the lines know about each other

Each `ascribe build` of a project with `[releases]` writes `.ascribe/build/_ascribe/versions.json`: for each versioned build, its lines, and for each page the lines that publish it, with its route in each. A site reads it to link a page to the same page in another line. The [output layout contract](../contracts/output-layout.md#15-the-versions-file) has its format.

## Publish with Astro

[`@ascribed/astro`](../astro/index.md) serves every line from one site, with no change to `astro.config.mjs` or `src/content.config.ts`. The collection has every line's pages: the current line's with their usual ids, and an older line's with the line in front, `3.4/install`, so the route `[...slug].astro` gives each is where Ascribe's links point. Each entry's data has `line` and `current`.

`<Versions entry={entry} />`, from `@ascribed/astro/Versions.astro`, lists the lines that publish the page, each a link to it there, with the page's own line marked `aria-current="page"`. It renders nothing for a page only its own line has, or a build that isn't versioned. The sidebar shows the pages of the page's own line when the layout passes the entry: `sidebar(nav, entries, { entry })`. `npx @ascribed/astro init` writes both into the layout.

`astro build` and `astro dev` build every line, and `astro dev` rebuilds them all on a change. Two things stay with the current line for now:

- **A permalink forwards to the current line,** whichever lines have the page: it's the address a product prints, and it should land on the newest docs.
- **Only the current line has a Markdown version and `llms.txt`.** An older line's pages point to neither, and the site's sitemap lists the current line's pages alone.
