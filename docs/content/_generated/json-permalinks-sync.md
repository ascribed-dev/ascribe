@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/permalinks-sync.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/permalinks-sync.schema.json`]({repo}/blob/v{version}/schemas/permalinks-sync.schema.json):

**`PermalinksSyncReport`**: What `ascribe permalinks sync` does to the list.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `changed` | `boolean` | yes | no | Whether the list's file changes: it's new, or its text differs. |
| `written` | `boolean` | yes | no | Whether the list's file was written: it changes, and nothing said to write nothing (`--check`, `--dry-run`). |
| `created` | `boolean` | yes | no | Whether the project had no list, so this makes one. |
| `added` | `string[]` | yes | no | The names added, by name. |
| `moved` | `object[]` | yes | no | The names whose place changed, by name. |
| `moved[].name` | `string` | yes | no | Its name. |
| `moved[].from` | `string` | yes | no | Where the list had it. |
| `moved[].to` | `string` | yes | no | Where it's declared now. |
| `aliased` | `string[]` | yes | no | The names moved to another permalink recorded, new or leading somewhere new, by name. |
| `forwarded` | `string[]` | yes | no | The names moved to a page recorded, new or leading somewhere new, by name. |
| `retired` | `string[]` | yes | no | The retired permalinks recorded, new or with a new reason, by name. |
| `removed` | `string[]` | yes | no | The names the list keeps that the project no longer has, by name. `ascribe check` warns about each. |
