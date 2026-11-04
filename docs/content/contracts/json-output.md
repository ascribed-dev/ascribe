---
type: reference
title: JSON output contract
description: "The format of the `json` output: each page as the build resolved it, with every node's place in its source, for your own tools."
permalink: json-output-contract
---

Each build writes a `json` output beside its site and plain Markdown outputs: each page the build publishes, as the build resolved it, with every node's place in its source, for your own tools. This contract says what each page's document holds. The [output-layout contract](output-layout.md) says where the files are.

| Who | Uses this contract to |
|---|---|
| `ascribe build` | Write the `json` output |
| A tool of your own | Read the pages: a search index, a check of your own, another renderer |

## 1. The files

The `json` output has one document per page, at the page's source path with `.json` for `.md`: `guides/install.md` is `guides/install.json`. Beside the pages, `_ascribe/nav.json` is the build's navigation, as the site output has it, with each formatted label as pieces (the [navigation contract](navigation.md)).

Each page's document is a JSON object, whose schema is [`schemas/json-page.schema.json`]({repo}/blob/v{version}/schemas/json-page.schema.json): JSON Schema (draft 2020-12), generated from the code that writes the documents, so the two can't disagree. [The schema](json-output/schema.md) has its fields as tables, generated from it as the [JSON report contract](json-reports.md#reading-the-tables) says.

## 2. Rules

The [JSON report contract](json-reports.md)'s rules hold here too:

- `schema_version`, `2`, changes only when a field is removed or changes meaning. Fields can be added without a new version, so **ignore fields you don't know**. The schema allows fields it doesn't list.
- A field whose values are a fixed set of names, such as a block's `type`, may gain a value without a new version. A reader skips a block or an inline whose `type` it doesn't know.
- A field the schema doesn't list as required is left out when it has no value. A required field that can be `null` is always written.
- Keys are snake_case, and so are the names a field's values are.

A **span** is `[start, end]`, counted in UTF-8 bytes from the start of the file, frontmatter included. **Lines** count from 1. Every `file` is a path relative to the content root, so a tool can point back at the source.

## 3. The page

| Field | Type | Meaning |
|---|---|---|
| `schema_version` | number | `2` |
| `format` | string | `"ascribe-page"` |
| `build` | string | The build's name |
| `path` | string | The page's source path, relative to the content root |
| `route` | string | The page's root-relative URL, as the router made it |
| `site` | string or null | `[consumer] site`, the origin to put before a root-relative URL |
| `title` | string or null | The frontmatter `title`, phrases substituted, as plain text |
| `frontmatter` | object or null | The frontmatter, with phrases substituted in the fields that ask for it, and fields that set `inline = "code"` as plain text. Its keys are the content model's, as written |
| `formatted` | object, when there is one | Each field that sets `inline = "code"`, as a list of `{type: "text" \| "code", value}` |
| `availability` | object or null | The page's own availability (frontmatter `available`) |
| `headings` | array | Every surviving heading: `level`, `text`, `id` (its page id), `source_id`, `explicit`, `source` |
| `assets` | array | Every surviving asset reference on the page: `source` (its source path), `path` (the copy's path in this output), `reference` (how this page refers to it), `kind` (`image` or `link`), `fragment`, `written_in` (the file the reference is written in), `span` |
| `blocks` | array | The page's blocks |

## 4. Blocks

Every block has a `type`, and:

| Field | Meaning |
|---|---|
| `source` | `file`, `span`, `lines` (first and last), and `via`: the includes it came through, outermost first (`file`, `span`, `line`), absent for a page's own blocks |
| `availability` | Its effective availability below page level: the innermost spec, with `enclosing` chaining outward to the scope it sits in. Absent when nothing but the page's availability applies |
| `links` | The block's own links and images, resolved: `span`, `kind`, `destination`, `target` |
| `glossary` | Glossary terms linked in its prose, and its links to a term's heading: `term`, `text`, `url`, `definition` |
| `substitutions` | Phrases replaced in its text: `span` (over the source), `key`, `value` |
| `snippet` | For a `code` block a `@snippet` became: `address` (as written), `path` (the code file, from the project root), and `lines` (the first and last lines of the file it covers), and `tested` when the file is a tested example. Its `source` is the `@snippet` line |

`links`, `glossary`, and `substitutions` are absent when empty, and `snippet` is absent on every other block.

A snippet's `tested` is present only when its source's `ascribe.examples.toml` makes the file an example ([tested examples](tested-examples.md)): `status`, one of `current`, `stale`, and `unobserved`, and `observations`, the observations current for the file in name order, each with its `name` and its `environment` as the observation has it, `os` and `arch` first, such as `{ "os": "linux", "arch": "x64", "node": "v24.9.0" }`. `observations` is empty unless the status is `current`.

| `type` | Fields |
|---|---|
| `heading` | `level`, `id`, `source_id`, `explicit`, `text`, `inlines`, and `summary` when its section has one |
| `paragraph` | `inlines` |
| `code` | `fenced`, `info`, `literal` |
| `block_quote` | `children` |
| `list` | `ordered`, `start`, `tight`, `items` (each `{children}`) |
| `html` | `literal` |
| `thematic_break` | |
| `table` | `alignments`: each column's `none`, `left`, `center`, or `right`; `rows`: each `{header, cells}`, a cell being `{inlines}`, and a row's own `availability` when it has one |
| `directive` | A line-form directive that survived: `name`, `attributes`, `title`, `primary`, `binding` (`own`, `heading`, `following_block`, `unbound`), `annotation` |
| `operation` | An `@operation` that names an operation the OpenAPI spec has: `api`, `operation_id` when it has one, `method`, `path`, `summary` when it has one, `deprecated`, and `address`, the operation's in the published reference. Its `source` is the `@operation` line |
| `container` | `name`, `attributes`, `title`, `children` |
| `group` | `name`, `arms`: each `{label, attributes, title, span, children}` |

An `@available` that survived the build is a `directive` with an `annotation`: `text` (the availability spec as shown; for a feature key, the availability spec it stands for), `display` (with the content model's labels), and `feature`. `attributes` is a list of `{key, values}` (a set has several values, a bare key none). A `title` and a text `primary` are inline lists. A `primary` is `{kind: "text", inlines}`, `{kind: "identifier", text}`, or `{kind: "line", text}`.

An `@operation` whose link names no operation the OpenAPI spec has, which `ascribe check` reports, stays a `directive` block, with its link as its `identifier` primary.

A heading's `summary` is its section's: `form`, `text` for `@summary: text` and `paragraph` for `@summary` above a paragraph; `inlines`, the summary's text; and `source`, the `@summary` line for the text form and the paragraph for the other. The `@summary` line is a `directive` block too, where it's written, and the paragraph a `paragraph`.

A group keeps its surviving arms only: a group a selection reduced to one arm isn't a group in the tree, but that arm's blocks, and one with none is gone. An arm's `label` is its title as plain text, or the display labels of the values it names.

## 5. Inlines

Every inline has a `type` and a `span` in the block's file: `text` (`value`, decoded), `code` (`value`), `soft_break`, `hard_break`, `html` (`value`), `emphasis` and `strong` (`children`), `link`, `image`, `phrase` (`key`, for a phrase the resolution left, which it doesn't).

A `link` has `destination` (a page link's is its route and id, an external one's is as written), `title`, `children`, and `target`: the block's resolved link at the same span, or `null` for a glossary link (an ordinary link, listed in the block's `glossary`). An `image` has `destination`, `title`, `alt` (plain text), `attributes` (`{key, values}`, such as `width`), and `target`.

A `target` has a `type`:

| `type` | Fields |
|---|---|
| `external` | |
| `page` | `page` (its source path), `id` (a page id, or null), `url`, `text_filled` (the link had no text and took the page's title), and, for a link out of a line of a versioned build, `line` (the line it leads to) |
| `asset` | `source`, `path` (the copy), `reference` (as written on this page), `url`, `fragment` |
| `operation` | `api` (its `[apis.<name>]`), `operation_id`, or `method` and `path` for an operation with no id, and `address`, the operation's in the published reference |
| `unresolved` | The build couldn't resolve it; the checks report why |

## 6. Example

```json
{
  "schema_version": 2,
  "format": "ascribe-page",
  "build": "cloud",
  "path": "keys.md",
  "route": "/keys/",
  "site": "https://docs.quill.dev",
  "title": "API keys",
  "headings": [{ "level": 2, "text": "Create a key", "id": "create-key", "source_id": "create-key", "explicit": true, "source": { "file": "keys.md", "span": [93, 108], "lines": [5, 5] } }],
  "assets": [],
  "blocks": [{ "type": "heading", "level": 2, "id": "create-key", "…": "…" }]
}
```
