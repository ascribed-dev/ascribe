---
type: how-to
title: Check a JSON example against its schema
description: "Mark a JSON example in a guide as an operation's request or response, and have ascribe check it against that body's schema in the OpenAPI spec, each disagreement at its place in the block."
available: ascribe planned
---

A guide shows the JSON a request sends and the JSON a response returns, and when the API changes, nothing tells you the guide's JSON is wrong. Mark the block with the operation it's an example of, and `ascribe check` checks the JSON against that body's schema in the spec you declared in `[apis]` (see [Link to your API reference](index.md)).

## Mark an example

Add `request="<link>"` or `response="<link> <status>"` to a `json` block's info string, where `<link>` is a link to the operation, as you'd write it in the page:

````markdown
```json request="../api/openapi.yaml#createUser"
{ "name": "Ada", "email": "ada@example.com" }
```

```json response="../api/openapi.yaml#createUser 201"
{ "id": "u_1", "name": "Ada", "email": "ada@example.com" }
```
````

- **The link** is a link to an operation, by its `operationId` or [by its method and path](no-id.md), and it's checked as one: an operation the spec doesn't have is `link-operation-missing`, with the closest operations as its fixes.
- **A response's status** is written as the spec's key is: `201`, `2XX`, or `default`.
- **A media type** goes after the status, or after the link for a request, when the body has more than one JSON media type: `response="../api/openapi.yaml#getUser 404 application/problem+json"`. Otherwise the body's one JSON media type is used, `application/json` or any `…+json`.

The marker sits beside `title` and `tested`, and the block stays a plain fenced block, shown as it is by every Markdown viewer. The published page doesn't change.

A marker on a block that isn't a `json` block, or whose code isn't JSON, is `example-not-json`. One that names a body the operation doesn't have, a request body it doesn't take or a status it has no response for, is `example-body-missing`.

## What's checked

The block's JSON is checked against the schema of the body the marker names, with the schema's `$ref`s into the same file followed. Each disagreement is an error, `example-schema-mismatch`, at its place in the block:

```console
$ ascribe check
error[example-schema-mismatch]: `name` is a number; the request body of `createUser` says it's a string
  --> guides/users.md:12:11
```

- A value of the wrong type, out of range, or not one the schema lists is reported at the value.
- A property the schema doesn't allow is reported at its key.
- A required property that's missing is reported at its object's opening brace.

The editor shows them as you type.

The schema is read as the spec means it. OpenAPI 3.1's and 3.2's schemas are JSON Schema 2020-12. OpenAPI 3.0's and Swagger 2.0's are read as JSON Schema draft 4, with `nullable: true` (and Swagger's `x-nullable`) allowing `null`. A `readOnly` property, which the server writes, isn't required in a request, and a `writeOnly` one, such as a password, isn't required in a response. Formats such as `date-time`, `email`, `uri`, and `uuid` are checked; OpenAPI's own, such as `int64` and `password`, aren't.

## What isn't checked

Some schemas can't be checked whole, and each is advice, `example-schema-unsupported`, at the marker:

- **A pattern the validator can't read**, such as one with a lookbehind, is left out, and the rest of the schema is still checked.
- **A `$ref` into another file** isn't followed, and the example isn't checked. Declare the spec's bundled file, the one your renderer is given, so every schema is in it.

Only JSON is checked: an example in YAML, or in JSON with comments, isn't, and takes no marker.
