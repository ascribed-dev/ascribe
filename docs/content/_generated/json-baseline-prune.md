@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/baseline-prune.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/baseline-prune.schema.json`]({repo}/blob/main/schemas/baseline-prune.schema.json):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe baseline prune --format json` writes.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "changed": {
      "description": "Whether the file changed: some entry was removed. With `--dry-run`,\nwhether it would.",
      "type": "boolean"
    },
    "entries": {
      "description": "How many problems the baseline holds, after this.",
      "minimum": 0,
      "type": "integer"
    },
    "file": {
      "description": "The file pruned, from the project's folder: `ascribe.baseline`.",
      "type": "string"
    },
    "files": {
      "description": "How many files they're in.",
      "minimum": 0,
      "type": "integer"
    },
    "found": {
      "description": "Whether the project has a baseline. Without one, there's nothing to\nprune, and nothing is written.",
      "type": "boolean"
    },
    "removed": {
      "description": "How many entries it removed.",
      "minimum": 0,
      "type": "integer"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "written": {
      "description": "Whether the file was written: it changed, without `--dry-run`.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "file",
    "found",
    "changed",
    "written",
    "entries",
    "files",
    "removed"
  ],
  "title": "BaselinePruneReport",
  "type": "object"
}
```
