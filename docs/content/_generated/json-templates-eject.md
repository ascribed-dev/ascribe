<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/templates-eject.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/templates-eject.schema.json`]({repo}/blob/main/schemas/templates-eject.schema.json):

```json
{
  "$defs": {
    "Added": {
      "description": "A type eject added to `ascribe.toml`.",
      "properties": {
        "files": {
          "description": "The pattern its `files` has, from the content root.",
          "type": "string"
        },
        "folder": {
          "description": "The folder of the pages it covers, from the folder `ascribe.toml` is\nin.",
          "type": "string"
        },
        "guide": {
          "description": "Its `guide`, from the folder `ascribe.toml` is in.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "name": {
          "description": "The type's name, `[types.<name>]`, which is the template's.",
          "type": "string"
        },
        "summary": {
          "description": "Their index's summary of it.",
          "type": "string"
        },
        "template": {
          "description": "Its `template`, from the folder `ascribe.toml` is in.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "summary",
        "files",
        "folder",
        "template",
        "guide"
      ],
      "type": "object"
    },
    "Refusal": {
      "description": "A template eject refused, and why.",
      "properties": {
        "message": {
          "description": "Why, in a sentence.",
          "type": "string"
        },
        "name": {
          "description": "The name given.",
          "type": "string"
        },
        "path": {
          "description": "The file in the way, from the folder `ascribe.toml` is in, when that's\nthe reason; otherwise null.",
          "type": [
            "string",
            "null"
          ]
        },
        "reason": {
          "$ref": "#/$defs/RefusalReason",
          "description": "Why it was refused."
        }
      },
      "required": [
        "name",
        "reason",
        "path",
        "message"
      ],
      "type": "object"
    },
    "RefusalReason": {
      "description": "Why a template was refused.",
      "oneOf": [
        {
          "const": "unknown",
          "description": "The set has no template of that name.",
          "type": "string"
        },
        {
          "const": "declared",
          "description": "`ascribe.toml` declares a type of that name already.",
          "type": "string"
        },
        {
          "const": "file_in_the_way",
          "description": "A file is where one of its files would be written.",
          "type": "string"
        }
      ]
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe templates eject` did: the files it wrote and the types it\nadded, or, when it refused anything, what it refused, and then it wrote\nnothing.",
  "properties": {
    "added": {
      "description": "The types added to `ascribe.toml`, in the order they were named.\nEmpty when anything was refused.",
      "items": {
        "$ref": "#/$defs/Added"
      },
      "type": "array"
    },
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "page_declared": {
      "description": "Whether `ascribe.toml` declared no types, so that every page was of\nthe implicit type `page`, and `[types.page]` was written out first,\nas it was, so the pages outside the types added keep it.",
      "type": "boolean"
    },
    "refused": {
      "description": "What was refused, in the order it was named. When it isn't empty,\nnothing was written.",
      "items": {
        "$ref": "#/$defs/Refusal"
      },
      "type": "array"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "written": {
      "description": "The files written, from the folder `ascribe.toml` is in,\n`/`-separated, in the order they were written: each template and its\nguide, then `templates.lock`, then `ascribe.toml`. Empty when anything\nwas refused.",
      "items": {
        "type": "string"
      },
      "type": "array"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "written",
    "added",
    "refused",
    "page_declared"
  ],
  "title": "TemplatesEjectReport",
  "type": "object"
}
```
