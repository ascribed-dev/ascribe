<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/permalinks-sync.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/permalinks-sync.schema.json`]({repo}/blob/main/schemas/permalinks-sync.schema.json):

```json
{
  "$defs": {
    "MovedEntry": {
      "description": "A permalink whose place changed.",
      "properties": {
        "from": {
          "description": "Where the list had it.",
          "type": "string"
        },
        "name": {
          "description": "Its name.",
          "type": "string"
        },
        "to": {
          "description": "Where it's declared now.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "from",
        "to"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe permalinks sync` does to the list.",
  "properties": {
    "added": {
      "description": "The names added, by name.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "aliased": {
      "description": "The aliases recorded, new or leading somewhere new, by name.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "changed": {
      "description": "Whether the list's file changes: it's new, or its text differs.",
      "type": "boolean"
    },
    "created": {
      "description": "Whether the project had no list, so this makes one.",
      "type": "boolean"
    },
    "moved": {
      "description": "The names whose place changed, by name.",
      "items": {
        "$ref": "#/$defs/MovedEntry"
      },
      "type": "array"
    },
    "removed": {
      "description": "The names the list keeps that the project no longer has, by name.\n`ascribe check` warns about each.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "retired": {
      "description": "The retired permalinks recorded, new or with a new reason, by name.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "changed",
    "created",
    "added",
    "moved",
    "aliased",
    "retired",
    "removed"
  ],
  "title": "PermalinksSyncReport",
  "type": "object"
}
```
