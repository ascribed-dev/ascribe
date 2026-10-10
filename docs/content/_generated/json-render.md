<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/render.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/render.schema.json`]({repo}/blob/main/schemas/render.schema.json):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe render <PAGE> --format json` answers.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "build": {
      "description": "The build.",
      "type": "string"
    },
    "not_published": {
      "description": "Why the build doesn't publish the page, when it doesn't; `text` is\nempty then.",
      "type": [
        "string",
        "null"
      ]
    },
    "page": {
      "description": "The page, as a path from the content root.",
      "type": "string"
    },
    "route": {
      "description": "The page's route in the build's site.",
      "type": [
        "string",
        "null"
      ]
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    },
    "text": {
      "description": "The page as plain Markdown, as the build's `plain` output writes it:\nwith its frontmatter first when asked for.",
      "type": "string"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "page",
    "build",
    "not_published",
    "route",
    "text"
  ],
  "title": "RenderReport",
  "type": "object"
}
```
