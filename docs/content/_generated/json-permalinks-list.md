@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/permalinks-list.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/permalinks-list.schema.json`]({repo}/blob/v{version}/schemas/permalinks-list.schema.json):

**`PermalinksListReport`**: What `ascribe permalinks list` answers.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `has_list` | `boolean` | yes | no | Whether the project has `ascribe.permalinks`. Without it, every permalink is `not_listed`, and nothing is `removed`. |
| `permalinks` | `object[]` | yes | no | Every permalink the content declares, in path order and then in document order, a page's own first. A name declared in several places is here once for each. |
| `permalinks[].name` | `string` | yes | no | Its name. |
| `permalinks[].place` | `string` | yes | no | Where it's declared, as the list writes it: the file's path from the content root, and `#` and the id for a heading. |
| `permalinks[].file` | `string` | yes | no | The file it's declared in, from the project root, as `ascribe check` reports it. |
| `permalinks[].line` | `integer` | yes | no | The line of its `@id` or its frontmatter `permalink`, from 1. |
| `permalinks[].state` | `ListState` | yes | no | Whether the list has it: `listed` at one of the places its name is declared, `moved` when the list has it somewhere else, or `not_listed`. |
| `permalinks[].listed_at` | `string` | yes | yes | The place the list has for it, when that's somewhere else. |
| `aliases` | `object[]` | yes | no | The names `ascribe.addresses` moves to another permalink, by name. |
| `aliases[].name` | `string` | yes | no | Its name. |
| `aliases[].target` | `string` | yes | no | The name it was moved to. |
| `aliases[].state` | `ListState` | yes | no | Whether the list has it, leading there: `listed`, or `not_listed`. |
| `moved` | `object[]` | yes | no | The names `ascribe.addresses` moves to a page, by name. |
| `moved[].name` | `string` | yes | no | Its name. |
| `moved[].to` | `string` | yes | no | Where its address leads now, as written: a page, and `#` and a heading's id. |
| `moved[].state` | `ListState` | yes | no | Whether the list has it, leading there: `listed`, or `not_listed`. |
| `retired` | `object[]` | yes | no | The names `ascribe.addresses` retires, or moves to one that's retired, by name. |
| `retired[].name` | `string` | yes | no | Its name. |
| `retired[].reason` | `string` | yes | no | Why it was retired. |
| `retired[].state` | `ListState` | yes | no | Whether the list has it as retired, with this reason: `listed`, or `not_listed`. |
| `removed` | `object[]` | yes | no | The names the list has that the project no longer has, by name. |
| `removed[].name` | `string` | yes | no | Its name. |
| `removed[].was` | `ListedAs` | yes | no | What the list has it as. |
| `removed[].place` | `string` | yes | no | Where the list says a permalink was declared; for an alias, the name it led to; for a moved one, where; for a retired one, its reason. |

**`ListState`**: Whether the list has a permalink where it's declared. One of:

| Value | Description |
| --- | --- |
| `"listed"` | At one of the places its name is declared. |
| `"moved"` | At a place that doesn't declare it any more. |
| `"not_listed"` | Not at all. |

**`ListedAs`**: Which of the list's tables has a name. One of:

| Value | Description |
| --- | --- |
| `"permalink"` | `[permalinks]`: a permalink the content declared. |
| `"alias"` | `[aliases]`: a name moved to another permalink. |
| `"moved"` | `[moved]`: a name moved to a page. |
| `"retired"` | `[retired]`: a retired permalink. |
