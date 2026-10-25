# Phase 6: Coverage

Part of [OpenAPI](README.md). Needs phase 2.

## Goal

`ascribe check` and the inventory say which operations no page links to.

```console
$ ascribe check
advice[api-operation-unreferenced]: no page links to `deleteInvoice` (DELETE /invoices/{id}) in api/openapi.yaml
$ ascribe report inventory
…
APIs: public: 24 operations, 19 linked, 6 with a checked example
```

## Context

- The proposal's phase 4 paragraph.
- `crates/ascribe-check/src/across.rs`: `fragment-unused` and its kind, and `Project::uses`; `ascribe report inventory` and its JSON.

## Design

- **`api-operation-unreferenced`,** advice, in area `project`, beside `fragment-unused`, on the spec's declaration, one per operation. Only links count, by id or by method and path, and examples from phase 5 when it's built; a word in prose that matches an id doesn't.
- **The inventory** gains, per API: operations, operations linked, and operations with a checked example (zero before phase 5). A field added to its JSON (decision 4).
- **A deprecated operation** isn't reported, since it's on its way out.

## Tasks

1. `api-operation-unreferenced`, counted from every build's links, with conformance cases: linked by id, by method and path, unlinked, deprecated.
2. The inventory's counts, its schema, and output tests.
3. The decision files.
4. The docs: the section's page on coverage, and the inventory's reference; `changes/language/openapi-coverage.md`.

## Out of scope

The embed.

## Acceptance criteria

- The goal's commands, in tests.

## Verify

```sh
cargo test -p ascribe-conformance --test conformance
cargo test -p ascribe-cli --test all report
```

## Commits

1. "Report operations no page links to, and count them in the inventory"
