@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/agents-prompt.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/agents-prompt.schema.json`]({repo}/blob/main/schemas/agents-prompt.schema.json):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe agents prompt <NAME> --format json` writes: the prompt.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "name": {
      "description": "The prompt's name.",
      "type": "string"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "text": {
      "description": "The prompt, as `--format text` prints it.",
      "type": "string"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "name",
    "text"
  ],
  "title": "AgentsPromptReport",
  "type": "object"
}
```
