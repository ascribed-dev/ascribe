---
type: how-to
title: Link to operations in the editor
description: "Complete an OpenAPI operation after a link's #, see its method, path, and summary on hover, open it in the spec, and fix a link whose operation was renamed, in VS Code or any editor with the language server."
available: ascribe 0.4.4
---

The editor knows the operations of each spec `[apis]` declares, as it knows a page's headings: it completes them in a link, shows what a link names, opens it in the spec, and fixes a link when the spec renames its operation. This is the language server's, so it works in VS Code and in any editor that runs `ascribe lsp`. All of it applies to the link of an [`@operation`](operation.md#operation-editor) too.

## Complete an operation

Type a link to the spec and `#`, and the editor offers its operations:

```markdown
Fetch the user with [Get a user](../../api/openapi.yaml#us
```

Each operation is offered by its `operationId`, or by its method and path, `delete/users/{id}`, for one with none, with its method and path, `GET /users/{id}`, beside it, and its summary. What you type is matched against all of these, so `#user`, `#get`, `#/users`, and `#Get a` each find `getUser`. A deprecated operation is shown struck through. The completion writes the id percent-encoded, as a link needs one with a space.

## See an operation, and open it

Hover a link to an operation for its method and path, its summary, its first tag, and whether the spec marks it deprecated. A link to an operation the spec doesn't have says so.

Go to definition (F12 in VS Code) opens the spec at the operation's method. A spec outside the project's folder opens too, as long as it's in the project's repository.

## When the spec changes

The editor reads the spec again when it changes on disk: when you regenerate it, pull, or switch branches. A spec outside the folder the editor opened is watched as well. The links to its operations are checked again, so a link to an operation the spec lost shows `link-operation-missing` as soon as the file is written.

## Fix a link to a renamed operation

A regenerated spec can rename an operation: the code's method was renamed, or the generator's ids changed. When the spec loses `getUser` and the same method and path, `GET /users/{id}`, now have `fetchUser`, the link's `link-operation-missing` offers `fetchUser` first, as a safe fix, so **Apply the safe fixes in this page** applies it.

The editor knows where `getUser` was from the spec as it read it before the change, or, after the change, from the review's base: the branch you compare with in [Review](../editor/review.md) has the spec as it was. The fix is safe only when exactly one operation is at that method and path, with an id that's new there. Otherwise the closest operations are offered, as for any missing one, for you to choose.

`ascribe check` has no earlier spec to compare with, so on the command line the operation in its place is offered as the closest, without being safe.

## A deprecated operation

A link to an operation the spec marks `deprecated` is advice, `link-operation-deprecated`. The operation still works, and the reference marks it, but a guide that teaches it sends readers to something going away: link to the operation that replaces it, if the spec names one, or say in the text that it's deprecated.

## For agents

`ascribe link` checks a link to an operation as it checks a link to a page, and answers with the operation: its API, its id, or its method and path, its summary, whether it's deprecated, and its address in the reference. An agent with no shell asks the same with the MCP tool `ascribe_link`. See [`ascribe link`](../command/questions.md#cli-link).

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/link-operation.txt

With `--format json`, `kind` is `operation`, and `operation` has its `api`, `operation_id` (null for an operation with none), `method`, `path`, `summary`, `tag`, `deprecated`, and `address`, as the editor's build writes the link.
