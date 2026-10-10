# Proposal: documentation versions

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail. It follows the research in [Documentation versioning approaches](../reports/Documentation%20versioning%20approaches.md).

## The goal

Four levels, each costing a project only what it needs:

1. **No versions.** The default, and what Ascribe does today. Nothing to configure.
2. **One site that knows its versions.** The project says which versions exist. It still publishes one site, and gains checks and a "planned" state it no longer keeps by hand. Ascribe's own docs are this case: one site following `main`.
3. **A window of live versions.** One source, with content marked for where it applies, published as one site per live version.
4. **An archive.** A version that has left the window is rendered once more and frozen: readable, and no longer editable.

Each level is opted into separately. The research found that every large project ends up with the last two as a pair: a short window tied to the product's support policy, and a long tail of frozen copies. The two big users of single-source versioning, Elastic and GitHub, both annotate inside a window and freeze at its edge.

## What Ascribe has, and what's missing

It already has the hard part. `@available: self-managed preview 3.4` says where content applies, with lifecycle states and histories, checked against the content model. A build either marks everything with its availability or filters for one target at one version. The research found no other system that gets both a badge and a filtered build from one ordered annotation; most markup languages can't say "since 3.4" at all.

What's missing is everything around it:

- **Ascribe doesn't know which versions exist.** It can compare `3.3` with `3.4`, but not say which is current, which are supported, or which are gone. Every system in the research that understands version order has this list. Ascribe is the exception.
- **Nothing in the published site knows about versions:** no switcher, no banner on an old version, no canonical link.
- **Nothing is ever frozen.**

## 1. Say which versions exist

Versions belong to a target, such as `self-managed`, so the list does too. It gets a table of its own, `[releases]`. `[versions]` keeps the comparison scheme and nothing else, so a target named `scheme` can't collide with it.

```toml
[releases.self-managed]
live = ["3.4", "3.5"]     # published and maintained; the newest is current
planned = ["3.6"]         # may be named in content before it ships

[releases.self-managed.ends]
"3.4" = 2027-01-31        # when support for a live line stops

[releases.self-managed.archived]
"3.3" = { url = "https://docs.example.com/archive/3.3/", ended = 2026-06-30 }
"2.9" = { url = "https://v2.docs.example.com/" }
```

- **A version here is a release line.** `3.4` stands for every `3.4.x`.
- **The current line is the newest live one.** It isn't written down, so it can't disagree with the list. A line newer than current is planned, by definition.
- **A project with no `[releases]` works exactly as it does now.**

### What the list gives, with one site

This is the second level. Nothing is published differently.

- **A version that can't exist is an error.** `@available: self-managed 3.45` names a line newer than every declared one, which is a typo. Today nothing can know that.
- **A version from before the list isn't.** A project whose pages say "since 3.1" doesn't have to declare 3.1, 3.2, and 3.3 to adopt this. A version older than the oldest declared line is before the window, and is reported as a mark that no longer does anything (below), not as a mistake.
- **"Planned" is computed.** Content marked for 3.6 is shown as planned while 3.5 is current, and stops being planned the day 3.6 joins `live`.
- **Marks that no longer do anything can be found.** Annotations are positive and ordered, so this is exact. So is content that appears in no declared line at all, which is always a mistake.
- **A date can be checked.** A live line whose `ends` date has passed is reported.

### "Planned" isn't a lifecycle state

Ascribe's docs declare a lifecycle state, `unreleased`, and a feature, `next`, that points at it. With a list, the state is unnecessary: planned is something the list says about a version, not something an author writes.

That touches three contracts, and the proposal doesn't hide it:

- **The availability element** marks a target's entry as planned, with an attribute of its own beside the state, and the badge reads "planned" before the version: "Self-managed (planned, 3.6)". The label can be changed, as a state's can.
- **The plain output's availability line** says the same.
- **The JSON output** carries it as a field on the target.

The feature key stays. An author writing about unreleased work often doesn't know its number, so `@available: next` is still how it's marked, and `[features.next]` still names the line. What changes on release day: the line moves from `planned` to `live`, and nothing about a lifecycle state is edited. Pointing `next` at the following line, and rewriting the marks that meant this one, is still a step someone takes. A command could do both, and is listed under "Still open".

### One home for the current version

The docs keep a phrase, `version`, by hand, with a test to hold it to the release. A built-in phrase for each versioned target, resolving to the line of the build it's in, gives that fact one home. It also covers the commonest difference between versions, the number inside an install command, which otherwise has to be written into the prose, since availability can't mark part of a sentence.

## 2. Publish the window

### A build that's versioned

Today's filter can't do this. It removes everything that doesn't name its target, so filtering a site for `self-managed 3.4` would strip every page about the cloud service, and every page about another product. It also treats a version as a point, so content marked `self-managed 3.4.2` is later than `3.4` and would be dropped from the 3.4 site.

So a versioned build is a third thing, beside `badge` and `filter`:

```toml
[builds.site]
variants = "switch"
availability = "badge"
versions = "self-managed"    # one output for each live line of this target
```

In each line's output:

- **Content that names the target and isn't in that line is removed.** The line is taken whole: anything that begins at `3.4.2` is in line 3.4, and not in 3.3.
- **Everything else is kept and marked, as a badge build does.** Content about the cloud service, or about another product, is untouched.
- **The current line keeps planned content,** marked as planned, which is what a one-site project shows today. Older lines don't show what came after them.

So the day versions are turned on, the root's content changes in one way only: content marked as removed at or before the current line is no longer published there.

**Several products.** A build is versioned by one target. Another product's annotations stay as badges in it, so two versioned products don't multiply into a grid of builds. A site about two products picks the one its addresses follow, or publishes a versioned build for each.

### Names, and work saved

One build in `ascribe.toml` becomes several outputs, and each needs a name: for its directory, `--build`, `[editor] build`, the build lens, and the list of builds `check` reports.

- **The current line keeps the build's name:** `site`. Each other line is `site@3.4`. `@` isn't allowed in a build's name, so nothing a project declares can collide.
- **A page that resolves the same in every line is checked once,** and reported once, as "in every line". Most pages do. The review report lists a changed page once, with the lines it differs in, not once per line.

### Addresses

Settled: the current line is at the root, and every other live line is under its number.

| | 3.5 (current) | 3.4 | When 3.6 ships |
|---|---|---|---|
| A page at `install` | `/install/` | `/3.4/install/` | `/install/` now shows 3.6, and 3.5 moves to `/3.5/install/` |

- A project with no versions already publishes at the root, so turning versions on moves no URL.
- A link to the docs always shows the newest version, and canonical links point search engines at the root.
- The cost is accepted: a link someone saved while reading 3.5 becomes a link to 3.6 when it ships.

**The number goes after the base path.** A site at `/docs/` publishes 3.4 at `/docs/3.4/`. Routes come from one base path in `[consumer]` today, so each line's output is written with its own: every link inside the 3.4 pages starts with `/docs/3.4/`, and the plain output's absolute URLs follow.

**The Astro integration changes with it.** It takes one build and one collection today, and checks that Astro's `base` is the project's base path. For a versioned build it needs a collection for each line, or one route keyed by line, and a check that allows for the number.

### What every line knows about the others

The switcher on a 3.4 page has to know 3.5 exists, and whether this page is in it. The output layout contract says nothing is shared between builds. This is the one exception, and a new contract: a single file above the builds' outputs, which each of them reads.

It holds each target's lines, with status and address, and for each page the lines that publish it and the page that replaces it in the others (below). From it:

- **The switcher** offers a line only for a page that's in it, and says so otherwise. Switchers elsewhere match pages by path, and break when a page is renamed or missing.
- **The canonical link** points at the same page in the current line when there is one. The best-documented reader problem in the research is search engines sending people to old versions.
- **A banner** on anything but the current line says which version it is, when its support ends if `ends` gives a date, and links to current.

By default an older live line can still be indexed, with its canonical links pointing at current. A setting marks older lines `noindex` instead, for a project whose old pages keep outranking its new ones.

`@ascribed/astro` would supply the switcher, banner, and links. A site using another generator gets the file.

**For agents,** the claim is narrower than it sounds. Ascribe has no `llms.txt` yet; the [content checks plan](content-checks/phase-6-agent-output.md) adds it. And the plain output names a version only on a page that has an availability line. Each line's Markdown pages saying which version they're for is new output, to be added with this.

### Links to a page that isn't in a line

A page new in 3.5 isn't in the 3.4 output, and every link to it from shared content is an error there today (`link-page-dropped`). The spec's way out is a variant arm the build removes, and versions aren't variants. Marking each linking paragraph with `@available` would cost an annotation for every inbound link to every page added inside the window.

So in a versioned build, a link resolves in this order:

1. The page, when this line publishes it.
2. The page that replaces it, or that it replaces, in this line (below).
3. The page in the nearest line that does publish it, as a link out of this line, marked as one: "Configure tracing (3.5)".

It's an error only when no live line publishes the target. A link from the current line back into an older one is reported as advice, since it's usually a sign the newer page is missing.

### A page that replaces another

Settled: two files never publish to one address, and a page can say which page it replaces.

A page rewritten for 3.5 while 3.4 is live is a new file at a new address. The language has no "until", so the old page says when it stops with a history, and the new one names it:

```yaml
---
title: Install the agent (before 3.5)
available: self-managed (ga 3.0, removed 3.5)
---
```

```yaml
---
title: Install the agent
available: self-managed 3.5
replaces: install-legacy.md
---
```

- The switcher takes a reader of the old page to the new one, and back.
- Canonical links follow, and so do links from other pages, by the rule above.
- It's checked like a link: the file must exist and be a page, the two pages' availability mustn't overlap, and a page can't replace itself or form a loop.

Without the line, the two are simply different pages. A project that never rewrites a page never sees any of this.

`replaces` is a new reserved frontmatter key, beside `available` and `variant`. A project whose page types already declare a field with that name would have to rename it, so it's recorded as a behavior change when it ships.

When enough pages need `replaces` that it feels like bookkeeping, that's the sign the change is a rewrite, and the old version belongs in the archive.

### What changed between versions

The blocks whose availability differs between 3.4 and 3.5 are known from the annotations alone, with nothing built. A report of them (added, removed, and changed state, by page) is exact and cheap, and is what release notes and upgrade guides are written from. It doesn't need `ascribe diff`, which compares two revisions of the source.

## 3. Retire a version, and archive it

Retiring is the step the research found most expensive: GitHub's public runbook has 16 steps, with scripts to strip conditions and clean-up by hand. Ascribe can make it two commands, because it knows what each annotation means.

### Archive first

`ascribe archive --version "self-managed 3.3"` renders the source as it stands, for that line, into a folder of plain HTML. It's a last render, not a copy of the site that was live: the same source, the same line, drawn by Ascribe alone. That's why it has to run before the clean-up, while the source still holds what 3.3 needs.

- **It depends on nothing.** No site generator, no theme, no script or request to anywhere else. Several projects in the research say that rebuilding old source years later doesn't work, and that's why they keep output.
- **It has a way around.** The content model has no navigation, and Ascribe's own renderer draws one page with no layout. So the archive writes a contents page and a sidebar from the folders, the pages' titles, and their headings, in path order. If the content model gains navigation, the archive uses it.
- **It has search.** An index and a script inside the folder, working from a file opened in a browser. That's a new component, on the scale of the review report's script.
- **It says what it is,** on every page, in words that don't age: "Documentation for Self-managed 3.3. This version is no longer maintained."
- **It's marked** so search engines and agents leave it alone.
- **It prints.** A print stylesheet, so a browser can make a PDF of a page or of the whole set.

The archive doesn't go in git. The research found that copies of text are cheap there (eleven copies of Ascribe's own docs measured 1.29 times the size of one), and that the repositories that grew to gigabytes did it by committing built output. The project hosts the folder wherever it likes, and records the address in `[releases]`.

### Then clean up

`ascribe versions retire "self-managed 3.3"` moves the line from `live` to `archived`, and refuses if it has no address. Then it reports every annotation the smaller window has made dead, with a fix for each.

- **A mark is rewritten to the simplest spec that means the same in every live line.** Versions before the window go; states stay. With 3.4 and 3.5 live, `self-managed 3.3` becomes `self-managed`, and `self-managed deprecated 3.2` becomes `self-managed deprecated`, which is still worth showing. A history's steps before the window collapse into the one in effect when it opens. These are safe fixes: no line's output changes.
- **A mark is removed only when what's left says nothing:** every target its scope allows, generally available.
- **A block that's in no live line is offered for deletion,** not deleted. That removes content.
- **A page being deleted that another `replaces`** would leave that line dangling and lose the redirect. The command lists each old address with the page that took its place, so the redirect can be set up, and removes the `replaces` line with the page.

Archiving is what makes the clean-up safe: the old text still exists somewhere.

## Where this stops

**A major rewrite isn't a version.** When version 2 replaces most of version 1's pages, the research found that nobody uses conditional content: five of six projects checked froze the old site on its own address. So an archived entry can be any URL, including a site Ascribe didn't build. The list is how a reader gets there; Ascribe doesn't try to hold both in one source.

**The window is as wide as a team can maintain.** "One or two versions" in other tools comes from the cost of backporting across branches. In one source there's nothing to backport, and the limit is how many annotations a page can carry and still be readable. Ascribe doesn't set a number. It can report how many live lines a page's marks distinguish, and leave the judgment to the team.

**A planned line isn't published by itself.** Its content shows on the current line, marked as planned. Docs for a release candidate at an address of their own would be a setting on the list, and aren't proposed here.

## What this leaves out

- A copy of the source per version, in folders or branches. A team that wants branches can already have them; Ascribe adds nothing there.
- Hosting archives.
- A PDF engine. Print goes through the browser.
- Translating between version schemes. `numeric` stays the only one.
- A general selector for platforms, environments, frameworks, and languages (below).
- Freezing a team's real site. The archive is Ascribe's own render.
- Versioning one build by two products at once.

## Later, and bigger than versions

A reader choosing "Cloud" or "Self-managed 3.5" is choosing between a target with no versions and a version of another. That's one case of a wider need: choosing a platform, an environment, a framework, or a language, and having the site hold that choice. Ascribe has the parts (dimensions, and variants a site shows as tabs with a choice that holds across pages), but not one design for selectors and filters across a whole site. Versions shouldn't invent one. This proposal's switcher is limited to the lines of one target, and is built so a general selector can take it over.

Two more wait on other work:

- **Dates follow one rule.** A check on `ends` and the content checks plan's overdue-review check both depend on the day they run, and share whatever rule makes that testable.
- **Tested examples.** Mapping an [observation](tested-examples.md)'s environment to a target, so Ascribe can say an example was tested on a version the page doesn't claim, is worth returning to once tested examples exist.

## Still open

- **Should a release be a command?** Moving a line from `planned` to `live` is one edit. Pointing `next` at the following line, and rewriting the marks that meant this one, is a second, and it's the one that was done by hand after 0.2.0. `ascribe versions release` could do both.
- **Do redirects wait for the redirects feature?** Retiring lists the old addresses and where each should go. Whether Ascribe then writes them for the host, or only reports them, depends on the redirects idea in the [brainstorm](brainstorm.md).
- **What does a version between two declared lines mean?** With 3.2 and 3.4 declared, content naming 3.3 isn't newer than every line, so it isn't the typo case. It could be an error, or advice that the line isn't declared.
