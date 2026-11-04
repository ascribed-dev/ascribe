@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/templates-list.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/templates-list.schema.json`]({repo}/blob/v{version}/schemas/templates-list.schema.json):

**`TemplatesListReport`**: What `ascribe templates list --format json` writes: the templates Ascribe carries, and where they're from.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `repository` | `string` | yes | no | The repository of The Good Docs Project's templates. |
| `tag` | `string` | yes | no | The tag of theirs the templates were converted from. |
| `commit` | `string` | yes | no | The tag's commit. |
| `license` | `string` | yes | no | Their license. |
| `templates` | `object[]` | yes | no | The templates, in the set's order. |
| `templates[].name` | `string` | yes | no | Its name, which `ascribe templates eject` takes, and its type's. |
| `templates[].summary` | `string` | yes | no | What it's for. |
