@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/versions-changes.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/versions-changes.schema.json`]({repo}/blob/v{version}/schemas/versions-changes.schema.json):

**`VersionsChangesReport`**: What differs between two lines of one target.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `target` | `string` | yes | no | The target. |
| `from` | `string` | yes | no | The line compared from, as the list writes it. |
| `to` | `string` | yes | no | The line compared to. |
| `added` | `LineChange[]` | yes | no | What `to` has and `from` doesn't, in page order. |
| `removed` | `LineChange[]` | yes | no | What `from` has and `to` doesn't. |
| `changed` | `LineChange[]` | yes | no | What both have, in a different state. |

**`LineChange`**: Content one line has and another doesn't, or has in another state.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `file` | `string` | yes | no | The file its mark is in, from the project root, as `ascribe check` reports it. |
| `line` | `integer` | yes | no | The mark's line, from 1. |
| `kind` | `string` | yes | no | What the mark is on: `page` (the frontmatter's `available`), `section` (a heading's), `block` (the block after it), or `row` (a table row's). |
| `page` | `string` | yes | no | The page's title, or the file's path when it has none. |
| `text` | `string` | yes | no | Its first line, as written. |
| `from_state` | `string` | no | no | The state in `from`, for a change of state: a lifecycle state, such as `preview`, or `ga`. |
| `heading` | `string` | no | no | The heading it's under, or the section's own heading; absent at the top of the page, and for a page. |
| `in_place_of` | `object` | no | no | For a page added, the page `from` has in its place and `to` doesn't, when `replaces` links the two, either way: the pair is one change, so that page isn't listed as removed. |
| `in_place_of.file` | `string` | yes | no | Its file, from the project root. |
| `in_place_of.line` | `integer` | yes | no | Its mark's line, from 1. |
| `in_place_of.page` | `string` | yes | no | Its title, or the file's path when it has none. |
| `to_state` | `string` | no | no | The state in `to`, for a change of state. |
