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

Every problem the loader reports is in the [diagnostics reference](../checks/diagnostics/content-model.md), with its fix. References to "SPEC" are to the [Ascribe specification]({repo}/blob/main/SPEC.md). The rules a loader enforces, stated for someone writing one, are in the [`ascribe.toml` format contract](../contracts/content-model.md), with the same section numbers and key tables.

@available: ascribe 0.4.1
In VS Code, [Project Settings](../editor/settings.md#project-settings) shows a project's settings beside its `ascribe.toml`, each with its value, its default, and its line, and changes a value the file sets in place.

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

@include: ../_generated/content-model-project.md

**Rules:** both paths are relative to the project root (`model-path-absolute`). Neither folder can be inside the other, and they can't be one folder (`model-output-overlaps-content`): a build would read its own output as source, or delete source as stale output. Ascribe tells them apart after resolving `.`, `..`, and, for folders that are there, symbolic links, so a link can't hide one inside the other.

@include: ../_fragments/content-model/project-defaults.md

---

## 4. `[types.<name>]` and `[fragments]`

@include: ../_fragments/content-model/types.md

**The page title.** Every page type has `title`, a required `string` field, whether it declares it or not. It's the page's title everywhere Ascribe needs one: an empty link to the page shows it as its text, for one ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)). Declaring it, in `[frontmatter]` or in a type's own `frontmatter`, changes only how it's read: it can accept phrases (§11), and code spans (§5.3). A declared `title` that's optional, has a default, or isn't a string is an error at the field (`model-field-reserved`).

**Reserved keys.** `type` (the page's [content type](#4-typesname-and-fragments)), `available` ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)), `variant` ([SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)), `permalink` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files), a page's [permalink](../writing/frontmatter.md#a-pages-permalink)), `listing` (below), `replaces` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files), the page [a page replaces](../writing/frontmatter.md#a-page-that-replaces-another)), and `nav-label` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files), a page's name in the sidebar and the previous and next links) are reserved frontmatter keys. Any page can use them, whatever its type says, and they mean what SPEC says; a type, or `[frontmatter]`, that declares one is an error (`model-field-reserved`). Generated consumer schemas include them automatically, except `listing`, which reaches a site in the page's record (output-layout contract §1.3), `replaces`, which reaches it in the versions file (output-layout contract §1.5), and `nav-label`, which reaches it in the navigation (output-layout contract §1.6). `intended` ([SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended)) is reserved too, and accepted by every page and every fragment: it [acknowledges a check's problem](../writing/directives.md#intended), and since no output has it, generated consumer schemas leave it out. With the `astro` profile, a type can't declare `slug` either (`model-field-reserved`): Astro would take a page's `slug` as its entry id instead of its path, and publish the page at an address Ascribe never worked out.

`formatted` is reserved on page types too: the site output writes the formatted form of fields that set `inline` under it (§5.3), so a type must not declare it (`model-field-reserved`).

`source` is reserved on page types too: the site output writes each page's path from the content root under it, for a layout's edit link, so a type must not declare it (`model-field-reserved`).

### Listing a page

A page says whether readers that aren't people are offered it with `listing` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)), one of three values, from open to closed: `listed`, the default when a page doesn't set it, `unquoted`, and `unlisted`. What each tells search engines and agents is in [Publishing](../publishing/index.md#how-a-page-is-listed). Any other value, or one that isn't a string, is an error (`frontmatter-type-mismatch`).

```yaml
---
title: Migrating from 3.x
listing: unlisted
---
```

Each build writes every page's record, with its `listing`, to `_ascribe/pages.json` in the site output. The site output's frontmatter doesn't carry `listing`, since the record does.

A project can set `listing` for the whole site, a folder, or a content type, in [`[listing]`](checks.md#22-listing). A page under a rule can set the same value or a stricter one, never a looser one (`listing-loosened`).

@include: ../_fragments/content-model/types-frontmatter-and-fragments.md

**Reserved keys on fragments.** `type`, `available`, `variant`, `permalink`, `listing`, `replaces`, and `nav-label` are a page's, so a fragment that has one in its frontmatter gets an error, and `[fragments.frontmatter]` can't declare them (`model-field-reserved`). To mark what a fragment contributes, put `@available` inside it. A fragment can use `intended`, which `[fragments.frontmatter]` can't declare either.

---

## 5. Field and attribute types

@include: ../_fragments/content-model/field-types.md

@include: ../_generated/content-model-fields.md

@include: ../_fragments/content-model/field-types-nesting.md

### 5.3 Code in a field

A backtick in a `string` field is just a backtick. To show code in a page title, set `inline = "code"` on the field:

```toml
[types.reference.frontmatter]
title = { type = "string", inline = "code" }
```

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

Only a page type's top-level `string` fields can set it, its own or `[frontmatter]`'s, not fields in objects or the fragment schema, and `"code"` is its only value (`model-inline-field`).

### 5.4 Field roles

A check that needs a page's description or the date it's due for review, or a report that counts pages by owner, reads the field with that role.

@available: next
A field named for a role has it, when its type is one the role takes:

```toml
[frontmatter]
description = "string?"
review-date = "date?"
owner = "enum(docs, platform)"
```

A field's name is yours to choose, so a field of another name takes a role with `role`, and the field named for it then has none:

```toml
[types.guide.frontmatter]
summary = { type = "string?", role = "description" }
review-by = { type = "date?", role = "review-date" }
team = { type = "enum(docs, platform)", role = "owner" }
```

| Role | The field's type | What reads it |
|---|---|---|
| `description` | `string`, optional or not | [`page-description-missing`](../checks/diagnostics/content-checks.md#ASC150) reports a page of the type without one. A required field, or one with a default, always has one, so the check has nothing to report: a page that leaves a field with a default out, or empty, has the default as its description, in `llms.txt` and the page records too. [`description-link`](../checks/diagnostics/content-checks.md#ASC198) reports a Markdown link in one: a description is plain text, so the link is shown as written. |
| `review-date` | `date`, optional or not | [`review-overdue`](../checks/diagnostics/content-checks.md#ASC153) reports a page whose date has passed. |
| `owner` | `string` or `enum`, optional or not | [`ascribe report inventory`](../checks/report-command.md#cli-report) counts pages by owner. |

Without a field with the role, those checks don't look at the type's pages, and the report counts its pages as having no owner. A role changes nothing in the outputs.

@available: ascribe 0.4.0
In VS Code, the sidebar's [This page](../editor/sidebar.md#this-page) shows a page's description against its limit, and when it's due for review, from the fields with these roles.

**Rules.** A role is one of the three (`model-field-role`), on a field of its type, in a page type's top-level fields, its own or `[frontmatter]`'s: not a field in an object, nor in the fragment schema. Two fields of one table can't have the same role. A role a type gives one of its own fields with `role` is that field's, and not a field's in `[frontmatter]`. A field named for a role has it only when no other field of the type is given it, and its type is one the role takes: a `description` that's a number is a field like any other.
