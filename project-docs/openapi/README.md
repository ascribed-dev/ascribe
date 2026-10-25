# OpenAPI

A writer links a guide to an operation in the project's OpenAPI spec, and the link is checked like a link to a heading, completed and shown in the editor, and written as the operation's address in the published reference. A JSON example that claims to be an operation's request or response is checked against its schema, and a report says which operations no page links to. Ascribe reads the spec and checks the pages; Redoc, or another renderer through a template, draws the reference. The idea, the research, the two spikes, and the maintainer's three settled answers are in the [proposal](../proposals/openapi.md). This is the plan for building it.

## Who it's for

- **A writer of guides** for an API, whose links into the reference break silently today when the spec is regenerated.
- **A team whose spec is generated from code,** often beside the docs rather than in them (the proposal's Settled 1), and often with mechanical or missing ids (its spikes).
- **An agent writing a guide,** which can check a link to an operation before writing it (phase 3).

## What exists today

The proposal's "What exists" holds. The pieces this plan stands on, checked against `main` on 24 October:

- **The resolver:** `resolve_reference` in `crates/ascribe-resolve/src/references.rs` returns a `Resolution` (`crates/ascribe-resolve/src/project.rs`: `External`, `Source`, `SourceMissing`, `Asset`, `AssetMissing`, `Route`, …), and a build's link becomes a `LinkTarget` (`crates/ascribe-resolve/src/build/tree.rs`: `External`, `Page`, `Asset`). Check, the build, the language server, and `ascribe link` all go through it. A link to `api/openapi.yaml#getUser` is an `Asset` with its fragment kept and unchecked.
- **The boundary:** `Layout::is_allowed` (`crates/ascribe-resolve/src/layout.rs`) allows a copied file inside the content root or the project root (SPEC §9.4). `FileSystem::read_file` (`crates/ascribe-resolve/src/fs.rs`) reads a path relative to the project root that may start with `..`, once the caller has decided it may: it's how a snippet reads code through a `[sources]` folder.
- **The same-repository rule:** `crates/ascribe-model/src/loader.rs` checks a source's folder against `repository_root` and reports `model-source-outside-repository`; the spec reuses that rule.
- **The outputs' links:** the site output writes a link in `crates/ascribe-emit/src/site/inline.rs` (`link_inline`); the JSON output's `TargetJson` (`crates/ascribe-emit/src/json.rs`) is `External`, `Page`, or `Asset`.
- **The editor:** completion after `#` offers a page's headings (`crates/ascribe-lsp/src/complete.rs`, `headings_of`); hover is `hover.rs`; go to definition into a non-Markdown file is how a snippet's region opens (`examples.rs`).
- **Unused things:** `fragment-unused` and its kind in `crates/ascribe-check/src/across.rs`, and `ascribe report inventory`.
- **A built site in a test:** `examples/astro-site`'s end-to-end tests (`test/e2e/`) build the site and drive it with Playwright, the way to prove a link lands on Redoc.
- **The spikes' material,** to reuse: the spec with awkward tags and ids, the Redoc addresses and the landing check, and the corpus sample, described in the proposal's "What the spikes found".

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

The proposal's Settled 1 to 3 and its design hold, except where this list narrows them. From this plan:

1. **The thread is in `examples/astro-site`,** not quill: it's the example whose test builds a site and follows links in a browser. Its spec is `api/openapi.yaml`, beside `content/`, and Redoc's page is an Astro page that loads Redoc from a pinned `devDependency`, never a CDN.
2. **The Redoc profile is Redoc 3's.** Ascribe can't tell which Redoc a site runs. An operation's address, `<at>#operation/<operationId>`, lands in Redoc 2 and 3 alike (the spike); the address for an operation with no id, `<at>#paths/<pointer>/<method>`, lands only in Redoc 3, so the docs say a link by method and path needs Redoc 3, and nothing reports it, since Ascribe can't know. The maintainer approved this narrowing of the proposal's Settled 3 on 24 October 2026.
3. **The reader is loose and owns nothing else:** a module in `ascribe-resolve` reading each operation's method, path, `operationId`, `summary`, `tags`, and `deprecated` from `paths`, in Swagger 2.0 and OpenAPI 3.0 to 3.2, with `serde_yaml_ng` and `serde_json`, ignoring the rest; phase 5 adds request and response schemas. It never validates the spec.
4. **A link's fragment is percent-decoded** before it's matched, since ids hold `/` and spaces (the spikes).
5. **A link to a declared spec is an operation link only with a fragment.** Without one it's an asset, as today, when the spec is inside the project's folder, and an error when it's outside (the proposal's Settled 1).
6. **The JSON output's target for an operation link** is a new kind, `operation`, with the API's name, the `operationId` or the method and path, and the address. Its shape is settled in phase 1, with its decision.
7. **The example marker is decided before phase 5** (phase 4), with a spike of the validator first: what a JSON Schema validator adds to the binary, and how a disagreement is put at its place in the block, since `serde_json` keeps no positions.

The contracts, and the phase that records each in `project-docs/decisions/`: `[apis]` (content-model contract), the operation link (SPEC §5.2), the spec outside the project (SPEC §9.4), the JSON output's `operation` target, and the first diagnostics (phase 1); the method-and-path form (phase 2); `ascribe link`'s answer and the MCP tool's (phase 3); the example marker (phase 4) and its diagnostics (phase 5); `api-operation-unreferenced` and the inventory's counts (phase 6); `@operation` and its element (phase 7).

## The thread

**The use case:** a writer links a guide in `examples/astro-site` to `getUser`; `ascribe check` fails with `link-operation-missing` when the spec loses it; and in the built site, the link opens `getUser` in Redoc.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The model | `ascribe-model` | `[apis.<name>]`, `spec` and `reference`, and the same-repository rule for a spec outside the folder |
| Resolve | `ascribe-resolve` | The reader, and an operation link as a new `Resolution` and `LinkTarget` |
| Check | `ascribe-check` | `link-operation-missing` and the spec's own diagnostics |
| Emit | `ascribe-emit` | The address in the site and plain outputs, and the `operation` target in the JSON output |
| The command | `ascribe-cli` | Nothing new to say: `check` and `build` |
| Astro | `examples/astro-site` | Redoc's page, and the end-to-end test that follows the link |
| The contracts | SPEC, the content-model contract | §5.2, §9.4, `[apis]`, the JSON output |
| The docs | `docs/content/openapi/` | A page: link to your API reference |

**The seam it tests** is the address in a URL space Ascribe doesn't own: the link, written by the site output, followed in a browser, lands on the operation in Redoc at its pinned version, and the template's address does the same for Stoplight Elements. If Redoc's address doesn't land from a built site as it did from the spike's page, the profile changes before anything else is built on it.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| A link to an operation with no id, by method and path | 2 |
| Completion, hover, go to definition, a renamed operation's fix, deprecation, `ascribe link`, and the MCP tool | 3 |
| How a JSON example names its operation, and the validator | 4 (the decision), 5 |
| Operations no page links to | 6 |
| `@operation` | 7 |

## Phases

| Phase | Result |
|---|---|
| [1: The thread](phase-1-thread.md) | A link to an operation is checked, and the built site's link opens it in Redoc. |
| [2: Operations with no id](phase-2-no-id.md) | A writer links an operation the spec gives no id, by its method and path. |
| [3: In the editor and for agents](phase-3-editor.md) | Operations are completed after `#`, shown on hover, opened in the spec, and fixed when renamed; an agent checks a link with `ascribe_link`. |
| [4: Decide the example marker](phase-4-example-decision.md) | The maintainer settles how a JSON block names its operation, from a spike of the validator. |
| [5: Checked examples](phase-5-examples.md) | A JSON example marked as an operation's request or response is checked against its schema, each disagreement at its place. |
| [6: Coverage](phase-6-coverage.md) | `ascribe check` and the inventory say which operations no page links to. |
| [7: The thin embed](phase-7-embed.md) | `@operation` puts an operation's method, path, summary, and deprecation in a page, linked to its reference. |

Phase 1 is the thread, through the riskiest seam. After it, by risk: phase 2 is a second link form, a language contract, while the thread's code is fresh; phase 3 is surfaces over phase 1's resolution; phase 4 decides the most design risk left, a new syntax and a new dependency, before phase 5 builds it; phase 6 and 7 are the smallest, a report over links that already resolve and an element over the same.

## Carved out, and waiting

The proposal's "What this leaves out": generated reference pages and schemas, overlays and multi-file specs, a spec in another repository, a spec per release line, availability from the spec, drift between the spec and the API, other renderers by name, and AsyncAPI and GraphQL.

## The other plans

[Redirects](../redirects/README.md) and [a page in the site output](../resolve-emit/README.md) are planned and touch none of this, except that both change `ascribe.toml`'s sections: a phase that adds a key reads the content-model contract as it is on `main`.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **Outputs change only as each phase says.** A project with no `[apis]` writes what it writes today: `node scripts/compare/outputs.ts --base main` shows no change outside `examples/astro-site` and the conformance cases a phase adds.
- **Contracts change only by a recorded decision,** in the phase that changes them, as the list above says.
- **Libraries don't print, and don't panic on input.** A spec that can't be read is a diagnostic. The spec is read through `ascribe_resolve::FileSystem`, `read_file` for a spec outside the folder, as a source's code is.
- **Behavior changes come with tests at the layer that proves them** ([writing-tests](../../.claude/skills/writing-tests/SKILL.md)): the reader in `ascribe-resolve`'s tests, each diagnostic and link form in `tests/conformance/cases/`, the outputs in `crates/ascribe-emit/tests/all/`, the editor's answers in `ascribe-lsp`'s tests, and the built site in `examples/astro-site`'s end-to-end tests.
- **Diagnostics** are added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs` in the same order.
- **The map of the surface** is regenerated when a phase changes it (`node scripts/surface/inventory.ts`), and its diff read against the conventions.
- **User-visible changes** update their page under `docs/content/openapi/`, marked `@available: ascribe planned` where they're new, and add an entry in `changes/` in the same phase. The section is a feature folder: each phase adds to it, whatever kind of page it is.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- Everything in the proposal's "What this leaves out".
- A check that a link's operation kept its method and path across a base revision, for an id that changed meaning (springdoc's suffixes, in the spikes).
