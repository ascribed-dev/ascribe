---
title: "`ascribe.toml` reference"
description: "The content model: content types, dimensions, availability, phrases, widgets, and builds."
permalink: content-model
---

`ascribe.toml` is a project's content model: its schema (SPEC §7). It sits at the project root and declares the content types and their frontmatter, the dimensions content varies along, phrases, the glossary, project widgets, how the site output fits Astro, and the builds. The editor, `ascribe check`, and `ascribe build` all read it, so they can't disagree about what's valid.

A file containing only `spec = "0.1"` is valid; every section has a default ([§21](#21-defaults-what-an-absent-section-means)). Example files:

- [`examples/content-models/minimal.toml`]({repo}/blob/main/examples/content-models/minimal.toml): the smallest valid model.
- [`examples/quill/ascribe.toml`]({repo}/blob/main/examples/quill/ascribe.toml): a small, complete project.
- [`examples/content-models/full.toml`]({repo}/blob/main/examples/content-models/full.toml): every section and key.

Every problem the loader reports is in the [diagnostics reference](diagnostics.md#the-content-model), with its fix. References to "SPEC" are to the [Ascribe specification]({repo}/blob/main/SPEC.md).

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
21. [Defaults: what an absent section means](#21-defaults-what-an-absent-section-means)
22. [Validation](#22-validation)

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
| `content-root` | string (path) | `"docs"` | The content root ([SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)): the directory holding every source file. Paths in links and includes that begin with `/` are relative to it. It must exist and be a directory. |
| `output-dir` | string (path) | `".ascribe/build"` | Where `ascribe build` writes output. Each build and emitter writes under `<output-dir>/<build>/<emitter>/`. It need not exist. It may be outside the project's folder, with `..`, and a build then writes there. A build never writes or removes through a symbolic link on the way to it, or in it ([output-layout contract](../contracts/output-layout.md#2-what-ascribe-owns)). |

**Rules:** both paths are relative (`model-path-absolute`). The output directory must not be inside the content root, the content root must not be inside the output directory, and they must not be the same directory (`model-output-overlaps-content`); otherwise a build would read its own output as source, or delete source as stale output. Paths are compared after normalizing `.` and `..` segments and, when both exist, after resolving symbolic links.

@include: ../_fragments/content-model/project-defaults.md

---

## 4. `[types.<name>]` and `[fragments]`

@include: ../_fragments/content-model/types.md

**The page title.** Every page type must declare `title` as a required `string` field (`model-type-title`). The frontmatter `title` is the page's title wherever the spec needs one, such as the replacement text of an empty link to a page ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)). The field may accept phrases (§11), and code spans (§5.3).

**Reserved keys.** `available` ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)), `variant` ([SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)), and `permalink` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files), a page's [permalink](directives.md#permalinks)) are reserved frontmatter keys. Every page accepts them, with the meaning the spec gives, whether or not its type mentions them, and a type must not declare them (`model-field-reserved`). Generated consumer schemas include them automatically. `intended` ([SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)) is reserved too, and accepted by every page and every fragment: it [acknowledges a check's problem](directives.md#intended), and since no output has it, generated consumer schemas leave it out. Under the `astro` profile, `slug` is reserved too: Astro's content loader uses a page's frontmatter `slug` as its entry id in place of its path, which would publish the page at a URL Ascribe never computed, so a type must not declare it (`model-field-reserved`).

@available: ascribe 0.2.0
`formatted` is reserved on page types too: the site output writes the formatted form of fields that set `inline` under it (§5.3), so a type must not declare it (`model-field-reserved`).

@available: next
`source` is reserved on page types too: the site output writes each page's path from the content root under it, for a layout's edit link, so a type must not declare it (`model-field-reserved`).

@include: ../_fragments/content-model/types-frontmatter-and-fragments.md

**Reserved keys on fragments.** The spec defines `available`, `variant`, and `permalink` for pages only. A fragment's frontmatter must not use them (an error on the fragment), and `[fragments.frontmatter]` must not declare them (`model-field-reserved`). Use `@available` inside the fragment instead. `intended` is for fragments too, and `[fragments.frontmatter]` must not declare it either.

---

## 5. Field and attribute types

@include: ../_fragments/content-model/field-types.md

| Key | Type | Default | Description |
|---|---|---|---|
| `type` | string | **required** | A field type or attribute type (§5.1). In table form it may also be `object`, `list(object)`, or bare `enum`, with the keys below. |
| `fields` | table of fields | required when `type` is `object`, `object?`, `list(object)`, or `list(object)?`; not allowed otherwise | The nested fields of an object, in the same syntax (short or table form). Field types only. |
| `values` | array of strings | required when `type` uses bare `enum`; not allowed otherwise | The enumeration's values, for values that aren't simple words, such as `["Getting started", "How-to"]`. Values must be distinct. Values of a `set(enum)` attribute must be tokens. |
| `default` | any TOML value | none | The value used when the field or attribute is absent. It must have the declared type (a TOML string for `string` and `enum`, integer or float for `number`, boolean for `boolean`, local date for `date`, array for `list`, inline table for `object`, a string or array of strings for `set`). A field with a default is optional; adding `?` as well is allowed and changes nothing. |
| `phrases` | boolean | `false` | Whether phrases ([SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)) are substituted in this field's value. Field types only; allowed only on `string` and `list(string)` fields (and their optional forms), including fields nested in objects. See §11. |
| `inline` | string | none | `"code"` reads the value's code spans. Allowed only on a content type's top-level `string` fields (and `string?`). See §5.3. |
| `description` | string | none | Help text shown by the editor (hover and completion) and emitted into generated schemas as documentation. |
| `role` | string | none | What the field is for, which a check reads: `"description"`, `"review-date"`, or `"owner"`. Allowed only on a content type's top-level fields. See §5.4. |

@include: ../_fragments/content-model/field-types-nesting.md

### 5.3 Code in a field

@available: ascribe 0.2.0
A `string` field is plain text: a backtick in it is a backtick. To show code in a page title, set `inline = "code"` on the field:

@include: ../_fragments/content-model/inline-code-example.md

```yaml
---
title: "`ascribe.toml` reference"
---
```

The value is then read as Markdown that knows only code spans. A run of backticks opens a code span, and the next run of the same length closes it. Outside code spans, a backslash escapes punctuation, so `` \` `` is a literal backtick. Emphasis, links, and HTML stay literal text, since a title also becomes the text of other links. With `phrases = true` as well, phrases are substituted outside code spans, never inside them.

Each output has the title both ways: as plain text, `ascribe.toml reference`, and formatted, with the code spans kept.

- **Empty link text** and the plain-markdown output's heading keep the code spans: ``[`ascribe.toml` reference](…)``.
- **The site output** writes the field as plain text, for a layout's `<title>`, search, and sorting, and its formatted form as HTML under `formatted`, for the page's heading and navigation. The generated schema types both, so a layout uses `entry.data.formatted.title` with `set:html`. See [Astro](../guides/astro.md#the-layout).
- **The JSON output** has the plain text in `title` and `frontmatter`, and the formatted form in `formatted.title`, a list of text and code pieces.
- **Review** shows the formatted title where a page's title is a heading or a list entry: the heading of VS Code's page preview, the HTML report's list of pages and its page headings, and the list of comments in the editor and the site preview. **Ascribe: Changed Pages**, which can show only text, writes the code spans between backticks. Tooltips, buttons, and search keep the plain text.

Only a content type's own `string` fields can set it, not fields in objects or the fragment schema, and `"code"` is its only value (`model-inline-field`).

### 5.4 Field roles

@available: next
A field's name is yours to choose, so a check that needs a page's description or the date it's due for review, or a report that counts pages by owner, reads the field the content type marks with a `role`:

```toml
[types.guide.frontmatter]
title = "string"
summary = { type = "string?", role = "description" }
review-by = { type = "date?", role = "review-date" }
team = { type = "enum(docs, platform)", role = "owner" }
```

| Role | The field's type | What reads it |
|---|---|---|
| `description` | `string`, optional or not | [`page-description-missing`](diagnostics.md#ASC150) reports a page of the type without one. A required field, or one with a default, always has one, so the check has nothing to report. |
| `review-date` | `date`, optional or not | [`review-overdue`](diagnostics.md#ASC153) reports a page whose date has passed. |
| `owner` | `string` or `enum`, optional or not | [`ascribe report inventory`](cli.md#cli-report) counts pages by owner. |

Without a role, those checks don't look at the type's pages, and the report counts its pages as having no owner. A role changes nothing in the outputs.

**Rules.** A role is one of the three (`model-field-role`), on a field of its type, in a content type's own fields: not a field in an object, nor in the fragment schema. Two fields of one type can't have the same role.

---

## 6. `[dimensions.<name>]`

@include: ../_fragments/content-model/dimensions.md

**Rules.** A value must belong to only one dimension (`model-dimension-value-shared`); otherwise a bare target in an availability spec would be ambiguous. Dimension names and values also take part in the one-role rule: a name can be a dimension name, a dimension value, a lifecycle state, or a feature key, but only one of them (`model-name-multiple-roles`).

---

## 7. `[versions]`

@include: ../_fragments/content-model/versions.md

| Key | Type | Default | Description |
|---|---|---|---|
| `scheme` | string | `"numeric"` | The version scheme. Spec 0.1 defines one: `"numeric"`. |
| `current` {available=next} | string (version) | none | The latest release, such as `"0.2.0"`. With it, the [`availability-left-behind`](diagnostics.md#content-checks) check reports a feature whose versions are all at or before it. Set it to each release as it ships. |

@include: ../_fragments/content-model/versions-numeric.md

No other scheme is defined. The table exists so a later spec version can add one without changing the file's shape.

**Rules.** `scheme` is one of the schemes (`model-invalid-value`), and `current` is a version, quoted (`model-wrong-type`).

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

**Matching:** occurrences match whole words only; the longest matching term wins where terms overlap (`API key` over `API`); matching applies to prose only, never to headings, link text, code, raw HTML, or text inside a directive's primary identifier. A term with no `link` isn't linked in the site or plain-markdown output; its definition still reaches the editor and the JSON output.

@include: ../_fragments/content-model/glossary-rules.md

---

## 13. `[images]`

@include: ../_fragments/content-model/images.md

---

## 14. `[widgets.<name>]`

@include: ../_fragments/content-model/widgets.md

**Rules:**

@include: ../_fragments/content-model/widgets-rules.md

**An example: a landing page.** Ascribe's own landing page is a page of Ascribe, laid out with three widgets that [`site/home/ascribe.toml`]({repo}/blob/main/site/home/ascribe.toml) declares and the site's stylesheet draws: `hero-banner`, the opening, with a `status` attribute the site shows as a badge; `claim-example`, a claim with its heading, its sentences, and the example under them, which can carry an [`@available`](directives.md#available) mark like any other block; and `source-and-result`, whose content is a `@snippet` of a page's file and whose `page` attribute names that page, which the site draws beside it. The lists and the install commands needed no widget.

```markdown
@claim-example:
### Checked

A broken link is an error, in the editor as you type and in CI.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/getting-started-check.txt
@end
```

What a widget can't say is the site's to do: Ascribe doesn't know that `page` names a page, so it doesn't check that the page exists, and the site's own test does.

---

## 15. `[consumer]`

@include: ../_fragments/content-model/consumer.md

| Key | Type | Default | Description |
|---|---|---|---|
| `profile` | string | `"astro"` | The consumer profile. Supported: `"astro"`. |
| `site` | string (URL) | none | The published site's origin, such as `"https://docs.quill.dev"`: an `http` or `https` URL with no path, query, or fragment. The plain-markdown output needs it to write absolute links ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)). Without it, plain-markdown links are root-relative (they start with `base-path`) and `ascribe build` warns. |
| `base-path` | string | `"/"` | **Routing.** The URL path every route starts with, such as `"/docs/"`, including any locale prefix (`"/en/"`). It must start with `/`. A trailing `/` is optional and doesn't change the meaning. |
| `trailing-slash` | string | `"always"` | **Routing.** Whether page URLs end in `/`: `"always"` (`/guides/setup/`) or `"never"` (`/guides/setup`). Match the consumer's own setting (Astro's `trailingSlash` and `build.format`). |
| `slugger` | string | `"github"` | **Slugging.** The algorithm for heading ids ([SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)), which must be the one the consumer uses. Supported: `"github"`, a port of `github-slugger`, which Astro uses. |
| `html` | boolean | `true` | **HTML passthrough.** Whether the consumer renders raw HTML in markdown. The site output's custom elements need it, so the `astro` profile supports only `true`. |
| `agents` {available=next} | boolean | `false` | **Outputs for agents.** Whether each build also publishes what AI agents read, as the [Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) lays it out: `llms.txt`, the index of every page, and each page's Markdown at its URL with `.md`. It needs `site`, since every link agents read is absolute. See below. |
| `permalink-path` {available=next} | string | `go/` under `base-path` | **Permalinks.** Where each [permalink](directives.md#permalinks)'s address is published, as a path from the site's root, such as `"/go/"`: `/go/cli-check/` forwards to the heading marked `cli-check`. It must start with `/`. Set it when the addresses a product prints shouldn't move when `base-path` does. Under the `astro` profile it must be under `base-path`, since an Astro site serves nothing outside it. |

**Heading ids, image attributes, and assets** ([SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)) are part of the profile, not keys. The `astro` profile has exactly one way to do each:

- **Heading ids, image attributes, and glossary links:** the site output writes each as a `<ascribe-attributes>` marker that the consumer's markdown plugin applies, so the consumer keeps its own heading, table-of-contents, and image processing, and a glossary link carries `data-ascribe-term`.
- **Assets:** copies mirror their source paths inside each output, and images are referenced relatively so Astro's image processing still applies. Other files a page links to are published under `_ascribe/files/`.

A later profile that offers a choice will add a key for it.

**How file paths become routes** (the `astro` profile): a page's route is `base-path`, then its path relative to the content root with the `.md` extension removed and each segment slugged the way Astro's content loader computes entry ids; a final `index` segment is dropped (`guides/index.md` → `/guides/`); then the trailing slash per `trailing-slash`. The root `index.md` (Astro's entry id `index`) is at `base-path`, which under `trailing-slash = "never"` loses its final `/` unless it's `/`. Two pages with one entry id (`My File.md` and `my-file.md`, or `index.md` and `index/index.md`) can't both be published, and `ascribe build --emit site` fails, naming them. The same router answers the reverse question, which page a route-like link names, for the `link-route` warning and its fix. Source files never contain routes ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)).

**The outputs for agents** (`agents = true`) change what the plain-markdown output is, and add to the site output:

- **The plain-markdown output is laid out by URL,** so it can be copied as it is to `base-path` beside the site: each page at its route with `.md` (`Guides/My Setup.md` → `guides/my-setup.md`, the root `index.md` → `index.md`), each opening with a line pointing at `llms.txt`; the files they link to under `_ascribe/files/`, linked by absolute URL; and `llms.txt`.
- **`llms.txt`** lists every page the build publishes as `- [Title](url): description`, linking to its Markdown. Its heading is the title of the page at `base-path`, and its summary that page's description. Pages in a folder are a section named after the folder (the title of its `index.md`, when it has one); pages outside every folder come first, under "Pages". An index over 50,000 characters is split into one `llms.txt` per folder, which the root file links to. A page's description is the field its type marks [`role = "description"`](#54-field-roles), as plain text; a type without one lists its pages without descriptions.
- **Each page of the site output** opens with a pointer to `llms.txt` and the page's Markdown, which the element library hides from sight ([element contract §8]({repo}/blob/main/packages/elements/CONTRACT.md#8-ascribe-for-agents)).

`@ascribed/astro` publishes the plain output with the site, and serves it in `astro dev`. With another generator, copy `<output-dir>/<build>/plain/` to the site's `base-path`. `ascribe check` then reports two advice checks: a description too long for its line in `llms.txt` (`description-too-long`), and an `llms.txt` still over the limit when it's split (`llms-section-large`).

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

A source is a folder outside the project's content, or another repository, that its pages may take code examples from, with [`@snippet`](directives.md#snippet) ([SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)). It's the only way a page can read a file outside the project's folder.

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["crates/**", "examples/**"]    # what's readable; everything else isn't
ignore = ["**/target/**"]

[sources.api]
git = "https://github.com/acme/api.git"   # another repository
branch = "main"                           # what `ascribe sources update` follows
include = ["src/**", "examples/**"]
```

A page names a file through its source as `<source>:<path>`, with the path relative to the source's folder: `@snippet: code:examples/quill/ascribe.toml#dimensions`. Both kinds of source are addressed the same way, so moving code to another repository, or back, changes `ascribe.toml` and no page.

| Key | Type | Default | Description |
|---|---|---|---|
| `path` | string (path) | **required** without `git` | The source's folder, relative to the project root. It must exist, be a directory, and be inside the git repository the project is in, when the project is in one. |
| `git` | string (URL) | **required** without `path` | Another repository, as `git` takes it: `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path`. |
| `branch` | string | the repository's default branch | The branch `ascribe sources update` moves the pin to. |
| `include` | array of patterns | every file | The files a snippet may read, matched against their paths relative to the source's folder (§1.3). |
| `ignore` | array of patterns | none | Files left out even when `include` matches them. |

`<name>` follows the rules for keys (§1.2). A project can declare several sources, of either kind.

**Rules.** `path` is relative (`model-path-absolute`); its folder must exist and be a directory (`model-source-path-missing`) inside the project's repository (`model-source-outside-repository`). With no repository, as in a copy made with `git archive`, `ascribe build` and the other commands read a source wherever `path` leads, but the editor reads one only inside a folder of the workspace it opened, and `ascribe mcp` only inside the folder it was started in: open the folder that contains the source. A source has `path` or `git`, never both, and `branch` only with `git` (`model-source-remote`).

### A source in another repository

`check` and `build` never fetch anything. A source with `git` is read from copies kept in the project:

- `ascribe.lock`, beside `ascribe.toml`, pins each such source to a commit, and records the hash of each file copied from it, and which files snippets name that aren't in the repository at that commit.
- The files the project's snippets use are copied, whole, into `sources/<name>/` beside `ascribe.toml`, at the paths they have in their repository. Only those files are copied, not the repository, and not everything `include` matches. `sources/` can't be inside the content root.

Both are written by [`ascribe sources fetch` and `ascribe sources update`](cli.md#cli-sources), and committed with the rest of the docs. `check` reports a copy that was changed by hand, a copy the lock doesn't list, a copy no snippet uses any more, and a snippet whose file hasn't been copied yet, or isn't in the repository at the pinned commit because it was moved or deleted there. The copies are published to everyone who can read the docs repository: for a private code repository and public docs, that's a decision to make knowingly.

---

## 19. `[checks]`
@available: next

How loudly each check about your content's quality speaks: whether it's reported as advice, a warning, or an error, or not at all. A diagnostic about whether the project is valid, such as a broken link, can't be set here: it's always reported at its own severity.

```toml
[checks]
code-language-missing = "warning"

[checks.page-size]
level = "warning"
limit = 40_000
```

| Key | Type | Default | Description |
|---|---|---|---|
| `<check>` | string or table | the check's own level | A check's name, as the [diagnostics reference](diagnostics.md) lists it. The value is its level: `"off"`, `"advice"`, `"warning"`, or `"error"`. Or it's a table with `level` and the check's own settings, which its entry in the reference lists; without `level`, the check keeps its own. |

Only a check the [diagnostics reference](diagnostics.md#setting-a-checks-level) marks configurable can be named; that section lists them, with their settings. Two have one, `limit`: for `page-size`, a whole number of characters above 0, `50_000` unless set; for `image-large`, a size, `"500 KB"` unless set. `[checks.vale]` and `[checks.links]` are reserved for the settings of tools Ascribe runs.

For example, a site whose sidebar lists every page doesn't need to hear about pages nothing links to, and a team that wants smaller images can lower the limit:

```toml
[checks]
page-orphan = "off"
title-duplicate = "warning"

[checks.image-large]
limit = "200 KB"
```

| Setting | Of | Type | Default | Description |
|---|---|---|---|---|
| `limit` | `page-size` | integer | `50_000` | The size, in characters of a page's plain Markdown in a build, at which the page is reported. Above 0. |
| `limit` | `image-large` | size | `"500 KB"` | The size over which an image is reported: a number of bytes, or a string with `B`, `KB`, or `MB`, such as `"1.5 MB"`. A kilobyte is 1,000 bytes. |

Advice is shown in the editor and by `ascribe check`, and never fails the check, even with `--deny-warnings`. A check set to `off` isn't reported at all.

**Rules.** A key that isn't a check is an unknown key, with the closest check suggested (`model-unknown-key`), and so is a setting the check doesn't have. A check that isn't configurable can't be named (`model-check-not-configurable`). A level is one of the four (`model-invalid-value`). `page-size`'s `limit` is a whole number above 0, and `image-large`'s a size (`model-wrong-type`).

### `[[intended]]`: acknowledgements
@id: intended-acknowledgements
@available: next

Says that a check's problem with a content model entry or an image is intended, as [`@intended`](directives.md#intended) does for a block and the `intended` frontmatter key for a page. Each table names one check, one entry, and why.

@include: ../_fragments/content-model/intended-example.md

| Key | Type | Default | Description |
|---|---|---|---|
| `check` | string | required | The check, by its name. It must be a review check that reports its problems at an entry, as the [diagnostics reference](diagnostics.md) says. |
| `reason` | string | required | Why the problem is intended. Not empty. |
| `phrase`, `feature`, `term`, or `image` | string | required | The entry: a phrase's key, a feature's key, a glossary term's id, or an image's path under the content root. Exactly one. |

A glossary term whose only occurrences are inside a phrase's value is reported by `glossary-term-unused`, since the search for uses doesn't read phrase values. Acknowledge it here with `term`.

**Rules.** A check that doesn't exist, isn't a review check, or reports its problems somewhere other than an entry is an error (`model-intended-check`). A table without exactly one entry key, or naming an entry that isn't declared, is an error (`model-intended-entry`).

### `[checks.vale]`
@available: next

Checks the prose with [Vale](https://vale.sh), a program the project installs: Ascribe gives it each page's prose, with phrases replaced by their text and without frontmatter, directive lines, attributes, or code, and reports its alerts as `prose` diagnostics at their place in the source. The editor runs it on a page when it's opened and saved; `ascribe check` runs it with `--vale`, or always with `in-check = true`. [Prose, through Vale](../guides/vale.md) has the steps.

```toml
[checks.vale]
preset = "quiet"
off = ["Ascribe.Repeated"]
```

| Key | Type | Default | Description |
|---|---|---|---|
| `preset` | string | | Ascribe's own selection of Vale rules: `quiet`, rules that are almost never wrong. Nothing is downloaded: the preset is written under `.ascribe/vale/`. |
| `config` | string | | Instead of a preset, the path of the project's own `.vale.ini`, from the project root. |
| `off` | array of strings | `[]` | Rules of the preset to turn off, by the names alerts give them, such as `Ascribe.Repeated`. Only with `preset`. |
| `command` | string | `"vale"` | The command that runs Vale. A path with a separator is from the project root; a bare name is looked up as the shell would. A path is a program in the project: the editor asks before it runs one, and the agents' hooks don't run one ([Prose, through Vale](../guides/vale.md#where-vale-runs)). |
| `in-check` | boolean | `false` | Whether `ascribe check`, and the agents' hook, check the prose without `--vale`. |
| `max-level` | string | `"error"` | The most an alert can be: `"advice"`, `"warning"`, or `"error"`. Vale's `suggestion`, `warning`, and `error` are advice, a warning, and an error up to it; with `"advice"`, Vale's alerts never fail `--deny-warnings`. |

Ascribe writes the project's own words (phrase values, glossary terms and their aliases, and the labels of dimensions, their values, and features) to a Vale vocabulary, `Ascribe`, under `.ascribe/vale/`, so Vale doesn't take them for misspellings. It's turned on for a preset and for the project's own configuration alike; nothing the project wrote is changed.

The level of every Vale alert can be set at once with `prose` in `[checks]`, and the advice that Vale couldn't be run with `prose-not-checked`.

**Rules.** `[checks.vale]` has exactly one of `preset` and `config`, and `off` goes with `preset` and names its rules (`model-checks-vale`). `preset` is one of Ascribe's presets, and `max-level` one of the three levels (`model-invalid-value`).

### `[checks.links]`
@available: next

How `ascribe report links` checks the project's external links, with [lychee](https://lychee.cli.rs), a program the project installs. `ascribe check` never checks external links. [The project report](../guides/report.md#external-links) has the steps.

```toml
[checks.links]
ignore = ["localhost", "*.internal.example.com"]
```

| Key | Type | Default | Description |
|---|---|---|---|
| `command` | string | `"lychee"` | The command that runs lychee. A path with a separator is from the project root; a bare name is looked up as the shell would. |
| `ignore` | array of strings | `[]` | The hosts whose links aren't checked: a host name, such as `example.com`, or `*.` and a host name, which covers every host under it but not the name itself. Case doesn't matter. |
| `reuse-days` | whole number | `14` | How many days an address that answered isn't asked about again. Each address's own limit falls between half of this and all of it, so addresses first checked together don't all come due on one day. `0` asks about every address on every run. |
| `confirm-days` | whole number | `3` | How many days a link that answered before and fails now is listed as unconfirmed, and not reported, before it's reported. A 404 or a 410 is reported at once. `0` reports every failure at once. |

An answer is reused only when it had nothing to report: a link that moved, failed, or timed out is asked about on every run, so every problem in a report was found in that run. The answers are kept in `.ascribe/cache/links/`; deleting it costs the next run time, and nothing else. `ascribe report links --recheck` asks about every address.

A link is held as unconfirmed only when an earlier run got an answer from its address: what each address last did is kept in `.ascribe/history/links.json`. Deleting it costs one noisier report, which reports every failure as a first run does, and never a missed problem. `ascribe report links --no-history` reports every failure too.

A broken link is `link-external-broken` and a moved one `link-external-moved`; both are configurable in `[checks]`, and a broken one is acknowledged with `@intended` above its block.

**Rules.** `ignore` names hosts, without a scheme, path, or port, and `reuse-days` and `confirm-days` are whole numbers, 0 or more (`model-wrong-type`).

---

## 20. `[permalinks]`
@available: next

Names for something outside the docs to link to that no heading or page declares: a [permalink](directives.md#permalinks) ended on purpose, and a second name for one. Each is published at an address, as a permalink is, and recorded in `ascribe.permalinks` when the project has it (SPEC §7.6). The [guide](../guides/permalinks.md#moving-aliasing-and-retiring) says when to use each.

```toml
[permalinks.retired]
rotate-keys = { to = "guides/security.md", reason = "Merged into the security guide" }
legacy-sync = { reason = "The sync agent was removed in 4.0" }

[permalinks.aliases]
agent-settings = "settings"
cli-settings = "settings"
```

`[permalinks.retired]` has an entry for each retired permalink, keyed by its name:

| Key | Type | Default | Description |
|---|---|---|---|
| `reason` | string | required | Why it was retired. A reader whose link leads there sees it, as text. Not empty. |
| `to` | string | | Where its address leads now: a page, by its path from the content root, and `#` and a heading's id, as `guides/security.md#rotate-keys`. Without it, or in a build that doesn't publish the page, the address shows that what was there was removed, the reason, and a link to the docs' home. |

`[permalinks.aliases]` has an entry for each alias, keyed by its name, whose value is the name it leads to: a permalink a heading or page declares, or another alias that leads to one. An alias's address leads where its permalink's does, in every build that publishes it, so repointing the alias changes where it leads and nothing else.

[`ascribe permalinks retire`](cli.md#cli-permalinks-retire) writes `[permalinks.retired]` for you, and the editor's quick fix on a removed permalink writes an entry with a placeholder reason for you to replace. To bring a retired permalink back, delete its entry and declare the name again.

**Rules.** A name has the characters of an `@id` (`model-invalid-name`). A retired permalink has a `reason`, not empty and not the quick fix's placeholder, and no keys but `reason` and `to` (`model-missing-key`, `model-empty-text`, `model-permalinks`, `model-unknown-key`). A name is one thing: retired, an alias, or declared by the content, never two (`model-permalinks`, `permalink-name-conflict`). An alias leads, through aliases, to a permalink the content declares, never back to itself or to a retired permalink (`model-permalinks`, `permalink-alias-target-missing`). A `to` leads to a page, not a fragment, and to a heading on it when it names one (`permalink-retired-target-missing`).

---

## 21. Defaults: what an absent section means

@include: ../_fragments/content-model/defaults.md

---

## 22. Validation

A content model with errors is reported, and nothing else is checked, since every other check depends on it. Its warnings are reported with the rest of the diagnostics. Every rule the loader enforces is listed in the [diagnostics reference](diagnostics.md#the-content-model), with its code and how to fix it.
