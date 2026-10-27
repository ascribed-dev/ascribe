@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/permalinks-sync.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
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
      "description": "The names moved to another permalink recorded, new or leading\nsomewhere new, by name.",
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
    "forwarded": {
      "description": "The names moved to a page recorded, new or leading somewhere new, by\nname.",
      "items": {
        "type": "string"
      },
      "type": "array"
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
      "minimum": 0,
      "type": "integer"
    },
    "written": {
      "description": "Whether the list's file was written: it changes, and nothing said to\nwrite nothing (`--check`, `--dry-run`).",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "changed",
    "written",
    "created",
    "added",
    "moved",
    "aliased",
    "forwarded",
    "retired",
    "removed"
  ],
  "title": "PermalinksSyncReport",
  "type": "object"
}
```
