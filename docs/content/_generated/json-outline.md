<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/outline.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/outline.schema.json`]({repo}/blob/main/schemas/outline.schema.json):

```json
{
  "$defs": {
    "OutlineHeading": {
      "description": "A heading a link can name.",
      "properties": {
        "explicit_id": {
          "description": "Whether the id is the heading's `@id`, which stays when its text\nchanges.",
          "type": "boolean"
        },
        "file": {
          "description": "The file it's written in, from the project root.",
          "type": "string"
        },
        "fragment": {
          "description": "The fragment it comes from, as a path from the content root; `null`\nfor the page's own.",
          "type": [
            "string",
            "null"
          ]
        },
        "id": {
          "description": "Its id: what a link writes after `#`.",
          "type": "string"
        },
        "level": {
          "description": "1 to 6.",
          "format": "uint8",
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "line": {
          "description": "Its line in that file, from 1.",
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "text": {
          "description": "Its text, phrases replaced by their values.",
          "type": "string"
        }
      },
      "required": [
        "level",
        "text",
        "id",
        "explicit_id",
        "file",
        "line",
        "fragment"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe outline <PAGE>` answers.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "build": {
      "description": "The build the headings are limited to, with `--build`.",
      "type": [
        "string",
        "null"
      ]
    },
    "file": {
      "description": "The page's file from the project root, as `ascribe check` reports it.",
      "type": "string"
    },
    "fragment": {
      "description": "Whether it's a fragment, which pages include rather than link to.",
      "type": "boolean"
    },
    "headings": {
      "description": "The headings a link to the page can name, in document order, each id\nonce.",
      "items": {
        "$ref": "#/$defs/OutlineHeading"
      },
      "type": "array"
    },
    "not_published": {
      "description": "Why that build doesn't publish the page, when it doesn't; there are\nno headings then.",
      "type": [
        "string",
        "null"
      ]
    },
    "page": {
      "description": "The page's path from the content root, as links write it.",
      "type": "string"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    },
    "title": {
      "description": "Its title (frontmatter `title`).",
      "type": [
        "string",
        "null"
      ]
    },
    "type": {
      "description": "Its content type; `null` for a fragment, or when no one type applies.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "page",
    "file",
    "fragment",
    "title",
    "type",
    "build",
    "not_published",
    "headings"
  ],
  "title": "OutlineReport",
  "type": "object"
}
```
