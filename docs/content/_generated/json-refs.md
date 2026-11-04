@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/refs.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/refs.schema.json`]({repo}/blob/v{version}/schemas/refs.schema.json):

**`RefsReport`**: What `ascribe refs <TARGET>` answers.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `target` | `string` | yes | no | The target, as given. |
| `kind` | `TargetKind` | yes | no | What the target is. |
| `exists` | `boolean` | yes | no | Whether the target exists. When it doesn't, the only places are the memo files that name it. |
| `places` | `object[]` | yes | no | The places that use it, file by file in path order and in document order within a file, then the memo files about it, in path order; at most `shown` of them. |
| `places[].file` | `string` | yes | no | The file, from the project root, as `ascribe check` reports it. |
| `places[].line` | `integer` | yes | no | The line, from 1. |
| `places[].column` | `integer` | yes | no | The column, from 1, in Unicode characters. |
| `places[].use` | `string` | yes | no | How it uses the target: `link`, `include`, `phrase`, `availability`, `term`, `variant`, `note`, `widget`, `image`, an image that shows it, `html` or `frontmatter`, raw HTML or a frontmatter field that names it, or `memo`, a memo file about it. |
| `places[].personal` | `boolean` | yes | no | Whether it's a memo file kept on this machine only, in `.ascribe/memos/`, rather than the project's `memos/`; `false` for every place that isn't a memo. |
| `places[].spec` | `string` | yes | yes | The availability spec, as written, for an `availability` place; null for every other place. |
| `total` | `integer` | yes | no | How many places use it. |
| `shown` | `integer` | yes | no | How many are listed. |
| `truncated` | `boolean` | yes | no | Whether the list was cut: `shown` is less than `total`. |
| `next_command` | `string` | yes | yes | The command that lists them all, when the list was cut. |

**`TargetKind`**: What a target is. One of:

| Value | Description |
| --- | --- |
| `"page"` | A page. |
| `"fragment"` | A fragment. |
| `"heading"` | A heading, by its page and id. |
| `"image"` | An image under the content root. |
| `"phrase"` | A phrase, `phrase:<key>`. |
| `"feature"` | A feature, `feature:<key>`. |
| `"term"` | A glossary term, `term:<id>`. |
| `"dimension"` | A dimension, `dimension:<name>`. |
| `"note"` | A note type, `note:<type>`. |
| `"widget"` | A project widget, `widget:<name>`. |
| `"release"` | A line of a release target, `release:<target>@<version>`. |
