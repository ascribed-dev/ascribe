---
type: how-to
title: Link to your API reference
description: "Declare your OpenAPI spec in `ascribe.toml`, link a guide to an operation by its `operationId`, and have the link checked against the spec and written as the operation's address in Redoc or another renderer."
---

A guide that says "call `getUser`" should link to `getUser` in the API reference, and the link should break the build when the spec loses it. Declare the spec in `ascribe.toml`, and a link to it with an `operationId` as its fragment is a link to that operation: `ascribe check` checks the spec has it, the editor completes it after `#` and shows it on hover ([in the editor](editor.md)), and the build writes the operation's address in your published reference.

## Declare the spec

Give the spec a name in `[apis]`, its file, and where its reference is published:

```toml
[apis.public]
spec = "api/openapi.yaml"
reference = { renderer = "redoc", at = "/api/" }
```

`spec` is a path from the project root, to a YAML or JSON file of Swagger 2.0 or OpenAPI 3.0 to 3.2. A spec generated from code often lives beside the docs rather than in them, so it may be outside the project's folder, `../services/api/openapi.yaml`, as long as it's in the same git repository. A spec that isn't there, or isn't YAML or JSON, is an error at `spec` (`api-spec-unreadable`).

Ascribe reads each operation's `operationId`, method, path, summary, first tag, and whether it's `deprecated` from `paths`, and its request and response schemas, which a [marked JSON example](examples.md) is checked against, and nothing else: it doesn't validate the spec, so one that a strict validator rejects still works.

## Link to an operation

Link to the spec's file, with the operation's `operationId` as the fragment:

```markdown
Fetch the user with [Get a user](../../api/openapi.yaml#getUser).
```

The path is relative to the page, as any link's is. An id with `/` or a space is written percent-encoded, `#activity%2Flist%20events`, and matched decoded.

- **It's checked.** An id the spec doesn't have is an error at the link, `link-operation-missing`, with the closest ids the spec has as its fixes, or the operation it was [renamed to](#a-renamed-id). A link to an operation the spec marks deprecated is advice, `link-operation-deprecated`.
- **It's written as the reference's address.** For Redoc, `getUser`'s is `/api/#operation/getUser`, under `[consumer] base-path`: `/docs/api/#operation/getUser` with a base path of `/docs/`. The plain output writes it absolute when `[consumer] site` is set, and the JSON output gives the link an `operation` target with the address.
- **A link to the spec with no fragment** is a link to the file, as to any file. When the spec is outside the project's folder, that's an error (`link-spec-outside-project`): only a link to one of its operations is allowed.

## A renamed id

A generator or a change to the code can rename an `operationId`, which breaks every link to it. Spelling alone would offer the wrong operation: when `createWidget` becomes `widgets_create`, the closest id by spelling can be `getWidget`. So `ascribe check` reads the spec as it was at the base `ascribe diff` compares with, the merge base of `HEAD` and the default branch, or `--base`. When the missing id named an operation there, the operation the spec now has at that method and path is offered first, as a safe fix:

```text
`api/openapi.yaml` has no operation `createWidget`; `createWidget` is now `widgets_create` (POST /widgets)
```

`ascribe check --fix` applies it, and `ascribe link`, and the MCP server's `ascribe_link`, answer with it first. The base is only read when a link names an operation the spec doesn't have.

It's safe only when exactly one operation is at that method and path, with an id that's new there. When the path moved too, nothing is there, or there's no history to read, as in a shallow clone, the closest ids are offered as for any missing one, and nothing more is reported. The editor does the same: see [Fix a link to a renamed operation](editor.md#fix-a-link-to-a-renamed-operation).

## When the spec changes

A spec edit can change published pages: an [`@operation`](operation.md) shows the operation's method, path, summary, and deprecation, and a link writes its address. [`ascribe diff`](../review/diff.md#cli-diff) reads the spec at the base too, and lists each page whose output changed through it, naming the operation as the page names it: `guides/api.md: 1 changed (through api/openapi.yaml#createWidget)`. Its JSON, its agent prompt, and review in the editor and the site name it the same way. A change no page shows, such as to a schema or a description, lists nothing.

## Publish the reference

Ascribe doesn't draw the reference: a renderer does, on a page of your site, and the link goes to the address that renderer gives the operation.

**Redoc.** `{ renderer = "redoc", at = "<the page's address>" }`, where `at` is relative to the base path, or absolute for a reference on another site. An operation's address is `<at>#operation/<operationId>`, which Redoc 2 and Redoc 3 both open, scrolled to the operation. In an Astro site, load Redoc from a pinned package rather than a CDN, on a page of its own:

```astro
---
import redoc from "redoc/bundles/redoc.standalone.js?url";
import spec from "../../api/openapi.yaml?url&no-inline";
---

<html lang="en">
  <body>
    <redoc spec-url={spec} disable-telemetry="true"></redoc>
    <script is:inline type="module" src={redoc}></script>
  </body>
</html>
```

Redoc 3 sends usage data to Redocly unless `disable-telemetry` is `"true"`.

**Another renderer.** Write `reference` as an address pattern: an operation's address, with `{operationId}` where the id goes and `{tag}` for its first tag. Each is filled in percent-encoded.

| Renderer | `reference` |
|---|---|
| Stoplight Elements, with `router="hash"` | `"/api/#/operations/{operationId}"` |
| Swagger UI, with deep linking | `"/api/#/{tag}/{operationId}"` |

A renderer whose address is a slug of the id, such as starlight-openapi's or Docusaurus OpenAPI docs', or that addresses an operation by its method and path, such as Scalar, can't be written as an address pattern, and isn't supported.

## Give each operation an id

A link names an operation by its `operationId`. An operation with none can be linked to by its method and path, `#delete/users/{id}`, with Redoc 3 only: see [Link to an operation with no id](no-id.md). Some generators write none by default (.NET's own and Swashbuckle), and some write ids that can change meaning between builds: springdoc gives a second method of the same name `findAll_1`, which one depending on scan order. Set each operation's `operationId` in the code it's generated from, so the id a guide links to stays the operation's.
