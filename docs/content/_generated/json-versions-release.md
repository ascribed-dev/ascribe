@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/versions-release.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/versions-release.schema.json`]({repo}/blob/main/schemas/versions-release.schema.json):

```json
{
  "$defs": {
    "FileJson": {
      "description": "A source file the release rewrites.",
      "properties": {
        "file": {
          "description": "The file, from the project root.",
          "type": "string"
        },
        "marks": {
          "description": "Each mark rewritten.",
          "items": {
            "$ref": "#/$defs/MarkJson"
          },
          "type": "array"
        }
      },
      "required": [
        "file",
        "marks"
      ],
      "type": "object"
    },
    "MarkJson": {
      "description": "A mark the release rewrites.",
      "properties": {
        "from": {
          "description": "The spec as written.",
          "type": "string"
        },
        "line": {
          "description": "Its line, from 1.",
          "minimum": 0,
          "type": "integer"
        },
        "to": {
          "description": "The spec as rewritten.",
          "type": "string"
        }
      },
      "required": [
        "line",
        "from",
        "to"
      ],
      "type": "object"
    },
    "OutputJson": {
      "description": "One output the release changes.",
      "properties": {
        "added": {
          "description": "The files it gains, by their path in the output.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "build": {
          "description": "The build: `site`, or an older line's, `site@3.4`. Empty for the\nfile the lines share.",
          "type": "string"
        },
        "changed": {
          "description": "The files that change.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "output": {
          "description": "The output: `site`, `plain`, or `json`, or `_ascribe` for the file\nthe lines share.",
          "type": "string"
        },
        "removed": {
          "description": "The files it loses.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "build",
        "output",
        "added",
        "removed",
        "changed"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe versions release --format json` writes: the line released,\nthe files and marks rewritten, the features moved, and what the release\nchanges in the outputs.",
  "properties": {
    "archived": {
      "description": "The line that was current, moved to `archived` with no address,\nbecause no build is versioned by the target.",
      "type": [
        "string",
        "null"
      ]
    },
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "files": {
      "description": "Each source file whose marks are rewritten, from the project root.",
      "items": {
        "$ref": "#/$defs/FileJson"
      },
      "type": "array"
    },
    "from": {
      "description": "The `ascribe.toml` the target's lines are written in, from the\ncurrent folder, when the project's `[from]` takes them from another\nproject's: the release is made in that project, and `files` are from\nits root. `null` when they're written in the project's own.",
      "type": [
        "string",
        "null"
      ]
    },
    "line": {
      "description": "The line released, as given.",
      "type": "string"
    },
    "next": {
      "description": "The line planned after the released one, from `--next`.",
      "type": [
        "string",
        "null"
      ]
    },
    "next_added": {
      "description": "Whether `next` was added to `planned`, from `--next`.",
      "type": "boolean"
    },
    "outputs": {
      "description": "What the release changes in the builds' outputs, by build and\noutput.",
      "items": {
        "$ref": "#/$defs/OutputJson"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "target": {
      "description": "The target.",
      "type": "string"
    },
    "was_planned": {
      "description": "Whether the line was in `planned`; a line that wasn't is a release\nnobody planned, such as a patch.",
      "type": "boolean"
    },
    "written": {
      "description": "Whether the files were written: false with `--dry-run`.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "written",
    "target",
    "line",
    "was_planned",
    "next",
    "next_added",
    "archived",
    "files",
    "outputs",
    "from"
  ],
  "title": "VersionsReleaseReport",
  "type": "object"
}
```
