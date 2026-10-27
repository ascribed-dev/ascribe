@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/sources-fetch.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/sources-fetch.schema.json`]({repo}/blob/main/schemas/sources-fetch.schema.json):

```json
{
  "$defs": {
    "Failure": {
      "description": "A file a snippet names that couldn't be copied.",
      "properties": {
        "path": {
          "description": "Its path in the source.",
          "type": "string"
        },
        "reason": {
          "description": "Why.",
          "type": "string"
        }
      },
      "required": [
        "path",
        "reason"
      ],
      "type": "object"
    },
    "SourceFetch": {
      "description": "What `fetch` did for one source.",
      "properties": {
        "commit": {
          "description": "Its pin, when it has one.",
          "type": [
            "string",
            "null"
          ]
        },
        "copied": {
          "description": "The copies written.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "failed": {
          "description": "The files snippets name that couldn't be copied.",
          "items": {
            "$ref": "#/$defs/Failure"
          },
          "type": "array"
        },
        "first_copy": {
          "description": "Whether these are the first files copied from the source, which\npublishes them to everyone who can read the project.",
          "type": "boolean"
        },
        "name": {
          "description": "The source.",
          "type": "string"
        },
        "pinned": {
          "description": "Whether this run pinned it, to the head of its branch.",
          "type": "boolean"
        },
        "removed": {
          "description": "The copies removed.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "name",
        "commit",
        "pinned",
        "copied",
        "removed",
        "failed",
        "first_copy"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe sources fetch --format json` writes: what was copied and\nremoved for each source, and the files that couldn't be copied.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "lock_written": {
      "description": "Whether `ascribe.lock` was written.",
      "type": "boolean"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "sources": {
      "description": "Each source looked at, in declaration order.",
      "items": {
        "$ref": "#/$defs/SourceFetch"
      },
      "type": "array"
    },
    "unpinned": {
      "description": "The pins removed, of sources ascribe.toml no longer declares in\nanother repository.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "written": {
      "description": "Whether the copies and the lock were written: false with\n`--dry-run`, which says what it would copy, remove, and write.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "written",
    "sources",
    "unpinned",
    "lock_written"
  ],
  "title": "SourcesFetchReport",
  "type": "object"
}
```
