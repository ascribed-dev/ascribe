---
type: reference
title: "`ascribe.toml` reference"
description: "The content model: content types, dimensions, availability, phrases, widgets, and builds."
permalink: content-model
---

`ascribe.toml` is a project's content model: its schema (SPEC §7). It sits at the project root and declares the content types and their frontmatter, the dimensions content varies along, phrases, the glossary, project widgets, how the site output fits Astro, and the builds. The editor, `ascribe check`, and `ascribe build` all read it, so they can't disagree about what's valid.

A file containing only `spec = "0.1"` is valid; every section has a default ([§23](checks.md#23-defaults-what-an-absent-section-means)). Example files:

- [`examples/content-models/minimal.toml`]({repo}/blob/main/examples/content-models/minimal.toml): the smallest valid model.
- [`examples/quill/ascribe.toml`]({repo}/blob/main/examples/quill/ascribe.toml): a small, complete project.
- [`examples/content-models/full.toml`]({repo}/blob/main/examples/content-models/full.toml): every section and key.

Every problem the loader reports is in the [diagnostics reference](../checks/diagnostics/content-model.md), with its fix. References to "SPEC" are to the [Ascribe specification]({repo}/blob/main/SPEC.md).

## Contents

1. [Conventions](index.md#1-conventions)
2. [Top level: `spec`](index.md#2-top-level-spec)
3. [`[project]`](index.md#3-project)
4. [`[types.<name>]` and `[fragments]`](index.md#4-typesname-and-fragments)
5. [Field and attribute types](index.md#5-field-and-attribute-types)
6. [`[dimensions.<name>]`](content.md#dimensions)
7. [`[versions]`](content.md#7-versions)
8. [`[lifecycle.<state>]`](content.md#8-lifecyclestate)
9. [`[features.<key>]`](content.md#9-featureskey)
10. [`[notes.<type>]`](content.md#10-notestype)
11. [`[phrases]`](content.md#phrases)
12. [`[glossary]`](content.md#12-glossary)
13. [`[images]`](content.md#13-images)
14. [`[widgets.<name>]`](content.md#14-widgetsname)
15. [`[consumer]`](output.md#consumer)
16. [`[builds.<name>]`](output.md#16-buildsname)
17. [`[editor]`](output.md#17-editor)
18. [`[sources.<name>]`](output.md#18-sourcesname)
19. [`[checks]`](checks.md#19-checks)
20. [`[permalinks]`](checks.md#20-permalinks)
21. [`[crawlers]`](checks.md#21-crawlers)
22. [`[listing]`](checks.md#22-listing)
23. [Defaults: what an absent section means](checks.md#23-defaults-what-an-absent-section-means)
24. [Validation](checks.md#24-validation)

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

**Reserved keys.** `available` ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)), `variant` ([SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)), `permalink` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files), a page's [permalink](../writing/directives.md#permalinks)), `listing` (below), and `replaces` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files), the page [a page replaces](../writing/directives.md#a-page-that-replaces-another)) are reserved frontmatter keys. Every page accepts them, with the meaning the spec gives, whether or not its type mentions them, and a type must not declare them (`model-field-reserved`). Generated consumer schemas include them automatically, except `listing`, which reaches a site in the page's record (output-layout contract §1.3), and `replaces`, which reaches it in the versions file (output-layout contract §1.5). `intended` ([SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)) is reserved too, and accepted by every page and every fragment: it [acknowledges a check's problem](../writing/directives.md#intended), and since no output has it, generated consumer schemas leave it out. Under the `astro` profile, `slug` is reserved too: Astro's content loader uses a page's frontmatter `slug` as its entry id in place of its path, which would publish the page at a URL Ascribe never computed, so a type must not declare it (`model-field-reserved`).

@available: ascribe 0.2.0
`formatted` is reserved on page types too: the site output writes the formatted form of fields that set `inline` under it (§5.3), so a type must not declare it (`model-field-reserved`).

@available: ascribe 0.3.0
`source` is reserved on page types too: the site output writes each page's path from the content root under it, for a layout's edit link, so a type must not declare it (`model-field-reserved`).

### Listing a page
@available: ascribe 0.3.0

A page says whether readers that aren't people are offered it with `listing` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)), one of three values, from open to closed:

| Value | What it means |
|---|---|
| `listed` | The default, when a page doesn't set it. Search engines may list it, and agents are offered it in `llms.txt`. |
| `unquoted` | Listed, and asks not to be quoted (`nosnippet`): a search result, or an AI answer inside a search product, shows no text from it. It's in `llms.txt` and the sitemap. |
| `unlisted` | Asks not to be listed (`noindex`), and isn't offered: it's out of `llms.txt` and the sitemap. It's still built, at its address, with its Markdown version. |

```yaml
---
title: Migrating from 3.x
listing: unlisted
---
```

Each build writes every page's record, with its `listing`, to `_ascribe/pages.json` in the site output, and `llms.txt` is written from the record. In an Astro site, `@ascribed/astro` writes the page's head, the sitemap, and `robots.txt` from it too ([the `@ascribed/astro` reference](../astro/reference.md#search-engines-and-crawlers)). The site output's frontmatter doesn't carry `listing`, since the record does. A record's canonical address needs [`[consumer] site`](output.md#consumer): without it, a site gets no canonical links and no sitemap, which needs absolute addresses. Any other value, or one that isn't a string, is an error (`frontmatter-type-mismatch`).

A project can set `listing` for the whole site, a folder, or a content type, in [`[listing]`](checks.md#22-listing). A page under a rule can set the same value or a stricter one, never a looser one (`listing-loosened`).

What isn't handled yet:

- **The Markdown version of a page that isn't listed can still be indexed** unless the host sends it with `X-Robots-Tag: noindex`, since only a response header keeps a file that isn't HTML out of a search index. With [`[consumer] headers`](output.md#consumer) set, the build gives the host the header; without it, the header is in `_ascribe/headers.json`, and the host sends it only if a deploy step gives it the list.

@include: ../_fragments/content-model/types-frontmatter-and-fragments.md

**Reserved keys on fragments.** The spec defines `available`, `variant`, `permalink`, `listing`, and `replaces` for pages only. A fragment's frontmatter must not use them (an error on the fragment), and `[fragments.frontmatter]` must not declare them (`model-field-reserved`). Use `@available` inside the fragment instead. `intended` is for fragments too, and `[fragments.frontmatter]` must not declare it either.

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
- **The site output** writes the field as plain text, for a layout's `<title>`, search, and sorting, and its formatted form as HTML under `formatted`, for the page's heading and navigation. The generated schema types both, so a layout uses `entry.data.formatted.title` with `set:html`. See [Astro](../astro/index.md#the-layout).
- **The JSON output** has the plain text in `title` and `frontmatter`, and the formatted form in `formatted.title`, a list of text and code pieces.
- **Review** shows the formatted title where a page's title is a heading or a list entry: the heading of VS Code's page preview, the HTML report's list of pages and its page headings, and the list of comments in the editor and the site preview. **Ascribe: Changed Pages**, which can show only text, writes the code spans between backticks. Tooltips, buttons, and search keep the plain text.

Only a content type's own `string` fields can set it, not fields in objects or the fragment schema, and `"code"` is its only value (`model-inline-field`).

### 5.4 Field roles

@available: ascribe 0.3.0
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
| `description` | `string`, optional or not | [`page-description-missing`](../checks/diagnostics/content-checks.md#ASC150) reports a page of the type without one. A required field, or one with a default, always has one, so the check has nothing to report. [`description-link`](../checks/diagnostics/content-checks.md#ASC198) reports a Markdown link in one: a description is plain text, so the link is shown as written. |
| `review-date` | `date`, optional or not | [`review-overdue`](../checks/diagnostics/content-checks.md#ASC153) reports a page whose date has passed. |
| `owner` | `string` or `enum`, optional or not | [`ascribe report inventory`](../review/diff.md#cli-report) counts pages by owner. |

Without a role, those checks don't look at the type's pages, and the report counts its pages as having no owner. A role changes nothing in the outputs.

**Rules.** A role is one of the three (`model-field-role`), on a field of its type, in a content type's own fields: not a field in an object, nor in the fragment schema. Two fields of one type can't have the same role.
