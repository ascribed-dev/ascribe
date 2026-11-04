@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/sources-update.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/sources-update.schema.json`]({repo}/blob/v{version}/schemas/sources-update.schema.json):

**`SourcesUpdateReport`**: What `ascribe sources update --format json` writes: how each source's pin moved, and the pages whose examples the new copies change.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `changed` | `boolean` | yes | no | Whether any file changed: a copy, or the lock. With `--dry-run`, whether any would. |
| `written` | `boolean` | yes | no | Whether the copies and the lock were written: false with `--dry-run`, which lists no pages, since it wrote no copies. |
| `sources` | `object[]` | yes | no | What the update did for each source, in declaration order. |
| `sources[].name` | `string` | yes | no | The source. |
| `sources[].git` | `string` | yes | no | Its repository. |
| `sources[].followed` | `string` | yes | no | The revision followed: `--to`'s, the branch, or `HEAD`. |
| `sources[].from` | `string` | yes | yes | The pin before, if it had one. |
| `sources[].to` | `string` | yes | no | The pin now. |
| `sources[].moved` | `boolean` | yes | no | Whether the pin moved. |
| `sources[].back` | `boolean` | yes | no | Whether the new pin isn't after the old one: it moved back, or to another line of history. No commits are counted then. |
| `sources[].commits` | `object` | yes | yes | The commits after the old pin up to the new one, when it moved forward from one and the repository could say. |
| `sources[].commits.count` | `integer` | yes | no | How many. |
| `sources[].commits.newest` | `object[]` | yes | no | The newest, newest first, at most 20. |
| `sources[].commits.newest[].commit` | `string` | yes | no | Its full hash. |
| `sources[].commits.newest[].subject` | `string` | yes | no | The first line of its message. |
| `sources[].files` | `object[]` | yes | no | The copies that changed. |
| `sources[].files[].path` | `string` | yes | no | Its path in the source. |
| `sources[].files[].change` | `FileChange` | yes | no | What happened to it. |
| `sources[].examples` | `object[]` | yes | no | The tested examples snippets use whose status the new pin changed: each one that went stale, lost its observation, or became current, by path. |
| `sources[].examples[].path` | `string` | yes | no | Its path in the source. |
| `sources[].examples[].status` | `ExampleState` | yes | no | Its status at the new pin. |
| `sources[].examples[].was` | `ExampleState` | yes | yes | Its status before; `null` when it wasn't an example, or had no copy. |
| `sources[].failed` | `object[]` | yes | no | The files snippets name that couldn't be copied. |
| `sources[].failed[].path` | `string` | yes | no | Its path in the source. |
| `sources[].failed[].reason` | `string` | yes | no | Why. |
| `sources[].first_copy` | `boolean` | yes | no | Whether these are the first files copied from the source. |
| `pages` | `object[]` | yes | yes | The pages whose examples the new copies change, as `ascribe drift --format json` lists them; `null` when nothing changed, or when that can't be told. |
| `pages[].path` | `string` | yes | no | The page's content path. |
| `pages[].route` | `string` | yes | no | Its route, in the first of the builds it's in. |
| `pages[].builds` | `string[]` | yes | no | The builds that show it with a changed example, in the order asked for. |
| `pages[].page_changed` | `boolean` | yes | no | Whether the page changed apart from its examples: its own file, or its resolved content through anything else it uses (a fragment, the content model), in any of those builds. When it didn't, the example changed and the words around it didn't. |
| `pages[].examples` | `object[]` | yes | no | The examples that changed, by address. Empty when only `broken` isn't. |
| `pages[].examples[].address` | `string` | yes | no | Its address, as the page writes it. |
| `pages[].examples[].source` | `string` | yes | no | The source the address names. |
| `pages[].examples[].file` | `string` | yes | no | The code file now, from the repository's root. |
| `pages[].examples[].was_file` | `string` | yes | yes | The code file at the base, when it was renamed since. |
| `pages[].examples[].added` | `integer` | yes | no | Lines only in the code now. |
| `pages[].examples[].removed` | `integer` | yes | no | Lines only in the code at the base. |
| `pages[].broken` | `object[]` | yes | no | The examples that resolved at the base and don't now, by address. |
| `pages[].broken[].address` | `string` | yes | no | Its address, as the page writes it. |
| `pages[].broken[].source` | `string` | yes | no | The source the address names. |
| `pages[].broken[].problem` | `string` | yes | no | The slug of the diagnostic `ascribe check` reports for it (`snippet-region-missing`). |
| `pages[].broken[].reason` | `string` | yes | no | Why it doesn't resolve, in a few words. |
| `pages_unavailable` | `string` | yes | yes | Why there are no `pages`, when there aren't. |

**`FileChange`**: What happened to a copy. One of:

| Value | Description |
| --- | --- |
| `"added"` | It's new. |
| `"changed"` | Its contents changed. |
| `"removed"` | It's gone. |

**`ExampleState`**: Where a tested example stands against the observation copied with it. One of:

| Value | Description |
| --- | --- |
| `"current"` | The observation lists the file as it is: the page says what it was tested with. |
| `"stale"` | The file changed, and the observation didn't follow. |
| `"unobserved"` | No observation lists the file. |
