<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/permalinks-list.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/permalinks-list.schema.json`]({repo}/blob/main/schemas/permalinks-list.schema.json):

```json
{
  "$defs": {
    "DeclaredEntry": {
      "description": "A permalink the content declares.",
      "properties": {
        "file": {
          "description": "The file it's declared in, from the project root, as `ascribe check`\nreports it.",
          "type": "string"
        },
        "line": {
          "description": "The line of its `@id` or its frontmatter `permalink`, from 1.",
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "listed_at": {
          "description": "The place the list has for it, when that's somewhere else.",
          "type": [
            "string",
            "null"
          ]
        },
        "name": {
          "description": "Its name.",
          "type": "string"
        },
        "place": {
          "description": "Where it's declared, as the list writes it: the file's path from the\ncontent root, and `#` and the id for a heading.",
          "type": "string"
        },
        "state": {
          "$ref": "#/$defs/ListState",
          "description": "Whether the list has it: `listed` at one of the places its name is\ndeclared, `moved` when the list has it somewhere else, or\n`not_listed`."
        }
      },
      "required": [
        "name",
        "place",
        "file",
        "line",
        "state",
        "listed_at"
      ],
      "type": "object"
    },
    "ListState": {
      "description": "Whether the list has a permalink where it's declared.",
      "oneOf": [
        {
          "const": "listed",
          "description": "At one of the places its name is declared.",
          "type": "string"
        },
        {
          "const": "moved",
          "description": "At a place that doesn't declare it any more.",
          "type": "string"
        },
        {
          "const": "not_listed",
          "description": "Not at all.",
          "type": "string"
        }
      ]
    },
    "RemovedEntry": {
      "description": "A name the list has that nothing declares.",
      "properties": {
        "name": {
          "description": "Its name.",
          "type": "string"
        },
        "place": {
          "description": "Where the list says it was.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "place"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe permalinks list` answers.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "has_list": {
      "description": "Whether the project has `ascribe.permalinks`. Without it, every\npermalink is `not_listed`, and nothing is `removed`.",
      "type": "boolean"
    },
    "permalinks": {
      "description": "Every permalink the content declares, in path order and then in\ndocument order, a page's own first. A name declared in several\nplaces is here once for each.",
      "items": {
        "$ref": "#/$defs/DeclaredEntry"
      },
      "type": "array"
    },
    "removed": {
      "description": "The names the list has that the content no longer declares, by name.",
      "items": {
        "$ref": "#/$defs/RemovedEntry"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "has_list",
    "permalinks",
    "removed"
  ],
  "title": "PermalinksListReport",
  "type": "object"
}
```
