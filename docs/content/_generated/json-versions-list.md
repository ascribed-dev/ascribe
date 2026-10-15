<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/versions-list.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/versions-list.schema.json`]({repo}/blob/main/schemas/versions-list.schema.json):

```json
{
  "$defs": {
    "LineOutput": {
      "description": "A live line's output in one build.",
      "properties": {
        "base": {
          "description": "The path its pages are under, ending in `/`.",
          "type": "string"
        },
        "build": {
          "description": "The output's build: the build's own name for the current line,\n`<build>@<line>` for an older one.",
          "type": "string"
        },
        "pages": {
          "description": "How many pages it publishes.",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "build",
        "base",
        "pages"
      ],
      "type": "object"
    },
    "ListedLine": {
      "description": "One line of a target.",
      "properties": {
        "ended": {
          "description": "For an archived line, the day its support ended.",
          "type": [
            "string",
            "null"
          ]
        },
        "ends": {
          "description": "For a live line, the last day it's supported, from `ends`.",
          "type": [
            "string",
            "null"
          ]
        },
        "line": {
          "description": "The line, as the list writes it.",
          "type": "string"
        },
        "outputs": {
          "description": "For a live line, its output in each build versioned by the target.",
          "items": {
            "$ref": "#/$defs/LineOutput"
          },
          "type": "array"
        },
        "status": {
          "description": "`current` (the newest live line), `live`, `planned`, or `archived`.",
          "type": "string"
        },
        "url": {
          "description": "For an archived line, where it can still be read; absent when\nnowhere.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "line",
        "status",
        "outputs"
      ],
      "type": "object"
    },
    "TargetLines": {
      "description": "One target's lines.",
      "properties": {
        "builds": {
          "description": "The builds versioned by it, which give each live line an output.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "lines": {
          "description": "Its lines, newest first.",
          "items": {
            "$ref": "#/$defs/ListedLine"
          },
          "type": "array"
        },
        "target": {
          "description": "The target, a versioned dimension value.",
          "type": "string"
        }
      },
      "required": [
        "target",
        "builds",
        "lines"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "Every target's release lines.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "targets": {
      "description": "Each target with a list in `[releases]`, in the order `ascribe.toml`\ndeclares them.",
      "items": {
        "$ref": "#/$defs/TargetLines"
      },
      "type": "array"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "targets"
  ],
  "title": "VersionsListReport",
  "type": "object"
}
```
