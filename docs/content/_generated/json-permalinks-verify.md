@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/permalinks-verify.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/permalinks-verify.schema.json`]({repo}/blob/main/schemas/permalinks-verify.schema.json):

```json
{
  "$defs": {
    "RetiredName": {
      "description": "A name the list has as retired.",
      "properties": {
        "name": {
          "description": "The name.",
          "type": "string"
        },
        "reason": {
          "description": "Why it was retired, as the docs' authors wrote it.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "reason"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe permalinks verify` answers.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "found": {
      "description": "The names the list has, as permalinks, or moved to another permalink\nor a page, in the order given, each once.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "list": {
      "description": "The list checked against, as given.",
      "type": "string"
    },
    "missing": {
      "description": "The names it doesn't have at all, in the order given, each once.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "retired": {
      "description": "The names the list has as retired, in the order given, each once.",
      "items": {
        "$ref": "#/$defs/RetiredName"
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
    "list",
    "found",
    "retired",
    "missing"
  ],
  "title": "PermalinksVerifyReport",
  "type": "object"
}
```
