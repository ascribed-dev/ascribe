@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/baseline-write.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/baseline-write.schema.json`]({repo}/blob/v{version}/schemas/baseline-write.schema.json):

**`BaselineWriteReport`**: What `ascribe baseline write --format json` writes.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `file` | `string` | yes | no | The file written, from the project's folder: `ascribe.baseline`. |
| `created` | `boolean` | yes | no | Whether the project had no baseline, so this made one. |
| `changed` | `boolean` | yes | no | Whether the file changed: it's new, or its text differs. With `--dry-run`, whether it would. |
| `written` | `boolean` | yes | no | Whether the file was written: it changed, without `--dry-run`. |
| `entries` | `integer` | yes | no | How many problems the baseline holds, after this. |
| `files` | `integer` | yes | no | How many files they're in. |
| `added` | `integer` | yes | no | How many of them this added. |
| `skipped` | `object` | yes | no | How many errors and warnings it left out, which a baseline never holds. |
| `skipped.errors` | `integer` | yes | no | How many errors. |
| `skipped.warnings` | `integer` | yes | no | How many warnings. |
