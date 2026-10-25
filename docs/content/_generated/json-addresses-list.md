@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/addresses-list.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
[`schemas/addresses-list.schema.json`]({repo}/blob/main/schemas/addresses-list.schema.json):

```json
{
  "$defs": {
    "Gone": {
      "description": "An address retired on purpose.",
      "properties": {
        "entry": {
          "description": "The `ascribe.addresses` key it comes from.",
          "type": "string"
        },
        "path": {
          "description": "The address, from the host's root.",
          "type": "string"
        },
        "reason": {
          "description": "Why it was retired.",
          "type": "string"
        }
      },
      "required": [
        "path",
        "reason",
        "entry"
      ],
      "type": "object"
    },
    "Redirect": {
      "description": "An address that leads somewhere else now.",
      "properties": {
        "entry": {
          "description": "The `ascribe.addresses` key the pair comes from.",
          "type": "string"
        },
        "from": {
          "description": "The old address, from the host's root, in the published case and\nslash form.",
          "type": "string"
        },
        "status": {
          "description": "The HTTP status: 301.",
          "maximum": 65535,
          "minimum": 0,
          "type": "integer"
        },
        "to": {
          "description": "Where it leads now, from the host's root.",
          "type": "string"
        }
      },
      "required": [
        "from",
        "to",
        "status",
        "entry"
      ],
      "type": "object"
    },
    "Redirects": {
      "description": "Where the addresses a build used to publish lead now, and which were\nretired: `_ascribe/redirects.json`.",
      "properties": {
        "build": {
          "description": "The build's name.",
          "type": "string"
        },
        "gone": {
          "description": "Each retired address, sorted by `path`.",
          "items": {
            "$ref": "#/$defs/Gone"
          },
          "type": "array"
        },
        "redirects": {
          "description": "Each pair, sorted by `from`.",
          "items": {
            "$ref": "#/$defs/Redirect"
          },
          "type": "array"
        },
        "version": {
          "description": "The format's version.",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "version",
        "build",
        "redirects",
        "gone"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `ascribe addresses list --format json` writes.",
  "properties": {
    "ascribe_version": {
      "description": "The version of Ascribe that wrote it.",
      "type": "string"
    },
    "builds": {
      "description": "Each build's redirects and retired addresses, as its\n`_ascribe/redirects.json` has them, in the content model's order.",
      "items": {
        "$ref": "#/$defs/Redirects"
      },
      "type": "array"
    },
    "has_file": {
      "description": "Whether the project has `ascribe.addresses`. Without it, every\nbuild's lists are empty.",
      "type": "boolean"
    },
    "schema_version": {
      "description": "The version of this schema. It changes only when a field is removed\nor changes meaning.",
      "minimum": 0,
      "type": "integer"
    }
  },
  "required": [
    "schema_version",
    "ascribe_version",
    "has_file",
    "builds"
  ],
  "title": "AddressesListReport",
  "type": "object"
}
```
