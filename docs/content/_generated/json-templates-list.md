@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/templates-list.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/templates-list.schema.json`]({repo}/blob/main/schemas/templates-list.schema.json):

```json
{
  "$defs": {
    "ListTemplate": {
      "description": "A template in `ascribe templates list --format json`.",
      "properties": {
        "name": {
          "description": "Its name, which `ascribe templates eject` takes, and its type's.",
          "type": "string"
        },
        "summary": {
          "description": "What it's for.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "summary"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe templates list --format json` writes: the templates\nAscribe carries, and where they're from.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "commit": {
      "description": "The tag's commit.",
      "type": "string"
    },
    "license": {
      "description": "Their license.",
      "type": "string"
    },
    "repository": {
      "description": "The repository of The Good Docs Project's templates.",
      "type": "string"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "tag": {
      "description": "The tag of theirs the templates were converted from.",
      "type": "string"
    },
    "templates": {
      "description": "The templates, in the set's order.",
      "items": {
        "$ref": "#/$defs/ListTemplate"
      },
      "type": "array"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "repository",
    "tag",
    "commit",
    "license",
    "templates"
  ],
  "title": "TemplatesListReport",
  "type": "object"
}
```
