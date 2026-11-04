@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/agents-prompt.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/agents-prompt.schema.json`]({repo}/blob/v{version}/schemas/agents-prompt.schema.json):

**`AgentsPromptReport`**: What `ascribe agents prompt <NAME> --format json` writes: the prompt.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `name` | `string` | yes | no | The prompt's name. |
| `text` | `string` | yes | no | The prompt, as `--format text` prints it. |
