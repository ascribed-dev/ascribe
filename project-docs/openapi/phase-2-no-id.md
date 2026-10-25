# Phase 2: Operations with no id

Part of [OpenAPI](README.md). Needs phase 1.

## Goal

A writer links an operation the spec gives no id, by its method and path.

```markdown
Delete the user with [Delete a user](../api/openapi.yaml#delete/users/{id}).
```

The link is checked against the spec's operations, and written for Redoc as `/docs/api/#paths/~1users~1{id}/delete`, which lands in Redoc 3. Under a template, the same link is an error saying a link by method and path needs `renderer = "redoc"`. A link by method and path to an operation that has an id is advice, with a fix that writes the id.

## Context

- README decisions 2 and 4; the proposal's Settled 3; phase 1's resolution and addresses.
- The spike's Redoc 3 address for an operation with no id, and its landing check.
- The spikes' finding that .NET's generators write no ids by default.

## Design

- **The form:** a fragment that starts with a lowercase HTTP method and `/`, then the path as the spec writes it, percent-decoded. A fragment that matches both an id and a method and path is the id, since an id is what a writer chose; the docs say so.
- **Redoc's address:** `<at>#paths/<pointer>/<method>`, the path as a JSON pointer (`~1` for `/`, `~0` for `~`), the form Redoc 3 accepts without a tag. Redoc 2 needs its own slug of the tag there; README decision 2 says nothing reports it.
- **Under a template:** `link-operation-form-unsupported`, an error: no template makes the renderers' addresses for an operation with no id.
- **An operation that has an id:** `link-operation-by-path`, advice, with a safe fix that writes `#<operationId>`.

## Tasks

1. The form in the resolver, with tests: a method and path, one with a parameter, one whose path holds `~`, the id winning when both match.
2. Redoc's address for it, with an emit test, and the end-to-end test following a link to the thread's operation with no id.
3. `link-operation-form-unsupported` and `link-operation-by-path` with its fix, each with its conformance case.
4. SPEC §5.2's second form, and its decision file.
5. The docs: the section's page on linking an operation with no id, that it needs Redoc 3, and why setting ids is better; `changes/language/openapi-no-id.md`.

## Out of scope

Everything for phases 3 to 7.

## Acceptance criteria

- The goal's link, in a conformance case and an emit test.
- The end-to-end test lands on the operation with no id in Redoc 3.

## Verify

```sh
cargo test -p ascribe-resolve openapi
cargo test -p ascribe-conformance --test conformance
pnpm --filter @ascribed/example-astro-site test:e2e
```

## Commits

1. "Link an operation with no id by its method and path"
