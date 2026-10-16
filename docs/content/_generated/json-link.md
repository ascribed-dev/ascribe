@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/link.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/link.schema.json`]({repo}/blob/main/schemas/link.schema.json):

```json
{
  "$defs": {
    "LinkKind": {
      "description": "What a link target is.",
      "oneOf": [
        {
          "const": "page",
          "description": "A page.",
          "type": "string"
        },
        {
          "const": "heading",
          "description": "A heading on a page.",
          "type": "string"
        },
        {
          "const": "fragment",
          "description": "A fragment, which pages include and links can't name.",
          "type": "string"
        },
        {
          "const": "file",
          "description": "A file that isn't a page, which a build copies: an image or a\ndownload.",
          "type": "string"
        },
        {
          "const": "external",
          "description": "A URL with a scheme.",
          "type": "string"
        },
        {
          "const": "missing",
          "description": "Nothing: no file has that path.",
          "type": "string"
        }
      ]
    },
    "LinkLine": {
      "description": "Where a link goes in one line of a versioned build (SPEC §5.2).",
      "properties": {
        "build": {
          "description": "The line's build: the declared build for the current line, and\n`<build>@<line>` for each other.",
          "type": "string"
        },
        "leads_to": {
          "description": "For a link out of the line, the line it leads to.",
          "type": [
            "string",
            "null"
          ]
        },
        "line": {
          "description": "The line, as `[releases.<target>]` writes it.",
          "type": "string"
        },
        "page": {
          "description": "The page it lands on, as a path from the content root: the target,\nor the page that takes its place in the line. `null` when no live\nline publishes the target, which is `link-page-dropped`.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "build",
        "line",
        "page",
        "leads_to"
      ],
      "type": "object"
    },
    "LinkSuggestion": {
      "description": "A target a link could name instead.",
      "properties": {
        "href": {
          "description": "The destination to write.",
          "type": "string"
        },
        "id": {
          "description": "The heading id, if it's a heading.",
          "type": [
            "string",
            "null"
          ]
        },
        "path": {
          "description": "The page, as a path from the content root.",
          "type": "string"
        },
        "title": {
          "description": "The page's title, or the heading's text.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "href",
        "path",
        "id",
        "title"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe link <TARGET> --from <PAGE>` answers.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "closest": {
      "description": "The closest targets that exist, best first, when it doesn't.",
      "items": {
        "$ref": "#/$defs/LinkSuggestion"
      },
      "type": "array"
    },
    "exists": {
      "description": "Whether a link from that page to the target works. An external URL\ncounts as existing; it isn't checked.",
      "type": "boolean"
    },
    "from": {
      "description": "The page the link is written on, as the command was given it: a path\nfrom the folder it was run in, as `ascribe check` names a file.",
      "type": "string"
    },
    "href": {
      "description": "The destination to write on that page, when the target exists.",
      "type": [
        "string",
        "null"
      ]
    },
    "id": {
      "description": "The heading id after `#`, if any.",
      "type": [
        "string",
        "null"
      ]
    },
    "kind": {
      "$ref": "#/$defs/LinkKind",
      "description": "What the target is."
    },
    "lines": {
      "description": "Where a link to a page goes in each line of each versioned build,\nnewest first; absent when the target isn't a page or no build is\nversioned.",
      "items": {
        "$ref": "#/$defs/LinkLine"
      },
      "type": "array"
    },
    "path": {
      "description": "The file it names, as a path from the content root; `null` for an\nexternal URL, or a file that doesn't exist.",
      "type": [
        "string",
        "null"
      ]
    },
    "problem": {
      "description": "Why it doesn't work, when it doesn't.",
      "type": [
        "string",
        "null"
      ]
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "target": {
      "description": "The target, as given.",
      "type": "string"
    },
    "title": {
      "description": "The page's title, or the heading's text: what a link with no text\nshows.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "from",
    "target",
    "exists",
    "kind",
    "path",
    "id",
    "title",
    "href",
    "problem",
    "closest"
  ],
  "title": "LinkReport",
  "type": "object"
}
```
