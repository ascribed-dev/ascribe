# Proposal: OpenAPI

A proposal, not a plan. It says what would be built and in what order, so the idea can be judged before anything is designed in detail. It picks up section 3 of the [brainstorm](../brainstorm.md#3-openapi), and draws on the research in [OpenAPI documentation pain points](../reports/OpenAPI%20documentation%20pain%20points.md).

## The goal

A writer links from a guide to an operation in the project's OpenAPI spec, and the link is checked like a link to a heading: completed in the editor, shown on hover, and an error when the spec no longer has the operation. A JSON example in a page that claims to be an operation's request or response is checked against that operation's schema. A report says which operations no page mentions.

Ascribe reads the spec and checks the pages against it. It doesn't draw the API reference. A renderer the team already uses (Scalar, Redoc, starlight-openapi, or a hosted portal) publishes the reference pages, and Ascribe's links land on them.

## Why checks and not a renderer

The research found the market split in two.

- **Rendering a spec is crowded and mostly solved.** Every tool surveyed draws a reference, and most embed an operation in an authored page. The hard parts are expensive and never finished: `oneOf`, `allOf` and discriminators (Swagger UI's most upvoted issue, open since 2017), nullability written two ways, four spec versions, try-it consoles, auth, code samples, and specs of 80,000 lines. Vendors with whole teams lag a year behind OpenAPI 3.2. In Ascribe the cost doubles, since `render_site_html` and the Astro plugin must agree, and `@ascribed/elements`' contract would grow with every schema shape.
- **The joins between the spec and the pages are unchecked.** Links from prose into operations break silently in every tool but Fern. Examples in guides are validated by nothing. No tool reports operations no guide mentions. These are what Ascribe already does for pages: links, completion, hover, rename, diagnostics, and reports of unused things.

So the brainstorm's level 1 is the core of this proposal, level 2 is built in a thin form, and level 3 is left to the renderers.

## What exists

- **Links to files that aren't pages** (SPEC §5.2, the [assets contract](../../docs/content/contracts/assets.md)). A link to `api/openapi.yaml` is an asset: checked for existence with exact case, copied into every output, and rewritten. Its fragment is kept and never checked, so `[Get a user](../api/openapi.yaml#getUser)` passes today and opens the raw YAML.
- **One resolver for every link.** `resolve_reference` (`crates/ascribe-resolve/src/references.rs`) serves check, the build, the language server, and `ascribe link`, and returns a `Resolution`. A new kind of target is a new variant there, and every surface picks it up.
- **The boundary.** A link's target may be inside the content root or the project root (`Layout::is_allowed`). With `ascribe.toml` at the repository's root, `api/openapi.yaml` is in reach; with `ascribe.toml` inside `docs/`, it isn't.
- **The editor's link features.** Completion after `#` offers a page's headings (`crates/ascribe-lsp/src/complete.rs`); hover previews the target (`hover.rs`); go to definition can already land inside a file that isn't Markdown, as a snippet's region does (`examples.rs`).
- **Reports of unused things.** `fragment-unused`, `image-unused`, and the others are advice in `crates/ascribe-check/src/across.rs`, counted by `Project::uses`, and listed by `ascribe report inventory`.
- **YAML and JSON parsers** (`serde_yaml_ng`, `serde_json`) are workspace dependencies. Nothing validates against a JSON Schema: `jsonschema` isn't a dependency.

Nothing in the code reads OpenAPI.

## The design

### The spec is declared

```toml
[apis.public]
spec = "api/openapi.yaml"
reference = "/api/#tag/{tag}/operation/{operationId}"
```

| Key | Meaning |
|---|---|
| `spec` | The spec's file, from the project root. One file, in YAML or JSON. |
| `reference` | The address of an operation's page in the published reference, as a template. Relative to `[consumer] base-path`, or absolute when the reference is published elsewhere. Its placeholders are the operation's `{operationId}`, `{tag}` (its first), `{method}`, and `{path}`. |

The section is `[apis]`, not `[openapi]`, so an AsyncAPI spec can be declared beside it later, with a `format` key added then.

### Reading the spec, loosely

Ascribe reads the part of the spec its checks need: each operation's method, path, `operationId`, `summary`, `tags`, and `deprecated`, and, from phase 3, its request and response schemas and the components they reference. It reads that part the same way in Swagger 2.0 and OpenAPI 3.0, 3.1, and 3.2, since `paths` hasn't changed shape across them, and it ignores everything else. It doesn't validate the spec; Spectral and Redocly do that.

This is what keeps Ascribe out of the version lag the research found. A reader that models the whole spec has to be rewritten for each version; one that reads `paths` and skips the rest reads 3.2 today. It also accepts the documents that strict readers reject: a spec labelled 3.1 that still writes `nullable: true` makes the most common validator error in one analysis of published specs, and its links are as good as any.

Ascribe reads the spec as readers see it: the file the team's pipeline publishes, after bundling and overlays. It doesn't apply overlays or follow `$ref`s into other files. A team whose spec is split across files, or patched by an overlay, points `spec` at the bundled result their renderer is given. A spec generated from code has to be on disk when Ascribe runs, as it does for the renderer.

The spec is read through `FileSystem::read_file`, and must be inside the project root.

### A link to an operation

```markdown
To fetch one user, call [Get a user](../api/openapi.yaml#getUser).
```

A link to a declared spec with a fragment is a link to an operation: the fragment is the `operationId`. A link to the spec without a fragment is an asset, as today, for "download the spec". A link to an undeclared YAML file is an asset, as today.

- **Checked.** An `operationId` the spec doesn't have is an error, `link-operation-missing`, with the closest ids suggested, at the level `link-id-missing` has. A spec that can't be read or parsed is an error on its declaration.
- **Written as the renderer's address.** The site output writes the `reference` template filled in for the operation; the plain output writes it absolute when `[consumer] site` is set or the template is absolute; the JSON output gives the link's target as an operation: the API's name, the `operationId`, and the address. That JSON is a contract, and its shape is settled in phase 1.
- **Unambiguous.** A link names its spec by path, so two specs with the same `operationId` never collide, where Mintlify takes the first match in alphabetical order.

The `operationId` is the address because it's readable, survives a path being renamed, and is what the renderers key their own pages by. Its weakness is that generated ids are mechanical (`createItemItemsPost`) and sometimes missing. Phase 1 measures how often they're missing, and the first open question asks what a link names when there's no id.

### Why these choices

- **A path and a fragment, not a new scheme.** A link to the spec is already a link to a file, relative like any other, completed and checked by the resolver every surface uses. A scheme such as Fern's `api:GET/users` would be a second link syntax, and an agent or a writer would have to learn which one to use.
- **A template, not knowledge of each renderer.** Ascribe doesn't know which renderer a team uses, and renderers change their addresses between releases. A template is one string the team owns. The risk is that a renderer's addresses can't be written as a template, which is what phase 1 tests.
- **No overlays, no multi-file specs.** Both are the pipeline's job, and the research found their tooling still settling. Reading the bundled file gives the same answer and keeps Ascribe's reader small.

## The thread

- **The use case:** a writer links a guide to `getUser`; `ascribe check` fails when the spec is regenerated without it; and in the built site, the link opens `getUser` in the reference a renderer publishes beside the guides.
- **The layers:** the content model (`[apis]`), resolve (reading the spec, a new `Resolution` variant), check (the diagnostics), emit (the address in the site, plain, and JSON outputs), the CLI, the contracts, and the docs. The language server shows the diagnostic without new work, since it runs the same check.
- **The seam it tests:** the renderer's address. Ascribe has never written a link into a URL space it doesn't own. A renderer's address for an operation is made from some of the tag, the `operationId`, the method, and the path, each slugged its own way. If phase 1 finds that a common renderer's addresses can't be written as a template (a tag slugged by rules a template can't express, or an address that isn't stable across the renderer's releases), the design changes to named profiles, one per renderer, as the `Router` is one per consumer.
- **Measured on real specs.** Phase 1 runs the reader over a corpus: a sample of APIs.guru, and specs generated by FastAPI, springdoc, Swashbuckle, and NestJS. It records which specs it can't read, how many operations have no `operationId`, and how many ids are mechanical. That's the first open question's input.
- **Left out, and where it comes back:** completion, hover, go to definition, and the fix for a moved operation (phase 2); `ascribe link` and the MCP server (phase 2); examples checked against schemas (phase 3); the report of operations no page mentions (phase 4); the embed (phase 5).

## The phases

| Phase | What a user can do after it | Layers |
|---|---|---|
| 1. Checked links | Link a page to an operation, see the build fail when the operation goes, and land on it in the published reference | content model, resolve, check, emit, CLI, contracts, docs |
| 2. In the editor and for agents | Complete an operation after `#`, see its method, path, and summary on hover, jump to it in the spec, and fix a link whose operation was renamed | language server, extension, `ascribe link`, MCP |
| 3. Checked examples | Mark a JSON block as an operation's request or response, and see where it disagrees with the schema | language, resolve, check, language server |
| 4. Coverage | See which operations no page mentions, in the check and in the inventory | check, report, contracts |
| 5. The thin embed | Put an operation's method, path, summary, and deprecation in a page, with a link to its reference page | language, emit, both renderers, elements |

**Phase 1** builds the thread. `examples/quill` gains a small spec and a renderer's reference beside its pages, and a page that links to two operations. Conformance cases hold a link to an operation, a missing one, a link to the spec with no fragment, and an undeclared YAML file. The content-model reference gains `[apis]`, SPEC §5.2 gains the new kind of target, and the docs gain a page, "Link to your API reference". *Done when:* `ascribe check` in `examples/quill` reports `link-operation-missing` after `getUser` is removed from the spec; `ascribe build` writes the link as the renderer's address, and following it in the built site opens that operation; and the corpus's results are recorded in the pull request.

**Phase 2** brings the links into the editor and to agents. Completion after `openapi.yaml#` lists the spec's operations, matched on id, method, path, or summary. Hover shows the method, the path, the summary, and whether the operation is deprecated. Go to definition opens the spec at the operation. The spec is re-read when it changes on disk. When an `operationId` disappears and an operation with the same method and path has a new one, which is what a regenerated spec does, the diagnostic offers that id as a safe fix. A link to a deprecated operation is advice, `link-operation-deprecated`. `ascribe link` and `ascribe_link` answer with the operation, so an agent writing a guide can check a link before writing it.

**Phase 3** checks examples. A JSON code block marked as an operation's request or response body (the second open question) is validated against that body's schema, and each disagreement is reported at its place in the block. 3.1's schemas are JSON Schema; 3.0's and 2.0's are translated first, `nullable` among the rest. It needs a JSON Schema validator, a new dependency, and the components a schema references, still within the one file. It's before coverage because it carries the most design risk left: a new syntax, a new dependency, and the translation.

**Phase 4** reports operations no page links to, as advice, `api-operation-unreferenced`, in area `project`, beside `fragment-unused`. The inventory gains a count per API: operations, operations linked, and operations with a checked example. Only links count: a page that mentions `getUser` in prose isn't counted, since that can't be told reliably from a word that happens to match.

**Phase 5** adds the embed: `@operation: ../api/openapi.yaml#getUser` writes a block with the operation's method, path, summary, and a deprecation mark, linked to its reference page. The name has no hyphen, since a hyphenated name is a project's widget (SPEC §6). The block is checked as the link is, and the plain output writes it as a line of text with the link. It renders no schema: a page that wants the parameters links to the reference, or shows an example that phase 3 checks.

## What this leaves out

- **Generated reference pages, and rendering schemas.** The renderers do it, and do it better than a first version would. If teams ask for one look across guides and reference, which is why Stripe, Adyen, and GitHub built their own, generated pages can be proposed then, on top of the links, the addresses, and the reader this builds.
- **Applying overlays and following `$ref`s into other files.** The pipeline bundles; Ascribe reads the result.
- **A spec from another repository.** A `[sources]` git source copies only the files snippets name, and its files aren't assets. A team whose spec lives elsewhere commits a copy, or a later proposal makes a source's file declarable as a spec.
- **A spec per release line.** A versioned build reads the same spec for every line. Lines whose APIs differ need the spec as of each line, which waits for the versions work to say how a line reads another file's history.
- **Availability from the spec.** The brainstorm's `x-ascribe-available` would map onto availability specs (`parse_availability`). Each renderer already reads its own extension for the same fact, and a fifth would make the team mark it again.
- **Drift between the spec and the running API.** Contract testing (Schemathesis, Prism) proves that. Ascribe proves the pages agree with the spec.
- **AsyncAPI and GraphQL.** The `[apis]` section and the link form leave room for them; neither is built here.

## Open questions

1. **What does a link name when an operation has no `operationId`?** Either such an operation can't be linked, and `link-operation-missing` says why, or a second fragment form names it by method and path, such as `#get/users/{id}`. Recommended: decide after phase 1's corpus. If missing ids are rare, the first; if not, the second, since a link a writer can't write is worse than a second form.
2. **How does a JSON block say which operation it belongs to** (phase 3)? An attribute in the info string, as `title` and `tested` are (```` ```json request="openapi.yaml#createUser" ````, with `response="…#createUser 201"`), or a directive around the block. Recommended: the info string, since it keeps the block a plain fenced block that every Markdown viewer still shows; whichever, it's a language change with its own decision before phase 3.
