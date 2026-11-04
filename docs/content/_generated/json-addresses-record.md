@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/addresses-record.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/addresses-record.schema.json`]({repo}/blob/v{version}/schemas/addresses-record.schema.json):

**`AddressesRecordReport`**: What `ascribe addresses move` and `retire` write with `--format json`.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `file` | `string` | yes | no | The file, from the project's folder: `ascribe.addresses`. |
| `table` | `string` | yes | no | The table the entry is in: `moved` or `retired`. |
| `key` | `string` | yes | no | The entry's key: a page's path from the content root, a file's path with `#` and an id, an address starting with `/`, or a permalink, `go:<name>`. |
| `value` | `string` | yes | no | The entry's value: where it moved, or why it was retired. |
| `changed` | `boolean` | yes | no | Whether the file changed: an entry already the same changes nothing. |
| `written` | `boolean` | yes | no | Whether the change was written: the MCP server's tools write nothing, and give the edits to apply. |
| `edits` | `object[]` | yes | no | The edits that make the change, each replacing the text of its range in the file as it was; none when nothing changed. |
| `edits[].range` | `object` | yes | no | The text it replaces. |
| `edits[].range.start` | `Pos` | yes | no | Its first character. |
| `edits[].range.end` | `Pos` | yes | no | Just past its last character. |
| `edits[].new_text` | `string` | yes | no | The text that replaces it. |

**`Pos`**: A position in a file.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `line` | `integer` | yes | no | The line, from 1. |
| `column` | `integer` | yes | no | The column, from 1, in Unicode characters (not bytes or UTF-16 units). |
| `offset` | `integer` | yes | no | The byte offset from the start of the file. |
