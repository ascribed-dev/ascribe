@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/json-page.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/json-page.schema.json`]({repo}/blob/main/schemas/json-page.schema.json):

```json
{
  "$defs": {
    "Annotation": {
      "description": "An `@available`'s spec.",
      "properties": {
        "display": {
          "description": "The spec with the content model's display labels.",
          "type": [
            "string",
            "null"
          ]
        },
        "feature": {
          "description": "The feature key it was written as, when it was one.",
          "type": [
            "string",
            "null"
          ]
        },
        "text": {
          "description": "The spec as shown; for a feature key, the spec it stands for.",
          "type": "string"
        }
      },
      "required": [
        "text",
        "display",
        "feature"
      ],
      "type": "object"
    },
    "Arm": {
      "description": "A surviving arm of a group.",
      "properties": {
        "attributes": {
          "description": "Its attributes.",
          "items": {
            "$ref": "#/$defs/Attribute"
          },
          "type": "array"
        },
        "children": {
          "description": "Its blocks.",
          "items": {
            "$ref": "#/$defs/Block"
          },
          "type": "array"
        },
        "label": {
          "description": "The arm's label: its title as plain text, or the display labels of the\nvalues it names.",
          "type": [
            "string",
            "null"
          ]
        },
        "span": {
          "$ref": "#/$defs/Span",
          "description": "Its span."
        },
        "title": {
          "description": "Its title.",
          "items": {
            "$ref": "#/$defs/Inline"
          },
          "type": [
            "array",
            "null"
          ]
        }
      },
      "required": [
        "label",
        "attributes",
        "title",
        "span",
        "children"
      ],
      "type": "object"
    },
    "Asset": {
      "description": "An asset reference on the page.",
      "properties": {
        "fragment": {
          "description": "The reference's fragment, without its `#`.",
          "type": [
            "string",
            "null"
          ]
        },
        "kind": {
          "description": "`image` or `link`.",
          "type": "string"
        },
        "path": {
          "description": "The copy's path in this output.",
          "type": "string"
        },
        "reference": {
          "description": "How this page refers to the copy.",
          "type": "string"
        },
        "source": {
          "description": "The asset's source path, relative to the content root.",
          "type": "string"
        },
        "span": {
          "$ref": "#/$defs/Span",
          "description": "The reference's span in that file."
        },
        "written_in": {
          "description": "The file the reference is written in, relative to the content root.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "source",
        "path",
        "reference",
        "kind",
        "fragment",
        "written_in",
        "span"
      ],
      "type": "object"
    },
    "Attribute": {
      "description": "A directive's attribute.",
      "properties": {
        "key": {
          "description": "Its key.",
          "type": "string"
        },
        "values": {
          "description": "Its values: several for a set, none for a bare key.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "key",
        "values"
      ],
      "type": "object"
    },
    "Availability": {
      "description": "An availability: the page's own, or a block's effective one.",
      "properties": {
        "display": {
          "description": "The spec with the content model's display labels.",
          "type": "string"
        },
        "enclosing": {
          "anyOf": [
            {
              "$ref": "#/$defs/Availability"
            },
            {
              "type": "null"
            }
          ],
          "description": "The scope it sits in, whose availability it inherits too. The chain\nstops before the page's availability, which is in the page's own\n`availability` and applies to every block."
        },
        "feature": {
          "description": "The feature key it was written as, when it was one.",
          "type": [
            "string",
            "null"
          ]
        },
        "scope": {
          "description": "What it applies to: `page`, `section`, `block`, or `row`.",
          "type": "string"
        },
        "source": {
          "anyOf": [
            {
              "$ref": "#/$defs/SourceRef"
            },
            {
              "type": "null"
            }
          ],
          "description": "Where it's written."
        },
        "targets": {
          "description": "Each target as the availability element gives it, when the project\nlists release lines in `[releases]`.",
          "items": {
            "$ref": "#/$defs/AvailabilityTarget"
          },
          "type": [
            "array",
            "null"
          ]
        },
        "text": {
          "description": "The spec as written (for a feature key, the spec it stands for).",
          "type": "string"
        }
      },
      "required": [
        "text",
        "display",
        "scope",
        "feature",
        "source"
      ],
      "type": "object"
    },
    "AvailabilityTarget": {
      "description": "One target of an availability spec: the element's attributes and its\ntext (element contract §4).",
      "properties": {
        "dimension": {
          "description": "The dimension it's a value of.",
          "type": "string"
        },
        "planned": {
          "description": "`true` when the target's last version is in a planned line; absent\notherwise.",
          "type": "boolean"
        },
        "states": {
          "description": "Its lifecycle states.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "target": {
          "description": "The target's name.",
          "type": "string"
        },
        "text": {
          "description": "The target as text.",
          "type": "string"
        },
        "versions": {
          "description": "The versions it names; absent when it names none.",
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "target",
        "dimension",
        "states",
        "text"
      ],
      "type": "object"
    },
    "Block": {
      "description": "A block: its `type` and the fields of that type, and what every block\nhas.",
      "oneOf": [
        {
          "description": "A heading.",
          "properties": {
            "explicit": {
              "description": "Whether an `@id` gave it its id.",
              "type": "boolean"
            },
            "id": {
              "description": "Its page id.",
              "type": [
                "string",
                "null"
              ]
            },
            "inlines": {
              "description": "Its text.",
              "items": {
                "$ref": "#/$defs/Inline"
              },
              "type": "array"
            },
            "level": {
              "description": "Its level, 1 to 6.",
              "maximum": 255,
              "minimum": 0,
              "type": "integer"
            },
            "source_id": {
              "description": "Its source id.",
              "type": [
                "string",
                "null"
              ]
            },
            "summary": {
              "anyOf": [
                {
                  "$ref": "#/$defs/Summary"
                },
                {
                  "type": "null"
                }
              ],
              "description": "The section's summary (SPEC §4.12), when it has one."
            },
            "text": {
              "description": "Its text, as plain text.",
              "type": [
                "string",
                "null"
              ]
            },
            "type": {
              "const": "heading",
              "type": "string"
            }
          },
          "required": [
            "type",
            "level",
            "id",
            "source_id",
            "explicit",
            "text",
            "inlines"
          ],
          "type": "object"
        },
        {
          "description": "A paragraph.",
          "properties": {
            "inlines": {
              "description": "Its text.",
              "items": {
                "$ref": "#/$defs/Inline"
              },
              "type": "array"
            },
            "type": {
              "const": "paragraph",
              "type": "string"
            }
          },
          "required": [
            "type",
            "inlines"
          ],
          "type": "object"
        },
        {
          "description": "A code block.",
          "properties": {
            "fenced": {
              "description": "Whether it's fenced, rather than indented.",
              "type": "boolean"
            },
            "info": {
              "description": "Its info string.",
              "type": "string"
            },
            "literal": {
              "description": "Its code.",
              "type": "string"
            },
            "type": {
              "const": "code",
              "type": "string"
            }
          },
          "required": [
            "type",
            "fenced",
            "info",
            "literal"
          ],
          "type": "object"
        },
        {
          "description": "A block quote.",
          "properties": {
            "children": {
              "description": "Its blocks.",
              "items": {
                "$ref": "#/$defs/Block"
              },
              "type": "array"
            },
            "type": {
              "const": "block_quote",
              "type": "string"
            }
          },
          "required": [
            "type",
            "children"
          ],
          "type": "object"
        },
        {
          "description": "A list.",
          "properties": {
            "items": {
              "description": "Its items.",
              "items": {
                "$ref": "#/$defs/Item"
              },
              "type": "array"
            },
            "ordered": {
              "description": "Whether it's numbered.",
              "type": "boolean"
            },
            "start": {
              "description": "The first item's number, for a numbered list.",
              "minimum": 0,
              "type": [
                "integer",
                "null"
              ]
            },
            "tight": {
              "description": "Whether it's tight: no blank line between its items.",
              "type": "boolean"
            },
            "type": {
              "const": "list",
              "type": "string"
            }
          },
          "required": [
            "type",
            "ordered",
            "start",
            "tight",
            "items"
          ],
          "type": "object"
        },
        {
          "description": "A block of HTML.",
          "properties": {
            "literal": {
              "description": "The HTML.",
              "type": "string"
            },
            "type": {
              "const": "html",
              "type": "string"
            }
          },
          "required": [
            "type",
            "literal"
          ],
          "type": "object"
        },
        {
          "description": "A thematic break.",
          "properties": {
            "type": {
              "const": "thematic_break",
              "type": "string"
            }
          },
          "required": [
            "type"
          ],
          "type": "object"
        },
        {
          "description": "A table.",
          "properties": {
            "alignments": {
              "description": "Each column's alignment: `none`, `left`, `center`, or `right`.",
              "items": {
                "type": "string"
              },
              "type": "array"
            },
            "rows": {
              "description": "Its rows.",
              "items": {
                "$ref": "#/$defs/Row"
              },
              "type": "array"
            },
            "type": {
              "const": "table",
              "type": "string"
            }
          },
          "required": [
            "type",
            "alignments",
            "rows"
          ],
          "type": "object"
        },
        {
          "description": "A line-form directive that survived: `@available` with its annotation,\nor a `@note`, `@steps`, `@details`, or widget that binds the block\nthat follows it.",
          "properties": {
            "annotation": {
              "anyOf": [
                {
                  "$ref": "#/$defs/Annotation"
                },
                {
                  "type": "null"
                }
              ],
              "description": "An `@available`'s spec."
            },
            "attributes": {
              "description": "Its attributes.",
              "items": {
                "$ref": "#/$defs/Attribute"
              },
              "type": "array"
            },
            "binding": {
              "description": "What it binds: `own`, `heading`, `following_block`, or `unbound`.",
              "type": [
                "string",
                "null"
              ]
            },
            "name": {
              "description": "Its name.",
              "type": "string"
            },
            "primary": {
              "anyOf": [
                {
                  "$ref": "#/$defs/Primary"
                },
                {
                  "type": "null"
                }
              ],
              "description": "Its primary value."
            },
            "title": {
              "description": "Its title.",
              "items": {
                "$ref": "#/$defs/Inline"
              },
              "type": [
                "array",
                "null"
              ]
            },
            "type": {
              "const": "directive",
              "type": "string"
            }
          },
          "required": [
            "type",
            "name",
            "attributes",
            "title",
            "primary",
            "binding",
            "annotation"
          ],
          "type": "object"
        },
        {
          "description": "An `@operation` that names an operation the spec has (SPEC §4.13).",
          "properties": {
            "address": {
              "description": "The operation's address in the published reference.",
              "type": "string"
            },
            "api": {
              "description": "Its API, the name of its `[apis.<name>]`.",
              "type": "string"
            },
            "deprecated": {
              "description": "Whether the spec marks it deprecated.",
              "type": "boolean"
            },
            "method": {
              "description": "The operation's HTTP method.",
              "type": "string"
            },
            "operation_id": {
              "description": "The operation's id, when it has one.",
              "type": [
                "string",
                "null"
              ]
            },
            "path": {
              "description": "The operation's path.",
              "type": "string"
            },
            "summary": {
              "description": "The operation's summary, when it has one.",
              "type": [
                "string",
                "null"
              ]
            },
            "type": {
              "const": "operation",
              "type": "string"
            }
          },
          "required": [
            "type",
            "api",
            "method",
            "path",
            "deprecated",
            "address"
          ],
          "type": "object"
        },
        {
          "description": "A container directive.",
          "properties": {
            "attributes": {
              "description": "Its attributes.",
              "items": {
                "$ref": "#/$defs/Attribute"
              },
              "type": "array"
            },
            "children": {
              "description": "Its blocks.",
              "items": {
                "$ref": "#/$defs/Block"
              },
              "type": "array"
            },
            "name": {
              "description": "Its name.",
              "type": "string"
            },
            "title": {
              "description": "Its title.",
              "items": {
                "$ref": "#/$defs/Inline"
              },
              "type": [
                "array",
                "null"
              ]
            },
            "type": {
              "const": "container",
              "type": "string"
            }
          },
          "required": [
            "type",
            "name",
            "attributes",
            "title",
            "children"
          ],
          "type": "object"
        },
        {
          "description": "A group, such as `@variant`, with its surviving arms.",
          "properties": {
            "arms": {
              "description": "Its surviving arms.",
              "items": {
                "$ref": "#/$defs/Arm"
              },
              "type": "array"
            },
            "name": {
              "description": "Its name.",
              "type": "string"
            },
            "type": {
              "const": "group",
              "type": "string"
            }
          },
          "required": [
            "type",
            "name",
            "arms"
          ],
          "type": "object"
        }
      ],
      "properties": {
        "availability": {
          "anyOf": [
            {
              "$ref": "#/$defs/Availability"
            },
            {
              "type": "null"
            }
          ],
          "description": "Its effective availability below page level: the innermost spec, with\n`enclosing` chaining outward. Absent when nothing but the page's\navailability applies."
        },
        "glossary": {
          "description": "Glossary terms linked in its prose, and its links to a term's heading;\nabsent when it has none.",
          "items": {
            "$ref": "#/$defs/GlossaryTerm"
          },
          "type": "array"
        },
        "links": {
          "description": "The block's own links and images, resolved; absent when it has none.",
          "items": {
            "$ref": "#/$defs/Link"
          },
          "type": "array"
        },
        "snippet": {
          "anyOf": [
            {
              "$ref": "#/$defs/Snippet"
            },
            {
              "type": "null"
            }
          ],
          "description": "For a `code` block a `@snippet` became, where its code comes from;\nabsent on every other block."
        },
        "source": {
          "$ref": "#/$defs/Source",
          "description": "Where it's written."
        },
        "substitutions": {
          "description": "Phrases replaced in its text; absent when it has none.",
          "items": {
            "$ref": "#/$defs/Substitution"
          },
          "type": "array"
        }
      },
      "required": [
        "source"
      ],
      "type": "object"
    },
    "Cell": {
      "description": "A table cell.",
      "properties": {
        "inlines": {
          "description": "Its text.",
          "items": {
            "$ref": "#/$defs/Inline"
          },
          "type": "array"
        }
      },
      "required": [
        "inlines"
      ],
      "type": "object"
    },
    "Environment": {
      "additionalProperties": {
        "type": "string"
      },
      "description": "An environment's names and values, in its order.",
      "type": "object"
    },
    "GlossaryTerm": {
      "description": "A glossary term linked in a block.",
      "properties": {
        "definition": {
          "description": "The term's definition, as plain text.",
          "type": "string"
        },
        "term": {
          "description": "The term.",
          "type": "string"
        },
        "text": {
          "description": "The text that links it.",
          "type": "string"
        },
        "url": {
          "description": "The term's URL.",
          "type": "string"
        }
      },
      "required": [
        "term",
        "text",
        "url",
        "definition"
      ],
      "type": "object"
    },
    "Heading": {
      "description": "A heading of the page.",
      "properties": {
        "explicit": {
          "description": "Whether an `@id` gave it its id.",
          "type": "boolean"
        },
        "id": {
          "description": "Its page id: the id it has on the page.",
          "type": "string"
        },
        "level": {
          "description": "Its level, 1 to 6.",
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "source": {
          "$ref": "#/$defs/Source",
          "description": "Where it's written."
        },
        "source_id": {
          "description": "Its source id: the id it has in its source file.",
          "type": "string"
        },
        "text": {
          "description": "Its text, as plain text.",
          "type": "string"
        }
      },
      "required": [
        "level",
        "text",
        "id",
        "source_id",
        "explicit",
        "source"
      ],
      "type": "object"
    },
    "Inline": {
      "description": "An inline: its `type` and the fields of that type, and its span in the\nblock's file.",
      "oneOf": [
        {
          "description": "Text.",
          "properties": {
            "type": {
              "const": "text",
              "type": "string"
            },
            "value": {
              "description": "The text, decoded.",
              "type": "string"
            }
          },
          "required": [
            "type",
            "value"
          ],
          "type": "object"
        },
        {
          "description": "A code span.",
          "properties": {
            "type": {
              "const": "code",
              "type": "string"
            },
            "value": {
              "description": "Its content.",
              "type": "string"
            }
          },
          "required": [
            "type",
            "value"
          ],
          "type": "object"
        },
        {
          "description": "A soft line break.",
          "properties": {
            "type": {
              "const": "soft_break",
              "type": "string"
            }
          },
          "required": [
            "type"
          ],
          "type": "object"
        },
        {
          "description": "A hard line break.",
          "properties": {
            "type": {
              "const": "hard_break",
              "type": "string"
            }
          },
          "required": [
            "type"
          ],
          "type": "object"
        },
        {
          "description": "Inline HTML.",
          "properties": {
            "type": {
              "const": "html",
              "type": "string"
            },
            "value": {
              "description": "The HTML.",
              "type": "string"
            }
          },
          "required": [
            "type",
            "value"
          ],
          "type": "object"
        },
        {
          "description": "Emphasis.",
          "properties": {
            "children": {
              "description": "What it emphasizes.",
              "items": {
                "$ref": "#/$defs/Inline"
              },
              "type": "array"
            },
            "type": {
              "const": "emphasis",
              "type": "string"
            }
          },
          "required": [
            "type",
            "children"
          ],
          "type": "object"
        },
        {
          "description": "Strong emphasis.",
          "properties": {
            "children": {
              "description": "What it emphasizes.",
              "items": {
                "$ref": "#/$defs/Inline"
              },
              "type": "array"
            },
            "type": {
              "const": "strong",
              "type": "string"
            }
          },
          "required": [
            "type",
            "children"
          ],
          "type": "object"
        },
        {
          "description": "A link.",
          "properties": {
            "children": {
              "description": "Its text.",
              "items": {
                "$ref": "#/$defs/Inline"
              },
              "type": "array"
            },
            "destination": {
              "description": "A page link's route and id, or an external link's destination as\nwritten.",
              "type": "string"
            },
            "target": {
              "anyOf": [
                {
                  "$ref": "#/$defs/Target"
                },
                {
                  "type": "null"
                }
              ],
              "description": "The block's resolved link at the same span, or `null` for a\nglossary link."
            },
            "title": {
              "description": "Its title.",
              "type": [
                "string",
                "null"
              ]
            },
            "type": {
              "const": "link",
              "type": "string"
            }
          },
          "required": [
            "type",
            "destination",
            "title",
            "target",
            "children"
          ],
          "type": "object"
        },
        {
          "description": "An image.",
          "properties": {
            "alt": {
              "description": "Its alternative text, as plain text.",
              "type": "string"
            },
            "attributes": {
              "description": "Its attributes, such as `width`.",
              "items": {
                "$ref": "#/$defs/Attribute"
              },
              "type": "array"
            },
            "destination": {
              "description": "Its destination.",
              "type": "string"
            },
            "target": {
              "anyOf": [
                {
                  "$ref": "#/$defs/Target"
                },
                {
                  "type": "null"
                }
              ],
              "description": "The block's resolved image at the same span."
            },
            "title": {
              "description": "Its title.",
              "type": [
                "string",
                "null"
              ]
            },
            "type": {
              "const": "image",
              "type": "string"
            }
          },
          "required": [
            "type",
            "destination",
            "title",
            "target",
            "alt",
            "attributes"
          ],
          "type": "object"
        },
        {
          "description": "A phrase the resolution left, which it doesn't.",
          "properties": {
            "key": {
              "description": "The phrase's key.",
              "type": "string"
            },
            "type": {
              "const": "phrase",
              "type": "string"
            }
          },
          "required": [
            "type",
            "key"
          ],
          "type": "object"
        }
      ],
      "properties": {
        "span": {
          "$ref": "#/$defs/Span",
          "description": "Its span in the block's file."
        }
      },
      "required": [
        "span"
      ],
      "type": "object"
    },
    "Item": {
      "description": "A list item.",
      "properties": {
        "children": {
          "description": "Its blocks.",
          "items": {
            "$ref": "#/$defs/Block"
          },
          "type": "array"
        }
      },
      "required": [
        "children"
      ],
      "type": "object"
    },
    "Link": {
      "description": "A link or image of a block, resolved.",
      "properties": {
        "destination": {
          "description": "Its destination, as written.",
          "type": "string"
        },
        "kind": {
          "description": "`link` or `image`.",
          "type": "string"
        },
        "span": {
          "$ref": "#/$defs/Span",
          "description": "Its span."
        },
        "target": {
          "$ref": "#/$defs/Target",
          "description": "What it resolved to."
        }
      },
      "required": [
        "span",
        "kind",
        "destination",
        "target"
      ],
      "type": "object"
    },
    "Observation": {
      "description": "An observation current for an example.",
      "properties": {
        "environment": {
          "$ref": "#/$defs/Environment",
          "description": "Its environment as it has it, in its order: `os`, `arch`, then each\nentry's kept line, such as `\"node\": \"v24.9.0\"`."
        },
        "name": {
          "description": "Its name: `default`, or the name `ascribe observe --name` gave it.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "environment"
      ],
      "type": "object"
    },
    "Primary": {
      "description": "A directive's primary value.",
      "oneOf": [
        {
          "description": "Text.",
          "properties": {
            "inlines": {
              "description": "The text.",
              "items": {
                "$ref": "#/$defs/Inline"
              },
              "type": "array"
            },
            "kind": {
              "const": "text",
              "type": "string"
            }
          },
          "required": [
            "kind",
            "inlines"
          ],
          "type": "object"
        },
        {
          "description": "An identifier.",
          "properties": {
            "kind": {
              "const": "identifier",
              "type": "string"
            },
            "text": {
              "description": "The identifier.",
              "type": "string"
            }
          },
          "required": [
            "kind",
            "text"
          ],
          "type": "object"
        },
        {
          "description": "The rest of the line, as written.",
          "properties": {
            "kind": {
              "const": "line",
              "type": "string"
            },
            "text": {
              "description": "The line.",
              "type": "string"
            }
          },
          "required": [
            "kind",
            "text"
          ],
          "type": "object"
        }
      ]
    },
    "Row": {
      "description": "A table row.",
      "properties": {
        "availability": {
          "anyOf": [
            {
              "$ref": "#/$defs/Availability"
            },
            {
              "type": "null"
            }
          ],
          "description": "The row's own availability, in its table's (SPEC §4.4)."
        },
        "cells": {
          "description": "Its cells.",
          "items": {
            "$ref": "#/$defs/Cell"
          },
          "type": "array"
        },
        "header": {
          "description": "Whether it's the header row.",
          "type": "boolean"
        }
      },
      "required": [
        "header",
        "cells"
      ],
      "type": "object"
    },
    "Snippet": {
      "description": "Where a code block that a `@snippet` became takes its code from.",
      "properties": {
        "address": {
          "description": "The address, as written.",
          "type": "string"
        },
        "lines": {
          "description": "The lines of the code file it covers, first and last.",
          "items": {
            "minimum": 0,
            "type": "integer"
          },
          "maxItems": 2,
          "minItems": 2,
          "type": [
            "array",
            "null"
          ]
        },
        "path": {
          "description": "The code file, from the project root.",
          "type": "string"
        },
        "tested": {
          "anyOf": [
            {
              "$ref": "#/$defs/Tested"
            },
            {
              "type": "null"
            }
          ],
          "description": "Whether the file was tested, when it's a tested example\n(decision #458c); absent on any other file."
        }
      },
      "required": [
        "address",
        "path",
        "lines"
      ],
      "type": "object"
    },
    "Source": {
      "description": "Where a node is written.",
      "properties": {
        "file": {
          "description": "The file the node is written in, relative to the content root.",
          "type": [
            "string",
            "null"
          ]
        },
        "lines": {
          "description": "First and last line, counted from 1.",
          "items": {
            "minimum": 0,
            "type": "integer"
          },
          "maxItems": 2,
          "minItems": 2,
          "type": [
            "array",
            "null"
          ]
        },
        "span": {
          "$ref": "#/$defs/Span",
          "description": "Its span in that file."
        },
        "via": {
          "description": "The includes it came through, outermost first; absent for a page's\nown nodes.",
          "items": {
            "$ref": "#/$defs/Via"
          },
          "type": "array"
        }
      },
      "required": [
        "file",
        "span",
        "lines"
      ],
      "type": "object"
    },
    "SourceRef": {
      "description": "A place in a source file.",
      "properties": {
        "file": {
          "description": "The file, relative to the content root.",
          "type": [
            "string",
            "null"
          ]
        },
        "span": {
          "$ref": "#/$defs/Span",
          "description": "The span in that file."
        }
      },
      "required": [
        "file",
        "span"
      ],
      "type": "object"
    },
    "Span": {
      "description": "A byte range in a source file, `[start, end]`, counted in UTF-8 bytes from\nthe start of the file (frontmatter included).",
      "maxItems": 2,
      "minItems": 2,
      "prefixItems": [
        {
          "minimum": 0,
          "type": "integer"
        },
        {
          "minimum": 0,
          "type": "integer"
        }
      ],
      "type": "array"
    },
    "Substitution": {
      "description": "A phrase replaced in a block's text.",
      "properties": {
        "key": {
          "description": "The phrase's key.",
          "type": "string"
        },
        "span": {
          "$ref": "#/$defs/Span",
          "description": "Its span over the source."
        },
        "value": {
          "description": "The text that replaced it.",
          "type": "string"
        }
      },
      "required": [
        "span",
        "key",
        "value"
      ],
      "type": "object"
    },
    "Summary": {
      "description": "A section's summary: its text, and the form that gives it, `text` for\n`@summary: text` and `paragraph` for `@summary` above a paragraph.",
      "properties": {
        "form": {
          "description": "`text` or `paragraph`.",
          "type": "string"
        },
        "inlines": {
          "description": "The summary's text.",
          "items": {
            "$ref": "#/$defs/Inline"
          },
          "type": "array"
        },
        "source": {
          "$ref": "#/$defs/Source",
          "description": "The `@summary` line for the text form, and the paragraph for the\nother."
        }
      },
      "required": [
        "form",
        "inlines",
        "source"
      ],
      "type": "object"
    },
    "Target": {
      "description": "What a link or image resolved to.",
      "oneOf": [
        {
          "description": "An external link.",
          "properties": {
            "type": {
              "const": "external",
              "type": "string"
            }
          },
          "required": [
            "type"
          ],
          "type": "object"
        },
        {
          "description": "A page the build publishes.",
          "properties": {
            "id": {
              "description": "The page id it names, if any.",
              "type": [
                "string",
                "null"
              ]
            },
            "line": {
              "description": "The line a link out of a line leads to; absent for any other\nlink.",
              "type": [
                "string",
                "null"
              ]
            },
            "page": {
              "description": "The page's source path.",
              "type": "string"
            },
            "text_filled": {
              "description": "Whether the link had no text and took the page's title.",
              "type": "boolean"
            },
            "type": {
              "const": "page",
              "type": "string"
            },
            "url": {
              "description": "Its URL.",
              "type": "string"
            }
          },
          "required": [
            "type",
            "page",
            "id",
            "url",
            "text_filled"
          ],
          "type": "object"
        },
        {
          "description": "An asset.",
          "properties": {
            "fragment": {
              "description": "The fragment, without its `#`.",
              "type": [
                "string",
                "null"
              ]
            },
            "path": {
              "description": "The copy's path in this output.",
              "type": "string"
            },
            "reference": {
              "description": "How this page refers to the copy.",
              "type": "string"
            },
            "source": {
              "description": "The asset's source path.",
              "type": "string"
            },
            "type": {
              "const": "asset",
              "type": "string"
            },
            "url": {
              "description": "The copy's URL.",
              "type": [
                "string",
                "null"
              ]
            }
          },
          "required": [
            "type",
            "source",
            "path",
            "reference",
            "url",
            "fragment"
          ],
          "type": "object"
        },
        {
          "description": "An operation of an API spec.",
          "properties": {
            "address": {
              "description": "The operation's address in the published reference.",
              "type": "string"
            },
            "api": {
              "description": "Its API, the name of its `[apis.<name>]`.",
              "type": "string"
            },
            "method": {
              "description": "The operation's HTTP method, for an operation with no id.",
              "type": [
                "string",
                "null"
              ]
            },
            "operation_id": {
              "description": "The operation's id, when it has one.",
              "type": [
                "string",
                "null"
              ]
            },
            "path": {
              "description": "The operation's path, for an operation with no id.",
              "type": [
                "string",
                "null"
              ]
            },
            "type": {
              "const": "operation",
              "type": "string"
            }
          },
          "required": [
            "type",
            "api",
            "address"
          ],
          "type": "object"
        },
        {
          "description": "The build couldn't resolve it; the checks report why.",
          "properties": {
            "type": {
              "const": "unresolved",
              "type": "string"
            }
          },
          "required": [
            "type"
          ],
          "type": "object"
        }
      ]
    },
    "Tested": {
      "description": "A tested example's status, and the observations current for it.",
      "properties": {
        "observations": {
          "description": "The observations current for it, in name order: empty unless it's\n`current`.",
          "items": {
            "$ref": "#/$defs/Observation"
          },
          "type": "array"
        },
        "status": {
          "description": "`current`, `stale`, or `unobserved`.",
          "type": "string"
        }
      },
      "required": [
        "status",
        "observations"
      ],
      "type": "object"
    },
    "Via": {
      "description": "An include a node came through.",
      "properties": {
        "file": {
          "description": "The file the `@include` is written in.",
          "type": [
            "string",
            "null"
          ]
        },
        "line": {
          "description": "The `@include`'s line, counted from 1.",
          "minimum": 0,
          "type": [
            "integer",
            "null"
          ]
        },
        "span": {
          "$ref": "#/$defs/Span",
          "description": "The `@include`'s span in that file."
        }
      },
      "required": [
        "file",
        "span",
        "line"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "A page of the JSON output: the page as the build resolved it, with every\nnode's place in its source (the JSON output contract). Spans are UTF-8\nbyte offsets, `[start, end]`, from the start of the file, frontmatter\nincluded; lines count from 1.",
  "properties": {
    "assets": {
      "description": "Every asset reference on the page that survived the build.",
      "items": {
        "$ref": "#/$defs/Asset"
      },
      "type": "array"
    },
    "availability": {
      "anyOf": [
        {
          "$ref": "#/$defs/Availability"
        },
        {
          "type": "null"
        }
      ],
      "description": "The page's own availability (frontmatter `available`)."
    },
    "blocks": {
      "description": "The page's blocks.",
      "items": {
        "$ref": "#/$defs/Block"
      },
      "type": "array"
    },
    "build": {
      "description": "The build's name.",
      "type": "string"
    },
    "format": {
      "description": "`\"ascribe-page\"`.",
      "type": "string"
    },
    "formatted": {
      "additionalProperties": true,
      "description": "Each field that sets `inline = \"code\"`, as a list of text and code\npieces (`{type, value}`); its plain text is in `frontmatter`.",
      "type": "object"
    },
    "frontmatter": {
      "description": "The frontmatter, with phrases substituted in the fields that ask for\nit, and each field that sets `inline = \"code\"` as plain text."
    },
    "headings": {
      "description": "Every heading that survived the build, in order.",
      "items": {
        "$ref": "#/$defs/Heading"
      },
      "type": "array"
    },
    "path": {
      "description": "The page's source path, relative to the content root.",
      "type": "string"
    },
    "route": {
      "description": "The page's root-relative URL, as the router made it.",
      "type": "string"
    },
    "schema_version": {
      "description": "The format's version, `2`. It changes only when a field is removed or\nchanges meaning.",
      "minimum": 0,
      "type": "integer"
    },
    "site": {
      "description": "`[consumer] site`: the origin to put before a root-relative URL.",
      "type": [
        "string",
        "null"
      ]
    },
    "title": {
      "description": "The frontmatter `title`, phrases substituted, as plain text.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "schema_version",
    "format",
    "build",
    "path",
    "route",
    "site",
    "title",
    "frontmatter",
    "availability",
    "headings",
    "assets",
    "blocks"
  ],
  "title": "JsonPage",
  "type": "object"
}
```
