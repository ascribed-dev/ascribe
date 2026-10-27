@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/agents-sync.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/agents-sync.schema.json`]({repo}/blob/main/schemas/agents-sync.schema.json):

```json
{
  "$defs": {
    "Agent": {
      "description": "An agent that reads a file `sync` writes, as `sync` names it.",
      "oneOf": [
        {
          "const": "claude_code",
          "description": "Claude Code.",
          "type": "string"
        },
        {
          "const": "codex",
          "description": "Codex.",
          "type": "string"
        },
        {
          "const": "copilot",
          "description": "GitHub Copilot, in an editor or on GitHub.",
          "type": "string"
        },
        {
          "const": "copilot_cloud",
          "description": "Copilot's cloud agent only.",
          "type": "string"
        },
        {
          "const": "cursor",
          "description": "Cursor.",
          "type": "string"
        },
        {
          "const": "gemini_cli",
          "description": "Gemini CLI.",
          "type": "string"
        }
      ]
    },
    "SyncFile": {
      "description": "A file in `ascribe agents sync --format json`.",
      "properties": {
        "changed": {
          "description": "Whether it changed: with `check`, whether it's out of date.",
          "type": "boolean"
        },
        "path": {
          "description": "Its path, from the current directory, `/`-separated.",
          "type": "string"
        },
        "read_by": {
          "description": "The agents that read it.",
          "items": {
            "$ref": "#/$defs/Agent"
          },
          "type": "array"
        }
      },
      "required": [
        "path",
        "changed",
        "read_by"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe agents sync --format json` writes: each file it writes,\nand what it has to say.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "check": {
      "description": "Whether it ran with `--check`, which writes nothing.",
      "type": "boolean"
    },
    "files": {
      "description": "Each file, in the order it was first planned.",
      "items": {
        "$ref": "#/$defs/SyncFile"
      },
      "type": "array"
    },
    "mcp_settings": {
      "description": "With `--cloud` and no `--agent`, the MCP server's JSON to paste into\nthe repository's settings for Copilot's cloud agent; else `null`."
    },
    "notes": {
      "description": "The notes for the person running it, which it also writes to\nstandard error.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "unread": {
      "description": "What an agent doesn't read of what's written, and the target that\nadds it, a sentence each.",
      "items": {
        "type": "string"
      },
      "type": "array"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "check",
    "files",
    "notes",
    "unread",
    "mcp_settings"
  ],
  "title": "AgentsSyncReport",
  "type": "object"
}
```
