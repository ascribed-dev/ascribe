@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/fmt.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/fmt.schema.json`]({repo}/blob/v{version}/schemas/fmt.schema.json):

**`FmtReport`**: What `ascribe fmt --format json` writes: one document. Fields can be added without a new `schema_version`, so a reader ignores fields it doesn't know.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `written` | `boolean` | yes | no | Whether the files were rewritten: `false` with `--check` or `--dry-run`, which write nothing. |
| `files` | `object[]` | yes | no | Each file that changed, or with `--check` or `--dry-run` would change, in path order, with the edits that format it. |
| `files[].file` | `string` | yes | no | The file, relative to the project root (the folder of `ascribe.toml`), with `/` separators; a file outside it as it was found. |
| `files[].edits` | `object[]` | yes | no | The edits, each replacing the text of its range in the file as it was before formatting. They don't overlap, and are in file order. |
| `files[].edits[].range` | `object` | yes | no | The text it replaces. |
| `files[].edits[].range.start` | `Pos` | yes | no | Its first character. |
| `files[].edits[].range.end` | `Pos` | yes | no | Just past its last character. |
| `files[].edits[].new_text` | `string` | yes | no | The text that replaces it. |
| `refused` | `object[]` | yes | no | The files left alone because a symbolic link on the way to them leads to a file that isn't a source file of the content root. |
| `refused[].file` | `string` | yes | no | The file, as a formatted file's `file` is. |
| `refused[].reason` | `string` | yes | no | Why, as `ascribe check` words it. |

**`Pos`**: A position in a file.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `line` | `integer` | yes | no | The line, from 1. |
| `column` | `integer` | yes | no | The column, from 1, in Unicode characters (not bytes or UTF-16 units). |
| `offset` | `integer` | yes | no | The byte offset from the start of the file. |
