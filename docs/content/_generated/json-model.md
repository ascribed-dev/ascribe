@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/model.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/model.schema.json`]({repo}/blob/main/schemas/model.schema.json):

```json
{
  "$defs": {
    "ModelApi": {
      "description": "An API: an OpenAPI spec that pages link into.",
      "properties": {
        "name": {
          "description": "Its name, as `@operation` and `api:` addresses write it.",
          "type": "string"
        },
        "reference": {
          "description": "Where its reference is: the renderer's page, or an address\npattern, as written.",
          "type": "string"
        },
        "renderer": {
          "description": "The renderer its reference is published with, `redoc`; absent when\n`reference` is an address pattern.",
          "type": [
            "string",
            "null"
          ]
        },
        "spec": {
          "description": "The spec's file, from the folder `ascribe.toml` is in.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "spec",
        "reference"
      ],
      "type": "object"
    },
    "ModelAttribute": {
      "description": "An attribute a widget accepts.",
      "properties": {
        "default": {
          "description": "The value used when it's left out, as written (a set's members joined\nby `|`).",
          "type": [
            "string",
            "null"
          ]
        },
        "description": {
          "description": "What it's for.",
          "type": [
            "string",
            "null"
          ]
        },
        "key": {
          "description": "The key.",
          "type": "string"
        },
        "required": {
          "description": "Whether every use must give it.",
          "type": "boolean"
        },
        "type": {
          "description": "The value's type: `string`, `number`, `boolean`, `enum`, `set`, or\n`note-type`.",
          "type": "string"
        },
        "values": {
          "description": "The values it allows, for `enum` and a `set` of named values; empty\notherwise.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "key",
        "type",
        "values",
        "required",
        "default",
        "description"
      ],
      "type": "object"
    },
    "ModelBuild": {
      "description": "A build.",
      "properties": {
        "availability": {
          "description": "How it treats availability: `badge`, or the target it filters for,\nsuch as `filter self-managed 3.3`.",
          "type": "string"
        },
        "editor": {
          "description": "Whether it's the build the editor checks (`[editor] build`).",
          "type": "boolean"
        },
        "line": {
          "description": "For a line of a versioned build, the line, as `[releases.<target>]`\nwrites it.",
          "type": [
            "string",
            "null"
          ]
        },
        "name": {
          "description": "Its name.",
          "type": "string"
        },
        "variants": {
          "description": "Which variant arms it keeps: `switch` for all of them, or the\nselection, such as `deployment=cloud`.",
          "type": "string"
        },
        "versions": {
          "description": "For a line of a versioned build, the target it's versioned by.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "name",
        "variants",
        "availability",
        "editor"
      ],
      "type": "object"
    },
    "ModelDimension": {
      "description": "A dimension.",
      "properties": {
        "label": {
          "description": "Its label.",
          "type": "string"
        },
        "name": {
          "description": "Its name.",
          "type": "string"
        },
        "values": {
          "description": "Its values, in display order.",
          "items": {
            "$ref": "#/$defs/ModelDimensionValue"
          },
          "type": "array"
        }
      },
      "required": [
        "name",
        "label",
        "values"
      ],
      "type": "object"
    },
    "ModelDimensionValue": {
      "description": "A value of a dimension.",
      "properties": {
        "label": {
          "description": "Its label.",
          "type": "string"
        },
        "value": {
          "description": "The value.",
          "type": "string"
        },
        "versionless": {
          "description": "Whether it has no versions, so an availability spec gives it a state\nbut no version.",
          "type": "boolean"
        }
      },
      "required": [
        "value",
        "label",
        "versionless"
      ],
      "type": "object"
    },
    "ModelExamples": {
      "description": "A source's tested examples.",
      "properties": {
        "files": {
          "description": "How many of the files the source makes readable its config makes\nexamples.",
          "minimum": 0,
          "type": "integer"
        },
        "observations": {
          "description": "Its observations, in name order: those that can't be read left out.",
          "items": {
            "$ref": "#/$defs/ModelObservation"
          },
          "type": "array"
        },
        "valid": {
          "description": "Whether its `ascribe.examples.toml` can be read. One that can't makes\nno file an example.",
          "type": "boolean"
        }
      },
      "required": [
        "valid",
        "files",
        "observations"
      ],
      "type": "object"
    },
    "ModelFeature": {
      "description": "A feature.",
      "properties": {
        "availability": {
          "description": "Its availability spec, as written.",
          "type": "string"
        },
        "key": {
          "description": "Its key, which an availability spec can name.",
          "type": "string"
        },
        "name": {
          "description": "Its name.",
          "type": "string"
        }
      },
      "required": [
        "key",
        "name",
        "availability"
      ],
      "type": "object"
    },
    "ModelField": {
      "description": "A frontmatter field.",
      "properties": {
        "declared": {
          "description": "Which table declares a page type's field: `frontmatter` for\n`[frontmatter]`, which every type shares, `type` for the type's own\n`frontmatter`, or `implicit` for the required string `title` neither\ndeclares. Absent on an object's fields.",
          "type": [
            "string",
            "null"
          ]
        },
        "description": {
          "description": "What it's for, as the content model describes it.",
          "type": [
            "string",
            "null"
          ]
        },
        "fields": {
          "description": "An object's fields; empty for any other type.",
          "items": {
            "$ref": "#/$defs/ModelField"
          },
          "type": "array"
        },
        "name": {
          "description": "Its name.",
          "type": "string"
        },
        "required": {
          "description": "Whether every page of the type must give it.",
          "type": "boolean"
        },
        "type": {
          "description": "Its type in short form: `string`, `list(string)`, `enum(a, b)`.",
          "type": "string"
        },
        "values": {
          "description": "The values it allows, for an enumeration or a list of one; empty\notherwise.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "name",
        "type",
        "required",
        "values",
        "description",
        "fields"
      ],
      "type": "object"
    },
    "ModelFragments": {
      "description": "`[fragments]`.",
      "properties": {
        "fields": {
          "description": "The fields a fragment's frontmatter may give, in declaration order.",
          "items": {
            "$ref": "#/$defs/ModelField"
          },
          "type": "array"
        },
        "patterns": {
          "description": "The patterns of the files that are fragments besides those whose\npath has a file or folder name starting with `_`, from the content\nroot.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "patterns",
        "fields"
      ],
      "type": "object"
    },
    "ModelFrom": {
      "description": "A file `[from]` takes tables or keys from.",
      "properties": {
        "file": {
          "description": "The file, from the folder `ascribe.toml` is in.",
          "type": "string"
        },
        "take": {
          "description": "What it takes, as `take` names it: `releases`, `consumer.site`.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "file",
        "take"
      ],
      "type": "object"
    },
    "ModelIntended": {
      "description": "An acknowledged problem: a check's problem about a content model entry\nor an image, which is intended.",
      "properties": {
        "check": {
          "description": "The check, by its slug.",
          "type": "string"
        },
        "kind": {
          "description": "What it's about: `phrase`, `feature`, or `image`.",
          "type": "string"
        },
        "name": {
          "description": "Its key, or the image's path.",
          "type": "string"
        },
        "reason": {
          "description": "Why the problem is intended.",
          "type": "string"
        }
      },
      "required": [
        "check",
        "kind",
        "name",
        "reason"
      ],
      "type": "object"
    },
    "ModelLifecycleState": {
      "description": "A lifecycle state.",
      "properties": {
        "available": {
          "description": "Whether content in it counts as available.",
          "type": "boolean"
        },
        "builtin": {
          "description": "Whether it's built in, rather than declared by the project.",
          "type": "boolean"
        },
        "label": {
          "description": "Its label.",
          "type": "string"
        },
        "name": {
          "description": "Its name, which an availability spec gives.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "label",
        "available",
        "builtin"
      ],
      "type": "object"
    },
    "ModelLine": {
      "description": "A release line.",
      "properties": {
        "builds": {
          "description": "The builds that are this line, one for each build versioned by the\ntarget: `site@3.4`, or `site` for the current line.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "line": {
          "description": "The line, as the list writes it.",
          "type": "string"
        },
        "status": {
          "description": "`current`, `live`, `planned`, or `archived`.",
          "type": "string"
        }
      },
      "required": [
        "line",
        "status",
        "builds"
      ],
      "type": "object"
    },
    "ModelNavPage": {
      "description": "A page of the navigation.",
      "properties": {
        "label": {
          "description": "Its label as plain text: its `nav-label`, else its title, else its\nfile name without `.md`.",
          "type": "string"
        },
        "source": {
          "description": "Its path from the content root.",
          "type": "string"
        },
        "type": {
          "description": "Its content type's name, when it has one.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "source",
        "label"
      ],
      "type": "object"
    },
    "ModelNavSection": {
      "description": "A section of the navigation: a folder and its pages, in order.",
      "properties": {
        "folder": {
          "description": "The folder, from the content root, ending in `/`; `\"\"` for the root.",
          "type": "string"
        },
        "label": {
          "description": "Its label: from `ascribe.nav`, else its own page's title as plain\ntext, else its folder's name with `-` and `_` as spaces and its first\nletter capitalized, and \"Start\" for the root.",
          "type": "string"
        },
        "listed": {
          "description": "Whether `ascribe.nav` lists it.",
          "type": "boolean"
        },
        "page": {
          "anyOf": [
            {
              "$ref": "#/$defs/ModelNavPage"
            },
            {
              "type": "null"
            }
          ],
          "description": "Its own page: its `index.md`, or the page beside the folder with its\nname, which is then one of the parent's `pages` too."
        },
        "pages": {
          "description": "Its pages in order, its own page not among them.",
          "items": {
            "$ref": "#/$defs/ModelNavPage"
          },
          "type": "array"
        },
        "sections": {
          "description": "Its nested sections with a page, in order: each one whose own page\nis among `pages` right after that page, then the rest.",
          "items": {
            "$ref": "#/$defs/ModelNavSection"
          },
          "type": "array"
        }
      },
      "required": [
        "folder",
        "label",
        "pages",
        "sections",
        "listed"
      ],
      "type": "object"
    },
    "ModelNoteType": {
      "description": "A note type.",
      "properties": {
        "builtin": {
          "description": "Whether it's built in, rather than declared by the project.",
          "type": "boolean"
        },
        "label": {
          "description": "Its label.",
          "type": "string"
        },
        "name": {
          "description": "Its name, which `@note`'s `type` gives.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "label",
        "builtin"
      ],
      "type": "object"
    },
    "ModelObservation": {
      "description": "An observation of a source's examples.",
      "properties": {
        "current": {
          "description": "How many of those are as it found them: the rest changed since, or\nare gone.",
          "minimum": 0,
          "type": "integer"
        },
        "environment": {
          "additionalProperties": {
            "type": "string"
          },
          "description": "Its environment as it has it, by name: `os`, `arch`, and each\nentry's kept line.",
          "type": "object"
        },
        "listed": {
          "description": "How many files it lists. For a source in another repository, only\nthose it has copies of, since the others weren't copied.",
          "minimum": 0,
          "type": "integer"
        },
        "name": {
          "description": "Its name: `default`, or the name `ascribe observe --name` gave it.",
          "type": "string"
        },
        "tested": {
          "description": "What a page says it was tested with, by this observation alone.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "environment",
        "tested",
        "listed",
        "current"
      ],
      "type": "object"
    },
    "ModelPhrase": {
      "description": "A phrase.",
      "properties": {
        "key": {
          "description": "Its key, written `{key}` in a page.",
          "type": "string"
        },
        "value": {
          "description": "Its value.",
          "type": "string"
        }
      },
      "required": [
        "key",
        "value"
      ],
      "type": "object"
    },
    "ModelReleases": {
      "description": "A target's release lines.",
      "properties": {
        "lines": {
          "description": "Its lines, newest first: planned, live, then archived.",
          "items": {
            "$ref": "#/$defs/ModelLine"
          },
          "type": "array"
        },
        "target": {
          "description": "The target: a versioned dimension value.",
          "type": "string"
        }
      },
      "required": [
        "target",
        "lines"
      ],
      "type": "object"
    },
    "ModelRemote": {
      "description": "The repository a source is in.",
      "properties": {
        "branch": {
          "description": "The branch an update follows; `null` for the repository's default.",
          "type": [
            "string",
            "null"
          ]
        },
        "url": {
          "description": "Its URL.",
          "type": "string"
        }
      },
      "required": [
        "url",
        "branch"
      ],
      "type": "object"
    },
    "ModelSetting": {
      "description": "A key of a section of settings, such as `[consumer]`, as the registry of\n`ascribe.toml`'s keys has it, with the value in effect.",
      "properties": {
        "default": {
          "description": "The value used when the file leaves it out, when that's one value."
        },
        "default_text": {
          "description": "What's used when the file leaves it out, in the reference's words,\nwhen that isn't one value.",
          "type": [
            "string",
            "null"
          ]
        },
        "from": {
          "description": "The file it's taken from, when `[from]` takes it: a path from the\nfolder `ascribe.toml` is in. Its value is that file's, and it's\n`set`.",
          "type": [
            "string",
            "null"
          ]
        },
        "key": {
          "description": "The key: in `checks`, a check's slug.",
          "type": "string"
        },
        "kind": {
          "description": "What its value is: `switch`, `choice`, `number`, `text`, `level`,\n`list`, or `table`.",
          "type": "string"
        },
        "set": {
          "description": "Whether `ascribe.toml` sets it.",
          "type": "boolean"
        },
        "table": {
          "description": "The table it's in, as `ascribe.toml` writes it: `consumer`,\n`checks.vale`.",
          "type": "string"
        },
        "value": {
          "description": "The value in effect: the file's, else the default. Absent when the\nfile leaves it out and its default isn't one value."
        }
      },
      "required": [
        "table",
        "key",
        "kind",
        "set"
      ],
      "type": "object"
    },
    "ModelSource": {
      "description": "A source: a folder of files, such as code, that pages can show.",
      "properties": {
        "examples": {
          "anyOf": [
            {
              "$ref": "#/$defs/ModelExamples"
            },
            {
              "type": "null"
            }
          ],
          "description": "Its tested examples, for a source with an `ascribe.examples.toml`:\nabsent for any other. For a source in another repository, they're\nread from its copies."
        },
        "git": {
          "anyOf": [
            {
              "$ref": "#/$defs/ModelRemote"
            },
            {
              "type": "null"
            }
          ],
          "description": "The repository it's in, for a source in another repository."
        },
        "ignore": {
          "description": "The patterns of the files it doesn't make readable, though `include`\nmatches them.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "include": {
          "description": "The patterns of the files it makes readable, relative to its folder;\nempty for every file.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "name": {
          "description": "Its name, as addresses write it: `code:` for a source named `code`.",
          "type": "string"
        },
        "path": {
          "description": "Its folder, relative to the project's folder: the `path` written,\nor, for a source in another repository, the folder of its copies.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "path",
        "include",
        "ignore",
        "git"
      ],
      "type": "object"
    },
    "ModelTerm": {
      "description": "A glossary term: a heading marked `@term` on a glossary page. Its\ndefinition is the paragraph under the heading, which differs by build:\nthe JSON output of a build has it.",
      "properties": {
        "aliases": {
          "description": "Other ways it's written.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "case_sensitive": {
          "description": "Whether an occurrence must match its case.",
          "type": "boolean"
        },
        "id": {
          "description": "Its id: its heading's id.",
          "type": "string"
        },
        "line": {
          "description": "The heading's line in that file, from 1.",
          "minimum": 0,
          "type": "integer"
        },
        "match": {
          "description": "Which occurrences are linked: `first` on each page, `every` one, or\nonly those an author links (`marked`).",
          "type": "string"
        },
        "page": {
          "description": "The file its heading is written in, from the content root: its\nglossary page, or a fragment the page includes.",
          "type": "string"
        },
        "term": {
          "description": "The term: its heading's text.",
          "type": "string"
        }
      },
      "required": [
        "id",
        "term",
        "aliases",
        "match",
        "case_sensitive",
        "page",
        "line"
      ],
      "type": "object"
    },
    "ModelType": {
      "description": "A page type.",
      "properties": {
        "fields": {
          "description": "Its frontmatter fields, in declaration order.",
          "items": {
            "$ref": "#/$defs/ModelField"
          },
          "type": "array"
        },
        "guide": {
          "description": "What to read before writing a page of the type, in the order to read\nit, from the folder `ascribe.toml` is in, from the type's `guide` or\nthe lock, as `template` is; absent when it has none.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "name": {
          "description": "Its name, which each page of the type gives as its `type`; the\nimplicit `page` type's pages give none.",
          "type": "string"
        },
        "purpose": {
          "description": "What a page of the type is for, as the content model says; absent\nwhen it says nothing.",
          "type": [
            "string",
            "null"
          ]
        },
        "template": {
          "description": "The page a page of the type starts from, from the folder\n`ascribe.toml` is in: the type's `template`, or the one\n`templates/templates.lock` gives a type that names neither a template\nnor a guide; absent when it has none.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "name",
        "fields"
      ],
      "type": "object"
    },
    "ModelWidget": {
      "description": "A project widget.",
      "properties": {
        "attributes": {
          "description": "Its attributes, in canonical order.",
          "items": {
            "$ref": "#/$defs/ModelAttribute"
          },
          "type": "array"
        },
        "container": {
          "description": "Whether it may be a container, closed by `@end`.",
          "type": "boolean"
        },
        "description": {
          "description": "What it's for, as the content model describes it.",
          "type": [
            "string",
            "null"
          ]
        },
        "line": {
          "description": "Whether it may be one line.",
          "type": "boolean"
        },
        "name": {
          "description": "Its name, written `@name`.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "description",
        "line",
        "container",
        "attributes"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe model --format json` answers: the content model's sections,\nor the one asked for.",
  "properties": {
    "apis": {
      "description": "APIs, in declaration order.",
      "items": {
        "$ref": "#/$defs/ModelApi"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "builds": {
      "description": "Builds, in declaration order, a versioned build as one for each live\nline, the current line first.",
      "items": {
        "$ref": "#/$defs/ModelBuild"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "checks": {
      "description": "`[checks]`: each key of `[checks.vale]` and `[checks.links]`, with its\nvalue in effect, and each check the file sets, by its slug.",
      "items": {
        "$ref": "#/$defs/ModelSetting"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "consumer": {
      "description": "`[consumer]`: each key, with its value in effect.",
      "items": {
        "$ref": "#/$defs/ModelSetting"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "crawlers": {
      "description": "`[crawlers]`: each key, with its value in effect.",
      "items": {
        "$ref": "#/$defs/ModelSetting"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "dimensions": {
      "description": "Dimensions, in declaration order.",
      "items": {
        "$ref": "#/$defs/ModelDimension"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "editor": {
      "description": "`[editor]`: each key, with its value in effect.",
      "items": {
        "$ref": "#/$defs/ModelSetting"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "features": {
      "description": "Features, in declaration order.",
      "items": {
        "$ref": "#/$defs/ModelFeature"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "fragments": {
      "anyOf": [
        {
          "$ref": "#/$defs/ModelFragments"
        },
        {
          "type": "null"
        }
      ],
      "description": "`[fragments]`: the files that are fragments, and the fields a\nfragment's frontmatter may give."
    },
    "from": {
      "description": "`[from]`: each file the project takes from, in the order `take`\nnames them, with what it takes.",
      "items": {
        "$ref": "#/$defs/ModelFrom"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "frontmatter": {
      "description": "`[frontmatter]`: the fields every page type has, as written, in\ndeclaration order. Each type's `fields` has them already.",
      "items": {
        "$ref": "#/$defs/ModelField"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "glossary": {
      "description": "Glossary terms, in the order of `[glossary] pages`, and on each page\nin document order.",
      "items": {
        "$ref": "#/$defs/ModelTerm"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "images": {
      "description": "`[images.attributes]`: the keys an image's attribute block may give,\nin declaration order.",
      "items": {
        "$ref": "#/$defs/ModelAttribute"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "intended": {
      "description": "`[[intended]]`: the acknowledged problems, in declaration order.",
      "items": {
        "$ref": "#/$defs/ModelIntended"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "lifecycle": {
      "description": "Lifecycle states: the built-in ones, as the project adjusts them,\nthen the ones it declares.",
      "items": {
        "$ref": "#/$defs/ModelLifecycleState"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "listing": {
      "description": "`[listing]`: each key, with its value in effect.",
      "items": {
        "$ref": "#/$defs/ModelSetting"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "navigation": {
      "anyOf": [
        {
          "$ref": "#/$defs/ModelNavSection"
        },
        {
          "type": "null"
        }
      ],
      "description": "The project's navigation, its root section: every page that isn't\n`unlisted`, in order, as each build's `_ascribe/nav.json` has its\nshare of it. Only in the whole report."
    },
    "notes": {
      "description": "Note types, the values of `@note`'s `type`: the built-in ones, as\nthe project labels them, then the ones it declares.",
      "items": {
        "$ref": "#/$defs/ModelNoteType"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "phrases": {
      "description": "Phrases, in declaration order.",
      "items": {
        "$ref": "#/$defs/ModelPhrase"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "project": {
      "description": "`[project]`: each key, with its value in effect.",
      "items": {
        "$ref": "#/$defs/ModelSetting"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "releases": {
      "description": "`[releases.<target>]`, in declaration order: each target's lines,\nwith their status.",
      "items": {
        "$ref": "#/$defs/ModelReleases"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "sources": {
      "description": "Sources, in declaration order.",
      "items": {
        "$ref": "#/$defs/ModelSource"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "types": {
      "description": "Page types, in declaration order: the implicit `page` type when none\nis declared.",
      "items": {
        "$ref": "#/$defs/ModelType"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "versions": {
      "description": "`[versions]`: each key, with its value in effect.",
      "items": {
        "$ref": "#/$defs/ModelSetting"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "widgets": {
      "description": "Project widgets, in declaration order.",
      "items": {
        "$ref": "#/$defs/ModelWidget"
      },
      "type": [
        "array",
        "null"
      ]
    }
  },
  "required": [
    "schema_version",
    "ascribe_version"
  ],
  "title": "ModelReport",
  "type": "object"
}
```
