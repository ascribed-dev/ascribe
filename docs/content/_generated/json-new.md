@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/new.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/new.schema.json`]({repo}/blob/v{version}/schemas/new.schema.json):

**`NewReport`**: What `ascribe new` wrote, or with `--dry-run` would write.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `path` | `string` | yes | no | The page, from the folder `ascribe.toml` is in, `/`-separated. |
| `type` | `string` | yes | no | Its type. |
| `title` | `string` | yes | yes | The title given, or `null` when the page keeps the template's. |
| `template` | `string` | yes | yes | The type's template, from the folder `ascribe.toml` is in, or `null` when it has none and the page is its frontmatter alone. |
| `guide` | `string[]` | yes | no | The type's guide, from the folder `ascribe.toml` is in, in the order to read it. |
| `placeholders` | `integer` | yes | no | How many places the page has that its template left to fill in, as `template-placeholder-left` finds them: its placeholders, and the frontmatter values still the template's. |
| `written` | `boolean` | yes | no | Whether the page was written: `false` with `--dry-run`. |
| `text` | `string` | yes | no | The page's text. |
