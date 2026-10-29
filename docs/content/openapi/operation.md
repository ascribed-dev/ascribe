---
type: how-to
title: Show an operation in a page
description: "Put an OpenAPI operation's method, path, and summary in a guide with `@operation`, linked to the operation in your API reference and checked against the spec."
---

A guide often names the call it's about before it explains it. `@operation` puts that call in the page, its method, path, and summary as the spec has them, linked to the operation in your published reference, so the line can't drift from the spec:

```markdown
## Fetch a user

@operation: ../api/openapi.yaml#getUser

Pass the user's id in the path.
```

The site shows `GET /users/{id}: Get a user`, linked to `getUser` in the reference. It needs the spec declared in `[apis]`: see [Link to your API reference](index.md).

## Name the operation

`@operation` takes one link to an operation, written as a link's destination names one: the spec's path from the page, `#`, and the operation's `operationId`. An operation with no id is named by its method in lowercase and its path, `../api/openapi.yaml#delete/users/{id}`, which needs Redoc 3: see [Link to an operation with no id](no-id.md).

It's checked as the link would be:

- An operation the spec doesn't have is an error, `link-operation-missing`, with the closest operations as its fixes.
- An operation named by its method and path when it has an id is advice, `link-operation-by-path`, and its fix writes the id.
- An operation the spec marks `deprecated` is advice, `link-operation-deprecated`, as a link to it is: a guide is usually better showing the call that replaces it.
- A link that isn't to an operation, such as a page, the spec with no `#`, or a URL, is an error, `operation-link-not-operation`. To link to a page, write a link in the text.

## What it shows

The operation's method, in capitals, and its path, then `: ` and its summary when it has one, as one line linked to the operation's address. When the spec marks the operation `deprecated`, the line ends with "(deprecated)".

| Output | What it writes |
|---|---|
| Site | An [`<ascribe-operation>` element](../astro/elements.md), with the operation's API, method, path, id, and deprecation as its attributes, holding the link |
| Plain | The same line, as a paragraph, its link absolute when `[consumer] site` is set |
| JSON | A block of type `operation`, with the API, `operationId`, method, path, summary, `deprecated`, and the address |

It never shows the operation's parameters, request, or responses: those are the reference's. A page that wants to show a request links to the reference, or shows an example of its own.

`@operation` is a block, so a directive that applies to the next block applies to it: `@available` directly above it says where the operation applies.

## In the editor
@id: operation-editor

The editor treats `@operation`'s link as it treats a link to an operation: after the spec's `#`, it offers the spec's operations; hovering the link shows the operation's method, path, summary, tag, and deprecation; and go to definition opens the spec at the operation. When the spec changes on disk, the line is checked again, so an operation the spec renamed is offered the one now in its place as a safe fix. See [Link to operations in the editor](editor.md).
