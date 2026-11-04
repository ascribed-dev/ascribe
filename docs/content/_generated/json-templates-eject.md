@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/templates-eject.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/templates-eject.schema.json`]({repo}/blob/v{version}/schemas/templates-eject.schema.json):

**`TemplatesEjectReport`**: What `ascribe templates eject` did: the files it wrote and the types it added, or, when it refused anything, what it refused, and then it wrote nothing.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `written` | `string[]` | yes | no | The files written, from the folder `ascribe.toml` is in, `/`-separated, in the order they were written: each template, its guide, and Ascribe's notes on its type, then `templates.lock`, then `ascribe.toml`, when it changed. Empty when anything was refused. With `dry_run`, the files it would write. |
| `dry_run` | `boolean` | yes | no | Whether it ran with `--dry-run`, and wrote nothing. |
| `added` | `object[]` | yes | no | The types added to `ascribe.toml`, in the order they were named. Empty when anything was refused. |
| `added[].name` | `string` | yes | no | The type's name, `[types.<name>]`, which is the template's. |
| `added[].summary` | `string` | yes | no | Their index's summary of it. |
| `added[].template` | `string` | yes | no | Its template, from the folder `ascribe.toml` is in. |
| `added[].guide` | `string[]` | yes | no | Its guide, from the folder `ascribe.toml` is in, in the order to read them: their guide, then Ascribe's notes on the type. |
| `added[].copied` | `string[]` | yes | no | The fields of its frontmatter declared as another of the project's types declares them, rather than as plain strings, in order. |
| `kept` | `object[]` | yes | no | The types `ascribe.toml` declared already and kept, with `--keep-type`, in the order they were named: each takes its template and guide from `templates/templates.lock`, or, written in another folder, was given its `template` and `guide`, and nothing else in it changed. Empty when anything was refused. |
| `kept[].name` | `string` | yes | no | The type's name, `[types.<name>]`, which is the template's. |
| `kept[].summary` | `string` | yes | no | Their index's summary of it. |
| `kept[].template` | `string` | yes | no | The template it was given, from the folder `ascribe.toml` is in. |
| `kept[].guide` | `string[]` | yes | no | The guide it was given, from the folder `ascribe.toml` is in, in the order to read them: their guide, then Ascribe's notes on the type. |
| `kept[].left` | `string[]` | yes | no | The type's keys left as they were, in the order `ascribe.toml` has them, such as `purpose` and `frontmatter`. |
| `refused` | `object[]` | yes | no | What was refused, in the order it was named. When it isn't empty, nothing was written. |
| `refused[].name` | `string` | yes | no | The name given. |
| `refused[].reason` | `RefusalReason` | yes | no | Why it was refused. |
| `refused[].path` | `string` | yes | yes | The file in the way, from the folder `ascribe.toml` is in, when that's the reason; otherwise null. |
| `refused[].message` | `string` | yes | no | Why, in a sentence. |
| `page_declared` | `boolean` | yes | no | Whether `ascribe.toml` declared no types, so that every page was of the implicit type `page`, and `[types.page]` was written out first, as it was, so the pages that aren't of a type added can name it. |
| `untyped` | `string[]` | yes | no | The project's pages that name no type, from the folder `ascribe.toml` is in, in order. With the types added the project declares types, so each needs `type:` in its frontmatter, naming its type. |

**`RefusalReason`**: Why a template was refused. One of:

| Value | Description |
| --- | --- |
| `"unknown"` | The set has no template of that name. |
| `"declared"` | `ascribe.toml` declares a type of that name already. |
| `"file_in_the_way"` | A file is where one of its files would be written. |
| `"type_not_table"` | Asked to keep the type, `ascribe.toml` writes it as an inline or a dotted table, which eject doesn't add to. |
| `"type_has_template"` | Asked to keep the type, it names a `template` or a `guide` already, or `templates/templates.lock` gives it them. |
| `"frontmatter_differs"` | Asked to keep the type, its frontmatter doesn't declare a field the template's has, or declares it as something other than a string. |
