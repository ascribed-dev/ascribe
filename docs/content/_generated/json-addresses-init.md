@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/addresses-init.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/addresses-init.schema.json`]({repo}/blob/v{version}/schemas/addresses-init.schema.json):

```json
{
  "$id": "https://raw.githubusercontent.com/ascribed-dev/ascribe/v0.4.5/schemas/addresses-init.schema.json",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe addresses init --format json` writes.",
  "properties": {
    "ascribe_version": {
      "description": "The release of Ascribe that wrote it, as `0.4.5`.",
      "type": "string"
    },
    "created": {
      "description": "Whether the project had no file, so this made one. A file already\nthere is left as it is, but for what's moved into it.",
      "type": "boolean"
    },
    "file": {
      "description": "The file, from the project's folder: `ascribe.addresses`.",
      "type": "string"
    },
    "moved": {
      "description": "The keys moved into the file from `ascribe.toml`'s `[permalinks]`,\neach `go:<name>`, in the order written there; empty when it has\nnone.",
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
    "written": {
      "description": "Whether a file was written: one made, or moved into, without\n`--dry-run`.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "file",
    "created",
    "moved",
    "written"
  ],
  "title": "AddressesInitReport",
  "type": "object"
}
```
