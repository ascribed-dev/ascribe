@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/search.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/search.schema.json`]({repo}/blob/main/schemas/search.schema.json):

```json
{
  "$defs": {
    "PageCount": {
      "description": "A page and how many places it shows: what `--summary` lists.",
      "properties": {
        "page": {
          "description": "The page, from the project root.",
          "type": "string"
        },
        "places": {
          "description": "How many places it shows.",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "page",
        "places"
      ],
      "type": "object"
    },
    "SearchPage": {
      "description": "A page that shows a place.",
      "properties": {
        "builds": {
          "description": "The builds that show it there, in the content model's order.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "heading": {
          "description": "The heading the place is under there, as a reader reads it; for a\nheading, the heading itself.",
          "type": [
            "string",
            "null"
          ]
        },
        "id": {
          "description": "That heading's id.",
          "type": [
            "string",
            "null"
          ]
        },
        "page": {
          "description": "The page's content path.",
          "type": "string"
        },
        "title": {
          "description": "Its title, as a reader reads it in the first build searched that\npublishes the page.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "page",
        "title",
        "heading",
        "id",
        "builds"
      ],
      "type": "object"
    },
    "SearchPlace": {
      "description": "A place a query matches: a line of the source to edit.",
      "properties": {
        "arm": {
          "description": "The arm of a group the block is written in, as its attributes are\nwritten (`deployment=cloud`), or else its title.",
          "type": [
            "string",
            "null"
          ]
        },
        "column": {
          "description": "The column of the match's first character, from 1, in Unicode\ncharacters.",
          "minimum": 0,
          "type": "integer"
        },
        "end_line": {
          "description": "For a snippet's code that can't be lined up with its file, the last\nline the snippet shows: the place is its lines, from `line`, and\nnot the line the match is on. `null` for every other place.",
          "minimum": 0,
          "type": [
            "integer",
            "null"
          ]
        },
        "file": {
          "description": "The file, from the project root, as `ascribe check` reports it: a\npage, a fragment, a snippet's code file, or `ascribe.toml` for a\nphrase.",
          "type": "string"
        },
        "in": {
          "description": "What it is: `title`, `description`, `heading`, `text`, `table`,\n`code`, or `phrase`, a phrase's value at its line in `ascribe.toml`.",
          "type": "string"
        },
        "line": {
          "description": "The line, from 1.",
          "minimum": 0,
          "type": "integer"
        },
        "match": {
          "description": "Where the match is in `text`: its first character and the one after\nits last, in Unicode characters; when `text` is cut before the\nmatch's last character, the end ends where it's cut, before the `…`.",
          "items": {
            "minimum": 0,
            "type": "integer"
          },
          "maxItems": 2,
          "minItems": 2,
          "type": "array"
        },
        "pages": {
          "description": "The pages that show it, in path order, at most three.",
          "items": {
            "$ref": "#/$defs/SearchPage"
          },
          "type": "array"
        },
        "pages_total": {
          "description": "How many pages show it; for a phrase, how many places in the source\nuse it.",
          "minimum": 0,
          "type": "integer"
        },
        "refs_command": {
          "description": "The `ascribe refs` command that lists where it's used, for a phrase,\nand for a fragment's place when `pages` was cut.",
          "type": [
            "string",
            "null"
          ]
        },
        "text": {
          "description": "The block's text as a reader reads it, or for a phrase, its value:\nwhole when it's at most 240 characters, and otherwise cut to about\nthat many around the match's first character, at words, with `…`\nwhere it's cut. `line` and `column` say where to read the rest.",
          "type": "string"
        },
        "through": {
          "description": "What the match reads through that the line doesn't spell, in order.",
          "items": {
            "$ref": "#/$defs/SearchThrough"
          },
          "type": "array"
        }
      },
      "required": [
        "file",
        "line",
        "column",
        "end_line",
        "in",
        "text",
        "match",
        "through",
        "arm",
        "pages",
        "pages_total",
        "refs_command"
      ],
      "type": "object"
    },
    "SearchThrough": {
      "description": "What a match reads through that its line doesn't spell.",
      "oneOf": [
        {
          "additionalProperties": false,
          "description": "A phrase's value, by its key.",
          "properties": {
            "phrase": {
              "type": "string"
            }
          },
          "required": [
            "phrase"
          ],
          "type": "object"
        },
        {
          "additionalProperties": false,
          "description": "The title of a page a link with no text takes, or the text of the\nheading it names: `page.md` or `page.md#id`.",
          "properties": {
            "title": {
              "type": "string"
            }
          },
          "required": [
            "title"
          ],
          "type": "object"
        },
        {
          "additionalProperties": false,
          "description": "A snippet's code: its address, as written (`code:app.py#main`).",
          "properties": {
            "code": {
              "type": "string"
            }
          },
          "required": [
            "code"
          ],
          "type": "object"
        }
      ]
    },
    "SourcePlace": {
      "description": "A place the source writes words no build searched publishes.",
      "properties": {
        "file": {
          "description": "The file, from the project root.",
          "type": "string"
        },
        "line": {
          "description": "The line, from 1.",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "file",
        "line"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe search <QUERY>` answers: what `--format json` writes.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "builds": {
      "description": "The builds searched, in the content model's order.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "next_command": {
      "description": "The command that lists them all, when the list was cut.",
      "type": [
        "string",
        "null"
      ]
    },
    "not_searched_builds": {
      "description": "When nothing matched, the builds that weren't searched that publish\nthe words, in the content model's order; empty otherwise.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "pages": {
      "description": "With `--summary`, in place of the places: each page that shows a\nplace and how many it shows, most first, then in path order; at\nmost `shown` of them. `null` without it.",
      "items": {
        "$ref": "#/$defs/PageCount"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "pages_matched": {
      "description": "How many pages show a place.",
      "minimum": 0,
      "type": "integer"
    },
    "places": {
      "description": "The places, at most `shown` of them: titles, then headings, then the\nrest; in each, the places written in pages, then those in fragments\nand code files, each in path order and in order within a file.\nEmpty with `--summary`.",
      "items": {
        "$ref": "#/$defs/SearchPlace"
      },
      "type": "array"
    },
    "query": {
      "description": "The query, as given.",
      "type": "string"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "shown": {
      "description": "How many places are listed, or with `--summary`, pages.",
      "minimum": 0,
      "type": "integer"
    },
    "source_places": {
      "description": "When nothing matched, the first places the source writes the words\nas given, at most three, though no build searched publishes them\nthere; empty when something matched, or when the words aren't\nwritten.",
      "items": {
        "$ref": "#/$defs/SourcePlace"
      },
      "type": "array"
    },
    "total": {
      "description": "How many places match.",
      "minimum": 0,
      "type": "integer"
    },
    "truncated": {
      "description": "Whether the list was cut: `shown` is less than `total`, or with\n`--summary`, than `pages_matched`.",
      "type": "boolean"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "query",
    "builds",
    "places",
    "pages",
    "pages_matched",
    "total",
    "shown",
    "truncated",
    "next_command",
    "source_places",
    "not_searched_builds"
  ],
  "title": "SearchReport",
  "type": "object"
}
```
