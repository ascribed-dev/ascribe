@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/permalinks-verify.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/permalinks-verify.schema.json`]({repo}/blob/v{version}/schemas/permalinks-verify.schema.json):

**`PermalinksVerifyReport`**: What `ascribe permalinks verify` answers.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `list` | `string` | yes | no | The list checked against, as given. |
| `found` | `string[]` | yes | no | The names the list has, as permalinks, or moved to another permalink or a page, in the order given, each once. |
| `retired` | `object[]` | yes | no | The names the list has as retired, in the order given, each once. |
| `retired[].name` | `string` | yes | no | The name. |
| `retired[].reason` | `string` | yes | no | Why it was retired, as the docs' authors wrote it. |
| `missing` | `string[]` | yes | no | The names it doesn't have at all, in the order given, each once. |
