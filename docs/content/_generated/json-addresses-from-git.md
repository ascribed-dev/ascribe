@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/addresses-from-git.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/addresses-from-git.schema.json`]({repo}/blob/main/schemas/addresses-from-git.schema.json):

```json
{
  "$defs": {
    "Edit": {
      "description": "One edit of a fix.",
      "properties": {
        "new_text": {
          "description": "The text that replaces it.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "The text it replaces."
        }
      },
      "required": [
        "range",
        "new_text"
      ],
      "type": "object"
    },
    "MoveJson": {
      "description": "A move `--from-git` records.",
      "properties": {
        "key": {
          "description": "The entry's key: where the page was.",
          "type": "string"
        },
        "value": {
          "description": "The entry's value: where it is now.",
          "type": "string"
        }
      },
      "required": [
        "key",
        "value"
      ],
      "type": "object"
    },
    "Pos": {
      "description": "A position in a file.",
      "properties": {
        "column": {
          "description": "The column, from 1, in Unicode characters (not bytes or UTF-16\nunits).",
          "minimum": 0,
          "type": "integer"
        },
        "line": {
          "description": "The line, from 1.",
          "minimum": 0,
          "type": "integer"
        },
        "offset": {
          "description": "The byte offset from the start of the file.",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "line",
        "column",
        "offset"
      ],
      "type": "object"
    },
    "Range": {
      "description": "A span of a file. `end` is just past its last character; an empty range\n(an insertion) has equal positions.",
      "properties": {
        "end": {
          "$ref": "#/$defs/Pos",
          "description": "Just past its last character."
        },
        "start": {
          "$ref": "#/$defs/Pos",
          "description": "Its first character."
        }
      },
      "required": [
        "start",
        "end"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe addresses move --from-git --format json` writes.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "base": {
      "description": "The base git's renames are seen from: the revision asked for, or the\ndefault branch, and its commit when that's something else.",
      "type": "string"
    },
    "changed": {
      "description": "Whether the file changed.",
      "type": "boolean"
    },
    "edits": {
      "description": "The edits that make the change, each replacing the text of its\nrange in the file as it was; none when nothing changed.",
      "items": {
        "$ref": "#/$defs/Edit"
      },
      "type": "array"
    },
    "file": {
      "description": "The file, from the project's folder: `ascribe.addresses`.",
      "type": "string"
    },
    "moves": {
      "description": "Each move recorded, in path order: the page's old path, from the\ncontent root, and its new one.",
      "items": {
        "$ref": "#/$defs/MoveJson"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "written": {
      "description": "Whether the change was written: the MCP server's tools write\nnothing, and give the edits to apply.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "file",
    "base",
    "moves",
    "changed",
    "written",
    "edits"
  ],
  "title": "AddressesFromGitReport",
  "type": "object"
}
```
