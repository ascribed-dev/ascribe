@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/templates-eject.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/templates-eject.schema.json`]({repo}/blob/main/schemas/templates-eject.schema.json):

```json
{
  "$defs": {
    "Added": {
      "description": "A type eject added to `ascribe.toml`.",
      "properties": {
        "copied": {
          "description": "The fields of its frontmatter declared as another of the project's\ntypes declares them, rather than as plain strings, in order.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "guide": {
          "description": "Its guide, from the folder `ascribe.toml` is in, in the order to\nread them: their guide, then Ascribe's notes on the type.",
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
          "description": "Its template, from the folder `ascribe.toml` is in.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "summary",
        "template",
        "guide",
        "copied"
      ],
      "type": "object"
    },
    "Kept": {
      "description": "A type `ascribe.toml` declared already, which eject kept, giving it only\nits template and guide.",
      "properties": {
        "guide": {
          "description": "The guide it was given, from the folder `ascribe.toml` is in, in the\norder to read them: their guide, then Ascribe's notes on the type.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "left": {
          "description": "The type's keys left as they were, in the order `ascribe.toml` has\nthem, such as `purpose` and `frontmatter`.",
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
          "description": "The template it was given, from the folder `ascribe.toml` is in.",
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
          "description": "Asked to keep the type, it names a `template` or a `guide` already,\nor `templates/templates.lock` gives it them.",
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
    "dry_run": {
      "description": "Whether it ran with `--dry-run`, and wrote nothing.",
      "type": "boolean"
    },
    "kept": {
      "description": "The types `ascribe.toml` declared already and kept, with `--keep-type`,\nin the order they were named: each takes its template and guide from\n`templates/templates.lock`, or, written in another folder, was given\nits `template` and `guide`, and nothing else in it changed. Empty when\nanything was refused.",
      "items": {
        "$ref": "#/$defs/Kept"
      },
      "type": "array"
    },
    "page_declared": {
      "description": "Whether `ascribe.toml` declared no types, so that every page was of\nthe implicit type `page`, and `[types.page]` was written out first,\nas it was, so the pages that aren't of a type added can name it.",
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
    "untyped": {
      "description": "The project's pages that name no type, from the folder `ascribe.toml`\nis in, in order. With the types added the project declares types, so\neach needs `type:` in its frontmatter, naming its type.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "written": {
      "description": "The files written, from the folder `ascribe.toml` is in,\n`/`-separated, in the order they were written: each template, its\nguide, and Ascribe's notes on its type, then `templates.lock`, then\n`ascribe.toml`, when it changed. Empty when anything was refused.\nWith `dry_run`, the files it would write.",
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
    "dry_run",
    "added",
    "kept",
    "refused",
    "page_declared",
    "untyped"
  ],
  "title": "TemplatesEjectReport",
  "type": "object"
}
```
