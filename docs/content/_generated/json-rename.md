@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/rename.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/rename.schema.json`]({repo}/blob/v{version}/schemas/rename.schema.json):

**`RenameReport`**: What `ascribe rename --format json` writes, and the MCP server's `ascribe_rename` gives.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `files` | `object[]` | yes | no | Each file the rename changes, in path order, with its whole text after. The file that moves is under where it was, and `ascribe.addresses` is one of them when the rename records a move. |
| `files[].file` | `string` | yes | no | The file, from the current folder. |
| `files[].text` | `string` | yes | no | Its whole text after the rename. |
| `files[].edits` | `integer` | yes | no | How many places in it change. |
| `recorded` | `object[]` | yes | no | The entries of `ascribe.addresses`'s `[moved]` the rename records, in key order, so the build forwards the old addresses; none when it records nothing, as for a phrase, a fragment, or a project with no `ascribe.addresses`. |
| `recorded[].key` | `string` | yes | no | The entry's key: where the page was. |
| `recorded[].value` | `string` | yes | no | The entry's value: where it is now. |
| `written` | `boolean` | yes | no | Whether the files were written and the file moved: `--dry-run`, and the MCP server's `ascribe_rename`, write nothing, and give the changes to make. |
| `moved` | `object` | no | no | The page or fragment that moves, when one does. |
| `moved.from` | `string` | yes | no | Where it is, from the current folder. |
| `moved.to` | `string` | yes | no | Where it goes, from the current folder. Its folder is made when it isn't there. |
