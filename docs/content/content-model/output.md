---
type: reference
title: "`ascribe.toml`: where pages are published"
description: "How the content model sets the site's consumer, the builds, how pages are listed, what crawlers are told, and where retired permalinks went."
---

Part of the [`ascribe.toml` reference](index.md), whose [contents](index.md#contents) list every section. The rules for a loader are in the contract's [where pages are published](../contracts/content-model/output.md).

## 15. `[consumer]`
@id {permalink=true}: consumer
@summary: The consumer profile: how the site output fits a specific consumer, such as Astro, with its site address, base path, trailing slash, and slugger.

@include: ../_fragments/content-model/consumer.md

@include: ../_generated/content-model-consumer.md

**Heading ids, image attributes, and assets** ([SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)) are part of the profile, not keys. The `astro` profile has exactly one way to do each:

- **Heading ids, image attributes, and glossary links:** the site output writes each as a `<ascribe-attributes>` marker that the consumer's markdown plugin applies, so the consumer keeps its own heading, table-of-contents, and image processing, and a glossary link carries `data-ascribe-term`.
- **Assets:** copies mirror their source paths inside each output, and images are referenced relatively so Astro's image processing still applies. Other files a page links to are published under `_ascribe/files/`.

A later profile that offers a choice will add a key for it.

**How file paths become routes** (the `astro` profile): a page's route is `base-path`, then its path relative to the content root with the `.md` extension removed and each segment slugged the way Astro's content loader computes entry ids; a final `index` segment is dropped (`guides/index.md` → `/guides/`); then the trailing slash per `trailing-slash`. The root `index.md` (Astro's entry id `index`) is at `base-path`, which under `trailing-slash = "never"` loses its final `/` unless it's `/`. Two pages with one entry id (`My File.md` and `my-file.md`, or `index.md` and `index/index.md`) can't both be published, and `ascribe build --emit site` fails, naming them. The same router answers the reverse question, which page a route-like link names, for the `link-route` warning and its fix. Source files never contain routes ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)).

**The outputs for agents** (`agents`, on by default when `site` is set) change what the plain-markdown output is, and add to the site output:

- **The plain-markdown output is laid out by URL,** so it can be copied as it is to `base-path` beside the site: each page at its route with `.md` (`Guides/My Setup.md` → `guides/my-setup.md`, the root `index.md` → `index.md`), each opening with a line pointing at `llms.txt`; the files they link to under `_ascribe/files/`, linked by absolute URL; and `llms.txt`. A link to a page the build publishes, a glossary link included, leads to that page's Markdown, so an agent that follows it stays in Markdown, and its `#fragment` to the heading's anchor there: the slug of the heading as the Markdown writes it. That's the heading's id on the page, unless it has an `@id`, or is in a variant arm, whose label it ends with in the Markdown.
- **`llms.txt`** lists every page the build publishes as `- [Title](../url): description`, linking to its Markdown. Its heading is the title of the page at `base-path`, and its summary that page's description. Its sections are the project's [navigation](../navigation/index.md)'s: the content root's pages first, under its label, **Start** unless `ascribe.nav` gives another, then each top section, in order, named by its label, with the pages of the sections nested in it in reading order. A page is listed by its title, not its `nav-label`. An index over 50,000 characters is split into one `llms.txt` per top section, which the root file links to. A page's description is the field its type marks [`role = "description"`](index.md#54-field-roles), as plain text, or the field's `default` when the page leaves the field out or empty; a type without one lists its pages without descriptions.
- **Each page of the site output** opens with a pointer to `llms.txt` and the page's Markdown, which the element library hides from sight ([element contract §8]({repo}/blob/main/packages/elements/CONTRACT.md#8-ascribe-for-agents)).

**The headers of a Markdown version.** Each build lists the response headers each Markdown version needs, `X-Robots-Tag` and a canonical `Link`, in `_ascribe/headers.json` in the site output ([output-layout contract](../contracts/output-layout.md#14-the-headers)), and with `host = "netlify"`, the build writes them in Netlify's `_headers` too, which `@ascribed/astro` places at the site's root, beside the `_redirects` that [`ascribe.addresses`](../publishing/redirects.md) gives. Why a host sends them is in [Publishing](../publishing/index.md#the-markdown-versions-headers).

`@ascribed/astro` publishes the plain output with the site, and serves it in `astro dev`. With another generator, copy `<output-dir>/<build>/plain/` to the site's `base-path`. `ascribe check` then reports two advice checks: a description too long for its line in `llms.txt` (`description-too-long`), on the page that writes it, or once on a field's `default` in `ascribe.toml`, and an `llms.txt` still over the limit when it's split (`llms-section-large`).

@include: ../_fragments/content-model/consumer-astro-settings.md

---

## 16. `[builds.<name>]`

@include: ../_fragments/content-model/builds.md

**Rules.** Two build names can't differ only in case (`model-build-name-case`): they become folder names, and some file systems ignore case. A versioned build needs a target with a live line, and can't filter as well (`model-build-versions`). A build that filters for a target its own selection leaves out, such as one selecting `deployment = "cloud"` and filtering for `self-managed 3.3`, builds, but it's almost certainly a mistake, so you get a warning (`model-build-filter-excluded`). A build that filters for a version older than every line `[releases.<target>]` declares is in no line the project knows, so `ascribe versions` doesn't list it and a retirement counts it as a build its rewrite changes: that's advice (`build-filter-line-undeclared`), to declare the line in `archived` or to publish the lines with `versions` instead.

A versioned build's outputs, their names and addresses, and what each line keeps and shows are in [Versions](../versions/index.md#build-a-site-for-each-line). An older line's output says nothing to agents: it has no `llms.txt`, and its pages point to no Markdown version.

---

## 17. `[listing]`

@include: ../_fragments/content-model/listing.md

What each value tells search engines and agents is in [Publishing](../publishing/index.md#how-a-page-is-listed).

In VS Code, the sidebar's [This page](../editor/sidebar.md#this-page) says whether a page is listed, and which rule says so; choosing the row opens the rule.

In the editor, `listing:` in a page's frontmatter completes its three values, and hovering it says which rule the page is under. A page looser than its rule has a quick fix that removes its `listing`.

**Rules.** Each value is one of the three (`model-invalid-value`). A folder is a path from the content root ending in `/`, with no `.` or `..` part or pattern characters, of a folder that's there, with its name's case as on disk (`model-listing-folder`). A type is one the project declares, or `page` when it declares none (`model-listing-type`). A rule isn't looser than a folder's it's under, or than `default` (`model-listing-looser`). A page isn't looser than a rule it's under (`listing-loosened`). No other key is allowed in `[listing]` (`model-unknown-key`).

In an older line of a versioned build, a page isn't looser than `older-lines` either (`listing-loosened-in-line`).

---

## 18. `[crawlers]`

@include: ../_fragments/content-model/crawlers.md

What each purpose's choice costs, and why a page is kept out of search by its listing and not by `robots.txt`, is in [Publishing](../publishing/index.md#what-crawlers-are-told).

`ascribe check` reads each search crawler the way it reads the file: the one group that applies to it. A page that isn't listed is reported when a search crawler may not fetch it, whether the lines for every crawler say so, a group of `lines` that names that crawler, or `search = "disallow"` (`listing-disallowed`).

### Crawlers by purpose

Each operator below runs a bot for each purpose, or a token that says what it may do with what its other bots fetch, and says on its own page which is which. A purpose's group names each of them. A bot whose operator states no single purpose isn't here, such as Common Crawl's `CCBot`, whose archive many train on, or Meta's `meta-externalagent`, which gathers for training and for indexing both; name such a bot in `lines`, in a group of its own. Each row says what its operator said on the day in its last column, linked to the page.

@include: ../_generated/crawlers.md

The list changes, as operators add bots and rename them. Ascribe's release process rereads every source, and a newer Ascribe may name more crawlers for a purpose than an older one did.

**Rules.** Each line is one line (`model-crawlers-line`) and not empty (`model-empty-text`). A purpose is `allow` or `disallow` (`model-invalid-value`). A crawler is named in one group: not in a group of `lines` that a disallowed purpose names, and not in two groups of `lines` (`model-crawlers-named-twice`). Projects that build one site must agree on each purpose, since it has one `robots.txt`: one that allows a purpose and one that disallows it fail the site's build, naming both. No other key is allowed in `[crawlers]` (`model-unknown-key`).

---

## 19. `[permalinks]`

`ascribe.toml` has no `[permalinks]` any more. What happened to a [permalink](../writing/directives.md#permalinks) that no heading or page declares now, retired with a reason, or moved to another permalink or to a page, is recorded in [`ascribe.addresses`](../publishing/redirects.md#ascribeaddresses), keyed `go:<name>`, with the rest of where the docs' addresses went (SPEC §7.10). The [guide](../permalinks/index.md#moving-aliasing-and-retiring) says when to use each.

[`ascribe addresses init`](../publishing/redirects.md#cli-addresses-init) moves a project's `[permalinks.retired]` and `[permalinks.aliases]` into `ascribe.addresses`, and removes the table. Until it does, the table is an error at the key that says so (`model-unknown-key`).
