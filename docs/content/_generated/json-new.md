@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/new.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/new.schema.json`]({repo}/blob/main/schemas/new.schema.json):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe new` wrote, or with `--dry-run` would write.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "guide": {
      "description": "The type's guide, from the folder `ascribe.toml` is in, in the order\nto read it.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "path": {
      "description": "The page, from the folder `ascribe.toml` is in, `/`-separated.",
      "type": "string"
    },
    "placeholders": {
      "description": "How many places the page has that its template left to fill in, as\n`template-placeholder-left` finds them: its placeholders, and the\nfrontmatter values still the template's.",
      "minimum": 0,
      "type": "integer"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "template": {
      "description": "The type's template, from the folder `ascribe.toml` is in, or `null`\nwhen it has none and the page is its frontmatter alone.",
      "type": [
        "string",
        "null"
      ]
    },
    "text": {
      "description": "The page's text.",
      "type": "string"
    },
    "title": {
      "description": "The title given, or `null` when the page keeps the template's.",
      "type": [
        "string",
        "null"
      ]
    },
    "type": {
      "description": "Its type.",
      "type": "string"
    },
    "written": {
      "description": "Whether the page was written: `false` with `--dry-run`.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "path",
    "type",
    "title",
    "template",
    "guide",
    "placeholders",
    "written",
    "text"
  ],
  "title": "NewReport",
  "type": "object"
}
```
