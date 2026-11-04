@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/versions-retire.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/versions-retire.schema.json`]({repo}/blob/v{version}/schemas/versions-retire.schema.json):

**`VersionsRetireReport`**: What `ascribe versions retire --format json` writes: the line, its address, the marks rewritten, what no line has any more, and where each of the line's addresses should go.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `written` | `boolean` | yes | no | Whether the files were written: false with `--dry-run`. |
| `retired` | `boolean` | yes | no | Whether this run retired the line: false when it was archived already, and the run only listed or removed what it left. |
| `target` | `string` | yes | no | The target. |
| `line` | `string` | yes | no | The line, as `[releases.<target>]` writes it. |
| `url` | `string` | yes | yes | Where the line's archive is read. |
| `ended` | `string` | yes | yes | The day the line's support ended: its `ends`, or the day it was retired. |
| `features` | `object[]` | yes | no | Each feature whose spec is rewritten. |
| `features[].key` | `string` | yes | no | Its key. |
| `features[].from` | `string` | yes | no | Its spec before. |
| `features[].to` | `string` | yes | no | Its spec after. |
| `features[].kept` | `string[]` | yes | no | The other targets its spec names, left as they were. |
| `files` | `object[]` | yes | no | Each source file whose marks are rewritten, from the project root. |
| `files[].file` | `string` | yes | no | The file, from the project root. |
| `files[].marks` | `object[]` | yes | no | Each mark rewritten. |
| `files[].marks[].line` | `integer` | yes | no | Its line, from 1. |
| `files[].marks[].from` | `string` | yes | no | The spec as written. |
| `files[].marks[].to` | `string` | yes | no | The spec as rewritten. |
| `unused` | `object[]` | yes | no | What no live or planned line has, and the marks that say nothing once rewritten, by file and line. |
| `unused[].file` | `string` | yes | no | The file, from the project root. |
| `unused[].line` | `integer` | yes | no | The mark's line, from 1. |
| `unused[].kind` | `string` | yes | no | `page`, `section`, `block`, `row`, or `mark`, for a mark that says nothing and can go while its content stays. |
| `unused[].text` | `string` | yes | no | Its first line: a page's title, a section's heading, a block's or a row's first line, or the mark. |
| `removed` | `boolean` | yes | no | Whether what `unused` lists was deleted, with `--remove-unused`. |
| `redirects` | `object[]` | yes | no | Where each address the line published should go, by address: the same page in the current line, the page that replaces it there, or its page in the archive. |
| `redirects[].from` | `string` | yes | no | The address, from the site's root. |
| `redirects[].to` | `string` | yes | no | Where it should go: a path from the site's root, or the archive's full address. |
| `recorded` | `boolean` | yes | no | Whether `ascribe.addresses` records the redirects that go to a page in the current line, in `[moved]` keyed by the address, so the build writes them: true when the project has the file and the line has one. With `--dry-run`, whether it would. Those to the archive are never recorded. |
| `outputs` | `OutputJson[]` | yes | no | The outputs of builds that aren't lines of the target that the rewrite changes, by build and output: they need `--force`, and a dry run lists them. |
| `expected` | `OutputJson[]` | yes | no | The outputs of builds that aren't lines of the target whose pages change only in the target's part of their availability lines, by build and output: expected, so they need no `--force`. |
| `from` | `string` | yes | yes | The `ascribe.toml` the target's lines are written in, from the current folder, when the project's `[from]` takes them from another project's: the retirement is made in that project, and `files` are from its root. `null` when they're written in the project's own. |

**`OutputJson`**: One output the release changes.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `build` | `string` | yes | no | The build: `site`, or an older line's, `site@3.4`. Empty for the file the lines share. |
| `output` | `string` | yes | no | The output: `site`, `plain`, or `json`, or `_ascribe` for the file the lines share. |
| `added` | `string[]` | yes | no | The files it gains, by their path in the output. |
| `removed` | `string[]` | yes | no | The files it loses. |
| `changed` | `string[]` | yes | no | The files that change. |
