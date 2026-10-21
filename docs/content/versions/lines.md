---
type: reference
title: Release lines
description: What a release line is, what each line's output holds and says about itself, how links go between lines, and what a site, the editor, and agents get from them.
---

A product a team supports in more than one version needs docs for each. Ascribe makes them from one source, the marks the pages already have, in four levels a project opts into one at a time:

1. **No versions.** The default. Nothing to configure, and nothing changes.
2. **One site that knows its versions.** [`[releases]`](../content-model/content.md#71-releasestarget) lists a target's lines: the live ones, the planned ones, when a line's support ends, and the ones that have left. One site still, with checks that the marks name versions the list knows, and "planned" worked out instead of kept by hand.
3. **A window of live versions.** `versions = "<target>"` on a build gives one output for each live line, with links from one line to another, and an older line that says what it is and where the current one is. [Publish a site for each version](index.md) sets it up.
4. **An archive.** A line rendered once more into a folder of HTML that needs nothing else, kept after it leaves the window. [Archive and retire a line](archive.md) makes one.

## Lines

A **line** is a version as a project writes it, and holds every version that starts with it: `3.4` holds `3.4.0`, `3.4.2`, and `3.4.2.1`. A line is **live** when a reader can be on it, **planned** before it ships, and **archived** once it has left; the newest live line is the **current line**. The [`[releases]` reference](../content-model/content.md#71-releasestarget) has the keys and their rules, and what `ascribe check` reports about the versions the marks name once the list is there.

A phrase can follow the list: `release = { line = "self-managed" }` makes `{release}` the current line, and in an older line's output of a versioned build, that line.

## What each line has

A line keeps content whose `available` doesn't name the target, and content that's available for the target at some version in the line. Content that begins at `3.4.2` is in 3.4; so is content removed at `3.4.2`, since it was there at `3.4.0`. What isn't in a line is removed from its output, as a filtered build removes it: a section, a table row, or a whole page. The current line also keeps content marked for a version after it, shown as it is in any badge build, such as "3.6+".

What's left is marked as a badge build marks it, but a version at or before the oldest live line isn't shown, since every reader has it: with 3.4 and 3.5 live, `self-managed 3.3` shows the target alone, `self-managed deprecated 3.2` shows "deprecated", and `self-managed 3.4.2` shows its version. A history keeps the state in effect when the window opens and what follows: with 3.5 and 3.6 live, `self-managed (preview 3.3, ga 3.5, deprecated 3.6)` shows `self-managed (ga 3.5, deprecated 3.6)`. A build that isn't versioned shows them all.

A versioned build's `availability` is `badge`: a build that filters is already one version of one target. With one live line, the build writes what it would without `versions`, apart from `versions.json`, and older marks no longer show their versions. Every address in an older line's output starts with its folder: links between pages, the files they link to, and the canonical addresses in its page records.

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

   A link to either page lands on whichever one the line has. Neither page links to the other by hand, and a link keeps its `#id` when the page it lands on has a heading with that id, and drops it otherwise, with [`link-fragment-dropped`](../checks/diagnostics/content-checks.md#ASC221) advice.
2. **The page in another line.** Otherwise the link leads to the page in the nearest line that has it, the closest newer line first. In 3.4, a link to a page new in 3.5 goes to `/docs/webhooks/`, and the line follows the link: "Webhooks (3.5)". A site styles it by its `data-ascribe-release-line` attribute; the plain output writes the line after the link.
3. **Nowhere.** When no live line has the page, the link is [`link-page-dropped`](../checks/diagnostics/pages.md#ASC121), as in any build.

A link from the current line to a page only an older line has is advice, [`link-into-older-line`](../checks/diagnostics/content-checks.md#ASC220): it's a reader on the newest docs sent back to an older line. [`replaces`](../writing/frontmatter.md#a-page-that-replaces-another) has its rules, and `ascribe link` says where a link goes in each line.

## What an older line says about itself

A reader who lands on an older line from a search should know it's older, and find the current one:

- **The canonical address** of a page in an older line is the same page in the current line, or the page that replaced it there, following `replaces` as a link does. A page the current line has neither of keeps its own. So a search engine indexes the current line's page, and an older line's page only when it's the one place the page is.
- **The sitemap** lists a page when it's listed and its canonical address is its own: each current page once, and an older line's page only when the current line has no such page.
- **The page's Markdown** opens with a line that says which version it is:

  > Documentation for Self-managed 3.4. Support ends on 31 January 2027. The current version is 3.5: <https://docs.example.com/docs/install/>.

  The current line's says its own version alone. The date comes from `ends`, and the sentence is left out without one; the target is named by its [label](../content-model/content.md#dimensions).
- **A site's banner** says the same to a person: [`VersionBanner.astro`](#the-astro-components) with Astro, or the site's own from `versions.json`.

Nothing is added to `robots.txt`. An older line is kept out of an index by its pages, not by a rule over its folder, which would also stop a crawler reading the canonical link.

### Keep the older lines out of an index

A project that doesn't want its older lines found at all says so under `[listing]`:

```toml
[listing]
older-lines = "unlisted"
```

Every page of every line but the current one is then `unlisted`: it asks not to be indexed, and it's left out of the sitemap and `llms.txt`. `unquoted` asks that they not be quoted. The rule only restricts, as every listing rule does: a page in an older line that sets a looser `listing`, such as `listing: listed`, is [`listing-loosened-in-line`](../checks/diagnostics/pages.md#ASC222). A folder's or a type's rule still applies on top of it.

## What the lines know about each other

Each `ascribe build` of a project with `[releases]` writes `.ascribe/build/_ascribe/versions.json`: for each versioned build, the target's label, its lines, and for each page the lines that publish it, with its route in each, and the page it replaces or that replaces it. The [output layout contract](../contracts/output-layout.md#15-the-versions-file) has its format.

A site that isn't built with Astro has all a banner and a list of versions need in it and in each line's page records (`_ascribe/pages.json`):

| To write | Read |
|---|---|
| "Documentation for Self-managed 3.4" | `builds.<name>.label`, and the line from `lines[]`, the one whose `base` the page is under |
| "Support ends on 31 January 2027" | that line's `ends` |
| The link to the current line | the page's `canonical` in its record when it isn't the page's own address; else the `base` of the line whose `status` is `current` |
| A page's other lines | `pages.<path>.lines`, following `replaces` and `replaced-by` |
| The lines that have left | the `lines[]` whose `status` is `archived`, and their `url` |

## The Astro components

[`@ascribed/astro`](../astro/index.md)'s collection has every line's pages: the current line's with their usual ids, and an older line's with the line in front, `3.4/install`, so the route `[...slug].astro` gives each is where Ascribe's links point. Each entry's data has `line` and `current`.

`<Versions entry={entry} />`, from `@ascribed/astro/Versions.astro`, lists the lines, each a link to the page there, with the page's own line marked `aria-current="page"`. A page that replaces another lists the lines of the page it replaces too, and the other way around, so `configure/` in 3.5 and `3.4/configure-legacy/` link to each other. A line that has neither is shown unlinked, "(not in this version)", and the archived lines that have a `url` follow, each linking to it, "(archived)". It renders nothing for a build that isn't versioned, or one with a single line to list.

`<VersionBanner entry={entry} />`, from `@ascribed/astro/VersionBanner.astro`, writes the banner on an older line's pages, and nothing on the current line's:

> Documentation for Self-managed 3.4. Support ends on 31 January 2027. See this page for 3.5.

The last sentence links to the page in the current line, or the page that replaced it, and to the current line's home otherwise, as "See the documentation for 3.5." Each sentence's words are a property, `documentation`, `supportEnds`, `samePage`, and `home`, with `locale` for the date, so a site can translate them; `Versions.astro`'s are `label`, `missing`, and `archived`.

The sidebar shows the pages of the page's own line when the layout passes the entry: `sidebar(entries, { entry })`, or, with a site's own list, `sidebar(entries, { nav, entry })`.

`astro build` and `astro dev` build every line, and `astro dev` rebuilds them all on a change. With `[consumer] agents = true`, each line's `llms.txt` and Markdown are published at its base path, `/docs/3.4/llms.txt`, with the response headers each line's pages need. A permalink forwards to the current line, whichever lines have the page: it's the address a product prints, and it should land on the newest docs.

## In the editor, in review, and in the tools

The lines show wherever a build does, and a page that's the same in every line shows once:

- **The editor.** Switch build lists a versioned build's lines under it, the current line first, and the build lens and the preview follow the line you choose: in 3.4, a page new in 3.5 is dimmed whole, and the preview is at the line's address, `/3.4/install/`. The editor checks every line of its build, and a problem only some lines have says which, such as "(only in build `site@3.4`)". A hover on a mark's version says which line it falls in, and a hover on `replaces` says which page each line publishes. [The editor](../editor/preview.md#the-status-bar) has the rest.
- **Review.** `ascribe diff` lists a changed page once with the lines it changed in: "in every line" for a page the lines share, "only in 3.4" for a change inside a section only 3.4 has. The HTML report lists the page once too, and switches between the lines it changed in. [Diff](../review/diff.md) has the details.
- **The tools.** `ascribe render install.md --build site@3.4` renders the page as that line publishes it, at its address under `/3.4/`, and so does `ascribe_render`. `ascribe model` has a `releases` section: each target's lines, with their status and the build that is each.

## For agents

With [`[consumer] agents = true`](../agents/index.md), each line has its own outputs for agents, in its plain output:

- **`llms.txt`** lists the line's pages, and its heading names the line: `# Loom (Self-managed 3.4)`. An older line's opens with a line pointing at the current line's: "This is an older version. The current version is 3.5: [llms.txt](https://docs.example.com/docs/llms.txt)."
- **Each page's Markdown** opens with the line that says which version it is, [above](#what-an-older-line-says-about-itself).
- **`ascribe agents index --build site@3.4`** writes the compact index of a line for a product's `AGENTS.md`, its heading naming the line as `llms.txt`'s does. Without `--build`, it's the current line's.

A line kept out of an index with `[listing] older-lines = "unlisted"` lists nothing in its `llms.txt`.
