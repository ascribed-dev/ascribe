<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/sources-status.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/sources-status.schema.json`]({repo}/blob/main/schemas/sources-status.schema.json):

```json
{
  "$defs": {
    "CopyState": {
      "description": "The state of a copy.",
      "oneOf": [
        {
          "const": "current",
          "description": "It's the file the lock pins, and a snippet uses it.",
          "type": "string"
        },
        {
          "const": "changed",
          "description": "It isn't the file the lock pins.",
          "type": "string"
        },
        {
          "const": "missing",
          "description": "The lock lists it, and it isn't there.",
          "type": "string"
        },
        {
          "const": "unlocked",
          "description": "It's there, and the lock doesn't list it.",
          "type": "string"
        },
        {
          "const": "unused",
          "description": "No snippet names it.",
          "type": "string"
        },
        {
          "const": "not_copied",
          "description": "A snippet names it, and it hasn't been copied.",
          "type": "string"
        },
        {
          "const": "not_at_pin",
          "description": "A snippet names it, and it isn't in the repository at the pin.",
          "type": "string"
        }
      ]
    },
    "CopyStatus": {
      "description": "A file of a source's copies.",
      "properties": {
        "path": {
          "description": "Its path in the source.",
          "type": "string"
        },
        "state": {
          "$ref": "#/$defs/CopyState",
          "description": "Its state."
        }
      },
      "required": [
        "path",
        "state"
      ],
      "type": "object"
    },
    "SourceStatus": {
      "description": "One source's pin and copies.",
      "properties": {
        "branch": {
          "description": "The branch an update follows; `None` for the repository's default.",
          "type": [
            "string",
            "null"
          ]
        },
        "commit": {
          "description": "Its pin, when it has one to this repository.",
          "type": [
            "string",
            "null"
          ]
        },
        "files": {
          "description": "Each copy, and each file snippets name with none, by path.",
          "items": {
            "$ref": "#/$defs/CopyStatus"
          },
          "type": "array"
        },
        "git": {
          "description": "Its repository.",
          "type": "string"
        },
        "name": {
          "description": "The source.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "git",
        "branch",
        "commit",
        "files"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe sources status --format json` writes: each source's pin and\nthe state of its copies, read from the files alone.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    },
    "sources": {
      "description": "Each source in another repository, in declaration order.",
      "items": {
        "$ref": "#/$defs/SourceStatus"
      },
      "type": "array"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "sources"
  ],
  "title": "SourcesStatusReport",
  "type": "object"
}
```
