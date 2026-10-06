<!-- Generated from the Rust types by crates/tessera-cli/src/shapes.rs, with schemas/check.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p tessera-cli shapes`. -->
[`schemas/check.schema.json`]({repo}/blob/main/schemas/check.schema.json):

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
    "Entry": {
      "description": "A diagnostic.",
      "properties": {
        "builds": {
          "description": "The builds a page-level diagnostic appears in, in `ascribe.toml`'s\norder. Empty for a file-level diagnostic, and for one in content no\nbuild publishes. With `--build`, only that build.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "code": {
          "description": "The code, such as `ASC036`.",
          "type": "string"
        },
        "file": {
          "description": "The file, relative to the project root (the directory of\n`ascribe.toml`), with `/` separators. `ascribe.toml` for a\ncontent-model problem.",
          "type": "string"
        },
        "fixes": {
          "description": "Edits that would fix it.",
          "items": {
            "$ref": "#/$defs/Fix"
          },
          "type": "array"
        },
        "message": {
          "description": "What's wrong, and what to do about it.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "Where in the file."
        },
        "related": {
          "description": "Other places that explain it.",
          "items": {
            "$ref": "#/$defs/Related"
          },
          "type": "array"
        },
        "severity": {
          "description": "`error` or `warning`.",
          "type": "string"
        },
        "slug": {
          "description": "The diagnostic's name, such as `link-target-missing`.",
          "type": "string"
        },
        "unpublished": {
          "description": "Whether it's in content that no build publishes.",
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "slug",
        "severity",
        "message",
        "file",
        "range",
        "related",
        "fixes",
        "builds",
        "unpublished"
      ],
      "type": "object"
    },
    "Fix": {
      "description": "Edits that would fix a diagnostic.",
      "properties": {
        "edits": {
          "description": "The edits, each replacing the text of its range.",
          "items": {
            "$ref": "#/$defs/Edit"
          },
          "type": "array"
        },
        "file": {
          "description": "The file the edits are in, as a diagnostic's `file` is.",
          "type": "string"
        },
        "title": {
          "description": "What the fix does.",
          "type": "string"
        }
      },
      "required": [
        "title",
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
    "Related": {
      "description": "Another place that explains a diagnostic.",
      "properties": {
        "file": {
          "description": "The file, as a diagnostic's `file` is.",
          "type": "string"
        },
        "message": {
          "description": "What it has to do with the diagnostic.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "Where in the file."
        }
      },
      "required": [
        "file",
        "range",
        "message"
      ],
      "type": "object"
    },
    "Summary": {
      "description": "How many diagnostics of each severity.",
      "properties": {
        "errors": {
          "description": "How many errors.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "warnings": {
          "description": "How many warnings.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "errors",
        "warnings"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe check --format json` and `ascribe build --format json`\nwrite: one document, whatever the outcome. Fields can be added without a\nnew `schema_version`, so a reader ignores fields it doesn't know.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "diagnostics": {
      "description": "Every diagnostic, in file order, and in source order within a file.",
      "items": {
        "$ref": "#/$defs/Entry"
      },
      "type": "array"
    },
    "error": {
      "description": "Why the project couldn't be checked (exit code 2), or `null`. When it\nisn't `null`, `diagnostics` holds what was found first: the content\nmodel's problems.",
      "type": [
        "string",
        "null"
      ]
    },
    "files_checked": {
      "description": "How many source files were checked.",
      "format": "uint",
      "minimum": 0,
      "type": "integer"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    },
    "summary": {
      "$ref": "#/$defs/Summary",
      "description": "How many errors and warnings."
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "error",
    "files_checked",
    "diagnostics",
    "summary"
  ],
  "title": "CheckReport",
  "type": "object"
}
```
