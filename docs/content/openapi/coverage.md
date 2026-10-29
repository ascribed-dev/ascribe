---
type: how-to
title: Find the operations no guide links to
description: "See which operations of your OpenAPI spec no page links to, with `ascribe check`'s `api-operation-unreferenced` and the inventory's count of operations linked for each API."
---

A spec grows faster than the guides that explain it. `ascribe check` says which operations no page links to, and `ascribe report inventory` counts them, so a new endpoint that ships without a guide shows up in the next check.

## What counts

An operation is linked when a link in any page or fragment names it, by its `operationId` or [by its method and path](no-id.md), as [Link to your API reference](index.md) describes, in the text or as an `@operation`'s link. Only links count: a word in prose that matches an id, such as `` `getUser` ``, isn't a link to it, and a link to an operation the spec doesn't have covers no other. A link counts for every build, whichever builds publish its page.

Each operation no link names is advice, `api-operation-unreferenced`, at the API's `spec` in `ascribe.toml`:

```text
ascribe.toml:7: [ASC282] no page links to `deleteInvoice` (DELETE /invoices/{id}) in api/openapi.yaml
```

An operation the spec marks `deprecated: true` isn't reported, since it's on its way out. An operation with no id is named by its method and path.

For an API whose reference is enough on its own, turn the check off in [`[checks]`](../content-model/checks.md#22-checks):

```toml
[checks]
api-operation-unreferenced = "off"
```

## The inventory's counts

`ascribe report inventory` counts, for each API whose spec could be read, its operations, how many of them a link names, and how many a checked example claims as its request or response:

```text
  APIs: public: 24 operations, 19 linked, 6 with a checked example.
```

Deprecated operations are counted with the rest. In the report's JSON, the inventory has `apis`, one object for each API with `api`, `operations`, `linked`, and `examples`; it's absent when the project declares no API.
