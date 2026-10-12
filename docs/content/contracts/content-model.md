---
title: "`ascribe.toml` format contract"
description: The normative contract for the content model file, with the rules a loader enforces and the decisions behind them.
---

This is the normative contract for `ascribe.toml`, the content model file (SPEC §7). It defines every table and key, the short syntax for field and attribute types, and every rule a loader enforces. The loader in `ascribe-model` implements it. For a guide to writing one, see the [`ascribe.toml` reference](../reference/content-model.md).

SPEC.md is normative for the language. This document is normative for the file format. Where it settles something the spec leaves open, the item is listed in [Decisions](#26-decisions).

Example files, each valid under this reference:

- [`examples/content-models/minimal.toml`]({repo}/blob/main/examples/content-models/minimal.toml): the smallest valid model.
- [`examples/content-models/quill.toml`]({repo}/blob/main/examples/content-models/quill.toml): the model SPEC Appendix B assumes.
- [`examples/content-models/full.toml`]({repo}/blob/main/examples/content-models/full.toml): every section and key.

## Contents

1. [Conventions](#1-conventions)
2. [Top level: `spec`](#2-top-level-spec)
3. [`[project]`](#3-project)
4. [`[types.<name>]` and `[fragments]`](#4-typesname-and-fragments)
5. [Field and attribute types](#5-field-and-attribute-types)
6. [`[dimensions.<name>]`](#6-dimensionsname)
7. [`[versions]`](#7-versions)
8. [`[lifecycle.<state>]`](#8-lifecyclestate)
9. [`[features.<key>]`](#9-featureskey)
10. [`[notes.<type>]`](#10-notestype)
11. [`[phrases]`](#11-phrases)
12. [`[glossary]`](#12-glossary)
13. [`[images]`](#13-images)
14. [`[widgets.<name>]`](#14-widgetsname)
15. [`[consumer]`](#15-consumer)
16. [`[builds.<name>]`](#16-buildsname)
17. [`[editor]`](#17-editor)
18. [`[sources.<name>]`](#18-sourcesname)
19. [`[checks]`](#19-checks)
20. [`[permalinks]`](#20-permalinks)
21. [`[crawlers]`](#21-crawlers)
22. [`[listing]`](#22-listing)
23. [Defaults: what an absent section means](#23-defaults-what-an-absent-section-means)
24. [Validation rules](#24-validation-rules)
25. [Map of SPEC §7.2 to this file](#25-map-of-spec-72-to-this-file)
26. [Decisions](#26-decisions)

---

## 1. Conventions

@include: ../_fragments/content-model/conventions.md

---

## 2. Top level: `spec`

@include: ../_fragments/content-model/spec.md

---

## 3. `[project]`

@include: ../_fragments/content-model/project.md

| Key | Type | Default | Description |
|---|---|---|---|
| `content-root` | string (path) | `"docs"` | The content root (SPEC §2.2): the directory holding every source file. Paths in links and includes that begin with `/` are relative to it. It MUST exist and be a directory. |
| `output-dir` | string (path) | `".ascribe/build"` | Where `ascribe build` writes output. Each build and emitter writes under `<output-dir>/<build>/<emitter>/` (see the [output-layout contract](output-layout.md)). It need not exist. |

**Rules** (§24): both paths are relative (`model-path-absolute`). The output directory MUST NOT be inside the content root, the content root MUST NOT be inside the output directory, and they MUST NOT be the same directory (`model-output-overlaps-content`); otherwise a build would read its own output as source, or delete source as stale output. Paths are compared after normalizing `.` and `..` segments and, when both exist, after resolving symbolic links.

@include: ../_fragments/content-model/project-defaults.md

---

## 4. `[types.<name>]` and `[fragments]`

@include: ../_fragments/content-model/types.md

**The page title.** Every page type MUST declare `title` as a required `string` field (`model-type-title`). The frontmatter `title` is the page's title wherever the spec needs one, such as the replacement text of an empty link to a page (SPEC §5.2). The field MAY accept phrases (§11), and MAY set `inline = "code"` for code spans in the title (§5.3).

**Reserved keys.** `available` (SPEC §4.4), `variant` (SPEC §4.3), `permalink` (SPEC §2.1), and `listing` (SPEC §2.1) are reserved frontmatter keys. Every page accepts them, with the meaning the spec gives, whether or not its type mentions them, and a type MUST NOT declare them (`model-field-reserved`). Generated consumer schemas include them automatically, except `listing`, which reaches a site in the page's record (output-layout contract §1.3). `intended` (SPEC §4.9) is reserved too, and every page and every fragment accepts it; no output has it, so generated consumer schemas leave it out, and a type MUST NOT declare it (`model-field-reserved`). Under the `astro` profile, `slug` is reserved too: Astro's content loader uses a page's frontmatter `slug` as its entry id in place of its path, which would publish the page at a URL Ascribe never computed, so a type MUST NOT declare it (`model-field-reserved`). `formatted` is reserved on page types as well: the site output writes the formatted form of `inline` fields under it (§5.3), so a type MUST NOT declare it (`model-field-reserved`). A page's own frontmatter can't use it either, since no type declares it. `source` is reserved on page types for the same reason: the site output writes each page's path from the content root under it (SPEC §9.6), so a type MUST NOT declare it (`model-field-reserved`), and a page can't write it.

@include: ../_fragments/content-model/types-frontmatter-and-fragments.md

**Reserved keys on fragments.** The spec defines `available`, `variant`, `permalink`, and `listing` for pages only. A fragment's frontmatter MUST NOT use them (an error on the fragment), and `[fragments.frontmatter]` MUST NOT declare them (`model-field-reserved`). Use `@available` inside the fragment instead. A fragment MAY use `intended`, and `[fragments.frontmatter]` MUST NOT declare it (`model-field-reserved`).

---

## 5. Field and attribute types

@include: ../_fragments/content-model/field-types.md

| Key | Type | Default | Description |
|---|---|---|---|
| `type` | string | **required** | A field type or attribute type (§5.1). In table form it may also be `object`, `list(object)`, or bare `enum`, with the keys below. |
| `fields` | table of fields | required when `type` is `object`, `object?`, `list(object)`, or `list(object)?`; not allowed otherwise | The nested fields of an object, in the same syntax (short or table form). Field types only. |
| `values` | array of strings | required when `type` uses bare `enum`; not allowed otherwise | The enumeration's values, for values that aren't simple words, such as `["Getting started", "How-to"]`. Values MUST be distinct. Values of a `set(enum)` attribute MUST be tokens. |
| `default` | any TOML value | none | The value used when the field or attribute is absent. It MUST have the declared type (a TOML string for `string` and `enum`, integer or float for `number`, boolean for `boolean`, local date for `date`, array for `list`, inline table for `object`, a string or array of strings for `set`). A field with a default is optional; adding `?` as well is allowed and changes nothing. |
| `phrases` | boolean | `false` | Whether phrases (SPEC §5.1) are substituted in this field's value. Field types only; allowed only on `string` and `list(string)` fields (and their optional forms), including fields nested in objects. See §11. |
| `inline` | string | none | The inline markup the value is read with. The one value is `"code"`: code spans. Allowed only on a content type's top-level `string` fields (and `string?`), not in objects or the fragment schema. See §5.3. |
| `description` | string | none | Help text shown by the editor (hover and completion) and emitted into generated schemas as documentation. |
| `role` | string | none | What the field is for, for the checks that read it (SPEC §7.2): `"description"`, `"review-date"`, or `"owner"`. Allowed only on a content type's top-level fields, not in objects or the fragment schema. See §5.4. |

@include: ../_fragments/content-model/field-types-nesting.md

### 5.3 Inline code in a field

A `string` field is plain text: a backtick in it is a backtick. A field that sets `inline = "code"` is read as inline Markdown that knows only code spans, so a page title can show code:

@include: ../_fragments/content-model/inline-code-example.md

```yaml
title: "`ascribe.toml` reference"
```

- **Code spans** are CommonMark's (CommonMark §6.1): a run of backticks opens one, the next run of the same length closes it, and a run with no closer is literal. Their content is normalized as CommonMark does: line endings become spaces, and one space is stripped from each end when both ends have one.
- **Escapes.** Outside code spans, a backslash before ASCII punctuation escapes it, as in CommonMark, so `` \` `` is a literal backtick. Inside them, a backslash is literal.
- **Nothing else is markup.** Emphasis, links, and HTML stay literal text. A title becomes the text of other links (empty link text, SPEC §5.2), where a link or emphasis would cause trouble.
- **Phrases** (`phrases = true`) are substituted in the text, never inside code spans, as in prose (SPEC §5.1).

Each output carries two forms of the value. The **plain text** is the value without markup: the code spans' content, with no backticks (`ascribe.toml reference`). The **formatted form** keeps the code spans. Where each goes:

| Output | Plain text | Formatted form |
|---|---|---|
| Site output (its frontmatter) | The field's frontmatter value, for a layout's `<title>`, search, and sorting | `formatted.<field>`, as HTML (`<code>ascribe.toml</code> reference`), for a layout's heading and navigation |
| Empty link text (SPEC §5.2) | Not used | The link text, with the title's code spans |
| Plain-markdown output | Not used | The page's level-1 heading |
| JSON output | `title` and `frontmatter.<field>` | `formatted.<field>`, a list of `{ "type": "text" \| "code", "value": … }` |
| Generated Zod schema | The field, as `z.string()` | `formatted`, with a `z.string()` for each field |

A field the page leaves out gets its default's formatted form, so `formatted.<field>` is there whenever the field has a value. The setting is per field, not on for every `string`, so a backtick in an existing field keeps its meaning.

---

### 5.4 Field roles
@available: next

A page type's top-level field MAY have a `role`, which says what the field is for, so a check or the report reads it whatever the field's name (SPEC §7.2):

| Role | The field's type | Read by |
|---|---|---|
| `description` | `string` or `string?` | `page-description-missing`, which reports a page of the type whose field is absent, null, or blank. A field that's required or has a default is never absent, so the check skips it. |
| `review-date` | `date` or `date?` | `review-overdue`, which reports a page whose date is before the day the check runs. |
| `owner` | `string`, `string?`, or an `enum`, optional or not | `ascribe report inventory`, which counts pages by the field's value; a page without one counts as having no owner. |

A type without a field with the role has none of that check's diagnostics. A role changes no output: not the generated schemas, the JSON, or the pages.

**Rules** (§24.3): the role is one of the three, on a field of its type, in a content type's own fields, and at most one field of a type has each role (`model-field-role`). A second field with a role keeps its type, and loses the role.

---

## 6. `[dimensions.<name>]`

@include: ../_fragments/content-model/dimensions.md

**Rules.** A value MUST belong to only one dimension (`model-dimension-value-shared`); otherwise a bare target in an availability spec would be ambiguous. Dimension names and values also take part in the one-role rule (§24).

---

## 7. `[versions]`

@include: ../_fragments/content-model/versions.md

| Key | Type | Default | Description |
|---|---|---|---|
| `scheme` | string | `"numeric"` | The version scheme. Spec 0.1 defines one: `"numeric"`. |
| `current` | string | none | The latest release: a version (SPEC Appendix A rule `version`), compared under `scheme`. Nothing in an output depends on it. The content check `availability-left-behind` reports a feature whose spec names a version for every target, none of them later than `current`; without `current` it reports nothing. |

@include: ../_fragments/content-model/versions-numeric.md

No other scheme is defined. The table exists so a later spec version can add one without changing the file's shape. The scheme is named `numeric`, not "semantic versioning", because it accepts any number of components and has no pre-release syntax.

---

## 8. `[lifecycle.<state>]`

@include: ../_fragments/content-model/lifecycle.md

---

## 9. `[features.<key>]`

@include: ../_fragments/content-model/features.md

---

## 10. `[notes.<type>]`

@include: ../_fragments/content-model/notes.md

---

## 11. `[phrases]`

@include: ../_fragments/content-model/phrases.md

---

## 12. `[glossary]`

@include: ../_fragments/content-model/glossary.md

**Matching**: occurrences match whole words only; the longest matching term wins where terms overlap (`API key` over `API`); matching applies to prose only, never to headings, link text, code, raw HTML, or text inside a directive's primary identifier. A term with no `link` isn't linked in the site or plain-markdown output; its definition still reaches the editor and the JSON output. SPEC §5.4 says only that occurrences are linked to definitions.

@include: ../_fragments/content-model/glossary-rules.md

---

## 13. `[images]`

@include: ../_fragments/content-model/images.md

---

## 14. `[widgets.<name>]`

@include: ../_fragments/content-model/widgets.md

**Rules** (§24):

@include: ../_fragments/content-model/widgets-rules.md

---

## 15. `[consumer]`

@include: ../_fragments/content-model/consumer.md

| Key | Type | Default | Description |
|---|---|---|---|
| `profile` | string | `"astro"` | The consumer profile. Supported: `"astro"`. |
| `site` | string (URL) | none | The published site's origin, such as `"https://docs.quill.dev"`: an `http` or `https` URL with no path, query, or fragment. The plain-markdown output needs it to write absolute links (SPEC §9.4). Without it, plain-markdown links are root-relative (they start with `base-path`) and `ascribe build` warns. |
| `base-path` | string | `"/"` | **Routing.** The URL path every route starts with, such as `"/docs/"`, including any locale prefix (`"/en/"`). It MUST start with `/`. A trailing `/` is optional and doesn't change the meaning. |
| `trailing-slash` | string | `"always"` | **Routing.** Whether page URLs end in `/`: `"always"` (`/guides/setup/`) or `"never"` (`/guides/setup`). Match the consumer's own setting (Astro's `trailingSlash` and `build.format`). |
| `slugger` | string | `"github"` | **Slugging.** The algorithm for heading ids (SPEC §5.5), which must be the one the consumer uses. Supported: `"github"`, a port of `github-slugger`, which Astro uses. |
| `html` | boolean | `true` | **HTML passthrough.** Whether the consumer renders raw HTML in markdown. The site output's custom elements need it, so the `astro` profile supports only `true`. |
| `agents` {available=next} | boolean | `false` | **Outputs for agents.** Whether each build also publishes what AI agents read, following the Web Documentation Delivery Spec 0.6.0: `llms.txt` and each page's Markdown version, in the plain-markdown output laid out by URL, and a pointer to them on each page of the site output. It requires `site`. See below. |
| `permalink-path` {available=next} | string | `go/` under `base-path` | **Permalinks.** The path each permalink's address is published under (SPEC §4.1), from the site's root: an address is the path, the permalink's id, and a final `/` when `trailing-slash` is `"always"`. It MUST start with `/` (`model-consumer-permalink-path`); a missing final `/` is added. The site output's `_ascribe/permalinks.json` gives it as `path` (contracts/output-layout.md §1.2). |

**Heading ids, image attributes, and assets** (SPEC §9.5) are part of the profile, not keys. The `astro` profile has exactly one way to do each, defined by the contracts:

- **Heading ids and image attributes:** the site output writes each as a `<ascribe-attributes>` marker that the consumer's markdown plugin applies, so the consumer keeps its own heading, table-of-contents, and image processing. See [`contracts/site-render.md`](site-render.md).
- **Assets:** copies mirror their source paths inside each output, and images are referenced relatively so Astro's image processing still applies. Other files a page links to are published under `_ascribe/files/`. See [`contracts/assets.md`](assets.md).

A later profile that offers a choice adds a key for it then.

**How file paths become routes** (the `astro` profile, verified against Astro): a page's route is `base-path`, then its path relative to the content root with the `.md` extension removed and each segment slugged the way Astro's content loader computes entry ids; a final `index` segment is dropped (`guides/index.md` → `/guides/`); then the trailing slash per `trailing-slash`. The root `index.md` (Astro's entry id `index`) is at `base-path`, which under `trailing-slash = "never"` loses its final `/` unless it's `/`. Two pages with one entry id (`My File.md` and `my-file.md`, or `index.md` and `index/index.md`) can't both be published, and `ascribe build --emit site` fails, naming them. The same router answers the reverse question, which page a route-like link names, for the `link-route` warning and its fix. Source files never contain routes (SPEC §5.2).

**The outputs for agents** (`agents = true`; decision 64). The plain-markdown output becomes what a site publishes for agents, laid out so that copying it to `base-path` publishes it (contracts/output-layout.md):

- Each page's file is its route's path with `.md`: Astro's entry id with `.md` added (`Guides/My Setup.md` → `guides/my-setup.md`, `guides/index.md` → `guides.md`, the root `index.md` → `index.md`). It opens with the blockquote `> For the complete documentation index, see [llms.txt](<url>).` and a blank line, `<url>` being `llms.txt`'s absolute URL.
- Files the pages link to are copied under `_ascribe/files/`, mirroring their source paths, and linked by absolute URL; the manifest gives each its root-relative `url`.
- `llms.txt` is the index of every page the build publishes, in page order: `# <name>`, a blank line, `> <summary>`, then a section per folder, each `## <heading>` followed by a line per page, `- [<title>](<url>)` or `- [<title>](<url>): <description>`, with `<url>` the absolute URL of the page's Markdown. `<name>` and `<summary>` are the title and description of the page whose entry id is `index`; without one, the site's host and a fixed sentence. A page's description is the field its type marks `role = "description"` (§5.4), as plain text on one line (its Markdown's text: a link is its text). Pages directly in the content root are the section "Pages", first, with the `index` page first among them; the rest are grouped by their entry id's first segment, in order of first appearance, each headed by the title of the folder's index page (entry id equal to the segment), which is listed first, or else the folder's name with `-` and `_` as spaces and its first letter capitalized.
- When `llms.txt` would be longer than 50,000 characters, it's split: the root file keeps the header and "Pages", and adds `## Sections` with a line per folder linking to `<segment>/llms.txt`, described by the folder's index page's description or "N pages". Each `<segment>/llms.txt` is a whole index of its own: the section's heading as its name, the folder's index page's description (or a sentence naming the site and the section) as its summary, and the section. A file still over the limit is written, and `ascribe check` reports it (`llms-section-large`).

Without `site` the model is invalid (`model-consumer-agents-site`), since every URL above is absolute.

@include: ../_fragments/content-model/consumer-astro-settings.md

---

## 16. `[builds.<name>]`

@include: ../_fragments/content-model/builds.md

---

## 17. `[editor]`

@include: ../_fragments/content-model/editor.md

---

## 18. `[sources.<name>]`
@available: ascribe 0.2.0

A source is a folder outside the project's content, or another repository, that its pages may take code examples from (SPEC §4.8, §7.3, §7.4). It's the only way a page can read a file outside the project's folder.

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["crates/**", "examples/**"]    # what's readable; everything else isn't
ignore = ["**/target/**"]

[sources.api]
git = "https://github.com/acme/api.git"
branch = "main"
include = ["src/**"]
```

`<name>` is the source's name (§1.2), which a snippet's address starts with: `code:examples/quill/ascribe.toml#dimensions`.

| Key | Type | Default | Description |
|---|---|---|---|
| `path` | string (path) | **required** without `git` | The source's folder, relative to the project root. It MUST exist and be a directory, and, when the project is in a git repository, it MUST be inside that repository. |
| `git` | string (URL) | **required** without `path` | Another repository: `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path`. Its files are read from copies in `sources/<name>/`, beside `ascribe.toml`. |
| `branch` | string | the repository's default branch | The branch `ascribe sources update` moves the pin to. Only with `git`. |
| `include` | array of patterns | every file | The files a snippet may read, matched against their paths relative to the source's folder (§1.3). |
| `ignore` | array of patterns | none | Files left out even when `include` matches them. |

A source has `path` or `git`, never both.

### 18.1 `ascribe.lock` and the copies

A source with `git` is pinned to a commit in `ascribe.lock`, beside `ascribe.toml`, and the files its snippets use are copied into `sources/<name>/` at the paths they have in their repository. Both are written by `ascribe sources fetch` and `ascribe sources update`, and committed. `check` and `build` read only the copies.

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

`check` reads the lock and the copies with the source files, and reports what doesn't match: a lock it can't read (`lock-invalid`); a pin of a source `ascribe.toml` doesn't declare with `git`, or declares with another `git` (`lock-source-unknown`); a copy that's missing or doesn't have its hash (`source-copy-changed`); a file in `sources/<name>/` the lock doesn't list (`source-copy-unlocked`); and a copy no snippet uses (`source-copy-unused`, a warning). A snippet that names a file with no copy is `snippet-file-missing`: its message says to run `ascribe sources fetch`, or, for a file the lock lists in `missing`, that the file isn't in the repository at the pinned commit, which `fetch` can't change. These are source-file diagnostics; see [the diagnostics reference](../reference/diagnostics.md).

**Rules** (§24.8): `path` is relative (`model-path-absolute`), and names an existing directory (`model-source-path-missing`) inside the project's repository (`model-source-outside-repository`). {available=next} With no repository around the project, the language server and `ascribe mcp` are bounded by what the user opened: a folder of the editor's workspace, or the folder `ascribe mcp` was started in. A source outside it is `model-source-outside-repository` with the variant `opened`. A command, such as `ascribe build` or `ascribe check`, reads a source wherever `path` leads, so a copy of a repository without `.git` still builds. A source has exactly one of `path` and `git`, `branch` only with `git`, a URL and a branch name `git` accepts, and copies outside the content root (`model-source-remote`).

---

## 19. `[checks]`
@available: next

How loudly each check about the content's quality speaks. A diagnostic about whether the project is valid can't be named here: its severity is the registry's. The two forms a key's value takes are:

```toml
[checks]
code-language-missing = "warning"     # a level

[checks.page-size]                    # or a table: the level, and the check's own settings
level = "off"
limit = 40_000
```

| Key | Type | Default | Description |
|---|---|---|---|
| `<check>` | string or table | the check's own level | A check's slug, as the [diagnostics reference](../reference/diagnostics.md) lists it. The value is a level: `"off"` (not reported), `"advice"`, `"warning"`, or `"error"`. Or it's a table with an optional `level` and the check's own settings, which the reference lists with the check; a table without `level` keeps the check's own level. |

Only a check the diagnostics registry marks configurable can be named; the [diagnostics reference](../reference/diagnostics.md#content-checks) lists them. The settings a check's table takes:

| Check | Setting | Type | Default | Description |
|---|---|---|---|---|
| `page-size` | `limit` | integer | `50000` | The size, in characters of a page's plain Markdown in a build, at which the page is reported. It MUST be above 0. |
| `image-large` | `limit` | integer or string | `"500 KB"` | The size over which an image file is reported: a whole number of bytes, or a number followed by `B`, `KB`, or `MB` (a kilobyte is 1,000 bytes, a megabyte 1,000,000), with or without a space. |

The names `vale` and `links` are reserved: they're tables of settings for the tools Ascribe runs, not checks, and no check has either name. `vale` is §19.2, and `links` §19.3.

A level changes how a diagnostic is reported everywhere: by `ascribe check` and `ascribe build`, and in the editor. Advice is shown, and never fails `ascribe check`, even with `--deny-warnings`; a check set to `off` isn't reported at all.

**Rules** (§24.9): every key is a check's slug, with a did-you-mean suggestion for one that isn't (`model-unknown-key`); the check is configurable (`model-check-not-configurable`); a level is one of the four (`model-invalid-value`); a table has only `level` and the check's settings (`model-unknown-key`); a value is a string or a table, `page-size`'s `limit` a whole number above 0, and `image-large`'s a size (`model-wrong-type`).

### 19.1 `[[intended]]`
@available: next

Acknowledges a review check's problem with a content model entry or an image as intended (SPEC §4.9): the problem isn't reported, and is counted and listed with the reason instead. Each table is one acknowledgement.

@include: ../_fragments/content-model/intended-example.md

| Key | Type | Default | Description |
|---|---|---|---|
| `check` | string | required | A check's slug. The registry gives it `next = "review"` and `place = "entry"`. |
| `reason` | string | required | Why the problem is intended. Not empty. |
| `phrase` | string | | A phrase's key, declared in `[phrases]`. |
| `feature` | string | | A feature's key, declared in `[features]`. |
| `term` | string | | A glossary term's id, declared in `[glossary]`. |
| `image` | string | | An image's path under the content root. |

Exactly one of `phrase`, `feature`, `term`, and `image`. The acknowledgement covers the check's problems that name that entry, in every build. One that covers nothing when every build is checked is advice (`intended-unused`), reported at its `[[intended]]` header with a fix that removes the table.

**Rules** (§24.9): the check exists, with a did-you-mean suggestion for one that doesn't, is a review check, and is reported at an entry (`model-intended-check`); the table names exactly one declared entry (`model-intended-entry`); `check` and `reason` are present (`model-missing-key`) and `reason` isn't empty (`model-empty-text`); and the table has no other key (`model-unknown-key`).

### 19.2 `[checks.vale]`
@available: next

How the prose is checked with Vale, a program the project installs. Ascribe runs it; it isn't linked or downloaded.

```toml
[checks.vale]
preset = "quiet"              # or: config = ".vale.ini"
off = ["Ascribe.Repeated"]    # rules of the preset to turn off
command = "vale"
in-check = false
max-level = "error"
```

| Key | Type | Default | Description |
|---|---|---|---|
| `preset` | string | | One of Ascribe's presets: `quiet`. A preset's name never changes once released; a rule added to it is a **Behavior change** in the changelog. |
| `config` | string | | The project's own `.vale.ini`, a path from the project root. |
| `off` | array of strings | `[]` | Rules of the preset to turn off, as `<Style>.<Rule>`. Only with `preset`. |
| `command` | string | `"vale"` | The command run. A value with a path separator is a path from the project root; a bare name is looked up on the `PATH`. A relative path, or an absolute one into the project's folder, is a program in the project, which the language server runs only once the person allows it, and the agents' hook never runs (decision 76). |
| `in-check` | boolean | `false` | Whether `ascribe check` checks the prose without `--vale`. |
| `max-level` | string | `"error"` | `"advice"`, `"warning"`, or `"error"`: the most a Vale alert is reported as. Vale's `suggestion`, `warning`, and `error` map to advice, warning, and error, capped at this. |

What Ascribe writes for Vale is under `.ascribe/vale/` in the project root: a preset's configuration and styles, and the vocabulary `Ascribe` of the project's own words (phrase values, glossary terms and aliases, dimension, value, and feature labels), one escaped regular expression to a line. Nothing outside it is written, except by `ascribe vale eject`.

Each alert is a `prose` diagnostic (`ASC165`) on the source file, with the rule as its JSON `rule`; one inside a phrase's text is at the phrase. A Vale that can't be run, fails, or runs out of time is one `prose-not-checked` advice (`ASC166`) at `[checks.vale]`, never a failed check, and so is one the caller doesn't start: the MCP server never starts Vale, the agents' hook never starts a program in the project, and the language server starts one only once the person allows it. Both are configurable in `[checks]`, and `[checks]` sets their level after `max-level` caps Vale's.

**Rules** (§24.9): exactly one of `preset` and `config` (`model-checks-vale`); `off` only with `preset`, and only naming its rules (`model-checks-vale`); `preset` and `max-level` take one of their values (`model-invalid-value`); no other key (`model-unknown-key`).

### 19.3 `[checks.links]`
@available: next

How `ascribe report links` checks external links: with lychee, a program the project installs, which Ascribe runs with the addresses on its standard input. `ascribe check` never reaches the network.

```toml
[checks.links]
command = "lychee"
ignore = ["localhost", "*.internal.example.com"]
```

| Key | Type | Default | Description |
|---|---|---|---|
| `command` | string | `"lychee"` | The command run, in the project root. A value with a path separator is a path from the project root; a bare name is looked up on the `PATH`. |
| `ignore` | array of strings | `[]` | Hosts not checked. `name` matches that host; `*.name` matches every host that ends in `.name`, and not `name` itself. Matched without regard to case, after a trailing `.` is dropped. |
| `reuse-days` | integer | `14` | How many days an answer is reused. An address that answered with nothing to report isn't asked about again while its answer is younger than its own limit: a number of days from half of `reuse-days`, rounded up, through all of it, fixed by a hash of the address. `0` reuses nothing. |
| `confirm-days` | integer | `3` | How many days a failure is held. A link that fails, other than with `404` or `410`, whose address the history records answering before, is unconfirmed, not a diagnostic, until it has failed on every run for `confirm-days` days from its first failure since that answer. `0` holds nothing. |

Answers are kept in `.ascribe/cache/links/` in the project's folder, one file an address, keyed by the address exactly as lychee is given it and a hash of the project's `lychee.toml`. Only an answer with nothing to report is kept, so every diagnostic in a report comes from an address asked about in that run. A file that can't be read, has another version, or holds another key counts as no answer. `ascribe check`, `build`, `diff`, `drift`, `fmt`, the language server, and the agents' hook never read or write the folder.

The history is `.ascribe/history/links.json` in the project's folder: one file, version 1, sorted, written whole through a rename after each run, holding for each address the pages link to the last day it answered, the day it started failing, and whether the run reported it. An address no longer linked to is dropped. A run that reads the file again before writing it and finds another run's writing keeps, for each address, the record that confirms its failure later. A file that can't be read or has another version is no history, and with none, or with `--no-history`, every failure is a diagnostic. A permanent redirect is an answer. An `@intended {check=link-external-broken}` covers an unconfirmed link as it covers a reported one. `ascribe check`, `build`, `diff`, `drift`, `fmt`, the language server, and the agents' hook never read or write it.

The links checked are every link whose destination is an `http` or `https` URL, in every source file, each address once. A link whose address answers with an error, doesn't answer in time, or can't be reached is a `link-external-broken` advice (`ASC168`, `next = "review"`, `place = "block"`); one whose redirects begin with a `301` or `308` is a `link-external-moved` advice (`ASC169`, `next = "choose"`), with an unsafe fix that writes the address the permanent redirects end at. Both are at the link, and configurable in `[checks]`. An `@intended {check=link-external-broken}` that covers no broken link is `intended-unused` only when `ascribe report links` ran; `ascribe check` doesn't report it.

**Rules** (§24.9): each `ignore` entry is a host name, or `*.` and one (`model-wrong-type`); `command` is a string, and `reuse-days` and `confirm-days` integers of 0 or more (`model-wrong-type`); no other key (`model-unknown-key`).

---

## 20. `[permalinks]`
@available: next

Names for something outside the docs to link to that no heading or page declares (SPEC §7.6): a permalink ended on purpose, and a second name for one.

```toml
[permalinks.retired]
rotate-keys = { to = "guides/security.md#keys", reason = "Merged into the security guide" }
legacy-sync = { reason = "The sync agent was removed in 4.0" }

[permalinks.aliases]
agent-settings = "settings"
```

| Key | Type | Default | Description |
|---|---|---|---|
| `retired.<name>` | table | | A retired permalink. `reason` (string, required, not empty) is why, which a reader whose link leads there sees. `to` (string) is where its address leads now: a page's path from the content root, and optionally `#` and the id of a heading on it. |
| `aliases.<name>` | string | | An alias: the name it leads to, a permalink a heading or page declares, or another alias that leads to one. |

No other key is allowed in `[permalinks]` or in a retired permalink's table. A name has the characters of an `@id`. `ascribe permalinks retire`, and the editor's quick fix on `permalink-removed`, write `[permalinks.retired]` in place; the quick fix's reason is the placeholder `why this permalink was retired`, which is an error until it's replaced.

**Rules** (§24.10): each name is valid (`model-invalid-name`); a retired permalink has a `reason` (`model-missing-key`), not empty (`model-empty-text`) and not the placeholder (`model-permalinks`), and no other key (`model-unknown-key`); no name is both retired and an alias, and no alias leads back to itself (`model-permalinks`). What needs the content is checked with the project, not by the loader: SPEC §8.2's `permalink-name-conflict`, `permalink-alias-target-missing`, and `permalink-retired-target-missing`.

---

## 21. `[crawlers]`
@available: next

@include: ../_fragments/content-model/crawlers.md

**Rules** (§24.11): each line is a string (`model-wrong-type`), one line (`model-crawlers-line`), and not empty (`model-empty-text`); no other key (`model-unknown-key`). What a line says isn't validated: a line that isn't a `User-agent`, `Allow`, or `Disallow` line is written as it is, and `ascribe check` reads only the lines it can.

---

## 22. `[listing]`
@available: next

@include: ../_fragments/content-model/listing.md

**Rules** (§24.12): each value is one of `listed`, `unquoted`, and `unlisted` (`model-invalid-value`); a folder is a path from the content root ending in `/`, without `.` or `..` parts or a pattern's characters, of a folder that exists by its exact name (`model-listing-folder`, a filesystem rule); a type is a content type the model declares, or `page` when it declares none (`model-listing-type`); no rule is looser than a folder's it's under, or than `default` (`model-listing-looser`); no other key (`model-unknown-key`). A page that's looser than a rule it's under is checked with the project, not by the loader: SPEC §8.2's `listing-loosened`.

---

## 23. Defaults: what an absent section means

@include: ../_fragments/content-model/defaults.md

---

## 24. Validation rules

A loader MUST enforce every rule below when it loads `ascribe.toml`, and report each violation at the span of the offending key or value. Every rule is an error unless marked **warning**. A model with errors doesn't load, and no document is checked against it; warnings don't stop loading.

- **Slugs** are stable identifiers, for tests and for the diagnostics registry, [`tests/conformance/diagnostics.toml`]({repo}/blob/main/tests/conformance/diagnostics.toml), which gives each one a code. Only `model-name-multiple-roles` corresponds to a row of SPEC §8.2 ("Content model"); the rest are loader rules this reference adds.
- **Messages** are templates. `{name}` is a placeholder. Where a rule has several messages, each covers one case of it.
- Rules about documents, such as a page matching no content type or an unknown frontmatter key, aren't loader rules.
- **Filesystem rules** (`model-content-root-missing`, `model-glossary-link`, `model-source-path-missing`, `model-source-outside-repository`, and `model-listing-folder` for a folder that isn't there) need the project directory. A loader given only the file's text, such as an unsaved editor buffer or a unit test, skips them; `ascribe check` and the language server always run them.

### 24.1 File and structure

| Slug | Rule | Message |
|---|---|---|
| `model-toml-syntax` | The file is valid TOML 1.0. | `` ascribe.toml isn't valid TOML: {detail} `` |
| `model-unknown-key` | Every key is one this reference defines for its table, except in tables keyed by names the project chooses. | `` unknown key `{key}` in `[{table}]` ``<br>`` unknown key `{key}` in `[{table}]`; did you mean `{suggestion}`? `` |
| `model-missing-key` | Every required key is present. | `` `[{table}]` is missing the required key `{key}` `` |
| `model-wrong-type` | Every value has the TOML type its key requires. | `` `{key}` must be {expected}, but it's {found} ``<br>`` `{key}` must be a string, but it's a number; quote it: {key} = "{value}" `` |
| `model-invalid-value` | A key with a fixed set of values has one of them. | `` `{key}` can't be "{value}"; use one of: {values} `` |
| `model-spec-unsupported` | `spec` is a version this processor implements. | `` ascribe.toml targets spec version "{spec}", but this processor implements {supported} `` |
| `model-invalid-name` | Every name follows its grammar (§1.2). | `` `{name}` isn't a valid {role} name: {rule} ``, for example `` `Deployment` isn't a valid dimension name: use a lowercase letter, then lowercase letters, digits, or hyphens `` |
| `model-empty-text` | Every `label`, `name`, `term`, `definition`, and `description` is non-empty. | `` `{key}` can't be empty `` |

### 24.2 Project

| Slug | Rule | Message |
|---|---|---|
| `model-path-absolute` | `content-root`, `output-dir`, and each source's `path` are relative paths. | `` `{key}` must be a path relative to ascribe.toml, not an absolute path `` |
| `model-content-root-missing` | The content root exists and is a directory. | `` content root `{path}` doesn't exist ``<br>`` content root `{path}` isn't a directory `` |
| `model-output-overlaps-content` | The output directory isn't inside the content root, the content root isn't inside the output directory, and they differ (derived from the spec). | `` output directory `{output}` is inside content root `{content}`; move it outside, or builds will read their own output as source ``<br>`` content root `{content}` is inside output directory `{output}`; builds could delete source files as stale output ``<br>`` output directory and content root are both `{path}` `` |

### 24.3 Content types, fragments, and fields

| Slug | Rule | Message |
|---|---|---|
| `model-type-multiple-defaults` | At most one type sets `default = true`. | `` only one content type can be the default, but `{a}` and `{b}` both set default = true `` |
| `model-type-unreachable` | Every type has `files` or `default = true`. | `` content type `{type}` has no `files` and isn't the default, so no page can use it `` |
| `model-type-title` | Every page type declares `title` as a required `string`. | `` content type `{type}` must declare title = "string": it's the page title, used for empty link text ``<br>`` content type `{type}`: `title` must be a required string, not "{found}" `` |
| `model-field-reserved` | No content type or fragment schema declares `available`, `variant`, `permalink`, or `listing`, and no content type declares `slug`, `formatted`, or `source`. | `` `{field}` is reserved by the Ascribe spec and every page accepts it; remove it from `[types.{type}.frontmatter]` ``<br>`` `{field}` is reserved by the Ascribe spec, and fragments can't use it ``<br>`` `slug` is reserved by the astro profile, which uses it as a page's URL id; remove it from `[types.{type}.frontmatter]` ``<br>`` `formatted` is reserved by the site output, which writes the formatted form of `inline` fields under it; remove it from `[types.{type}.frontmatter]` ``<br>`` `source` is reserved by the site output, which writes each page's path from the content root under it; remove it from `[types.{type}.frontmatter]` `` |
| `model-type-syntax` | Every field and attribute type parses under §5.1, and is allowed where it's used: no `date`, `list`, or `object` for attributes; no `set` for fields; `object`, `list(object)`, and bare `enum` only in table form. | `` "{type}" isn't a valid {kind} type: {detail} ``, for example `` "strng?" isn't a valid field type: expected string, number, boolean, date, enum(…), or list(…) `` |
| `model-type-fields` | `fields` is present exactly when the type is `object` or `list(object)`, optional or not. | `` field `{field}` is an object, so it needs `fields` ``<br>`` `fields` is only allowed on object fields, and `{field}` is "{type}" `` |
| `model-enum-values` | An enumeration has at least one value and no duplicates, and its values come from exactly one of `enum(…)` and `values`. `values` appears only with bare `enum`. | `` `{field}` has an empty enumeration ``<br>`` "{value}" appears twice in the enumeration for `{field}` ``<br>`` `{field}`: list enumeration values in enum(…) or in `values`, not both ``<br>`` `{field}` is a bare enum, so it needs `values` `` |
| `model-set-token` | Every value of a `set(enum)` attribute is a token (SPEC §3.3). | `` "{value}" can't be in a value set: members can't contain spaces or any of , \| { } = " `` |
| `model-default-type` | A `default` has the declared type, and an enumeration default is one of its values. | `` default for `{field}` must be {type}, but it's {found} ``<br>`` default "{value}" for `{field}` isn't one of: {values} `` |
| `model-phrases-field-type` | `phrases = true` is set only on `string` and `list(string)` fields. | `` phrases = true only works on string and list(string) fields, and `{field}` is "{type}" `` |
| `model-inline-field` | `inline` is `"code"`, and is set only on a content type's top-level `string` fields (§5.3). | `` inline = "code" only works on string fields, and `{field}` is "{type}" ``<br>`` inline must be "code", not "{value}" ``<br>`` inline = "code" only works on a content type's top-level fields, and `{field}` is in {place} `` |
| `model-pattern-syntax` | Every pattern parses under §1.3, doesn't start with `/`, and has no `..` segment. | `` "{pattern}" isn't a valid pattern: {detail} ``<br>`` pattern "{pattern}" is already relative to the content root; remove the leading / ``<br>`` pattern "{pattern}" can't contain .. `` |
| `model-field-role` | `role` is `"description"`, on a `string` field, `"review-date"`, on a `date` field, or `"owner"`, on a `string` or `enum` field; only on a content type's top-level fields; and a type has at most one field with each role (§5.4). | `` role = "{role}" only works on a {expected} field, and `{field}` is "{type}" ``<br>`` role must be "description", "review-date", or "owner", not "{value}" ``<br>`` role only works on a content type's top-level fields, and `{field}` is in {place} ``<br>`` `{other}` already has role = "{role}"; a content type has one field with each role `` |
| `model-attribute-reserved` | No image or widget attribute key is one HTML already gives a meaning on that element (SPEC §7.2): `src`, `alt`, or `title` on images; `heading` or `primary` on widgets; and on both, HTML's global attributes (such as `id`, `class`, `style`, and `title`), any key starting with `aria-`, and HTML's event-handler attributes (such as `onclick`, `onload`, and `onerror`). The lists are explicit, in `ascribe_core::reserved`, so keys that merely begin with `on`, such as `online` or `only-if`, are allowed. | `` `{key}` can't be an image attribute: HTML already uses it on the <img> element ``<br>`` `{key}` can't be an attribute of widget `{name}`: the site output already uses it on the widget's element `` |

### 24.4 Dimensions, names, lifecycle, notes, and features

| Slug | Rule | Message |
|---|---|---|
| `model-name-multiple-roles` | A name is used in at most one of these roles: dimension name, dimension value, lifecycle state (including built-in states), feature key (SPEC §7.2, §8.2). | `` `{name}` is used as both {role-a} and {role-b}; a name can have only one role, so availability specs stay unambiguous ``, for example `` `beta` is used as both a dimension value (in dimensions.channel) and a lifecycle state (built in); a name can have only one role, so availability specs stay unambiguous `` |
| `model-name-case` (**warning**) | No two names in those roles differ only in case. | `` `{a}` and `{b}` differ only in case; names are case-sensitive, so they're easy to confuse `` |
| `model-dimension-empty` | `values` has at least one value. | `` dimension `{dimension}` has no values `` |
| `model-dimension-value-duplicate` | No value appears twice in one dimension. | `` `{value}` appears twice in dimensions.{dimension}.values `` |
| `model-dimension-value-shared` | No value belongs to two dimensions. | `` `{value}` is a value of both `{a}` and `{b}`; a value can belong to only one dimension `` |
| `model-label-undeclared` | Every key in a dimension's `labels` is one of its values. | `` dimensions.{dimension}.labels has a label for `{value}`, which isn't one of its values: {values} `` |
| `model-versionless-undeclared` | Every entry in a dimension's `versionless` is one of its values. | `` dimensions.{dimension}.versionless lists `{value}`, which isn't one of its values: {values} `` |
| `model-lifecycle-available-required` | A new lifecycle state sets `available`. | `` new lifecycle state `{state}` must set available = true or available = false `` |
| `model-lifecycle-ga-unavailable` | `ga` counts as available. | `` `ga` must count as available: content with no lifecycle state is ga `` |
| `model-note-label-required` | A new note type sets `label`. | `` new note type `{type}` needs a label, such as label = "{Type}" `` |
| `model-availability-syntax` | A feature's `available` parses as an availability spec (SPEC Appendix A `availability`). | `` feature `{key}`: "{spec}" isn't a valid availability spec: {detail} `` |
| `model-availability-unknown-name` | Every target in a feature's spec is a declared dimension value or dimension name, and every state is a declared lifecycle state. | `` feature `{key}`: `{name}` isn't a declared dimension value or dimension name ``<br>`` feature `{key}`: `{name}` isn't a declared lifecycle state `` |
| `model-availability-versionless` | A versionless target, or a dimension name, has no versions in a feature's spec (SPEC §4.4). | `` feature `{key}`: `{target}` is versionless, so it takes a state but no version ``<br>`` feature `{key}`: `{target}` is a dimension name, so it takes a state but no version; name one of its values, such as `{example}`, to give a version `` |
| `model-availability-history-order` | A history in a feature's spec is in chronological order under the version scheme. | `` feature `{key}`: the history for `{target}` must be in chronological order, but {later} comes before {earlier} `` |
| `model-feature-nested` | A feature's `available` isn't itself a feature key. | `` feature `{key}`: available must be an availability spec, not another feature (`{other}`) `` |

### 24.5 Versions, phrases, glossary, and images

| Slug | Rule | Message |
|---|---|---|
| `model-phrase-value-type` | Every phrase value is a string. | `` phrase `{key}` must be a quoted string, but it's {found}; write {key} = "{value}" `` |
| `model-glossary-duplicate-term` | No two terms or aliases are the same text, ignoring case when either is case-insensitive. | `` "{text}" is declared by both glossary terms `{a}` and `{b}` `` |
| `model-glossary-link` | A term's `link` file exists under the content root and isn't a fragment. | `` glossary term `{id}` links to {path}, which doesn't exist ``<br>`` glossary term `{id}` links to {path}, which is a fragment; link to a page that includes it `` |

`[versions] scheme` is covered by `model-invalid-value`, and `[versions] current`, which MUST be a version, by `model-wrong-type`. Image attribute types are covered by the rules in §24.3.

### 24.6 Widgets

| Slug | Rule | Message |
|---|---|---|
| `model-widget-reserved-name` | The name doesn't start with `ascribe-` and isn't one of HTML's reserved custom-element names. | `` widget name `{name}` is reserved: names starting with ascribe- belong to Ascribe's element library ``<br>`` widget name `{name}` is reserved by HTML and can't be a custom element `` |
| `model-widget-forms` | `forms` is non-empty, has no duplicates, and contains only `"line"` and `"container"`. | `` forms must be ["line"], ["container"], or ["line", "container"] `` |
| `model-widget-binding` | `binding` is present when `forms` includes `"line"`, and absent otherwise. | `` widget `{name}` has a line form, so it needs a binding: "self", "heading", "block", or "heading-or-block" ``<br>`` widget `{name}` is container-only, and a container holds its own content; remove binding `` |
| `model-widget-container-primary` | With container form, the primary isn't required; a container-only widget's primary is `"none"`. | `` widget `{name}` has a container form, whose opener has no primary, so its primary can't be required; use "{kind}?" ``<br>`` widget `{name}` is container-only, so it can't take a primary `` |
| `model-widget-groupable-form` | A groupable widget is container-only. | `` widget `{name}` is groupable, so it must be container-only: forms = ["container"] `` |
| `model-widget-plain-content` | `plain-content` is set only on widgets that wrap content: container form, or binding `block` or `heading-or-block`. | `` widget `{name}` doesn't wrap content, so plain-content has no effect; remove it `` |

`primary`, `title`, and `binding` values are covered by `model-invalid-value`; attribute types by §24.3.

### 24.7 Consumer, builds, and editor

| Slug | Rule | Message |
|---|---|---|
| `model-consumer-unsupported` | Every consumer key has a value the profile supports (for example, `html = false` with `astro`). | `` the {profile} profile doesn't support {key} = {value}; use {values} `` |
| `model-consumer-site` | `site` is an absolute `http` or `https` URL with no path, query, or fragment. | `` site must be an origin such as "https://docs.example.com"; put any path in base-path `` |
| `model-consumer-base-path` | `base-path` starts with `/`. | `` base-path must start with "/", such as "/docs/" `` |
| `model-consumer-agents-site` | `agents = true` comes with `site`. | `` agents = true needs `site`: every link in llms.txt and the Markdown pages is an absolute URL `` |
| `model-build-name-case` | Build names are unique ignoring case. | `` builds `{a}` and `{b}` differ only in case, so they'd share an output directory on some file systems `` |
| `model-build-variants` | `variants` is `"switch"` or a non-empty table of dimension selections, each a value or a non-empty array of values. | `` variants must be "switch" or a selection such as { deployment = "cloud" } ``<br>`` build `{build}` selects nothing; write variants = "switch" `` |
| `model-build-unknown-dimension` | Every dimension a selection names is declared. | `` build `{build}` selects dimension `{dimension}`, which isn't declared `` |
| `model-build-unknown-value` | Every selected value is a value of its dimension. | `` build `{build}`: `{value}` isn't a value of `{dimension}`; values: {values} `` |
| `model-build-availability` | `availability` is `"badge"` or a table with exactly one key, `filter`. | `` availability must be "badge" or { filter = "<target> <version>" } `` |
| `model-build-filter-target` | The filter's target is a declared dimension value. | `` build `{build}` filters for `{target}`, which isn't a declared dimension value ``<br>`` build `{build}` filters for `{target}`, which is a dimension; filter for one of its values: {values} `` |
| `model-build-filter-version` | The filter has a version exactly when its target is versioned, and the version follows SPEC Appendix A's `version` rule. | `` build `{build}` filters for `{target}`, which is versioned, so it needs a version, such as "{target} 3.3" ``<br>`` build `{build}`: `{target}` is versionless, so the filter can't name a version ``<br>`` build `{build}`: "{version}" isn't a valid version `` |
| `model-build-filter-excluded` (**warning**) | A build's filter target isn't a value its own selection drops. | `` build `{build}` filters for `{target}`, but its selection keeps only {dimension} = {values}, so pages marked for `{target}` are dropped `` |
| `model-editor-build-unknown` | `editor.build` names a declared build. | `` editor.build is `{build}`, which isn't a declared build; builds: {builds} `` |
| `model-editor-build-required` | When there are several builds and none is named `site`, `editor.build` is set. | `` there are several builds and none is named site; set [editor] build to the one the editor should check `` |

### 24.8 Sources
@available: ascribe 0.2.0

| Slug | Rule | Message |
|---|---|---|
| `model-source-path-missing` | A source's `path` exists and is a directory. | `` the folder of source `{source}`, `{path}`, doesn't exist ``<br>`` the folder of source `{source}`, `{path}`, isn't a directory `` |
| `model-source-outside-repository` | A source's folder is inside the git repository the project is in, when it's in one; with none, in the language server and `ascribe mcp`, inside what the user opened (variant `opened`). | `` the folder of source `{source}`, `{path}`, is outside the git repository the project is in `` |
| `model-source-remote` | A source has `path` or `git`, not both; `branch` only with `git`; a URL `git` can be given; a branch name `git` accepts; and, with `git`, copies outside the content root. | `` source `{source}` has both `path` and `git`: a source is a folder in this repository or another repository, not both ``<br>`` source `{source}` needs `path`, a folder in this repository, or `git`, another repository's URL ``<br>`` source `{source}` has `branch` but no `git`: a branch is only for a source in another repository ``<br>`` `{url}` isn't a URL Ascribe can give `git`: use `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path` ``<br>`` `{branch}` isn't a branch name `git` accepts ``<br>`` the copies of source `{source}` go in `sources/{source}/`, which is inside the content root `{content}`, where they'd be read as pages; give the content root a folder of its own `` |

### 24.9 Checks
@available: next

| Slug | Rule | Message |
|---|---|---|
| `model-check-not-configurable` | `[checks]` names only checks the diagnostics registry marks configurable. | `` `{check}` can't be set in [checks]: it reports whether the project is valid, which a project can't lower or turn off `` |
| `model-intended-check` | `[[intended]]` names a review check that's acknowledged at a content model entry. | `` `{check}` isn't a check `` |
| `model-intended-entry` | `[[intended]]` names exactly one declared phrase, feature, glossary term, or image. | `` an acknowledgement in [[intended]] names what it's about with one of `phrase`, `feature`, `term`, or `image` `` |
| `model-checks-vale` | `[checks.vale]` has exactly one of `preset` and `config`, and `off` goes with `preset` and names its rules. | `` `[checks.vale]` has both `preset` and `config`; pick one: Ascribe's preset, or the project's own `.vale.ini` `` |

### 24.10 Permalinks
@available: next

| Slug | Rule | Message |
|---|---|---|
| `model-permalinks` | A name in `[permalinks]` is retired or an alias, not both; an alias leads, through aliases, to a name that isn't one; a retired permalink's reason isn't the quick fix's placeholder. | `` `{name}` is both retired and an alias; a name is one thing `` |

### 24.11 Crawlers
@available: next

| Slug | Rule | Message |
|---|---|---|
| `model-crawlers-line` | Each of `[crawlers] lines` is one line, with no line break. | `` `{key}` has a line break; each string in `lines` is one line of robots.txt `` |

### 24.12 Listing
@available: next

| Slug | Rule | Message |
|---|---|---|
| `model-listing-folder` | A key of `[listing.folders]` is a folder's path from the content root, ending in `/`, of a folder that exists by its exact name. | `` `{folder}` in `[listing.folders]` doesn't exist under the content root ``<br>`` `{folder}` in `[listing.folders]` isn't a folder ``<br>`` `{folder}` in `[listing.folders]` isn't a folder's path; write it from the content root, with `/` between its parts and at its end, such as `internal/` `` |
| `model-listing-type` | A key of `[listing.types]` is a content type. | `` `{type}` in `[listing.types]` isn't a content type; use one of: {values} `` |
| `model-listing-looser` | No rule is looser than a folder's it's under, or than `default`. | `` `{rule}` is looser than `{wider}` on line {line}, which it's under; a narrower rule can only restrict ``<br>`` `{rule}` is looser than the site's `{wider}` on line {line}; with that default, no folder or type can be listed more openly `` |

---

## 25. Map of SPEC §7.2 to this file

Every declaration SPEC §7.2 lists, and every setting SPEC §9.3 and §9.5 need, maps to one section:

| SPEC declaration | Section of `ascribe.toml` | Here |
|---|---|---|
| Spec version (§11) | `spec` | [§2](#2-top-level-spec) |
| Content root (§2.2); output directory (§9.4) | `[project]` | [§3](#3-project) |
| Content types: page schemas and the fragment schema (§2.1, §2.2) | `[types.<name>]`, `[fragments.frontmatter]` | [§4](#4-typesname-and-fragments) |
| Field roles (§7.2) | The `role` field option | [§5.4](#54-field-roles) |
| Fragment patterns (§2.2) | `[fragments] patterns` | [§4.3](#43-fragments) |
| Field types (§7.2) | Field type syntax | [§5](#5-field-and-attribute-types) |
| Directive schemas for project widgets (§3, §6) | `[widgets.<name>]` | [§14](#14-widgetsname) |
| Dimensions: names, values, display labels, versionless values (§4.3, §4.4) | `[dimensions.<name>]` | [§6](#6-dimensionsname) |
| Version scheme (§4.4) | `[versions]` | [§7](#7-versions) |
| Lifecycle states (§4.4) | `[lifecycle.<state>]` | [§8](#8-lifecyclestate) |
| Features registry (§4.4) | `[features.<key>]` | [§9](#9-featureskey) |
| Note types (§4.5) | `[notes.<type>]` | [§10](#10-notestype) |
| Phrases registry; phrases in frontmatter (§5.1) | `[phrases]`; the `phrases` field option | [§11](#11-phrases) |
| Glossary (§5.4) | `[glossary]` | [§12](#12-glossary) |
| Image attributes (§5.3) | `[images.attributes]` | [§13](#13-images) |
| Consumer profile (§9.5) | `[consumer]` | [§15](#15-consumer) |
| Builds (§9.3) | `[builds.<name>]` | [§16](#16-buildsname) |
| Name roles (§7.2, last paragraph) | Validation rule `model-name-multiple-roles` | [§24](#24-validation-rules) |
| The editor's default build | `[editor]` | [§17](#17-editor) |
| Sources: the folders outside the content a snippet may read (§4.8, §7.3) | `[sources.<name>]` | [§18](#18-sourcesname) |
| Acknowledgements of a content model entry's or an image's problems (§4.9) | `[[intended]]` | [§19.1](#191-intended) |
| Retired permalinks and aliases (§7.6) | `[permalinks]` | [§20](#20-permalinks) |
| The project's own `robots.txt` lines (§7.2, §9.4) | `[crawlers]` | [§21](#21-crawlers) |
| How the site's, a folder's, or a type's pages are listed (§2.1, §7.2) | `[listing]` | [§22](#22-listing) |

---

## 26. Decisions

Each item settles a gap in SPEC.md. All 21 were decided on 2026-09-28 as recommended below, except item 12, which was settled when the site-render and asset contracts were written. Items 1, 2, 3, 4, and 6 are now also stated in SPEC.md (§2.1, §5.2, §7.2, §8.2). Each item keeps the alternatives that were considered.

1. **Frontmatter diagnostics (SPEC §8.2).** §8.1 says file-level validation covers frontmatter, but §8.2 has no rows for it. *Decision:* add file-level error rows: unknown frontmatter key; missing required field; value doesn't match the field's type; reserved key (`available`, `variant`) on a fragment; page matches more than one content type; page matches no content type and there's no default.
2. **Assigning content types to pages (SPEC §7.2).** The spec doesn't say how a page gets its type. *Decision:* `files` patterns per type plus at most one `default = true` type; a page matching several types is an error, with no precedence. *Considered:* first match in file order (TOML tables are formally unordered); most specific pattern (hard to define); a frontmatter `type` key (would need a new reserved key).
3. **Page titles (SPEC §5.2).** Empty link text uses "the page title", but the spec doesn't say where it comes from. *Decision:* the frontmatter `title`, which every page type must declare as a required string. *Considered:* fall back to the first level-1 heading.
4. **Reserved keys in fragments (SPEC §2.1, §4.3, §4.4).** `available` and `variant` are defined for pages. *Decision:* fragments can't use them in spec 0.1; use `@available` inside the fragment. *Considered:* fragment `available` applies to everything the fragment contributes, like a section spec.
5. **Version scheme name (SPEC §4.4).** *Decision:* call the one scheme `numeric`: dotted numbers of any length, compared numerically with missing components as 0. The grammar has no pre-release syntax, so "semver" would overpromise. *Considered:* call it `semver` and cap versions at three components, which needs a new document diagnostic.
6. **Name rules beyond the one-role rule (SPEC §4.3, §4.4, §7.2).** *Decision:* (a) a dimension value belongs to only one dimension, since `cloud` in a spec must mean one thing; (b) dimension names follow the `key` rule, since they're attribute keys; (c) warn on names that differ only in case. (a) and (b) are stated in SPEC §7.2.
7. **Glossary (SPEC §5.4).** The spec gives no format or matching rules. *Decision:* terms with a required plain-text `definition` and an optional `link`; occurrences link to `link`; terms without it aren't linked in site or plain output. Whole-word matching, longest term wins, prose only (not headings, link text, or code), `first` per resolved page by default. No term element is needed: in the site output an occurrence is an ordinary link whose title is the definition, marked with `data-ascribe-term` (see the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md)).
8. **Widget plain fallback (SPEC §6, §9.4).** *Decision:* a static CommonMark string with phrases substituted and no attribute interpolation (no behavior); a widget that wraps content keeps that content in plain output unless `plain-content = "drop"`, since silently losing content is worse than showing it. The spec's "or nothing" then applies to the widget itself, not its content.
9. **Widget schema constraints (SPEC §3.5, §3.6, §6).** *Decision:* groupable widgets are container-only; widgets with container form have no required primary; names starting with `ascribe-` and HTML's reserved custom-element names are rejected.
10. **Absolute links in plain output (SPEC §9.4).** Plain-markdown links are "absolute URLs", which needs the site's origin. *Decision:* optional `[consumer] site`; without it, links are root-relative and `ascribe build` warns.
11. **HTML passthrough (SPEC §9.5).** The site output depends on raw HTML (custom elements). *Decision:* keep the key, but the `astro` profile accepts only `true` until a profile needs `false`, rather than defining a degraded site output now.
12. **Heading ids, image attributes, and asset placement (SPEC §9.4, §9.5).** *Decision:* no keys. The profile named by `profile` fixes all three, and the `astro` profile has one way to do each: a `<ascribe-attributes>` marker for heading ids and image attributes ([site-render contract](site-render.md)), and mirrored asset copies with relative image references ([asset contract](assets.md)). A key that accepts one value says nothing, and since unknown keys are errors, adding a key when a second profile needs a choice breaks no existing file, while removing one later would. *Considered:* the keys first drafted, `heading-ids` (`"attribute"` or `"html"`), `image-attributes` (`"attribute"` or `"html"`), `assets` (`"beside-page"` or `"directory"`), and `assets-dir`. Their alternatives were dropped: an `{#id}` attribute block is rewritten by Astro's default smartypants and GFM processing before a plugin sees it (quotes, `--` in ids), which the marker avoids; a raw HTML heading loses the consumer's inline processing and, in Astro, its table-of-contents entry; a raw `<img>` bypasses Astro's image processing; and a shared asset directory needs hashed names to avoid collisions, which mirroring avoids by construction. With them went the loader rule `model-consumer-assets-dir`.
13. **Filter builds on versioned targets (SPEC §9.3).** A filter is "given a target and, for versioned targets, a version". *Decision:* the version is required for versioned targets, and not allowed for versionless ones. *Considered:* a versioned target with no version means "at every version", but then the "state in effect" is undefined.
14. **Project defaults.** *Decision:* `content-root = "docs"`, `output-dir = ".ascribe/build"`; paths relative to `ascribe.toml`, `..` allowed, absolute paths rejected (keeps projects portable). The content root can't be `"."` by default, because the output directory couldn't then sit outside it.
15. **YAML flavor for frontmatter.** *Decision:* the YAML 1.2 core schema (`yes` is a string, `3.10` is a number), and `date` fields accept `YYYY-MM-DD` scalars, quoted or not.
16. **The editor's default build.** *Decision:* the only build, else the build named `site`, else required.
17. **Built-in lifecycle states and note types.** *Decision:* built-ins can be relabeled, and states' `available` flags changed, but not removed; `ga` must stay available; new states must set `available` explicitly; new note types need a `label`; built-in states take part in the one-role rule.
18. **Implicit type and build.** *Decision:* with no `[types]`, one default type `page` with a required `title`; with no `[builds]`, one build `site` (`switch`, `badge`). Declaring any type or build removes the implicit one.
19. **Features referring to features (SPEC §4.4).** *Decision:* not allowed, so there are no chains or cycles.
20. **Phrases in frontmatter (SPEC §5.1).** *Decision:* opt in per field with `phrases = true`, on `string` and `list(string)` fields only; off by default.
21. **Spec version matching (SPEC §11).** *Decision:* `spec` is a quoted string that must exactly equal a version the processor implements; `"0.1"` for now. Revisit compatibility ranges when 0.2 exists.
22. **Code in a page title (SPEC §7.2).** *Decision:* opt in per field with `inline = "code"`, on a content type's top-level `string` fields, and code spans only (§5.3). Every output carries the plain text and the formatted form. *Considered:* reading code spans in every title, which would change the meaning of every existing title with a backtick and make `title` the one string field that isn't plain text; and links or emphasis in titles, which go wrong where a title becomes the text of another link.
23. **Fields the checks read (SPEC §7.2).** A check of a page's description or its review date needs to know which field holds it, and a project names its fields. *Decision:* a `role` on the field (§5.4), so the type that has the field says what it is, and a type without one opts out of the check. *Considered:* a `field` setting in each check's `[checks]` table, which names one field for every type, so two types with different names for their description can't both be checked, and a type without the field is reported as missing it; and fixed names, such as `description`, which take a name from every project.
