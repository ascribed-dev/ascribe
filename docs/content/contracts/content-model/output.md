---
type: reference
title: "`ascribe.toml` format contract: consumer, builds, and sources"
nav-label: "Consumer, builds, and sources"
description: "The normative contract for the content model's consumer, builds, editor, and sources."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section. What these keys are for, with examples, is in the reference's [consumer, builds, and sources](../../content-model/output.md).

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

## 17. `[editor]`

@include: ../../_fragments/content-model/editor.md

---

## 18. `[sources.<name>]`

A source is a folder outside the content root, or another repository, that a page's snippets MAY read (SPEC §4.8, §7.3, §7.4); a page MUST NOT read a file outside the project's folder any other way. `<name>` is the source's name (§1.2), and a snippet's address starts with it.

@include: ../../_generated/content-model-sources.md

**Rules** (§25.8): a source MUST have exactly one of `path` and `git`, `branch` only with `git`, a URL `git` can be given, and a branch name `git` accepts (`model-source-remote`). `path` MUST be relative (`model-path-absolute`) and name a directory that exists (`model-source-path-missing`), inside the git repository the project is in, when it's in one (`model-source-outside-repository`). With no repository, the language server and `ascribe mcp` MUST read a source only inside what the user opened, a folder of the workspace or the folder `ascribe mcp` was started in, and report one outside it as `model-source-outside-repository` with the variant `opened`; a command reads a source wherever `path` leads. With `git`, `sources/` MUST be outside the content root (`model-source-remote`).

### 18.1 `ascribe.lock` and the copies

A source with `git` is pinned to a commit in `ascribe.lock`, beside `ascribe.toml`, and the files its snippets use are copied into `sources/<name>/` at the paths they have in their repository. So are the repository's `ascribe.examples.toml` and every file in its `examples.observations/`, when it has them, whatever the source's `include` says ([tested examples](../tested-examples.md#1-where-they-are)). Both are written by `ascribe sources fetch` and `ascribe sources update`, and committed. `check` and `build` read only the copies.

```toml
# Written by `ascribe sources fetch` and `ascribe sources update`. Don't edit it by hand.
version = 1

[[source]]
name = "api"
git = "https://github.com/acme/api.git"
commit = "9f2c41d0e0c4a1b2c3d4e5f60718293a4b5c6d7e"
missing = ["examples/quickstart.ts"]

[source.files]
"src/auth.rs" = "sha256:5d41402abc4b2a76b9719d911017c592ae2fd2b1f6b3b0f0d5ce64b4c7d0e1a2"
```

| Key | Type | Description |
|---|---|---|
| `version` | integer | The lock's format: `1`. |
| `source[].name` | string | The source's name. Each source is pinned once. |
| `source[].git` | string | The source's `git` when it was pinned. |
| `source[].commit` | string | The commit, in full: 40 or 64 hexadecimal digits. |
| `source[].files` | table | Each copy's path, relative to the source's folder, and the SHA-256 of its bytes: `sha256:` and 64 lowercase hexadecimal digits. |
| `source[].missing` | array of strings | Optional. The paths, relative to the source's folder, that snippets named and that weren't in the repository at the commit, so weren't copied. No path is in both `files` and `missing`. |

`check` reads the lock and the copies with the source files, and reports what doesn't match: a lock it can't read (`lock-invalid`); a pin of a source `ascribe.toml` doesn't declare with `git`, or declares with another `git` (`lock-source-unknown`); a copy that's missing or doesn't have its hash (`source-copy-changed`); a file in `sources/<name>/` the lock doesn't list (`source-copy-unlocked`); and a copy no snippet uses (`source-copy-unused`, a warning), which a tested examples' config or observation never is. A snippet that names a file with no copy is `snippet-file-missing`: its message says to run `ascribe sources fetch`, or, for a file the lock lists in `missing`, that the file isn't in the repository at the pinned commit, which `fetch` can't change. These are source-file diagnostics; see [the diagnostics reference](../../checks/diagnostics.md).
