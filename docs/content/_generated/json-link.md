@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/link.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/link.schema.json`]({repo}/blob/v{version}/schemas/link.schema.json):

**`LinkReport`**: What `ascribe link <TARGET> --from <PAGE>` answers.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `from` | `string` | yes | no | The page the link is written on, as the command was given it: a path from the folder it was run in, as `ascribe check` names a file. |
| `target` | `string` | yes | no | The target, as given. |
| `exists` | `boolean` | yes | no | Whether a link from that page to the target works. An external URL counts as existing; it isn't checked. |
| `kind` | `LinkKind` | yes | no | What the target is. |
| `path` | `string` | yes | yes | The file it names, as a path from the content root; `null` for an external URL, or a file that doesn't exist. |
| `id` | `string` | yes | yes | The heading id after `#`, if any. |
| `title` | `string` | yes | yes | The page's title, or the heading's text: what a link with no text shows. |
| `href` | `string` | yes | yes | The destination to write on that page, when the target exists. |
| `problem` | `string` | yes | yes | Why it doesn't work, when it doesn't. |
| `closest` | `object[]` | yes | no | The closest targets that exist, best first, when it doesn't. |
| `closest[].href` | `string` | yes | no | The destination to write. |
| `closest[].path` | `string` | yes | no | The page, as a path from the content root. |
| `closest[].id` | `string` | yes | yes | The heading id, if it's a heading. |
| `closest[].title` | `string` | yes | yes | The page's title, or the heading's text. |
| `lines` | `object[]` | no | no | Where a link to a page goes in each line of each versioned build, newest first; absent when the target isn't a page or no build is versioned. |
| `lines[].build` | `string` | yes | no | The line's build: the declared build for the current line, and `<build>@<line>` for each other. |
| `lines[].line` | `string` | yes | no | The line, as `[releases.<target>]` writes it. |
| `lines[].page` | `string` | yes | yes | The page it lands on, as a path from the content root: the target, or the page that takes its place in the line. `null` when no live line publishes the target, which is `link-page-dropped`. |
| `lines[].leads_to` | `string` | yes | yes | For a link out of the line, the line it leads to. |
| `operation` | `object` | no | no | The operation a link to a declared API's spec names, when the spec has it; absent for any other target. |
| `operation.api` | `string` | yes | no | The API's name. |
| `operation.operation_id` | `string` | yes | yes | Its `operationId`; `null` for an operation with none, which a link names by its method and path. |
| `operation.method` | `string` | yes | no | Its method, in lowercase. |
| `operation.path` | `string` | yes | no | Its path, as the spec writes it. |
| `operation.summary` | `string` | yes | yes | Its `summary`, when it has one. |
| `operation.tag` | `string` | yes | yes | Its first tag, when it has one. |
| `operation.deprecated` | `boolean` | yes | no | Whether the spec marks it `deprecated`. |
| `operation.address` | `string` | yes | yes | Its address in the published reference, as the editor's build writes the link; `null` when the reference has none for it: an operation with no id, under an address pattern. |

**`LinkKind`**: What a link target is. One of:

| Value | Description |
| --- | --- |
| `"page"` | A page. |
| `"heading"` | A heading on a page. |
| `"fragment"` | A fragment, which pages include and links can't name. |
| `"file"` | A file that isn't a page, which a build copies: an image or a download. |
| `"external"` | A URL with a scheme. |
| `"operation"` | An operation of a declared API's spec: a link to the spec with a fragment. |
| `"missing"` | Nothing: no file has that path. |
