# Phase 1: The thread

Part of [OpenAPI](README.md). Needs nothing.

## Goal

A link to an operation is checked, and the built site's link opens it in Redoc.

In `examples/astro-site`:

```toml
[apis.public]
spec = "api/openapi.yaml"
reference = { renderer = "redoc", at = "/api/" }
```

```markdown
To fetch one user, call [Get a user](../api/openapi.yaml#getUser).
```

```console
$ ascribe check
No problems.
$ # getUser removed from api/openapi.yaml
$ ascribe check
error[link-operation-missing]: api/openapi.yaml has no operation `getUser`; the closest is `getUsers`
```

`ascribe build` writes the link as `/docs/api/#operation/getUser` (under the project's `base-path`), and the end-to-end test, following it in the built site with Redoc at its pinned version, lands on the operation.

## Context

- README decisions 1 to 6; the proposal's "The design", Settled 1 and 2, and "What the spikes found".
- `crates/ascribe-resolve/src/references.rs`, `resolve_reference`, `SourceSet`; `crates/ascribe-resolve/src/project.rs`, `Resolution`; `crates/ascribe-resolve/src/build/tree.rs`, `LinkTarget`.
- `crates/ascribe-resolve/src/layout.rs`, `is_allowed`; `crates/ascribe-resolve/src/fs.rs`, `read_file`.
- `crates/ascribe-model/src/loader.rs`, the `[sources]` folder checks and `repository_root`; `crates/ascribe-model/src/sections.rs` for adding a section.
- `crates/ascribe-emit/src/site/inline.rs`, `link_inline`; the plain output's link writing; `crates/ascribe-emit/src/json.rs`, `TargetJson`.
- `tests/conformance/diagnostics.toml`'s `link-id-missing`, the level and the "closest" suggestion to match.
- `examples/astro-site/test/e2e/`, and the spike's landing check (the proposal's "Renderers' addresses").

## Design

### The model

`[apis.<name>]` with `spec` and `reference`, per the proposal's table. `spec` is a path from the project root; a path outside the project's folder is allowed when it's in the same git repository, checked as a source's folder is. `reference` is `{ renderer = "redoc", at = "…" }` or a template string with `{operationId}` and `{tag}`. A spec that can't be read or parsed is an error on its declaration, `api-spec-unreadable`; one with no operations is advice.

### The reader

`ascribe-resolve` reads each declared spec once per project load through `FileSystem`, `read_file` for a spec outside the folder, into each operation's method, path, `operationId`, `summary`, first tag, and `deprecated` (README decision 3). It returns what it read and what it couldn't; it never fails a project's load.

### The link

`resolve_reference` resolves a link whose path is a declared spec and that has a fragment to a new `Resolution`, an operation: the API's name and the operation, or, when the decoded fragment (README decision 4) isn't an id, the closest ids for `link-operation-missing`. A link to the spec with no fragment stays an asset inside the folder; outside it, it's an error, `link-spec-outside-project`, saying to link the published reference (README decision 5). A link's path that leaves the project's folder is allowed only when it reaches a declared spec: SPEC §9.4's exception, with its conformance case.

The build's `LinkTarget` gains an operation, with its address: Redoc's `<at>#operation/<operationId>`, or the template filled in, `{operationId}` and `{tag}` percent-encoded, joined to `[consumer] base-path` unless absolute.

### The outputs

The site output writes the address; the plain output writes it absolute when `[consumer] site` is set or the address is absolute, and as a route otherwise, as page links are. The JSON output's `TargetJson` gains `operation`: `api`, `operationId`, `address` (README decision 6).

### The thread's site

`examples/astro-site` gains `api/openapi.yaml`, built from the spike's spec (tags with spaces and accents, an id with a dot), an Astro page at `/api/` that loads Redoc 3 from a pinned `devDependency`, and a guide that links to two operations, one under a tag with a space. Its end-to-end test builds the site, opens the guide, follows each link, and asserts the operation's heading is at the top of the viewport, as the spike's check did. A second test project, with a template for Stoplight Elements, proves the template's address the same way.

### The corpus

The reader runs over the spikes' APIs.guru sample (re-downloaded by a script kept out of the test suite) and the generated specs, and the pull request records what it couldn't read, which should be nothing.

## Tasks

1. `[apis]` in `ascribe-model`, with the same-repository rule for `spec`, and its model diagnostics.
2. The reader in `ascribe-resolve`, with tests over Swagger 2.0, OpenAPI 3.0, 3.1, and 3.2 specs, YAML and JSON, and a spec outside the folder through `read_file`.
3. The operation `Resolution` and `LinkTarget`, the §9.4 exception, and `link-operation-missing`, `link-spec-outside-project`, and `api-spec-unreadable`, each with its conformance case: a link to an operation, a missing one, one whose id holds `/` and one with a space (percent-encoded), the spec with no fragment inside and outside the folder, and an undeclared YAML file.
4. The outputs, with emit tests: Redoc's address, a template's, under a `base-path`, absolute with `[consumer] site`, and the JSON target.
5. The thread's site, Redoc's page, and the end-to-end tests for Redoc and Stoplight Elements.
6. The corpus run, recorded in the pull request.
7. The contracts: the content-model contract's `[apis]`, SPEC §5.2's operation link and §9.4's exception, the JSON output's `operation`, and a decision file for each.
8. The docs: a new section, `docs/content/openapi/`, with `index.md` ("Link to your API reference", `@available: ascribe planned`): declaring a spec, inside or outside the folder; Redoc and a template, naming Swagger UI and Stoplight Elements as templates known to work and saying what isn't supported; what a link checks and writes; setting ids explicitly, since a generated id can change meaning. `ascribe.nav` places the section. `changes/language/openapi-links.md`.

## Out of scope

Everything in the README's "Left out" table. A link by method and path is a missing operation until phase 2.

## Acceptance criteria

- The goal's commands, in a test.
- The end-to-end tests land on each linked operation in Redoc, and on the template's in Stoplight Elements.
- Every conformance case task 3 lists passes.
- The corpus run reads every spec, recorded in the pull request.
- `node scripts/compare/outputs.ts --base main` changes nothing outside `examples/astro-site` and the new cases.

## Verify

```sh
cargo test -p ascribe-resolve openapi
cargo test -p ascribe-emit openapi
cargo test -p ascribe-conformance --test conformance
pnpm --filter @ascribed/example-astro-site test:e2e
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Declare an API's spec in [apis]"
2. "Read a spec's operations, loosely"
3. "Check a link to an operation, and write its address in the reference"
4. "Follow a link to an operation into Redoc, in astro-site's end-to-end test"
5. "Link to your API reference: the page, the contracts, and the decisions"
