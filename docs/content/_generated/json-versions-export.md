@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/versions-export.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/versions-export.schema.json`]({repo}/blob/v{version}/schemas/versions-export.schema.json):

**`VersionsExportReport`**: What `ascribe versions export --format json` writes: the line, the folder, and what's in it.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `target` | `string` | yes | no | The target. |
| `line` | `string` | yes | no | The line exported, as `[releases.<target>]` writes it. |
| `status` | `string` | yes | no | The line's status: `current`, `live`, or `planned`. |
| `build` | `string` | yes | no | The build the line is a line of, as it's declared. |
| `folder` | `string` | yes | no | The folder written, from the current folder, with `/`. |
| `contents` | `string` | yes | no | The contents page, from the folder: `index.html`, or `contents.html` when the line's root page is `index.html`. |
| `pages` | `integer` | yes | no | How many pages the archive has. |
| `assets` | `integer` | yes | no | How many copies of images and files they use it has. |
| `removed` | `integer` | yes | no | How many files of an archive written there before it no longer has, removed. |
| `search_index_bytes` | `integer` | yes | no | The size of the search index, `search-index.js`, in bytes. |
| `search_index_large` | `boolean` | yes | no | Whether the search index is large enough that a reader's first search waits for it to load. |
| `unlinked` | `object[]` | yes | no | Each link written as its text: a link out of the line, or to an address no page of the line has. |
| `unlinked[].file` | `string` | yes | no | The page it's on, from the project root. |
| `unlinked[].text` | `string` | yes | no | Its text. |
| `unlinked[].href` | `string` | yes | no | Where it led. |
| `unlinked[].line` | `string` | yes | yes | For a link out of the line, the line it led to. |
