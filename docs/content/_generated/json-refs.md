<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/refs.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/refs.schema.json`]({repo}/blob/main/schemas/refs.schema.json):

```json
{
  "$defs": {
    "Place": {
      "description": "A place a target is used.",
      "properties": {
        "column": {
          "description": "The column, from 1, in Unicode characters.",
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "file": {
          "description": "The file, from the project root, as `ascribe check` reports it.",
          "type": "string"
        },
        "line": {
          "description": "The line, from 1.",
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "use": {
          "description": "How it uses the target: `link`, `include`, `phrase`, `availability`,\n`term`, `variant`, `note`, or `widget`.",
          "type": "string"
        }
      },
      "required": [
        "file",
        "line",
        "column",
        "use"
      ],
      "type": "object"
    },
    "TargetKind": {
      "description": "What a target is.",
      "oneOf": [
        {
          "const": "page",
          "description": "A page.",
          "type": "string"
        },
        {
          "const": "fragment",
          "description": "A fragment.",
          "type": "string"
        },
        {
          "const": "heading",
          "description": "A heading, by its page and id.",
          "type": "string"
        },
        {
          "const": "phrase",
          "description": "A phrase, `phrase:<key>`.",
          "type": "string"
        },
        {
          "const": "feature",
          "description": "A feature, `feature:<key>`.",
          "type": "string"
        },
        {
          "const": "term",
          "description": "A glossary term, `term:<id>`.",
          "type": "string"
        },
        {
          "const": "dimension",
          "description": "A dimension, `dimension:<name>`.",
          "type": "string"
        },
        {
          "const": "note",
          "description": "A note type, `note:<type>`.",
          "type": "string"
        },
        {
          "const": "widget",
          "description": "A project widget, `widget:<name>`.",
          "type": "string"
        }
      ]
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe refs <TARGET>` answers.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "exists": {
      "description": "Whether the target exists. When it doesn't, there are no places.",
      "type": "boolean"
    },
    "kind": {
      "$ref": "#/$defs/TargetKind",
      "description": "What the target is."
    },
    "next_command": {
      "description": "The command that lists them all, when the list was cut.",
      "type": [
        "string",
        "null"
      ]
    },
    "places": {
      "description": "The places that use it, file by file in path order and in document\norder within a file; at most `shown` of them.",
      "items": {
        "$ref": "#/$defs/Place"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    },
    "shown": {
      "description": "How many are listed.",
      "format": "uint",
      "minimum": 0,
      "type": "integer"
    },
    "target": {
      "description": "The target, as given.",
      "type": "string"
    },
    "total": {
      "description": "How many places use it.",
      "format": "uint",
      "minimum": 0,
      "type": "integer"
    },
    "truncated": {
      "description": "Whether the list was cut: `shown` is less than `total`.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "target",
    "kind",
    "exists",
    "places",
    "total",
    "shown",
    "truncated",
    "next_command"
  ],
  "title": "RefsReport",
  "type": "object"
}
```
