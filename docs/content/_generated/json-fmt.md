<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/fmt.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/fmt.schema.json`]({repo}/blob/main/schemas/fmt.schema.json):

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
    "FmtFile": {
      "description": "A file and the edits that format it.",
      "properties": {
        "edits": {
          "description": "The edits, each replacing the text of its range in the file as it\nwas before formatting. They don't overlap, and are in file order.",
          "items": {
            "$ref": "#/$defs/Edit"
          },
          "type": "array"
        },
        "file": {
          "description": "The file, relative to the project root (the directory of\n`ascribe.toml`), with `/` separators; a file outside it as it was\nfound.",
          "type": "string"
        }
      },
      "required": [
        "file",
        "edits"
      ],
      "type": "object"
    },
    "Pos": {
      "description": "A position in a file.",
      "properties": {
        "column": {
          "description": "The column, from 1, in Unicode characters (not bytes or UTF-16\nunits).",
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "line": {
          "description": "The line, from 1.",
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "offset": {
          "description": "The byte offset from the start of the file.",
          "format": "uint",
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
    },
    "RefusedFile": {
      "description": "A file left alone.",
      "properties": {
        "file": {
          "description": "The file, as a formatted file's `file` is.",
          "type": "string"
        },
        "reason": {
          "description": "Why, as `ascribe check` words it.",
          "type": "string"
        }
      },
      "required": [
        "file",
        "reason"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe fmt --format json` writes: one document. Fields can be\nadded without a new `schema_version`, so a reader ignores fields it\ndoesn't know.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "files": {
      "description": "Each file that changed, or with `--check` would change, in path\norder, with the edits that format it.",
      "items": {
        "$ref": "#/$defs/FmtFile"
      },
      "type": "array"
    },
    "refused": {
      "description": "The files left alone because a symbolic link on the way to them\nleads to a file that isn't a source file of the content root.",
      "items": {
        "$ref": "#/$defs/RefusedFile"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    },
    "written": {
      "description": "Whether the files were rewritten: `false` with `--check`, which\nwrites nothing.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "written",
    "files",
    "refused"
  ],
  "title": "FmtReport",
  "type": "object"
}
```
