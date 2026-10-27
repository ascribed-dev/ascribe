---
type: reference
title: "`ascribe.toml` format contract: where pages are published"
nav-label: "Where pages are published"
description: "The normative contract for the content model's consumer, builds, listing, crawlers, and permalinks."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section. What these keys are for, with examples, is in the reference's [where pages are published](../../content-model/output.md).

## 15. `[consumer]`

@include: ../../_fragments/content-model/consumer.md

@include: ../../_generated/content-model-consumer.md

**Rules** (§25.7): each key MUST have a value the profile supports, and under `astro`, `html` MUST be `true` (`model-consumer-unsupported`). `site` MUST be an absolute `http` or `https` URL with no path, query, or fragment (`model-consumer-site`). `base-path` MUST start with `/` (`model-consumer-base-path`), and so MUST `permalink-path` (`model-consumer-permalink-path`), whose missing final `/` is added; the site output's `_ascribe/permalinks.json` gives it as `path` ([output-layout contract](../output-layout.md) §1.2). `agents` left out MUST be read as `true` when `site` is set and `false` when it isn't; an explicit `agents = true` MUST come with `site` (`model-consumer-agents-site`). `host` MUST be `"netlify"` (`model-invalid-value`); with it, the compiler MUST write the host's files in each build's `_ascribe/host/` ([output-layout contract](../output-layout.md) §1.9), and without it MUST NOT. The compiler MUST write each build's `_ascribe/headers.json` (§1.4) and `_ascribe/redirects.json` (§1.8) whatever it says. `headers` isn't a key: it's `model-unknown-key`, whose message names `host`.

**Heading ids, image attributes, and assets** (SPEC §9.5) have no keys: the profile fixes them. Under `astro`, heading ids and image attributes MUST be written as the `<ascribe-attributes>` markers of the [site-render contract](../site-render.md), and assets placed as the [asset contract](../assets.md) says.

**Routes** (the `astro` profile, verified against Astro). A page's route MUST be, in order: `base-path`; the page's path from the content root without `.md`, each segment slugged as Astro's content loader slugs an entry id, and without a final `index` segment (`guides/index.md` is `/guides/`); and a final `/` when `trailing-slash` is `"always"`. The route of the root `index.md`, entry id `index`, is `base-path`, without its final `/` under `"never"` unless it's `/`. When two pages have one entry id (`My File.md` and `my-file.md`, or `index.md` and `index/index.md`), `ascribe build --emit site` MUST fail, naming both. `link-route` and its fix MUST map a route back to its page with the same router. Source files contain no routes (SPEC §5.2).

**The outputs for agents** (`agents`, `true` by default with a `site`). The plain-markdown output becomes what a site publishes for agents, laid out so that copying it to `base-path` publishes it (contracts/output-layout.md):

- Each page's file is its route's path with `.md`: Astro's entry id with `.md` added (`Guides/My Setup.md` → `guides/my-setup.md`, `guides/index.md` → `guides.md`, the root `index.md` → `index.md`). It opens with the blockquote `> For the complete documentation index, see [llms.txt](../<url>).` and a blank line, `<url>` being `llms.txt`'s absolute URL.
- Files the pages link to are copied under `_ascribe/files/`, mirroring their source paths, and linked by absolute URL; the manifest gives each its root-relative `url`.
- `llms.txt` is the index of every page the build publishes and offers, in the sections of the build's navigation (the [navigation contract](../navigation.md)): `# <name>`, a blank line, `> <summary>`, then a section each, `## <heading>` followed by a line per page, `- [<title>](../<url>)` or `- [<title>](../<url>): <description>`, with `<url>` the absolute URL of the page's Markdown. `<name>` and `<summary>` are the title and description of the page whose entry id is `index`; without one, the site's host and a fixed sentence. `<title>` is the page's title, never its `nav-label`. A page's description is the field its type marks `role = "description"` (§5.4), as plain text on one line (its Markdown's text: a link is its text). The first section is the content root's own page and its pages, in order, headed by the root's label, "Start" unless `ascribe.nav` gives another; a page beside a top folder with its name isn't among them, but first in that folder's section. Then each top section, in order, with its own page first and its nested sections' pages folded in, in reading order, headed by its `label` from `ascribe.nav`, else its own page's title, else the folder's name with `-` and `_` as spaces and its first letter capitalized. A section with no page to list is left out.
- When `llms.txt` would be longer than 50,000 characters, it's split: the root file keeps the header and the root's section, and adds `## Sections` with a line per top section linking to `<segment>/llms.txt`, `<segment>` being the first segment of its pages' entry ids, described by its own page's description or "N pages". Each `<segment>/llms.txt` is a whole index of its own: the section's heading as its name, its own page's description (or a sentence naming the site and the section) as its summary, and the section. A file still over the limit is written, and `ascribe check` reports it (`llms-section-large`).


@include: ../../_fragments/content-model/consumer-astro-settings.md

---

## 16. `[builds.<name>]`

@include: ../../_fragments/content-model/builds.md

**Rules** (§25.7): build names MUST differ ignoring case (`model-build-name-case`). A versioned build's target MUST have a live line in `[releases]`, and the build MUST NOT also filter (`model-build-versions`). A filter's version MUST be given exactly when its target is versioned (`model-build-filter-version`). A build that filters for a target its own selection excludes is a warning (`model-build-filter-excluded`).

---

## 17. `[listing]`

@include: ../../_fragments/content-model/listing.md

**Rules** (§25.12): each value MUST be one of `listed`, `unquoted`, and `unlisted` (`model-invalid-value`). A key of `[listing.folders]` MUST be a path from the content root ending in `/`, without `.` or `..` parts or a pattern's characters, of a folder that exists by its exact name (`model-listing-folder`, a filesystem rule), and a key of `[listing.types]` a content type the model declares, or `page` when it declares none (`model-listing-type`). A rule MUST NOT be looser than a folder's it's under, or than `default` (`model-listing-looser`), and the table MUST have no other key (`model-unknown-key`). A page looser than a rule it's under is checked with the project, not by the loader: SPEC §8.2's `listing-loosened`.

`older-lines` is held to `default` by `model-listing-looser`, as a folder's and a type's rules are. A page looser than `older-lines`, which only an older line's pages are under, is SPEC §8.2's `listing-loosened-in-line`.

---

## 18. `[crawlers]`

@include: ../../_fragments/content-model/crawlers.md

**Rules** (§25.11): each of `lines` MUST be a string (`model-wrong-type`) of one line (`model-crawlers-line`), not empty (`model-empty-text`). `training`, `search`, and `on-request` MUST each be `"allow"` or `"disallow"` (`model-invalid-value`). A crawler a `User-agent` line of `lines` names, compared without case, MUST NOT be one a disallowed purpose names, nor one an earlier group of `lines` names (`model-crawlers-named-twice`), where consecutive `User-agent` lines open one group and `*` is never counted. The table MUST have no other key (`model-unknown-key`). A line is otherwise not validated: one that isn't a `User-agent`, `Allow`, or `Disallow` line MUST be written as it is, and `ascribe check` reads only the lines it can.

The crawlers of each purpose are data Ascribe ships, each with its operator, the operator's page about it, and the day it was read; a release MAY add to them. A disallowed purpose's group MUST name every one Ascribe knows of it.

---

## 19. `[permalinks]`

None. A permalink's record, retired or moved, is an entry of `ascribe.addresses` keyed `go:<name>` (SPEC §7.10), and `[permalinks]` is a replaced key: a loader MUST report it as an error at the key (`model-unknown-key`), whose fix says that `ascribe addresses init` moves its entries.
