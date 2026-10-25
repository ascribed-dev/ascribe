# Phase 7: The thin embed

Part of [OpenAPI](README.md). Needs phase 2.

## Goal

`@operation` puts an operation's method, path, summary, and deprecation in a page, linked to its reference.

```markdown
@operation: ../api/openapi.yaml#getUser
```

The site output writes an element with the method, the path, the summary, and a deprecation mark, linked to the operation's address; the plain output writes it as a line of text with the link; the JSON output as a block with the operation.

## Context

- The proposal's phase 5 paragraph: no hyphen in the name, since a hyphenated name is a project's widget (SPEC §6), and no schema rendered.
- `packages/elements/CONTRACT.md`, the site-render contract, and `tests/render/`, which both renderers must pass.
- The directives' registry in `ascribe-core` and the conformance cases for a directive.

## Design

- **`@operation: <link>`,** a line directive whose value is a link to an operation in either form, checked, completed, and renamed as phase 3's links are.
- **The element** (`<ascribe-operation>`), with its attributes, added to the elements' contract; `render_site_html` and the Astro plugin both pass its fixtures in `tests/render/`.
- **The plain output:** "GET /users/{id}: Get a user", with the link, and "(deprecated)" when it is.
- **No schema, no parameters:** a page that wants them links to the reference, or shows an example phase 5 checks.

## Tasks

1. The directive, with its conformance cases.
2. The element, its contract, the render fixtures, and both renderers.
3. The plain and JSON outputs, with emit tests.
4. SPEC's directive, the element's contract, and their decision files.
5. The docs: the section's page on `@operation`, and the directives' reference; `changes/language/openapi-embed.md`.

## Out of scope

Rendering a schema, parameters, or a try-it console.

## Acceptance criteria

- The goal's directive, in a conformance case, and its element in `tests/render/`, passed by both renderers.

## Verify

```sh
cargo test -p ascribe-conformance --test conformance
cargo test -p ascribe-emit operation
pnpm --filter @ascribed/astro test
```

## Commits

1. "Put an operation in a page with @operation"
