<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/permalinks-retire.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/permalinks-retire.schema.json`]({repo}/blob/main/schemas/permalinks-retire.schema.json):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe permalinks retire` did.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "list_changed": {
      "description": "Whether `ascribe.permalinks` was changed to record them. Only a name\nthe content no longer declares is recorded as retired.",
      "type": "boolean"
    },
    "reason": {
      "description": "The reason each was given.",
      "type": "string"
    },
    "retired": {
      "description": "The names retired, each written in `ascribe.toml`'s\n`[permalinks.retired]`, in order.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "to": {
      "description": "Where each address leads now, as given; none when it shows the\nreason.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "retired",
    "reason",
    "to",
    "list_changed"
  ],
  "title": "PermalinksRetireReport",
  "type": "object"
}
```
