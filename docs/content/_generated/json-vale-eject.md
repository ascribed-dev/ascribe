@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/vale-eject.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/vale-eject.schema.json`]({repo}/blob/v{version}/schemas/vale-eject.schema.json):

**`ValeEjectReport`**: What `ascribe vale eject --format json` writes: the preset written out, and the files written.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `preset` | `string` | yes | no | The preset written out. |
| `written` | `string[]` | yes | no | The files written, from the project's folder, `/`-separated, in the order they were written: `ascribe.toml` last. With `--dry-run`, the files it would write. |
| `dry_run` | `boolean` | yes | no | Whether it ran with `--dry-run`, and wrote nothing. |
