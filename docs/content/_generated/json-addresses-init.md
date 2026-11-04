@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/addresses-init.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/addresses-init.schema.json`]({repo}/blob/v{version}/schemas/addresses-init.schema.json):

**`AddressesInitReport`**: What `ascribe addresses init --format json` writes.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `file` | `string` | yes | no | The file, from the project's folder: `ascribe.addresses`. |
| `created` | `boolean` | yes | no | Whether the project had no file, so this made one. A file already there is left as it is, but for what's moved into it. |
| `moved` | `string[]` | yes | no | The keys moved into the file from `ascribe.toml`'s `[permalinks]`, each `go:<name>`, in the order written there; empty when it has none. |
| `written` | `boolean` | yes | no | Whether a file was written: one made, or moved into, without `--dry-run`. |
