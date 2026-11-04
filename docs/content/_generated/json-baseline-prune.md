@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/baseline-prune.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/baseline-prune.schema.json`]({repo}/blob/v{version}/schemas/baseline-prune.schema.json):

**`BaselinePruneReport`**: What `ascribe baseline prune --format json` writes.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `file` | `string` | yes | no | The file pruned, from the project's folder: `ascribe.baseline`. |
| `found` | `boolean` | yes | no | Whether the project has a baseline. Without one, there's nothing to prune, and nothing is written. |
| `changed` | `boolean` | yes | no | Whether the file changed: some entry was removed. With `--dry-run`, whether it would. |
| `written` | `boolean` | yes | no | Whether the file was written: it changed, without `--dry-run`. |
| `entries` | `integer` | yes | no | How many problems the baseline holds, after this. |
| `files` | `integer` | yes | no | How many files they're in. |
| `removed` | `integer` | yes | no | How many entries it removed. |
