@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/explain-list.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/explain-list.schema.json`]({repo}/blob/v{version}/schemas/explain-list.schema.json):

**`ExplainListReport`**: What `ascribe explain --list` answers: every diagnostic.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `diagnostics` | `object[]` | yes | no | Every diagnostic, in code order. |
| `diagnostics[].code` | `string` | yes | no | The code. |
| `diagnostics[].slug` | `string` | yes | no | The slug. |
| `diagnostics[].severity` | `string` | yes | no | `error`, `warning`, or `advice`. |
