# Phase 5: Checked examples

Part of [OpenAPI](README.md). Needs phase 4's decision, and phase 2.

## Goal

A JSON example marked as an operation's request or response is checked against its schema, each disagreement at its place.

````markdown
```json request="../api/openapi.yaml#createUser"
{ "name": 42 }
```
````

```console
$ ascribe check
error[example-schema-mismatch]: `name` is a number; createUser's request body says it's a string
  --> guides/users.md:12:11
```

The exact names and text are phase 4's.

## Context

- Phase 4's decision file; the proposal's phase 3 paragraph.
- The info string's attributes, as `title` and `tested` are parsed (SPEC §3 and the parser in `ascribe-syntax`).
- Phase 1's reader, which gains request bodies, responses, and components.

## Design

- **The marker** as phase 4 settled it, parsed with the info string's other attributes, and resolved as a link: checked, completed, hovered, and renamed by phase 3's code.
- **The reader** gains each operation's request body and responses by media type, and the components their schemas reference, in the same file; 3.0's and 2.0's schemas translated as phase 4 found.
- **The check:** the block parsed with positions, validated, and each disagreement reported at its place, or at the block when phase 4 found positions can't be had.
- **The editor** shows the disagreements as you type, as every page diagnostic is.

## Tasks

1. The marker in the syntax and the resolver, with conformance cases.
2. The reader's bodies, responses, and components, and the translation, with tests over 2.0, 3.0, and 3.1 schemas.
3. The validator and the positions, behind the decision's dependency, and the diagnostics with their conformance cases.
4. SPEC's change for the marker, the diagnostics, and their decision files.
5. The docs: the section's page on checking examples; `changes/language/openapi-examples.md`.

## Out of scope

Coverage and the embed. Examples in languages other than JSON.

## Acceptance criteria

- The goal's check, in a conformance case, with the disagreement at its place.
- An example that agrees passes, in 2.0, 3.0, and 3.1 specs.

## Verify

```sh
cargo test -p ascribe-resolve openapi
cargo test -p ascribe-conformance --test conformance
```

## Commits

1. "Mark a JSON block as an operation's request or response"
2. "Check a marked example against its schema"
