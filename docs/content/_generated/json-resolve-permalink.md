@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/resolve-permalink.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/resolve-permalink.schema.json`]({repo}/blob/main/schemas/resolve-permalink.schema.json):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe resolve go:<name> --format json` answers: the name's entry\nas the build's `_ascribe/help.json` writes it, with the file's `version`,\n`build`, and `line`.",
  "properties": {
    "address": {
      "description": "The permalink's address; absent without `[consumer] site`.",
      "type": [
        "string",
        "null"
      ]
    },
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "build": {
      "description": "The build, without its line.",
      "type": "string"
    },
    "choices": {
      "additionalProperties": {
        "additionalProperties": {
          "type": "string"
        },
        "type": "object"
      },
      "description": "For a section with a switcher, its text with each arm chosen: keyed\nby dimension, then by value, both in the content model's order.\nAbsent when the section has no switcher on a dimension.",
      "type": [
        "object",
        "null"
      ]
    },
    "line": {
      "description": "The line, for a line of a versioned build that has more than one.",
      "type": [
        "string",
        "null"
      ]
    },
    "name": {
      "description": "The permalink's name, as asked for.",
      "type": "string"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "source": {
      "description": "The page the permalink is recorded on, as a path from the content\nroot.",
      "type": "string"
    },
    "summary": {
      "description": "The section's summary; absent when it has none.",
      "type": [
        "string",
        "null"
      ]
    },
    "text": {
      "description": "The section as the plain output writes it, in Markdown, with each\nimage as its alt text.",
      "type": "string"
    },
    "title": {
      "description": "The heading's text, or the page's title for a page's permalink, as\nplain text.",
      "type": "string"
    },
    "version": {
      "description": "The version of `help.json`'s format, which this shares. A later\nversion adds keys and changes none of these.",
      "minimum": 0,
      "type": "integer"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "version",
    "build",
    "name",
    "title",
    "text",
    "source"
  ],
  "title": "ResolvePermalinkReport",
  "type": "object"
}
```
