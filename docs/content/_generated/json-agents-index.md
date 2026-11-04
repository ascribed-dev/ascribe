@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/agents-index.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/agents-index.schema.json`]({repo}/blob/v{version}/schemas/agents-index.schema.json):

**`AgentsIndexReport`**: What `ascribe agents index --format json` answers.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `build` | `string` | yes | no | The build. |
| `limit` | `integer` | yes | no | The most bytes the index holds. |
| `bytes` | `integer` | yes | no | How many bytes it holds. |
| `offered` | `integer` | yes | no | How many pages the build offers: those it publishes, apart from the `unlisted`. |
| `listed` | `string[]` | yes | no | The pages the index lists, as paths from the content root, in its order. |
| `left_out` | `string[]` | yes | no | The offered pages the index leaves out to stay under `limit`, as paths from the content root. |
| `without_description` | `string[]` | yes | no | The pages the index lists without their description, to stay under `limit`, as paths from the content root. |
| `file` | `string` | yes | yes | The file the index was written into, between its markers, from the folder the command ran in; `null` when it was printed. |
| `changed` | `boolean` | yes | no | Whether that file changed: with `--dry-run`, whether it would. |
| `written` | `boolean` | yes | no | Whether the file and the pages were written: false when the index was printed, or with `--dry-run`. |
| `folder` | `string` | yes | yes | The folder the pages were copied into, from the folder the command ran in; `null` without `--pages`. |
| `pages` | `object[]` | yes | no | The pages copied into the folder: every offered page, with `--pages`. |
| `pages[].source` | `string` | yes | no | The page, as a path from the content root. |
| `pages[].path` | `string` | yes | no | Its Markdown version's path in the folder. |
| `text` | `string` | yes | no | The index's Markdown. |
