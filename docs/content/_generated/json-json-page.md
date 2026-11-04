@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/json-page.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/json-page.schema.json`]({repo}/blob/v{version}/schemas/json-page.schema.json):

**`JsonPage`**: A page of the JSON output: the page as the build resolved it, with every node's place in its source (the JSON output contract). Spans are UTF-8 byte offsets, `[start, end]`, from the start of the file, frontmatter included; lines count from 1.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The format's version, `2`. It changes only when a field is removed or changes meaning. |
| `format` | `string` | yes | no | `"ascribe-page"`. |
| `build` | `string` | yes | no | The build's name. |
| `path` | `string` | yes | no | The page's source path, relative to the content root. |
| `route` | `string` | yes | no | The page's root-relative URL, as the router made it. |
| `site` | `string` | yes | yes | `[consumer] site`: the origin to put before a root-relative URL. |
| `title` | `string` | yes | yes | The frontmatter `title`, phrases substituted, as plain text. |
| `frontmatter` | `any` | yes | no | The frontmatter, with phrases substituted in the fields that ask for it, and each field that sets `inline = "code"` as plain text. |
| `availability` | `Availability` | yes | yes | The page's own availability (frontmatter `available`). |
| `headings` | `object[]` | yes | no | Every heading that survived the build, in order. |
| `headings[].level` | `integer` | yes | no | Its level, 1 to 6. |
| `headings[].text` | `string` | yes | no | Its text, as plain text. |
| `headings[].id` | `string` | yes | no | Its page id: the id it has on the page. |
| `headings[].source_id` | `string` | yes | no | Its source id: the id it has in its source file. |
| `headings[].explicit` | `boolean` | yes | no | Whether an `@id` gave it its id. |
| `headings[].source` | `Source` | yes | no | Where it's written. |
| `assets` | `object[]` | yes | no | Every asset reference on the page that survived the build. |
| `assets[].source` | `string` | yes | no | The asset's source path, relative to the content root. |
| `assets[].path` | `string` | yes | no | The copy's path in this output. |
| `assets[].reference` | `string` | yes | no | How this page refers to the copy. |
| `assets[].kind` | `string` | yes | no | `image` or `link`. |
| `assets[].fragment` | `string` | yes | yes | The reference's fragment, without its `#`. |
| `assets[].written_in` | `string` | yes | yes | The file the reference is written in, relative to the content root. |
| `assets[].span` | `Span` | yes | no | The reference's span in that file. |
| `blocks` | `Block[]` | yes | no | The page's blocks. |
| `formatted` | `object` | no | no | Each field that sets `inline = "code"`, as a list of text and code pieces (`{type, value}`); its plain text is in `frontmatter`. |

**`Availability`**: An availability: the page's own, or a block's effective one.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `text` | `string` | yes | no | The spec as written (for a feature key, the spec it stands for). |
| `display` | `string` | yes | no | The spec with the content model's display labels. |
| `scope` | `string` | yes | no | What it applies to: `page`, `section`, `block`, or `row`. |
| `feature` | `string` | yes | yes | The feature key it was written as, when it was one. |
| `source` | `object` | yes | yes | Where it's written. |
| `source.file` | `string` | yes | yes | The file, relative to the content root. |
| `source.span` | `Span` | yes | no | The span in that file. |
| `enclosing` | `Availability` | no | no | The scope it sits in, whose availability it inherits too. The chain stops before the page's availability, which is in the page's own `availability` and applies to every block. |
| `targets` | `object[]` | no | no | Each target as the availability element gives it, when the project lists release lines in `[releases]`. |
| `targets[].target` | `string` | yes | no | The target's name. |
| `targets[].dimension` | `string` | yes | no | The dimension it's a value of. |
| `targets[].states` | `string[]` | yes | no | Its lifecycle states. |
| `targets[].text` | `string` | yes | no | The target as text. |
| `targets[].planned` | `boolean` | no | no | `true` when the target's last version is in a planned line; absent otherwise. |
| `targets[].versions` | `string[]` | no | no | The versions it names; absent when it names none. |

**`Source`**: Where a node is written.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `file` | `string` | yes | yes | The file the node is written in, relative to the content root. |
| `span` | `Span` | yes | no | Its span in that file. |
| `lines` | `integer[]` | yes | yes | First and last line, counted from 1. |
| `via` | `object[]` | no | no | The includes it came through, outermost first; absent for a page's own nodes. |
| `via[].file` | `string` | yes | yes | The file the `@include` is written in. |
| `via[].span` | `Span` | yes | no | The `@include`'s span in that file. |
| `via[].line` | `integer` | yes | yes | The `@include`'s line, counted from 1. |

**`Span`**: A byte range in a source file, `[start, end]`, counted in UTF-8 bytes from the start of the file (frontmatter included). Its type is `[integer, integer]`.

**`Block`**: A block: its `type` and the fields of that type, and what every block has. Every option has these fields:

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `source` | `Source` | yes | no | Where it's written. |
| `availability` | `Availability` | no | no | Its effective availability below page level: the innermost spec, with `enclosing` chaining outward. Absent when nothing but the page's availability applies. |
| `glossary` | `object[]` | no | no | Glossary terms linked in its prose, and its links to a term's heading; absent when it has none. |
| `glossary[].term` | `string` | yes | no | The term. |
| `glossary[].text` | `string` | yes | no | The text that links it. |
| `glossary[].url` | `string` | yes | no | The term's URL. |
| `glossary[].definition` | `string` | yes | no | The term's definition, as plain text. |
| `links` | `object[]` | no | no | The block's own links and images, resolved; absent when it has none. |
| `links[].span` | `Span` | yes | no | Its span. |
| `links[].kind` | `string` | yes | no | `link` or `image`. |
| `links[].destination` | `string` | yes | no | Its destination, as written. |
| `links[].target` | `Target` | yes | no | What it resolved to. |
| `snippet` | `object` | no | no | For a `code` block a `@snippet` became, where its code comes from; absent on every other block. |
| `snippet.address` | `string` | yes | no | The address, as written. |
| `snippet.path` | `string` | yes | no | The code file, from the project root. |
| `snippet.lines` | `integer[]` | yes | yes | The lines of the code file it covers, first and last. |
| `snippet.tested` | `object` | no | no | Whether the file was tested, when it's a tested example (decision #458c); absent on any other file. |
| `snippet.tested.status` | `string` | yes | no | `current`, `stale`, or `unobserved`. |
| `snippet.tested.observations` | `object[]` | yes | no | The observations current for it, in name order: empty unless it's `current`. |
| `snippet.tested.observations[].name` | `string` | yes | no | Its name: `default`, or the name `ascribe observe --name` gave it. |
| `snippet.tested.observations[].environment` | `Environment` | yes | no | Its environment as it has it, in its order: `os`, `arch`, then each entry's kept line, such as `"node": "v24.9.0"`. |
| `substitutions` | `object[]` | no | no | Phrases replaced in its text; absent when it has none. |
| `substitutions[].span` | `Span` | yes | no | Its span over the source. |
| `substitutions[].key` | `string` | yes | no | The phrase's key. |
| `substitutions[].value` | `string` | yes | no | The text that replaced it. |

**`Block`** when `type` is `"heading"`: A heading.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"heading"` | yes | no | Always `"heading"`. |
| `level` | `integer` | yes | no | Its level, 1 to 6. |
| `id` | `string` | yes | yes | Its page id. |
| `source_id` | `string` | yes | yes | Its source id. |
| `explicit` | `boolean` | yes | no | Whether an `@id` gave it its id. |
| `text` | `string` | yes | yes | Its text, as plain text. |
| `inlines` | `Inline[]` | yes | no | Its text. |
| `summary` | `object` | no | no | The section's summary (SPEC §4.12), when it has one. |
| `summary.form` | `string` | yes | no | `text` or `paragraph`. |
| `summary.inlines` | `Inline[]` | yes | no | The summary's text. |
| `summary.source` | `Source` | yes | no | The `@summary` line for the text form, and the paragraph for the other. |

**`Block`** when `type` is `"paragraph"`: A paragraph.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"paragraph"` | yes | no | Always `"paragraph"`. |
| `inlines` | `Inline[]` | yes | no | Its text. |

**`Block`** when `type` is `"code"`: A code block.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"code"` | yes | no | Always `"code"`. |
| `fenced` | `boolean` | yes | no | Whether it's fenced, rather than indented. |
| `info` | `string` | yes | no | Its info string. |
| `literal` | `string` | yes | no | Its code. |

**`Block`** when `type` is `"block_quote"`: A block quote.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"block_quote"` | yes | no | Always `"block_quote"`. |
| `children` | `Block[]` | yes | no | Its blocks. |

**`Block`** when `type` is `"list"`: A list.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"list"` | yes | no | Always `"list"`. |
| `ordered` | `boolean` | yes | no | Whether it's numbered. |
| `start` | `integer` | yes | yes | The first item's number, for a numbered list. |
| `tight` | `boolean` | yes | no | Whether it's tight: no blank line between its items. |
| `items` | `object[]` | yes | no | Its items. |
| `items[].children` | `Block[]` | yes | no | Its blocks. |

**`Block`** when `type` is `"html"`: A block of HTML.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"html"` | yes | no | Always `"html"`. |
| `literal` | `string` | yes | no | The HTML. |

**`Block`** when `type` is `"thematic_break"`: A thematic break.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"thematic_break"` | yes | no | Always `"thematic_break"`. |

**`Block`** when `type` is `"table"`: A table.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"table"` | yes | no | Always `"table"`. |
| `alignments` | `string[]` | yes | no | Each column's alignment: `none`, `left`, `center`, or `right`. |
| `rows` | `object[]` | yes | no | Its rows. |
| `rows[].header` | `boolean` | yes | no | Whether it's the header row. |
| `rows[].cells` | `object[]` | yes | no | Its cells. |
| `rows[].cells[].inlines` | `Inline[]` | yes | no | Its text. |
| `rows[].availability` | `Availability` | no | no | The row's own availability, in its table's (SPEC §4.4). |

**`Block`** when `type` is `"directive"`: A line-form directive that survived: `@available` with its annotation, or a `@note`, `@steps`, `@details`, or widget that binds the block that follows it.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"directive"` | yes | no | Always `"directive"`. |
| `name` | `string` | yes | no | Its name. |
| `attributes` | `Attribute[]` | yes | no | Its attributes. |
| `title` | `Inline[]` | yes | yes | Its title. |
| `primary` | `Primary` | yes | yes | Its primary value. |
| `binding` | `string` | yes | yes | What it binds: `own`, `heading`, `following_block`, or `unbound`. |
| `annotation` | `object` | yes | yes | An `@available`'s spec. |
| `annotation.text` | `string` | yes | no | The spec as shown; for a feature key, the spec it stands for. |
| `annotation.display` | `string` | yes | yes | The spec with the content model's display labels. |
| `annotation.feature` | `string` | yes | yes | The feature key it was written as, when it was one. |

**`Block`** when `type` is `"operation"`: An `@operation` that names an operation the spec has (SPEC §4.13).

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"operation"` | yes | no | Always `"operation"`. |
| `api` | `string` | yes | no | Its API, the name of its `[apis.<name>]`. |
| `method` | `string` | yes | no | The operation's HTTP method. |
| `path` | `string` | yes | no | The operation's path. |
| `deprecated` | `boolean` | yes | no | Whether the spec marks it deprecated. |
| `address` | `string` | yes | no | The operation's address in the published reference. |
| `operation_id` | `string` | no | no | The operation's id, when it has one. |
| `summary` | `string` | no | no | The operation's summary, when it has one. |

**`Block`** when `type` is `"container"`: A container directive.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"container"` | yes | no | Always `"container"`. |
| `name` | `string` | yes | no | Its name. |
| `attributes` | `Attribute[]` | yes | no | Its attributes. |
| `title` | `Inline[]` | yes | yes | Its title. |
| `children` | `Block[]` | yes | no | Its blocks. |

**`Block`** when `type` is `"group"`: A group, such as `@variant`, with its surviving arms.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"group"` | yes | no | Always `"group"`. |
| `name` | `string` | yes | no | Its name. |
| `arms` | `object[]` | yes | no | Its surviving arms. |
| `arms[].label` | `string` | yes | yes | The arm's label: its title as plain text, or the display labels of the values it names. |
| `arms[].attributes` | `Attribute[]` | yes | no | Its attributes. |
| `arms[].title` | `Inline[]` | yes | yes | Its title. |
| `arms[].span` | `Span` | yes | no | Its span. |
| `arms[].children` | `Block[]` | yes | no | Its blocks. |

**`Target`**: What a link or image resolved to. One of these options:

**`Target`** when `type` is `"external"`: An external link.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"external"` | yes | no | Always `"external"`. |

**`Target`** when `type` is `"page"`: A page the build publishes.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"page"` | yes | no | Always `"page"`. |
| `page` | `string` | yes | no | The page's source path. |
| `id` | `string` | yes | yes | The page id it names, if any. |
| `url` | `string` | yes | no | Its URL. |
| `text_filled` | `boolean` | yes | no | Whether the link had no text and took the page's title. |
| `line` | `string` | no | no | The line a link out of a line leads to; absent for any other link. |

**`Target`** when `type` is `"asset"`: An asset.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"asset"` | yes | no | Always `"asset"`. |
| `source` | `string` | yes | no | The asset's source path. |
| `path` | `string` | yes | no | The copy's path in this output. |
| `reference` | `string` | yes | no | How this page refers to the copy. |
| `url` | `string` | yes | yes | The copy's URL. |
| `fragment` | `string` | yes | yes | The fragment, without its `#`. |

**`Target`** when `type` is `"operation"`: An operation of an API spec.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"operation"` | yes | no | Always `"operation"`. |
| `api` | `string` | yes | no | Its API, the name of its `[apis.<name>]`. |
| `address` | `string` | yes | no | The operation's address in the published reference. |
| `method` | `string` | no | no | The operation's HTTP method, for an operation with no id. |
| `operation_id` | `string` | no | no | The operation's id, when it has one. |
| `path` | `string` | no | no | The operation's path, for an operation with no id. |

**`Target`** when `type` is `"unresolved"`: The build couldn't resolve it; the checks report why.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"unresolved"` | yes | no | Always `"unresolved"`. |

**`Environment`**: An environment's names and values, in its order. Its type is `{ [name]: string }`.

**`Inline`**: An inline: its `type` and the fields of that type, and its span in the block's file. Every option has these fields:

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `span` | `Span` | yes | no | Its span in the block's file. |

**`Inline`** when `type` is `"text"`: Text.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"text"` | yes | no | Always `"text"`. |
| `value` | `string` | yes | no | The text, decoded. |

**`Inline`** when `type` is `"code"`: A code span.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"code"` | yes | no | Always `"code"`. |
| `value` | `string` | yes | no | Its content. |

**`Inline`** when `type` is `"soft_break"`: A soft line break.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"soft_break"` | yes | no | Always `"soft_break"`. |

**`Inline`** when `type` is `"hard_break"`: A hard line break.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"hard_break"` | yes | no | Always `"hard_break"`. |

**`Inline`** when `type` is `"html"`: Inline HTML.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"html"` | yes | no | Always `"html"`. |
| `value` | `string` | yes | no | The HTML. |

**`Inline`** when `type` is `"emphasis"`: Emphasis.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"emphasis"` | yes | no | Always `"emphasis"`. |
| `children` | `Inline[]` | yes | no | What it emphasizes. |

**`Inline`** when `type` is `"strong"`: Strong emphasis.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"strong"` | yes | no | Always `"strong"`. |
| `children` | `Inline[]` | yes | no | What it emphasizes. |

**`Inline`** when `type` is `"link"`: A link.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"link"` | yes | no | Always `"link"`. |
| `destination` | `string` | yes | no | A page link's route and id, or an external link's destination as written. |
| `title` | `string` | yes | yes | Its title. |
| `target` | `Target` | yes | yes | The block's resolved link at the same span, or `null` for a glossary link. |
| `children` | `Inline[]` | yes | no | Its text. |

**`Inline`** when `type` is `"image"`: An image.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"image"` | yes | no | Always `"image"`. |
| `destination` | `string` | yes | no | Its destination. |
| `title` | `string` | yes | yes | Its title. |
| `target` | `Target` | yes | yes | The block's resolved image at the same span. |
| `alt` | `string` | yes | no | Its alternative text, as plain text. |
| `attributes` | `Attribute[]` | yes | no | Its attributes, such as `width`. |

**`Inline`** when `type` is `"phrase"`: A phrase the resolution left, which it doesn't.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `type` | `"phrase"` | yes | no | Always `"phrase"`. |
| `key` | `string` | yes | no | The phrase's key. |

**`Attribute`**: A directive's attribute.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `key` | `string` | yes | no | Its key. |
| `values` | `string[]` | yes | no | Its values: several for a set, none for a bare key. |

**`Primary`**: A directive's primary value. One of these options:

**`Primary`** when `kind` is `"text"`: Text.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `kind` | `"text"` | yes | no | Always `"text"`. |
| `inlines` | `Inline[]` | yes | no | The text. |

**`Primary`** when `kind` is `"identifier"`: An identifier.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `kind` | `"identifier"` | yes | no | Always `"identifier"`. |
| `text` | `string` | yes | no | The identifier. |

**`Primary`** when `kind` is `"line"`: The rest of the line, as written.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `kind` | `"line"` | yes | no | Always `"line"`. |
| `text` | `string` | yes | no | The line. |
