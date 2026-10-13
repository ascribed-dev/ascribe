<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/observe.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/observe.schema.json`]({repo}/blob/main/schemas/observe.schema.json):

```json
{
  "$defs": {
    "EnvironmentRun": {
      "description": "An `[environment]` command that ran.",
      "properties": {
        "command": {
          "description": "The line of shell.",
          "type": "string"
        },
        "kept": {
          "description": "The first line it printed, as the observation records it.",
          "type": "string"
        },
        "key": {
          "description": "The entry's key.",
          "type": "string"
        }
      },
      "required": [
        "key",
        "command",
        "kept"
      ],
      "type": "object"
    },
    "TestRun": {
      "description": "The test command, and how it ended.",
      "properties": {
        "command": {
          "description": "The line of shell.",
          "type": "string"
        },
        "exit_code": {
          "description": "Its exit code; absent when a signal stopped it.",
          "type": [
            "integer",
            "null"
          ]
        },
        "passed": {
          "description": "Whether it exited with 0.",
          "type": "boolean"
        }
      },
      "required": [
        "command",
        "passed",
        "exit_code"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe observe --format json` writes: the commands it ran, how the\ntest ended, and the observation it wrote.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "config": {
      "description": "The config it ran, `ascribe.examples.toml`, in the current folder.",
      "type": "string"
    },
    "environment": {
      "description": "The `[environment]` commands, in the order the config has them, each\nwith the line it kept.",
      "items": {
        "$ref": "#/$defs/EnvironmentRun"
      },
      "type": "array"
    },
    "files": {
      "additionalProperties": {
        "type": "string"
      },
      "description": "The hash of each example, by path from the current folder. Empty when\nthe test failed.",
      "type": "object"
    },
    "observation": {
      "description": "The observation it wrote, from the current folder: none when the test\nfailed.",
      "type": [
        "string",
        "null"
      ]
    },
    "recorded": {
      "additionalProperties": {
        "type": "string"
      },
      "description": "The environment the observation records, by name: `os`, `arch`, and\neach entry's kept line. Empty when the test failed.",
      "type": "object"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "test": {
      "$ref": "#/$defs/TestRun",
      "description": "The test command, and how it ended."
    },
    "tested": {
      "description": "What a page says it was tested with: `Node 24.9.0 on Linux`. Absent\nwhen the test failed.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "config",
    "environment",
    "test",
    "observation",
    "recorded",
    "files",
    "tested"
  ],
  "title": "ObserveReport",
  "type": "object"
}
```
