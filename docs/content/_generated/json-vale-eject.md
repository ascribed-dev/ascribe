@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/vale-eject.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/vale-eject.schema.json`]({repo}/blob/main/schemas/vale-eject.schema.json):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe vale eject --format json` writes: the preset written out,\nand the files written.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "preset": {
      "description": "The preset written out.",
      "type": "string"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "written": {
      "description": "The files written, from the project's folder, `/`-separated, in the\norder they were written: `ascribe.toml` last.",
      "items": {
        "type": "string"
      },
      "type": "array"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "preset",
    "written"
  ],
  "title": "ValeEjectReport",
  "type": "object"
}
```
