---
type: how-to
title: Link to an operation with no id
description: "Link a guide to an OpenAPI operation the spec gives no operationId, by its method and path, which needs Redoc 3, and why setting ids is better."
available: ascribe 0.4.4
---

A link names an operation by its `operationId`, and some specs leave operations without one: .NET's generators write none by default. To link to an operation with no id, name it by its method and path instead.

## Link by method and path

Write the method in lowercase, then the path as the spec writes it, parameters and all, as the link's fragment:

```markdown
Delete the user with [Delete a user](../api/openapi.yaml#delete/users/{id}).
```

The link is checked against the spec as a link by id is: a method and path the spec doesn't have is `link-operation-missing`, with the closest operations as its fixes. The path is matched exactly, so `#delete/users/{userId}` doesn't name `/users/{id}`.

A fragment that's an operation's id names that operation, even when it's another's method and path too: an id is what a writer chose.

## It needs Redoc 3

Redoc 3 has an address for an operation with no id, by its path: `/api/#paths/~1users~1%7Bid%7D/delete`, under `[consumer] base-path`, the path as a JSON pointer, with `~1` for each `/`. Ascribe writes that address. Redoc 2 addresses the same operation under a slug of its tag, and Ascribe can't tell which Redoc your site runs, so nothing reports a link that Redoc 2 won't open: publish the reference with Redoc 3.

An address pattern has no placeholder for an operation with no id, so with one for `reference`, a link by method and path to one is an error, `link-operation-form-unsupported`. The closest operations a missing one is offered are then the ones with ids alone.

## Set ids instead

A link by method and path is a way to link before the spec has ids, not a better one. An id stays the operation's when its path changes, and every renderer addresses an operation by its id, so give each operation an `operationId` in the code it's generated from, and link to that. A link by method and path to an operation that has an id is advice, `link-operation-by-path`, and its fix writes the id: `ascribe check --fix` changes them all.
