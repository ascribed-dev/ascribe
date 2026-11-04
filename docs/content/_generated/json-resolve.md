@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/resolve.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/resolve.schema.json`]({repo}/blob/v{version}/schemas/resolve.schema.json):

**`ResolveReport`**: What `ascribe resolve <PAGE> --format json` answers.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `page` | `string` | yes | no | The page, as a path from the content root. |
| `build` | `string` | yes | no | The build. |
| `emit` | `Emit` | yes | no | Which output `text` is written as: `plain`, the build's plain output, or `site`, its site output. |
| `not_published` | `string` | yes | yes | Why the build doesn't publish the page, when it doesn't; `text` is empty then. |
| `route` | `string` | yes | yes | The page's route in the build's site. |
| `text` | `string` | yes | no | The page as the output `emit` names writes it: with its frontmatter first when asked for. |

**`Emit`**: Which output a page is written as, by the name `ascribe build --emit` gives it. One of:

| Value | Description |
| --- | --- |
| `"plain"` | The plain output: fully resolved CommonMark with no HTML. |
| `"site"` | The site output: Markdown with Ascribe's elements, as a site's integration receives it. |
