@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/drift.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/drift.schema.json`]({repo}/blob/v{version}/schemas/drift.schema.json):

**`DriftReport`**: The report, as `ascribe drift --format json` writes it.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `base` | `object` | yes | no | What was compared with. |
| `base.requested` | `string` | yes | no | The revision asked for, or the default branch used. |
| `base.commit` | `string` | yes | no | The commit it names. |
| `base.merge_base` | `string` | yes | yes | The merge base of that commit and `HEAD`, which the comparison reads; `null` with `--base-exact`, which reads `commit`. |
| `repository` | `object` | yes | no | Where the project is. |
| `repository.root` | `string` | yes | no | The repository's top-level folder, as git prints it. |
| `repository.project_prefix` | `string` | yes | no | The project's folder inside it, with a trailing `/`, or empty when the project is at the repository's root. |
| `pages` | `object[]` | yes | no | Every page with an example that changed or broke, in path order. |
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
