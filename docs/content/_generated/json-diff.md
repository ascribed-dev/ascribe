<!-- Generated from the Rust types by crates/tessera-cli/src/shapes.rs, with schemas/diff.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p tessera-cli shapes`. -->
[`schemas/diff.schema.json`]({repo}/blob/main/schemas/diff.schema.json):

```json
{
  "$defs": {
    "Anchor": {
      "description": "Where a block's text is written: the README's anchor grammar, the same\nstring the rendered page carries in `data-ascribe-source` and\n`data-ascribe-via`.",
      "properties": {
        "source": {
          "description": "`<path>:<first>-<last>`: the file's content path, percent-encoded by\nsegment, and the block's first and last lines, from 1.",
          "type": "string"
        },
        "via": {
          "description": "The includes the block came through, outermost first, each\n`<path>:<line>`. Empty for a block written in the page itself.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "source",
        "via"
      ],
      "type": "object"
    },
    "BaseInfo": {
      "description": "The base of a comparison, in the report.",
      "properties": {
        "commit": {
          "description": "The commit it names.",
          "type": "string"
        },
        "merge_base": {
          "description": "The merge base of that commit and `HEAD`, which the comparison reads;\n`null` with `--base-exact`, which reads `commit`.",
          "type": [
            "string",
            "null"
          ]
        },
        "requested": {
          "description": "The revision asked for, or the default branch used.",
          "type": "string"
        }
      },
      "required": [
        "requested",
        "commit",
        "merge_base"
      ],
      "type": "object"
    },
    "BuildDiff": {
      "description": "What changed in one build.",
      "properties": {
        "build": {
          "description": "The build's name.",
          "type": "string"
        },
        "pages": {
          "description": "The pages that changed, in path order.",
          "items": {
            "$ref": "#/$defs/PageDiff"
          },
          "type": "array"
        }
      },
      "required": [
        "build",
        "pages"
      ],
      "type": "object"
    },
    "Change": {
      "description": "One block's change.",
      "properties": {
        "after": {
          "anyOf": [
            {
              "$ref": "#/$defs/Anchor"
            },
            {
              "type": "null"
            }
          ],
          "description": "For a removed block, and a moved block's old place: the block of the\nnew version it came after, among its siblings. Absent when it was\nfirst."
        },
        "kind": {
          "$ref": "#/$defs/ChangeKind",
          "description": "What happened to it."
        },
        "now": {
          "anyOf": [
            {
              "$ref": "#/$defs/Anchor"
            },
            {
              "type": "null"
            }
          ],
          "description": "Where it's written now: every kind but `removed`."
        },
        "parent": {
          "anyOf": [
            {
              "$ref": "#/$defs/Anchor"
            },
            {
              "type": "null"
            }
          ],
          "description": "For a removed block, and a moved block's old place: the block of the\nnew version it was inside. Absent at the top of the page."
        },
        "text": {
          "description": "For a removed block, its text, whitespace collapsed, so it can be\nshown where it was.",
          "type": [
            "string",
            "null"
          ]
        },
        "was": {
          "anyOf": [
            {
              "$ref": "#/$defs/Anchor"
            },
            {
              "type": "null"
            }
          ],
          "description": "Where it was written: every kind but `added`."
        },
        "words": {
          "anyOf": [
            {
              "$ref": "#/$defs/Words"
            },
            {
              "type": "null"
            }
          ],
          "description": "For changed prose, the words that differ."
        }
      },
      "required": [
        "kind"
      ],
      "type": "object"
    },
    "ChangeKind": {
      "description": "The kinds of block change.",
      "oneOf": [
        {
          "const": "changed",
          "description": "The block's content changed.",
          "type": "string"
        },
        {
          "const": "added",
          "description": "The block is new.",
          "type": "string"
        },
        {
          "const": "removed",
          "description": "The block is gone.",
          "type": "string"
        },
        {
          "const": "moved",
          "description": "The same block is somewhere else.",
          "type": "string"
        }
      ]
    },
    "Counts": {
      "description": "How many changes of each kind a page has.",
      "properties": {
        "added": {
          "description": "Blocks only in the new version.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "changed": {
          "description": "Blocks whose content changed.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "moved": {
          "description": "Blocks in both, somewhere else.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "description": "Blocks only in the old version.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "changed",
        "added",
        "removed",
        "moved"
      ],
      "type": "object"
    },
    "PageDiff": {
      "description": "What changed on one page of a build.",
      "properties": {
        "because": {
          "description": "The other changed files the page's change can come from: fragments it\nincludes and pages its links take a title or a heading from, in path\norder; then the snippets whose code changed, by address\n(`code:app.py#main`); and `ascribe.toml` (last) when the content model\nis a cause.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "changes": {
          "description": "The block-level changes, in the page's order, a removed block where\nit was. Empty for an added or removed page.",
          "items": {
            "$ref": "#/$defs/Change"
          },
          "type": "array"
        },
        "counts": {
          "$ref": "#/$defs/Counts",
          "description": "How many changes of each kind."
        },
        "own_file_changed": {
          "description": "Whether the page's own file changed (or exists on one side only).",
          "type": "boolean"
        },
        "page_changed": {
          "description": "What changed about the page itself besides its blocks, in this\norder: `title`, `frontmatter`, `availability` (the page-level one),\nand `route`. Empty for an added or removed page.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "path": {
          "description": "The page's content path.",
          "type": "string"
        },
        "route": {
          "description": "Its route: the new one, or for a removed page the old one.",
          "type": "string"
        },
        "status": {
          "$ref": "#/$defs/PageStatus",
          "description": "Whether the build publishes it only now, only before, or in both\nwith a difference."
        }
      },
      "required": [
        "path",
        "route",
        "status",
        "own_file_changed",
        "because",
        "page_changed",
        "counts",
        "changes"
      ],
      "type": "object"
    },
    "PageStatus": {
      "description": "Whether a page is new, gone, or different.",
      "oneOf": [
        {
          "const": "added",
          "description": "The build publishes it now, and didn't before.",
          "type": "string"
        },
        {
          "const": "removed",
          "description": "The build published it before, and doesn't now.",
          "type": "string"
        },
        {
          "const": "changed",
          "description": "The build publishes it in both, and it differs.",
          "type": "string"
        }
      ]
    },
    "RepositoryInfo": {
      "description": "The repository, in the report.",
      "properties": {
        "project_prefix": {
          "description": "The project's folder inside it, with a trailing `/`, or empty when\nthe project is at the repository's root.",
          "type": "string"
        },
        "root": {
          "description": "The repository's top-level directory, as git prints it.",
          "type": "string"
        }
      },
      "required": [
        "root",
        "project_prefix"
      ],
      "type": "object"
    },
    "Words": {
      "description": "The words that differ inside a changed block of prose. Ranges are\n`[start, end)` in characters (Unicode scalar values) of each side's\n`text`, which is the block's text with whitespace collapsed.",
      "properties": {
        "now": {
          "description": "Ranges in `now_text`: words added or replacing others.",
          "items": {
            "items": {
              "format": "uint",
              "minimum": 0,
              "type": "integer"
            },
            "maxItems": 2,
            "minItems": 2,
            "type": "array"
          },
          "type": "array"
        },
        "now_text": {
          "description": "The block's text now.",
          "type": "string"
        },
        "was": {
          "description": "Ranges in `was_text`: words removed or replaced.",
          "items": {
            "items": {
              "format": "uint",
              "minimum": 0,
              "type": "integer"
            },
            "maxItems": 2,
            "minItems": 2,
            "type": "array"
          },
          "type": "array"
        },
        "was_text": {
          "description": "The block's text before.",
          "type": "string"
        }
      },
      "required": [
        "now",
        "was",
        "now_text",
        "was_text"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "The whole report, as `ascribe diff --format json` writes it.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "base": {
      "$ref": "#/$defs/BaseInfo",
      "description": "What was compared with."
    },
    "builds": {
      "description": "What changed, per build, in the order asked for.",
      "items": {
        "$ref": "#/$defs/BuildDiff"
      },
      "type": "array"
    },
    "repository": {
      "$ref": "#/$defs/RepositoryInfo",
      "description": "Where the project is."
    },
    "schema_version": {
      "description": "`SCHEMA_VERSION`.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    },
    "working_tree_errors": {
      "description": "How many errors `ascribe check` finds in the working tree, for the\nbuilds compared. The comparison runs regardless, but a page with an\nerror may not render as it will once it's fixed, so a reviewer should\nknow. `diff_project` sets it; `Report::new` leaves it zero.",
      "format": "uint",
      "minimum": 0,
      "type": "integer"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "base",
    "repository",
    "working_tree_errors",
    "builds"
  ],
  "title": "DiffReport",
  "type": "object"
}
```
