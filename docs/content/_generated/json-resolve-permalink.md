@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/resolve-permalink.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/resolve-permalink.schema.json`]({repo}/blob/v{version}/schemas/resolve-permalink.schema.json):

**`ResolvePermalinkReport`**: What `ascribe resolve go:<name> --format json` answers: the name's entry as the build's `_ascribe/help.json` writes it, with the file's `version`, `build`, and `line`.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `version` | `integer` | yes | no | The version of `help.json`'s format, which this shares. A later version adds keys and changes none of these. |
| `build` | `string` | yes | no | The build, without its line. |
| `name` | `string` | yes | no | The permalink's name, as asked for. |
| `title` | `string` | yes | no | The heading's text, or the page's title for a page's permalink, as plain text. |
| `text` | `string` | yes | no | The section as the plain Markdown output writes it, in Markdown, with each image as its alt text. |
| `source` | `string` | yes | no | The page the permalink is recorded on, as a path from the content root. |
| `address` | `string` | no | no | The permalink's address; absent without `[consumer] site`. |
| `choices` | `{ [name]: { [name]: string } }` | no | no | For a section with a switcher, its text with each arm chosen: keyed by dimension, then by value, both in the content model's order. Absent when the section has no switcher on a dimension. |
| `line` | `string` | no | no | The line, for a line of a versioned build that has more than one. |
| `summary` | `string` | no | no | The section's summary; absent when it has none. |
