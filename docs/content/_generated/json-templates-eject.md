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
          "description": "Its `guide`, from the folder `ascribe.toml` is in, in the order to\nread them: their guide, then Ascribe's notes on the type.",
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
    "Kept": {
      "description": "A type `ascribe.toml` declared already, which eject kept, adding only\nits `template` and `guide`.",
      "properties": {
        "guide": {
          "description": "The `guide` added, from the folder `ascribe.toml` is in, in the order\nto read them: their guide, then Ascribe's notes on the type.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "left": {
          "description": "The type's keys left as they were, in the order `ascribe.toml` has\nthem, such as `files` and `frontmatter`.",
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
          "description": "The `template` added, from the folder `ascribe.toml` is in.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "summary",
        "template",
        "guide",
        "left"
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
        },
        {
          "const": "type_not_table",
          "description": "Asked to keep the type, `ascribe.toml` writes it as an inline or a\ndotted table, which eject doesn't add to.",
          "type": "string"
        },
        {
          "const": "type_has_template",
          "description": "Asked to keep the type, it names a `template` or a `guide` already.",
          "type": "string"
        },
        {
          "const": "frontmatter_differs",
          "description": "Asked to keep the type, its frontmatter doesn't declare a field the\ntemplate's has, or declares it as something other than a string.",
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
    "kept": {
      "description": "The types `ascribe.toml` declared already and kept, with `--keep-type`,\nin the order they were named: each was given its `template` and\n`guide`, and nothing else in it changed. Empty when anything was\nrefused.",
      "items": {
        "$ref": "#/$defs/Kept"
      },
      "type": "array"
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
      "description": "The files written, from the folder `ascribe.toml` is in,\n`/`-separated, in the order they were written: each template, its\nguide, and Ascribe's notes on its type, then `templates.lock`, then\n`ascribe.toml`. Empty when anything was refused.",
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
    "kept",
    "refused",
    "page_declared"
  ],
  "title": "TemplatesEjectReport",
  "type": "object"
}
```
