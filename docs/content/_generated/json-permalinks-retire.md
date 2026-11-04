@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/permalinks-retire.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/permalinks-retire.schema.json`]({repo}/blob/v{version}/schemas/permalinks-retire.schema.json):

**`PermalinksRetireReport`**: What `ascribe permalinks retire` did.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `retired` | `string[]` | yes | no | The names retired, each written in `ascribe.addresses` as `go:<name>`: in `[retired]`, or, with `to`, in `[moved]`, in order. |
| `reason` | `string` | yes | yes | The reason each was given; none when each leads to `to`. |
| `to` | `string` | yes | yes | Where each address leads now, as written in `ascribe.addresses`; none when it shows the reason. |
| `created` | `boolean` | yes | no | Whether `ascribe.addresses` was made, since the project had none. |
| `list_changed` | `boolean` | yes | no | Whether `ascribe.permalinks` was changed to record them. Only a name the content no longer declares is recorded as retired. |
| `written` | `boolean` | yes | no | Whether the files were written: false with `--dry-run`, which says what would change. |
