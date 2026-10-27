@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/report.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
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
    "ApiCount": {
      "description": "How many of one API's operations the pages cover.",
      "properties": {
        "api": {
          "description": "The API's name, its key in `[apis]`.",
          "type": "string"
        },
        "examples": {
          "description": "How many of them a checked example claims as its request or\nresponse.",
          "minimum": 0,
          "type": "integer"
        },
        "linked": {
          "description": "How many of them a link names, by `operationId` or by method and\npath.",
          "minimum": 0,
          "type": "integer"
        },
        "operations": {
          "description": "How many operations its spec has.",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "api",
        "operations",
        "linked",
        "examples"
      ],
      "type": "object"
    },
    "BuildJson": {
      "description": "What one build leaves out that another build keeps, once for each mark\nthat leaves it out.",
      "properties": {
        "build": {
          "description": "The build.",
          "type": "string"
        },
        "content": {
          "$ref": "#/$defs/Capped_LeftContentJson",
          "description": "The marks that take content out of the pages it publishes, in file\nand source order: each `@available` and variant arm once, however\nmany blocks it takes out."
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
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
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
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
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
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
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
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
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
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
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
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
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
    "Capped_PageRecordJson": {
      "description": "A list of at most `--limit` items.",
      "properties": {
        "items": {
          "description": "The items, at most `--limit` of them.",
          "items": {
            "$ref": "#/$defs/PageRecordJson"
          },
          "type": "array"
        },
        "shown": {
          "description": "How many are listed.",
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
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
    "Capped_UnconfirmedEntry": {
      "description": "A list of at most `--limit` items.",
      "properties": {
        "items": {
          "description": "The items, at most `--limit` of them.",
          "items": {
            "$ref": "#/$defs/UnconfirmedEntry"
          },
          "type": "array"
        },
        "shown": {
          "description": "How many are listed.",
          "minimum": 0,
          "type": "integer"
        },
        "total": {
          "description": "How many there are.",
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
    "DeliveryJson": {
      "description": "`delivery`: the delivery spec's checks on a built site.",
      "properties": {
        "diagnostics": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "A diagnostic for each check that failed or warned that the hosting\nor Ascribe owns. One a page owns is `ascribe check`'s, and isn't\nrepeated."
        },
        "new": {
          "description": "How many checks fail or warn now that passed on the last report or\nweren't in it, or `null` when `previous_run` is. A check is told from\nanother by the site's address and its id.",
          "minimum": 0,
          "type": [
            "integer",
            "null"
          ]
        },
        "new_checks": {
          "description": "The ids of the checks `new` counts, in the checker's order; empty\nwhen `previous_run` is `null`.",
          "items": {
            "type": "string"
          },
          "type": "array"
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
        "previous_run": {
          "description": "The day of the last report that checked the same site,\n`YYYY-MM-DD`, or `null` on a first run, with `--no-history`, or when\nthe last report checked another site or none.",
          "type": [
            "string",
            "null"
          ]
        },
        "resolved": {
          "description": "How many checks that failed or warned on the last report pass now or\naren't in this one, or `null` when `previous_run` is.",
          "minimum": 0,
          "type": [
            "integer",
            "null"
          ]
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
        "diagnostics",
        "previous_run",
        "new",
        "resolved",
        "new_checks"
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
          "description": "The address of its entry in the diagnostics reference: the docs'\npermalink for its code, such as `https://ascribed-dev.com/go/ASC036/`,\nwhich forwards to the entry wherever it is.",
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
    "ExamplesCount": {
      "description": "How many snippets of one source show a tested example, by status.",
      "properties": {
        "current": {
          "description": "How many of those show an example an observation is current for.",
          "minimum": 0,
          "type": "integer"
        },
        "snippets": {
          "description": "How many `@snippet` lines, in pages and fragments, show one of its\nexamples.",
          "minimum": 0,
          "type": "integer"
        },
        "source": {
          "description": "The source's name.",
          "type": "string"
        },
        "stale": {
          "description": "How many show one that changed after every observation listing it.",
          "minimum": 0,
          "type": "integer"
        },
        "unobserved": {
          "description": "How many show one no observation lists.",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "source",
        "snippets",
        "current",
        "stale",
        "unobserved"
      ],
      "type": "object"
    },
    "FileCount": {
      "description": "How many diagnostics one file has.",
      "properties": {
        "advice": {
          "description": "How many advice.",
          "minimum": 0,
          "type": "integer"
        },
        "errors": {
          "description": "How many errors.",
          "minimum": 0,
          "type": "integer"
        },
        "file": {
          "description": "The file, relative to the project root, as a diagnostic's `file` is.",
          "type": "string"
        },
        "warnings": {
          "description": "How many warnings.",
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
        "apis": {
          "description": "How many of each declared API's operations the pages cover, for each\nAPI whose spec could be read, in the content model's order. Absent\nwhen the project declares no API.",
          "items": {
            "$ref": "#/$defs/ApiCount"
          },
          "type": [
            "array",
            "null"
          ]
        },
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
        "examples": {
          "description": "The tested examples the snippets show, for each source in the same\nrepository with an `ascribe.examples.toml`, in the content model's\norder. Absent when no source has one.",
          "items": {
            "$ref": "#/$defs/ExamplesCount"
          },
          "type": [
            "array",
            "null"
          ]
        },
        "orphans": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "The pages nothing links to (`page-orphan`)."
        },
        "overdue": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "The pages whose recheck date has passed (`recheck-overdue`)."
        },
        "pages": {
          "description": "How many pages.",
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
      "description": "A mark that takes content out of a page a build publishes.",
      "properties": {
        "file": {
          "description": "The file the mark is written in: the page, or a fragment it includes.",
          "type": "string"
        },
        "heading": {
          "description": "The text of the heading the mark is under, or `null`: the section's\nown heading, for a section's `@available`.",
          "type": [
            "string",
            "null"
          ]
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
          "description": "Where in the file: the `@available` line, the table row, or the\nvariant arm."
        },
        "scope": {
          "description": "What the mark takes out: `section`, `block`, `row` (a table row), or\n`arm` (a variant arm).",
          "type": "string"
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
        "heading",
        "kind",
        "scope",
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
        "kind": {
          "description": "`availability` when its `available` frontmatter leaves it out, or\n`variant` when its `variant` frontmatter does.",
          "type": "string"
        },
        "mark": {
          "description": "That frontmatter's value, as written: `self-managed 3.5`, or\n`deployment=cloud` for `variant`, with a dimension's values joined by\n`|`.",
          "type": "string"
        }
      },
      "required": [
        "file",
        "kind",
        "mark",
        "kept_by"
      ],
      "type": "object"
    },
    "LinksJson": {
      "description": "`links`: what the link checker said about the external links.",
      "properties": {
        "acknowledged": {
          "$ref": "#/$defs/Capped_AcknowledgedEntry",
          "description": "The broken links acknowledged as intended, confirmed or not."
        },
        "asked": {
          "description": "How many different addresses the link checker was asked about in\nthis run.",
          "minimum": 0,
          "type": "integer"
        },
        "checked": {
          "description": "How many different addresses were checked: `asked` and `reused`.",
          "minimum": 0,
          "type": "integer"
        },
        "diagnostics": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "A diagnostic for each link that moved or is broken, at its place,\nand for each acknowledgement of a broken link that covers nothing."
        },
        "ignored": {
          "description": "How many `[checks.links] ignore` left out.",
          "minimum": 0,
          "type": "integer"
        },
        "new": {
          "description": "How many of `diagnostics` the previous run didn't report, or `null`\nwhen `previous_run` is.",
          "minimum": 0,
          "type": [
            "integer",
            "null"
          ]
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
        "oldest_answer": {
          "description": "The day of the oldest answer reused, `YYYY-MM-DD`, or `null` when\nnone was.",
          "type": [
            "string",
            "null"
          ]
        },
        "previous_run": {
          "description": "The day of the run the link history is from, `YYYY-MM-DD`, or `null`\non a first run, or with `--no-history`.",
          "type": [
            "string",
            "null"
          ]
        },
        "resolved": {
          "description": "How many addresses the previous run reported that this one doesn't,\nfixed or no longer linked to, or `null` when `previous_run` is.",
          "minimum": 0,
          "type": [
            "integer",
            "null"
          ]
        },
        "reused": {
          "description": "How many different addresses' answers were reused from an earlier\nrun, within `[checks.links] reuse-days`. None of `diagnostics` is\nfrom a reused answer: only an answer with nothing to report is kept.",
          "minimum": 0,
          "type": "integer"
        },
        "unconfirmed": {
          "$ref": "#/$defs/Capped_UnconfirmedEntry",
          "description": "The broken links that answered before and aren't reported until\nthey've kept failing for `[checks.links] confirm-days`. They aren't\nin `diagnostics` or the summary, and don't fail `--exit-code`."
        }
      },
      "required": [
        "not_run",
        "checked",
        "asked",
        "reused",
        "oldest_answer",
        "ignored",
        "diagnostics",
        "acknowledged",
        "previous_run",
        "new",
        "resolved",
        "unconfirmed"
      ],
      "type": "object"
    },
    "ListingFromJson": {
      "description": "Where a page's listing comes from.",
      "properties": {
        "folder": {
          "description": "For a folder's rule, the folder, as `[listing.folders]` names it.",
          "type": [
            "string",
            "null"
          ]
        },
        "level": {
          "description": "`default` when nothing sets it, `page` for the page's own `listing`,\nor the level of the rule of `[listing]` it's under: `site`,\n`older-lines`, `folder`, or `type`.",
          "type": "string"
        },
        "line": {
          "description": "For a rule, its line in `ascribe.toml`.",
          "minimum": 0,
          "type": [
            "integer",
            "null"
          ]
        },
        "rule": {
          "description": "For a rule, the rule as written in `ascribe.toml`.",
          "type": [
            "string",
            "null"
          ]
        },
        "type": {
          "description": "For a type's rule, the type.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "level"
      ],
      "type": "object"
    },
    "NextCount": {
      "description": "How many problems have one kind of next step.",
      "properties": {
        "count": {
          "description": "How many.",
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
    "PageRecordJson": {
      "description": "What a build says about one page: its entry in `_ascribe/pages.json`,\nwith its route.",
      "properties": {
        "breadcrumbs": {
          "description": "Its place in the site, from the top down to the page itself.",
          "items": {
            "$ref": "#/$defs/StepJson"
          },
          "type": "array"
        },
        "canonical": {
          "description": "Its own address, absolute; absent without `[consumer] site`.",
          "type": [
            "string",
            "null"
          ]
        },
        "description": {
          "description": "Its description, as plain text on one line; absent when it has none.",
          "type": [
            "string",
            "null"
          ]
        },
        "listing": {
          "description": "`listed`, `unquoted`, or `unlisted`.",
          "type": "string"
        },
        "listing_from": {
          "$ref": "#/$defs/ListingFromJson",
          "description": "Where `listing` comes from."
        },
        "markdown": {
          "description": "Its Markdown version's address, from the host's root; absent when the\nbuild publishes none.",
          "type": [
            "string",
            "null"
          ]
        },
        "route": {
          "description": "The page's address, from the host's root, as the site output's links\nwrite it.",
          "type": "string"
        },
        "source": {
          "description": "The page's source path, from the content root.",
          "type": "string"
        },
        "title": {
          "description": "Its title, as plain text; absent when it has none.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "route",
        "source",
        "listing",
        "listing_from",
        "breadcrumbs"
      ],
      "type": "object"
    },
    "PagesJson": {
      "description": "One build's page records.",
      "properties": {
        "build": {
          "description": "The build.",
          "type": "string"
        },
        "not_listed": {
          "description": "How many of them aren't `listed`: `unquoted` or `unlisted`.",
          "minimum": 0,
          "type": "integer"
        },
        "published": {
          "description": "How many pages the build publishes.",
          "minimum": 0,
          "type": "integer"
        },
        "records": {
          "$ref": "#/$defs/Capped_PageRecordJson",
          "description": "The records, sorted by `route`: every page's, or with `--not-listed`\nthose that aren't `listed`. Each has the fields of its entry in the\nbuild's `_ascribe/pages.json` (output-layout contract §1.3), by the\nsame names, with `route`, the key it's under there."
        },
        "site": {
          "description": "The project's address, `[consumer] site`, as the build's\n`_ascribe/pages.json` has it; absent when it isn't set.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "build",
        "published",
        "not_listed",
        "records"
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
          "minimum": 0,
          "type": "integer"
        },
        "line": {
          "description": "The line, from 1.",
          "minimum": 0,
          "type": "integer"
        },
        "offset": {
          "description": "The byte offset from the start of the file.",
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
        "baselined": {
          "description": "How many advice `ascribe.baseline` holds back, which the counts\nleave out: the backlog the project has put off. Left out when there\nare none, and with `--no-baseline`.",
          "minimum": 0,
          "type": "integer"
        },
        "by_check": {
          "$ref": "#/$defs/Capped_CheckCount",
          "description": "How many diagnostics each check reports, most first."
        },
        "by_file": {
          "$ref": "#/$defs/Capped_FileCount",
          "description": "How many diagnostics each file has, most first."
        },
        "list_command": {
          "description": "The command that lists each problem: `ascribe check`, as JSON.",
          "type": "string"
        },
        "new": {
          "description": "How many problems the last report didn't list, or `null` when\n`previous_run` is. A problem is told from another by its check, its\nfile, the text it's about, and its order among those alike.",
          "minimum": 0,
          "type": [
            "integer",
            "null"
          ]
        },
        "new_items": {
          "$ref": "#/$defs/Capped_Entry",
          "description": "The problems the last report didn't list, in the order `ascribe\ncheck` reports them; none when `previous_run` is `null`."
        },
        "previous_run": {
          "description": "The day of the last report, `YYYY-MM-DD`, or `null` on a first run,\nwith `--no-history`, or when the last report checked other builds.",
          "type": [
            "string",
            "null"
          ]
        },
        "resolved": {
          "description": "How many problems the last report listed that this one doesn't,\nfixed or with the text they're about edited, or `null` when\n`previous_run` is.",
          "minimum": 0,
          "type": [
            "integer",
            "null"
          ]
        },
        "summary": {
          "$ref": "#/$defs/Summary",
          "description": "How many errors, warnings, and advice, and of each kind of next\nstep."
        }
      },
      "required": [
        "summary",
        "by_check",
        "by_file",
        "acknowledged",
        "list_command",
        "previous_run",
        "new",
        "resolved",
        "new_items"
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
    "StepJson": {
      "description": "A step of a page's breadcrumbs.",
      "properties": {
        "route": {
          "description": "The step's page's address, as `route` is.",
          "type": "string"
        },
        "title": {
          "description": "The step's page's title, as plain text.",
          "type": "string"
        }
      },
      "required": [
        "route",
        "title"
      ],
      "type": "object"
    },
    "Summary": {
      "description": "How many problems of each severity, and of each kind of next step.",
      "properties": {
        "advice": {
          "description": "How many advice.",
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
          "minimum": 0,
          "type": "integer"
        },
        "warnings": {
          "description": "How many warnings.",
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
    "TypeCount": {
      "description": "How many pages one content type has.",
      "properties": {
        "pages": {
          "description": "How many pages.",
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
    },
    "UnconfirmedEntry": {
      "description": "A broken link that answered before, not reported yet.",
      "properties": {
        "failing_since": {
          "description": "The day it started failing, `YYYY-MM-DD`.",
          "type": "string"
        },
        "file": {
          "description": "The file, as a diagnostic's `file` is.",
          "type": "string"
        },
        "last_answered": {
          "description": "The last day it answered, `YYYY-MM-DD`.",
          "type": "string"
        },
        "range": {
          "$ref": "#/$defs/Range",
          "description": "Where in the file: the link."
        },
        "reported_from": {
          "description": "The day it's reported from if it's still failing, `YYYY-MM-DD`.",
          "type": "string"
        },
        "status": {
          "description": "What the link checker said: a status, such as `503 Service\nUnavailable`, or why it didn't answer.",
          "type": "string"
        },
        "url": {
          "description": "The link's address.",
          "type": "string"
        }
      },
      "required": [
        "file",
        "range",
        "url",
        "status",
        "last_answered",
        "failing_since",
        "reported_from"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe report --format json` writes: one document, whatever the\noutcome. A reader ignores fields it doesn't know.",
  "properties": {
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
      "description": "The builds `problems` checked and `builds` and `pages` report on, in\n`ascribe.toml`'s order.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "delivery": {
      "anyOf": [
        {
          "$ref": "#/$defs/DeliveryJson"
        },
        {
          "type": "null"
        }
      ],
      "description": "`delivery`, when asked for."
    },
    "error": {
      "description": "Why the report couldn't be made (exit code 2), or `null`. When it\nisn't `null`, no section is there.",
      "type": [
        "string",
        "null"
      ]
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
    "pages": {
      "description": "`pages`, when asked for: one for each build checked.",
      "items": {
        "$ref": "#/$defs/PagesJson"
      },
      "type": [
        "array",
        "null"
      ]
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
      "minimum": 0,
      "type": "integer"
    },
    "sections": {
      "description": "The sections asked for, in the order the report has them:\n`problems`, `inventory`, `builds`, `pages`, `links`, `delivery`.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "summary": {
      "$ref": "#/$defs/Summary",
      "description": "How many problems of each severity the sections that ran have, and\nof each kind of next step: the diagnostics of `problems`, `links`,\nand `delivery`."
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "error",
    "sections",
    "builds_checked",
    "not_run",
    "summary",
    "next_command"
  ],
  "title": "ReportReport",
  "type": "object"
}
```
