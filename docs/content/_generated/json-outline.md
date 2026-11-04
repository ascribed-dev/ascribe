@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/outline.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/outline.schema.json`]({repo}/blob/v{version}/schemas/outline.schema.json):

**`OutlineReport`**: What `ascribe outline <PAGE>` answers.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `page` | `string` | yes | no | The page's path from the content root, as links write it. |
| `file` | `string` | yes | no | The page's file from the project root, as `ascribe check` reports it. |
| `fragment` | `boolean` | yes | no | Whether it's a fragment, which pages include rather than link to. |
| `title` | `string` | yes | yes | Its title (frontmatter `title`). |
| `type` | `string` | yes | yes | Its content type; `null` for a fragment, or when no one type applies. |
| `build` | `string` | yes | yes | The build the headings are limited to, with `--build`. |
| `not_published` | `string` | yes | yes | Why that build doesn't publish the page, when it doesn't; there are no headings then. |
| `memos` | `OutlineMemo[]` | yes | no | The memo files about the page as a whole, in path order. A detached memo isn't listed. |
| `headings` | `object[]` | yes | no | The headings a link to the page can name, in document order, each id once. |
| `headings[].level` | `integer` | yes | no | 1 to 6. |
| `headings[].text` | `string` | yes | no | Its text, phrases replaced by their values. |
| `headings[].id` | `string` | yes | no | Its id: what a link writes after `#`. |
| `headings[].explicit_id` | `boolean` | yes | no | Whether the id is the heading's `@id`, which stays when its text changes. |
| `headings[].file` | `string` | yes | no | The file it's written in, from the project root. |
| `headings[].line` | `integer` | yes | no | Its line in that file, from 1. |
| `headings[].fragment` | `string` | yes | yes | The fragment it comes from, as a path from the content root; `null` for the page's own. |
| `headings[].memos` | `OutlineMemo[]` | yes | no | The memo files about its section, or a block in it, in path order. |

**`OutlineMemo`**: A memo file about a page or a heading.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `file` | `string` | yes | no | The memo file, from the project root. |
| `state` | `OutlineMemoState` | yes | no | `placed`, on what its `about` names or the block it quotes, or `outdated`, when no block holds its quote. |
| `quote` | `string` | yes | yes | Its `quote`; `null` when it has none. |
| `first_line` | `string` | yes | no | The first line of its text that isn't blank. |
| `personal` | `boolean` | yes | no | Whether it's one of the memos kept on this machine only, rather than the project's `memos/`. |

**`OutlineMemoState`**: Where a listed memo is. One of:

| Value | Description |
| --- | --- |
| `"placed"` | On its page or section, or the block its quote is in. |
| `"outdated"` | Its quote isn't found: it's on its page or section, and no block. |
