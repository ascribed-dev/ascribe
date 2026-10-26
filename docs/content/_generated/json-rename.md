@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/rename.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/rename.schema.json`]({repo}/blob/main/schemas/rename.schema.json):

```json
{
  "$defs": {
    "FileJson": {
      "description": "A file a rename changes.",
      "properties": {
        "edits": {
          "description": "How many places in it change.",
          "minimum": 0,
          "type": "integer"
        },
        "file": {
          "description": "The file, from the current directory.",
          "type": "string"
        },
        "text": {
          "description": "Its whole text after the rename.",
          "type": "string"
        }
      },
      "required": [
        "file",
        "text",
        "edits"
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
    "MovedJson": {
      "description": "The file a rename moves.",
      "properties": {
        "from": {
          "description": "Where it is, from the current directory.",
          "type": "string"
        },
        "to": {
          "description": "Where it goes, from the current directory. Its folder is made when\nit isn't there.",
          "type": "string"
        }
      },
      "required": [
        "from",
        "to"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe rename --format json` writes, and the MCP server's\n`ascribe_rename` gives.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "files": {
      "description": "Each file the rename changes, in path order, with its whole text\nafter. The file that moves is under where it was, and\n`ascribe.addresses` is one of them when the rename records a move.",
      "items": {
        "$ref": "#/$defs/FileJson"
      },
      "type": "array"
    },
    "moved": {
      "anyOf": [
        {
          "$ref": "#/$defs/MovedJson"
        },
        {
          "type": "null"
        }
      ],
      "description": "The page or fragment that moves, when one does."
    },
    "recorded": {
      "description": "The entries of `ascribe.addresses`'s `[moved]` the rename records, in\nkey order, so the build forwards the old addresses; none when it\nrecords nothing, as for a phrase, a fragment, or a project with no\n`ascribe.addresses`.",
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
      "description": "Whether the files were written and the file moved: the MCP server's\n`ascribe_rename` writes nothing, and gives the changes to make.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "files",
    "recorded",
    "written"
  ],
  "title": "RenameReport",
  "type": "object"
}
```
