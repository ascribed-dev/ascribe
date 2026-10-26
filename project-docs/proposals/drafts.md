# Proposal: drafts

A proposal, not a plan. It says what would be built and in what order, so the idea can be judged before anything is designed in detail.

## The goal

A writer starts a page, or gets early feedback on one, and merges it without publishing it: `draft: true` in its frontmatter, and no build publishes it until the line goes. It's checked like any page while it waits, so it's clean the day it's published, and the editor, review, and a preview build show it, marked as a draft.

## Who it's for

- **A writer** who wants an early start on a page for a release, or to merge work in progress instead of keeping a long-lived branch.
- **A reviewer** giving early feedback on a page that isn't ready to publish, in review or on a deploy preview.
- **A team** that wants to see which pages have sat as drafts too long.

## What exists

- **Nothing marks a page unfinished.** The nearest features answer other questions:
  - **`listing: unlisted`** (SPEC §2.1) still builds the page at its address; it keeps it out of `llms.txt`, the navigation, and what offers pages to search engines and agents. Published, not advertised.
  - **Availability,** `planned` included, says what the product has, not whether the doc is done; the current line's build keeps `planned` content, with a badge.
  - **A project's own field,** such as `status = enum(draft, published)` (the content-model reference's example), is checked, and nothing acts on it.
  - **Review** shows a pull request's pages rendered, for feedback before merging; it doesn't cover merging early and publishing later.
- **A build already leaves pages out.** `ResolvedBuild::dropped` (`crates/ascribe-resolve/src/build/mod.rs`) gives a `DropReason` (`crates/ascribe-resolve/src/build/tree.rs`): `Variant`, a page whose `variant` conflicts with the build's selection, and `Unavailable`, one its `available` rules out in a filter build. Everything that lists a build's pages follows it: the outputs, the navigation, the sitemap, `llms.txt`, search, and the page records. A link to a page a build doesn't publish is an error, `link-page-dropped`.
- **Reserved frontmatter keys** (SPEC §2.1): `type`, `available`, `variant`, `intended`, `permalink`, `listing`, `replaces`, and `nav-label`. A fragment may use only `intended`.
- **Other tools agree on the name.** Hugo, Astro's content collections, Docusaurus, and Jekyll each read `draft: true`, and leave the page out of a production build.

## Settled

Answered by the maintainer on 25 October 2026.

1. **Pages only.** A draft is a whole page. An unfinished section of a published page, which would need a container such as `@draft`, waits until someone asks.
2. **A draft is built nowhere by default.** It isn't published at a hidden address for sharing: feedback goes through review or a preview build that includes drafts, so an unfinished page never sits on the live site at an address someone can guess or share.
3. **No default from the content model.** A folder or a content type can't make its pages drafts, as `listing` can; a page says so itself. Added if someone asks.

## The design

### The key

`draft: true` in a page's frontmatter, a ninth reserved key (SPEC §2.1). It's a boolean, `false` when absent; a value that isn't one is a frontmatter value of the wrong type. A fragment can't use it, as with the other reserved keys but `intended`. A content model that declares `draft` as a field of its own is an error, as for every reserved key; a project migrating from Hugo or Astro keeps its `draft: true` lines, which now mean what they meant there.

It's a key of its own, not a value of `listing` or of availability, because it's a third question: whether the doc is finished. `listing` says who a page is offered to, and availability what the product has.

### What a build does

- **Every build drops a draft,** with a third `DropReason`, `Draft`, so everything that already follows a dropped page follows it: no output, no navigation entry, no sitemap line, no `llms.txt` entry, no search entry, no page record, and its permalinks aren't in the build's `permalinks.json`.
- **A link to a draft from a published page** is `link-page-dropped`, as now, with a message of its own: the page is a draft, so publish it, or take the link out until it is. A writer who links ahead of publishing sees it in the check, not on the live site.
- **An include into a draft** is fine: a draft may include fragments, and a fragment only drafts include isn't unused.

### Seeing drafts

- **`ascribe build --drafts`** builds drafts as though published, and marks them: the site output's frontmatter for a draft has `draft: true`, the page records say so, and the manifest records `"drafts": true`, so an output built with drafts can be told from one that wasn't.
- **`@ascribed/astro`** takes a `drafts` option, for a deploy preview's build. A draft it builds shows a banner saying it's a draft and isn't published, gets `noindex`, and stays out of the sitemap and `llms.txt` even in that build, so a preview indexed by mistake doesn't spread it.
- **Not in `ascribe.toml`.** Whether a build includes drafts is a choice about where it's built, a preview or production, not about the content, so it's an option of the build command and the integration, not a setting a project commits.
- **The editor's preview** shows a draft with the same banner, and This page's About says "Draft: no build publishes it". The Pages view marks drafts.
- **Review** shows a draft a pull request adds or changes, rendered and marked, so a reviewer can comment on it, though no build publishes it. `ascribe diff`'s report lists it under its own heading, since a diff of builds would otherwise show nothing for it.
- **`ascribe resolve`** on a draft writes it, and says it's a draft, as it says why a build doesn't publish a page now.

### Publishing

Taking out `draft: true` publishes the page in every build that would otherwise publish it; nothing else changes. With [redirects](redirects.md)' `ascribe.addresses`, publishing a draft loses no address, since it had none, and turning a published page back into a draft loses its addresses, which `address-lost` reports, as a deliberate decision should be.

### Permalinks

A draft's permalink isn't published, so `ascribe permalinks sync` leaves it out of `ascribe.permalinks` until the page is published: a product's `ascribe permalinks verify` can't pass for a name whose address doesn't answer yet. A published page whose permalink moves onto a draft is `permalink-removed` for the published name, as any name that stops being published is.

### Drafts that sit

`ascribe report` gains a list of drafts: each page, and when it was last changed, from git, oldest first. A draft that hasn't changed in a long time is a page someone forgot, or one that should be deleted. The report says; nothing fails.

## The thread

- **The use case:** a writer adds a page with `draft: true` and a link to it from a published page; `ascribe check` reports the link; `ascribe build` leaves the page out; and a preview built with `@ascribed/astro`'s `drafts` option shows it with its banner, `noindex`, and out of the sitemap.
- **The layers:** the language (the reserved key), resolve (`DropReason::Draft`), check (the link's message), emit (`--drafts`, the marks), the CLI, Astro (the option and the banner), the contracts, and the docs.
- **The seam it tests:** a page no build publishes, built anyway and marked, through an output that must never be mistaken for production: the manifest's mark, the banner, and `noindex` together. If a draft's mark can't reach the integration through the site output's frontmatter, the design changes before the editor and review are built on it.
- **Left out, and where it comes back:** the editor's banner, This page's row, the Pages view's mark, and review (phase 2); the report of drafts and the permalinks list (phase 3).

## The phases

| Phase | What a user can do after it | Layers |
|---|---|---|
| 1. The thread | Mark a page a draft, see every build leave it out and a link to it reported, and preview it with a banner | language, resolve, check, emit, CLI, Astro, contracts, docs |
| 2. In the editor and review | See a draft's banner in the preview, its row in This page and mark in Pages, and comment on it in review | language server, extension, `ascribe diff`, review |
| 3. Drafts that sit, and permalinks | See which drafts are oldest in the report; a draft's permalink stays out of `ascribe.permalinks` until it's published | report, permalinks, agents |

**Phase 1** builds the thread. SPEC §2.1 gains the reserved key and §9.3 the drop, the output layout the manifest's `drafts` and the page record's mark, and the content-model contract the rule for a field named `draft`. `examples/astro-site` gains a draft and a link to it, and its end-to-end tests build with and without drafts. Conformance cases hold the key, a wrong type, a fragment with it, a content model declaring it, and a link to a draft. The docs gain a page, `docs/content/writing/drafts.md` ("Drafts"). *Done when:* `ascribe build` leaves the draft out of every output; `ascribe check` reports the link with the draft's message; and the end-to-end test's preview build shows the draft with its banner and `noindex`, and leaves it out of the sitemap and `llms.txt`.

**Phase 2** brings drafts to the editor and review: the preview's banner, This page's row, the Pages view's mark, review's rendering of a draft a pull request adds or changes, and `ascribe diff`'s heading for drafts, a field added to its JSON. `ascribe resolve` says a page is a draft.

**Phase 3** adds the report's list of drafts, from git, and `ascribe permalinks sync` leaving a draft's permalinks out. The rules `ascribe agents sync` writes say an agent may start a page as a draft, and that taking out `draft: true` publishes it, which is a decision for a person.

## What it changes that's a contract

Each with a decision recorded in the phase that changes it:

- **SPEC:** §2.1's reserved keys, §9.3's drop, and the diagnostics' rows.
- **The output layout:** the manifest's `drafts`, and the page record's and the site output's frontmatter marks.
- **The content model:** a field named `draft` is an error.
- **The commands' JSON:** `ascribe build --drafts`, `ascribe diff`'s drafts, `ascribe resolve`'s mark, and the report's list.
- **`@ascribed/astro`'s options:** `drafts`.

## What this leaves out

- **Sections as drafts** (Settled 1).
- **A hidden address for sharing a draft** (Settled 2).
- **Drafts by folder or content type** (Settled 3).
- **A scheduled publish date.** A page that publishes itself on a date needs a build on that date, which is the host's or CI's to run; a team can take out `draft: true` in a pull request merged that day.
- **Workflow states beyond draft and published,** such as "in review" or "approved". A project's own field can hold them, checked by the content model, and review covers the feedback.
