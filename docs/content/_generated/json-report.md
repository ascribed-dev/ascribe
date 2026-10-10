<!-- Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/report.schema.json. Change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. -->
[`schemas/report.schema.json`]({repo}/blob/main/schemas/report.schema.json):

```json
{
  "$defs": {
    "AcknowledgedEntry": {
      "description": "A problem acknowledged as intended.",
      "properties": {
        "at": {
          "$ref": "#/$defs/Place",
          "description": "Where the acknowledgement is written."
        },
        "builds": {
          "description": "The builds it appears in, as a diagnostic's `builds` are.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "code": {
          "description": "The code of the check that found it, such as `ASC036`.",
          "type": "string"
        },
        "file": {
          "description": "The file, as a diagnostic's `file` is.",
          "type": "string"
        },
        "message": {
          "description": "What the check found.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "Where in the file."
        },
        "reason": {
          "description": "Why it's intended: the acknowledgement's reason.",
          "type": "string"
        },
        "slug": {
          "description": "The check's name.",
          "type": "string"
        }
      },
      "required": [
        "code",
        "slug",
        "message",
        "file",
        "range",
        "builds",
        "reason",
        "at"
      ],
      "type": "object"
    },
    "AgentsJson": {
      "description": "`agents`: the delivery spec's checks on a built site.",
      "properties": {
        "diagnostics": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "A diagnostic for each check that failed or warned that the hosting\nor Ascribe owns. One a page owns is `ascribe check`'s, and isn't\nrepeated."
        },
        "not_run": {
          "anyOf": [
            {
              "$ref": "#/$defs/NotRunJson"
            },
            {
              "type": "null"
            }
          ],
          "description": "Why it couldn't run, or `null` when it ran. When it isn't `null`,\nthe rest is empty."
        },
        "results": {
          "description": "Each check's result, in the checker's order.",
          "items": {
            "$ref": "#/$defs/CheckResultJson"
          },
          "type": "array"
        },
        "site": {
          "description": "The site checked, as `--site` gives it; empty when none was given.",
          "type": "string"
        }
      },
      "required": [
        "not_run",
        "site",
        "results",
        "diagnostics"
      ],
      "type": "object"
    },
    "BuildJson": {
      "description": "What one build leaves out that another build keeps.",
      "properties": {
        "build": {
          "description": "The build.",
          "type": "string"
        },
        "content": {
          "$ref": "#/$defs/Capped_LeftContentJson",
          "description": "The content it takes out of the pages it publishes, in file and\nsource order."
        },
        "pages": {
          "$ref": "#/$defs/Capped_LeftPageJson",
          "description": "The pages it doesn't publish, in path order."
        }
      },
      "required": [
        "build",
        "pages",
        "content"
      ],
      "type": "object"
    },
    "Capped_AcknowledgedEntry": {
      "description": "A list of at most `--limit` items.",
      "properties": {
        "items": {
          "description": "The items, at most `--limit` of them.",
          "items": {
            "$ref": "#/$defs/AcknowledgedEntry"
          },
          "type": "array"
        },
        "shown": {
          "description": "How many are listed.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "truncated": {
          "description": "Whether some are left out: `shown` is less than `total`. The\nreport's `next_command` lists them all.",
          "type": "boolean"
        }
      },
      "required": [
        "items",
        "shown",
        "total",
        "truncated"
      ],
      "type": "object"
    },
    "Capped_CheckCount": {
      "description": "A list of at most `--limit` items.",
      "properties": {
        "items": {
          "description": "The items, at most `--limit` of them.",
          "items": {
            "$ref": "#/$defs/CheckCount"
          },
          "type": "array"
        },
        "shown": {
          "description": "How many are listed.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "truncated": {
          "description": "Whether some are left out: `shown` is less than `total`. The\nreport's `next_command` lists them all.",
          "type": "boolean"
        }
      },
      "required": [
        "items",
        "shown",
        "total",
        "truncated"
      ],
      "type": "object"
    },
    "Capped_Entry": {
      "description": "A list of at most `--limit` items.",
      "properties": {
        "items": {
          "description": "The items, at most `--limit` of them.",
          "items": {
            "$ref": "#/$defs/Entry"
          },
          "type": "array"
        },
        "shown": {
          "description": "How many are listed.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "truncated": {
          "description": "Whether some are left out: `shown` is less than `total`. The\nreport's `next_command` lists them all.",
          "type": "boolean"
        }
      },
      "required": [
        "items",
        "shown",
        "total",
        "truncated"
      ],
      "type": "object"
    },
    "Capped_FileCount": {
      "description": "A list of at most `--limit` items.",
      "properties": {
        "items": {
          "description": "The items, at most `--limit` of them.",
          "items": {
            "$ref": "#/$defs/FileCount"
          },
          "type": "array"
        },
        "shown": {
          "description": "How many are listed.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "truncated": {
          "description": "Whether some are left out: `shown` is less than `total`. The\nreport's `next_command` lists them all.",
          "type": "boolean"
        }
      },
      "required": [
        "items",
        "shown",
        "total",
        "truncated"
      ],
      "type": "object"
    },
    "Capped_LeftContentJson": {
      "description": "A list of at most `--limit` items.",
      "properties": {
        "items": {
          "description": "The items, at most `--limit` of them.",
          "items": {
            "$ref": "#/$defs/LeftContentJson"
          },
          "type": "array"
        },
        "shown": {
          "description": "How many are listed.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "truncated": {
          "description": "Whether some are left out: `shown` is less than `total`. The\nreport's `next_command` lists them all.",
          "type": "boolean"
        }
      },
      "required": [
        "items",
        "shown",
        "total",
        "truncated"
      ],
      "type": "object"
    },
    "Capped_LeftPageJson": {
      "description": "A list of at most `--limit` items.",
      "properties": {
        "items": {
          "description": "The items, at most `--limit` of them.",
          "items": {
            "$ref": "#/$defs/LeftPageJson"
          },
          "type": "array"
        },
        "shown": {
          "description": "How many are listed.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "truncated": {
          "description": "Whether some are left out: `shown` is less than `total`. The\nreport's `next_command` lists them all.",
          "type": "boolean"
        }
      },
      "required": [
        "items",
        "shown",
        "total",
        "truncated"
      ],
      "type": "object"
    },
    "CheckCount": {
      "description": "How many diagnostics one check reports.",
      "properties": {
        "code": {
          "description": "The code, such as `ASC036`.",
          "type": "string"
        },
        "count": {
          "description": "How many.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "next": {
          "description": "Its kind of next step, as a diagnostic's `next` is.",
          "type": "string"
        },
        "severity": {
          "description": "`error`, `warning`, or `advice`.",
          "type": "string"
        },
        "slug": {
          "description": "The check's name, such as `link-target-missing`.",
          "type": "string"
        }
      },
      "required": [
        "code",
        "slug",
        "severity",
        "next",
        "count"
      ],
      "type": "object"
    },
    "CheckResultJson": {
      "description": "One check of the delivery spec.",
      "properties": {
        "category": {
          "description": "Its category in the spec.",
          "type": "string"
        },
        "id": {
          "description": "The check's id, such as `llms-txt-exists`.",
          "type": "string"
        },
        "message": {
          "description": "What the checker said.",
          "type": "string"
        },
        "owner": {
          "description": "Who changes what it checks: `ascribe` (what Ascribe writes),\n`hosting` (where the site is hosted), or `pages` (the pages, which\n`ascribe check` reports on).",
          "type": "string"
        },
        "status": {
          "description": "`pass`, `warn`, `fail`, `skip`, or `error`.",
          "type": "string"
        }
      },
      "required": [
        "id",
        "category",
        "status",
        "message",
        "owner"
      ],
      "type": "object"
    },
    "Edit": {
      "description": "One edit of a fix.",
      "properties": {
        "new_text": {
          "description": "The text that replaces it.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "The text it replaces."
        }
      },
      "required": [
        "range",
        "new_text"
      ],
      "type": "object"
    },
    "Entry": {
      "description": "A diagnostic.",
      "properties": {
        "builds": {
          "description": "The builds a page-level diagnostic appears in, in `ascribe.toml`'s\norder. Empty for a file-level diagnostic, and for one in content no\nbuild publishes. With `--build`, only that build.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "code": {
          "description": "The code, such as `ASC036`.",
          "type": "string"
        },
        "docs": {
          "description": "The address of its entry in the diagnostics reference.",
          "type": "string"
        },
        "file": {
          "description": "The file, relative to the project root (the directory of\n`ascribe.toml`), with `/` separators. `ascribe.toml` for a\ncontent-model problem.",
          "type": "string"
        },
        "fixes": {
          "description": "Edits that would fix it.",
          "items": {
            "$ref": "#/$defs/Fix"
          },
          "type": "array"
        },
        "help": {
          "description": "How to fix it, in general: the diagnostics reference's advice for its\ncode.",
          "type": "string"
        },
        "message": {
          "description": "What's wrong, and what to do about it.",
          "type": "string"
        },
        "next": {
          "description": "The kind of next step: `fix` when Ascribe can make the edit,\n`choose` when the author picks among things Ascribe can list,\n`write` when it needs writing or judgment, `outside` when nothing in\nthe source can fix it, and `review` when it may be fine as it is.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "Where in the file."
        },
        "related": {
          "description": "Other places that explain it.",
          "items": {
            "$ref": "#/$defs/Related"
          },
          "type": "array"
        },
        "repeats": {
          "description": "For a problem in included content reported because one of its related\nplaces is in a path named: at how many other includes it's reported\ntoo, collapsed into this one. `0` otherwise.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "rule": {
          "description": "The rule of the program that found it, such as `Ascribe.Repeated`\nfrom Vale, for a `prose` diagnostic. Absent for Ascribe's own.",
          "type": [
            "string",
            "null"
          ]
        },
        "severity": {
          "description": "`error`, `warning`, or `advice`. Advice never fails the command.\nMore severities may be added; a reader treats one it doesn't know as\nit treats advice.",
          "type": "string"
        },
        "slug": {
          "description": "The diagnostic's name, such as `link-target-missing`.",
          "type": "string"
        },
        "unpublished": {
          "description": "Whether it's in content that no build publishes.",
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "slug",
        "severity",
        "next",
        "message",
        "file",
        "range",
        "related",
        "fixes",
        "builds",
        "unpublished",
        "help",
        "docs",
        "repeats"
      ],
      "type": "object"
    },
    "FileCount": {
      "description": "How many diagnostics one file has.",
      "properties": {
        "advice": {
          "description": "How many advice.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "errors": {
          "description": "How many errors.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "file": {
          "description": "The file, relative to the project root, as a diagnostic's `file` is.",
          "type": "string"
        },
        "warnings": {
          "description": "How many warnings.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "file",
        "errors",
        "warnings",
        "advice"
      ],
      "type": "object"
    },
    "Findings": {
      "description": "How many findings of each severity, and of each kind of next step.",
      "properties": {
        "advice": {
          "description": "How many advice.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "by_next": {
          "description": "How many of each kind of next step, in the order `fix`, `choose`,\n`write`, `review`, `outside`; a kind with none is left out.",
          "items": {
            "$ref": "#/$defs/NextCount"
          },
          "type": "array"
        },
        "errors": {
          "description": "How many errors.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "warnings": {
          "description": "How many warnings.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "errors",
        "warnings",
        "advice",
        "by_next"
      ],
      "type": "object"
    },
    "Fix": {
      "description": "Edits that would fix a diagnostic.",
      "properties": {
        "applicability": {
          "description": "`safe` when applying the edits as they are can't change what the page\nsays and leaves nothing to decide; `unsafe` otherwise.",
          "type": "string"
        },
        "edits": {
          "description": "The edits, each replacing the text of its range.",
          "items": {
            "$ref": "#/$defs/Edit"
          },
          "type": "array"
        },
        "file": {
          "description": "The file the edits are in, as a diagnostic's `file` is.",
          "type": "string"
        },
        "title": {
          "description": "What the fix does.",
          "type": "string"
        }
      },
      "required": [
        "title",
        "file",
        "edits",
        "applicability"
      ],
      "type": "object"
    },
    "InventoryJson": {
      "description": "`inventory`: what the project holds.",
      "properties": {
        "by_owner": {
          "description": "How many pages each owner has, most first: the value of the\nfrontmatter field the page's type marks `role = \"owner\"`.",
          "items": {
            "$ref": "#/$defs/OwnerCount"
          },
          "type": "array"
        },
        "by_type": {
          "description": "How many pages of each content type, most first.",
          "items": {
            "$ref": "#/$defs/TypeCount"
          },
          "type": "array"
        },
        "orphans": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "The pages nothing links to (`page-orphan`)."
        },
        "overdue": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "The pages whose review date has passed (`review-overdue`)."
        },
        "pages": {
          "description": "How many pages.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "unused": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "The fragments, phrases, features, glossary terms, and images nothing\nuses."
        }
      },
      "required": [
        "pages",
        "by_type",
        "by_owner",
        "overdue",
        "orphans",
        "unused"
      ],
      "type": "object"
    },
    "LeftContentJson": {
      "description": "Content a build takes out of a page it publishes.",
      "properties": {
        "file": {
          "description": "The file it's written in: the page, or a fragment it includes.",
          "type": "string"
        },
        "kept_by": {
          "description": "The builds that keep it.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "kind": {
          "description": "`variant` for a variant arm, or `availability` for content a\nfeature's availability takes out.",
          "type": "string"
        },
        "page": {
          "description": "The page it's taken out of, as `file` is.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "Where in the file."
        },
        "why": {
          "description": "Why, with the content model's labels: `Shows only os=linux`.",
          "type": "string"
        }
      },
      "required": [
        "file",
        "range",
        "page",
        "kind",
        "why",
        "kept_by"
      ],
      "type": "object"
    },
    "LeftPageJson": {
      "description": "A page a build doesn't publish.",
      "properties": {
        "file": {
          "description": "The page, relative to the project root, as a diagnostic's `file` is.",
          "type": "string"
        },
        "kept_by": {
          "description": "The builds that publish it.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "why": {
          "description": "Why, as the end of a sentence: `its variant frontmatter names …`.",
          "type": "string"
        }
      },
      "required": [
        "file",
        "why",
        "kept_by"
      ],
      "type": "object"
    },
    "LinksJson": {
      "description": "`links`: what the link checker said about the external links.",
      "properties": {
        "acknowledged": {
          "$ref": "#/$defs/Capped_AcknowledgedEntry",
          "description": "The broken links acknowledged as intended."
        },
        "checked": {
          "description": "How many different addresses were checked.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "diagnostics": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "A diagnostic for each link that moved or is broken, at its place,\nand for each acknowledgement of a broken link that covers nothing."
        },
        "ignored": {
          "description": "How many `[checks.links] ignore` left out.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "not_run": {
          "anyOf": [
            {
              "$ref": "#/$defs/NotRunJson"
            },
            {
              "type": "null"
            }
          ],
          "description": "Why it couldn't run, or `null` when it ran. When it isn't `null`,\nthe rest is empty."
        }
      },
      "required": [
        "not_run",
        "checked",
        "ignored",
        "diagnostics",
        "acknowledged"
      ],
      "type": "object"
    },
    "NextCount": {
      "description": "How many findings have one kind of next step.",
      "properties": {
        "count": {
          "description": "How many.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "next": {
          "description": "`fix`, `choose`, `write`, `review`, or `outside`, as a diagnostic's\n`next` is.",
          "type": "string"
        }
      },
      "required": [
        "next",
        "count"
      ],
      "type": "object"
    },
    "NotRunJson": {
      "description": "Why a section couldn't run, and what would let it.",
      "properties": {
        "how": {
          "description": "What to do so that it runs, as a sentence.",
          "type": "string"
        },
        "reason": {
          "description": "Why, as a sentence.",
          "type": "string"
        }
      },
      "required": [
        "reason",
        "how"
      ],
      "type": "object"
    },
    "OwnerCount": {
      "description": "How many pages one owner has.",
      "properties": {
        "owner": {
          "description": "The owner, as the frontmatter gives it; `null` for the pages without\none.",
          "type": [
            "string",
            "null"
          ]
        },
        "pages": {
          "description": "How many pages.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "owner",
        "pages"
      ],
      "type": "object"
    },
    "Place": {
      "description": "A place in a file.",
      "properties": {
        "file": {
          "description": "The file, as a diagnostic's `file` is.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "Where in the file."
        }
      },
      "required": [
        "file",
        "range"
      ],
      "type": "object"
    },
    "Pos": {
      "description": "A position in a file.",
      "properties": {
        "column": {
          "description": "The column, from 1, in Unicode characters (not bytes or UTF-16\nunits).",
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "line": {
          "description": "The line, from 1.",
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "offset": {
          "description": "The byte offset from the start of the file.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "line",
        "column",
        "offset"
      ],
      "type": "object"
    },
    "ProblemsJson": {
      "description": "`problems`: what `ascribe check` reports, counted.",
      "properties": {
        "acknowledged": {
          "$ref": "#/$defs/Capped_AcknowledgedEntry",
          "description": "The problems acknowledged as intended, with their reasons, in file\norder."
        },
        "by_check": {
          "$ref": "#/$defs/Capped_CheckCount",
          "description": "How many diagnostics each check reports, most first."
        },
        "by_file": {
          "$ref": "#/$defs/Capped_FileCount",
          "description": "How many diagnostics each file has, most first."
        },
        "findings": {
          "$ref": "#/$defs/Findings",
          "description": "How many errors, warnings, and advice, and of each kind of next\nstep."
        },
        "list_command": {
          "description": "The command that lists each problem: `ascribe check`, as JSON.",
          "type": "string"
        }
      },
      "required": [
        "findings",
        "by_check",
        "by_file",
        "acknowledged",
        "list_command"
      ],
      "type": "object"
    },
    "Range": {
      "description": "A span of a file. `end` is just past its last character; an empty range\n(an insertion) has equal positions.",
      "properties": {
        "end": {
          "$ref": "#/$defs/Pos",
          "description": "Just past its last character."
        },
        "start": {
          "$ref": "#/$defs/Pos",
          "description": "Its first character."
        }
      },
      "required": [
        "start",
        "end"
      ],
      "type": "object"
    },
    "Related": {
      "description": "Another place that explains a diagnostic.",
      "properties": {
        "file": {
          "description": "The file, as a diagnostic's `file` is.",
          "type": "string"
        },
        "message": {
          "description": "What it has to do with the diagnostic.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "Where in the file."
        }
      },
      "required": [
        "file",
        "range",
        "message"
      ],
      "type": "object"
    },
    "TypeCount": {
      "description": "How many pages one content type has.",
      "properties": {
        "pages": {
          "description": "How many pages.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "type": {
          "description": "The type's name; `null` for the pages no one type applies to.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "type",
        "pages"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe report --format json` writes: one document, whatever the\noutcome. A reader ignores fields it doesn't know.",
  "properties": {
    "agents": {
      "anyOf": [
        {
          "$ref": "#/$defs/AgentsJson"
        },
        {
          "type": "null"
        }
      ],
      "description": "`agents`, when asked for."
    },
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "builds": {
      "description": "`builds`, when asked for: one for each build checked.",
      "items": {
        "$ref": "#/$defs/BuildJson"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "builds_checked": {
      "description": "The builds `problems` checked and `builds` reports on, in\n`ascribe.toml`'s order.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "error": {
      "description": "Why the report couldn't be made (exit code 2), or `null`. When it\nisn't `null`, no section is there.",
      "type": [
        "string",
        "null"
      ]
    },
    "findings": {
      "$ref": "#/$defs/Findings",
      "description": "How many findings of each severity the sections that ran have, and\nof each kind of next step: the diagnostics of `problems`, `links`,\nand `agents`."
    },
    "inventory": {
      "anyOf": [
        {
          "$ref": "#/$defs/InventoryJson"
        },
        {
          "type": "null"
        }
      ],
      "description": "`inventory`, when asked for."
    },
    "links": {
      "anyOf": [
        {
          "$ref": "#/$defs/LinksJson"
        },
        {
          "type": "null"
        }
      ],
      "description": "`links`, when asked for."
    },
    "next_command": {
      "description": "The command that lists everything a list here leaves out, when one\ndoes; `null` otherwise.",
      "type": [
        "string",
        "null"
      ]
    },
    "not_run": {
      "description": "The sections asked for that couldn't run. Each says why in its own\n`not_run`. With `--exit-code`, any is exit code 2.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "problems": {
      "anyOf": [
        {
          "$ref": "#/$defs/ProblemsJson"
        },
        {
          "type": "null"
        }
      ],
      "description": "`problems`, when asked for."
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "format": "uint32",
      "minimum": 0,
      "type": "integer"
    },
    "sections": {
      "description": "The sections asked for, in the order the report has them:\n`problems`, `inventory`, `builds`, `links`, `agents`.",
      "items": {
        "type": "string"
      },
      "type": "array"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "error",
    "sections",
    "builds_checked",
    "not_run",
    "findings",
    "next_command"
  ],
  "title": "ReportReport",
  "type": "object"
}
```
