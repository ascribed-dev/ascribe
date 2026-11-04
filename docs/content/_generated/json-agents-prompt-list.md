@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/agents-prompt-list.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/agents-prompt-list.schema.json`]({repo}/blob/v{version}/schemas/agents-prompt-list.schema.json):

**`AgentsPromptListReport`**: What `ascribe agents prompt --list --format json` writes: every prompt, with its arguments.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `prompts` | `object[]` | yes | no | The prompts, in order. |
| `prompts[].name` | `string` | yes | no | Its name, which `ascribe agents prompt` takes. |
| `prompts[].title` | `string` | yes | no | A short name for people. |
| `prompts[].description` | `string` | yes | no | What it asks an agent to do. |
| `prompts[].arguments` | `object[]` | yes | no | Its arguments, which `--arg key=value` gives. |
| `prompts[].arguments[].name` | `string` | yes | no | Its name, the `key` of `--arg key=value`. |
| `prompts[].arguments[].description` | `string` | yes | no | What it is. |
| `prompts[].arguments[].required` | `boolean` | yes | no | Whether the prompt needs it. |
