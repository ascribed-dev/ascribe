@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/agents-index.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/agents-index.schema.json`]({repo}/blob/main/schemas/agents-index.schema.json):

```json
{
  "$defs": {
    "IndexPage": {
      "description": "A page copied into the folder an index points into.",
      "properties": {
        "path": {
          "description": "Its Markdown version's path in the folder.",
          "type": "string"
        },
        "source": {
          "description": "The page, as a path from the content root.",
          "type": "string"
        }
      },
      "required": [
        "source",
        "path"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe agents index --format json` answers.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "build": {
      "description": "The build.",
      "type": "string"
    },
    "bytes": {
      "description": "How many bytes it holds.",
      "minimum": 0,
      "type": "integer"
    },
    "changed": {
      "description": "Whether that file changed: with `--dry-run`, whether it would.",
      "type": "boolean"
    },
    "file": {
      "description": "The file the index was written into, between its markers, from the\nfolder the command ran in; `null` when it was printed.",
      "type": [
        "string",
        "null"
      ]
    },
    "folder": {
      "description": "The folder the pages were copied into, from the folder the command\nran in; `null` without `--pages`.",
      "type": [
        "string",
        "null"
      ]
    },
    "left_out": {
      "description": "The offered pages the index leaves out to stay under `limit`, as paths\nfrom the content root.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "limit": {
      "description": "The most bytes the index holds.",
      "minimum": 0,
      "type": "integer"
    },
    "listed": {
      "description": "The pages the index lists, as paths from the content root, in its\norder.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "offered": {
      "description": "How many pages the build offers: those it publishes, apart from the\n`unlisted`.",
      "minimum": 0,
      "type": "integer"
    },
    "pages": {
      "description": "The pages copied into the folder: every offered page, with\n`--pages`.",
      "items": {
        "$ref": "#/$defs/IndexPage"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "text": {
      "description": "The index's Markdown.",
      "type": "string"
    },
    "without_description": {
      "description": "The pages the index lists without their description, to stay under\n`limit`, as paths from the content root.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "written": {
      "description": "Whether the file and the pages were written: false when the index\nwas printed, or with `--dry-run`.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "build",
    "limit",
    "bytes",
    "offered",
    "listed",
    "left_out",
    "without_description",
    "file",
    "changed",
    "written",
    "folder",
    "pages",
    "text"
  ],
  "title": "AgentsIndexReport",
  "type": "object"
}
```
