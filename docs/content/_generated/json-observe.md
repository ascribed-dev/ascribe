<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/observe.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/observe.schema.json`]({repo}/blob/main/schemas/observe.schema.json):

```json
{
  "$defs": {
    "Changes": {
      "description": "What differs between an observation and the one after it, or the folder.",
      "properties": {
        "added": {
          "description": "Examples it lists that the earlier one didn't, by path.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "changed": {
          "description": "Examples both list, with different hashes.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "environment": {
          "description": "Environment values that differ, in the newer one's order, then those\nonly the earlier one had.",
          "items": {
            "$ref": "#/$defs/EnvironmentChange"
          },
          "type": "array"
        },
        "removed": {
          "description": "Examples the earlier one listed that it doesn't.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "added",
        "removed",
        "changed",
        "environment"
      ],
      "type": "object"
    },
    "EnvironmentChange": {
      "description": "An environment value that differs.",
      "properties": {
        "key": {
          "description": "The entry's key: `os`, `arch`, or the config's.",
          "type": "string"
        },
        "now": {
          "description": "What the newer one has; absent when it has none.",
          "type": [
            "string",
            "null"
          ]
        },
        "was": {
          "description": "What the earlier observation had; absent when it had none.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "key",
        "was",
        "now"
      ],
      "type": "object"
    },
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
  "description": "What `ascribe observe --format json` writes: the commands it ran, how the\ntest ended, the observation it wrote, and what changed since the one it\nreplaced. With `--check`, nothing ran: the observation it compared, and\nwhether it's current.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "changes": {
      "anyOf": [
        {
          "$ref": "#/$defs/Changes"
        },
        {
          "type": "null"
        }
      ],
      "description": "What changed since the observation it replaced: absent when there was\nnone, or the test failed. With `--check`, what differs between the\nobservation and the folder."
    },
    "check": {
      "description": "Whether it was run with `--check`, which runs nothing.",
      "type": "boolean"
    },
    "config": {
      "description": "The config it ran, `ascribe.examples.toml`, in the current folder.",
      "type": "string"
    },
    "current": {
      "description": "With `--check`, whether the observation is current: it lists every\nexample in the folder, with the hash it has now, and nothing else.\nAbsent without `--check`.",
      "type": [
        "boolean",
        "null"
      ]
    },
    "environment": {
      "description": "The `[environment]` commands, in the order the config has them, each\nwith the line it kept. Empty with `--check`.",
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
      "description": "The observation it wrote, from the current folder: none when the test\nfailed. With `--check`, the one it compared.",
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
      "anyOf": [
        {
          "$ref": "#/$defs/TestRun"
        },
        {
          "type": "null"
        }
      ],
      "description": "The test command, and how it ended. Absent with `--check`."
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
    "check",
    "environment",
    "test",
    "observation",
    "current",
    "changes",
    "recorded",
    "files",
    "tested"
  ],
  "title": "ObserveReport",
  "type": "object"
}
```
