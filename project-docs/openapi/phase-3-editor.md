# Phase 3: In the editor and for agents

Part of [OpenAPI](README.md). Needs phase 2.

## Goal

Operations are completed after `#`, shown on hover, opened in the spec, and fixed when renamed; an agent checks a link with `ascribe_link`.

Typing `[Get a user](../api/openapi.yaml#us` offers the spec's operations matched on id, method, path, or summary. Hovering the link shows `GET /users/{id}`, "Get a user", and "Deprecated" when it is. Go to definition opens the spec at the operation. When a regenerated spec drops `getUser` and the same method and path now have `fetchUser`, `link-operation-missing` offers `fetchUser` as a safe fix. `ascribe link "../api/openapi.yaml#getUser" --from guides/users.md` answers with the operation, and so does `ascribe_link`.

## Context

- The proposal's phase 2 paragraph; phase 1's reader and resolution.
- `crates/ascribe-lsp/src/complete.rs` (`link`, `headings_of`, `link_destination`), `hover.rs`, `definition.rs`, and `examples.rs` for opening a place in a file that isn't Markdown.
- `ascribe link`'s command and JSON, and `crates/ascribe-cli/src/mcp/tools.rs`' `ascribe_link`.

## Design

- **Completion** after `<spec>#` lists operations, each with its id (or method and path, for one with none), its method and path as detail, and its summary as documentation; matched on any of them.
- **Hover** shows the method and path, the summary, the tag, and deprecation.
- **Go to definition** opens the spec at the operation's key, from the reader's positions: the reader records each operation's line in the spec.
- **The spec is re-read** when it changes on disk, through the server's file watching, including a spec outside the project's folder.
- **The rename fix:** when an id is missing and the spec has an operation with the link's last-known method and path, which phase 1's resolution doesn't know, the fix comes from the base revision's spec where the server has one (review's base), or from an operation with the same method and path whose id is new. Safe only when exactly one operation matches.
- **Deprecation:** `link-operation-deprecated`, advice.
- **`ascribe link` and `ascribe_link`** answer an operation link with the operation: API, id or method and path, summary, deprecation, address. A field added to their JSON (decision 4); an answer kind added is a decision of its own (decision 56).

## Tasks

1. The reader's positions, and completion, hover, and go to definition, with the language server's tests.
2. The re-read on change, including outside the folder.
3. The rename fix and `link-operation-deprecated`, with conformance cases.
4. `ascribe link`'s and `ascribe_link`'s answer, their schema, output tests, and a decision file; the surface map regenerated.
5. The docs: the section's editor page and the agents' commands page; `changes/editor/openapi.md` and `changes/agents/openapi-link.md`.

## Out of scope

Examples, coverage, and the embed.

## Acceptance criteria

- The goal's completion, hover, definition, and fix, in the language server's tests.
- `ascribe link` and `ascribe_link` answer with the operation, in their tests.

## Verify

```sh
cargo test -p ascribe-lsp openapi
cargo test -p ascribe-cli --test all output
cargo test -p ascribe-cli --test all mcp
```

## Commits

1. "Complete, hover, and open an operation in a link"
2. "Fix a link whose operation was renamed, and advise on a deprecated one"
3. "ascribe link and ascribe_link answer with the operation"
