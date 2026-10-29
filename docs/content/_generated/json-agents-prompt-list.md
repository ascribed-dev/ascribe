@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/agents-prompt-list.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/agents-prompt-list.schema.json`]({repo}/blob/main/schemas/agents-prompt-list.schema.json):

```json
{
  "$defs": {
    "PromptArgument": {
      "description": "An argument of a prompt.",
      "properties": {
        "description": {
          "description": "What it is.",
          "type": "string"
        },
        "name": {
          "description": "Its name, the `key` of `--arg key=value`.",
          "type": "string"
        },
        "required": {
          "description": "Whether the prompt needs it.",
          "type": "boolean"
        }
      },
      "required": [
        "name",
        "description",
        "required"
      ],
      "type": "object"
    },
    "PromptListed": {
      "description": "A prompt in `ascribe agents prompt --list --format json`.",
      "properties": {
        "arguments": {
          "description": "Its arguments, which `--arg key=value` gives.",
          "items": {
            "$ref": "#/$defs/PromptArgument"
          },
          "type": "array"
        },
        "description": {
          "description": "What it asks an agent to do.",
          "type": "string"
        },
        "name": {
          "description": "Its name, which `ascribe agents prompt` takes.",
          "type": "string"
        },
        "title": {
          "description": "A short name for people.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "title",
        "description",
        "arguments"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe agents prompt --list --format json` writes: every prompt,\nwith its arguments.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "prompts": {
      "description": "The prompts, in order.",
      "items": {
        "$ref": "#/$defs/PromptListed"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "prompts"
  ],
  "title": "AgentsPromptListReport",
  "type": "object"
}
```
